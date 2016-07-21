package Meth::OverRegion;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);
use Bio::DB::HTS::Tabix;

my $PROM =  "Upstream";
my $BODY =  "Body";
my $DOWN =  "Downstream";

sub new{
    my $class     = shift;
    #print "Script:$FindBin::Bin\n";
    return bless {}, $class;  # built in bless function
}

sub drawMeth{
    my ($class, $opts_sub) = @_;
    foreach my $context(@{$opts_sub->{context}}){
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOverRegion_$context.txt";

	open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}_MethOverRegion_$context.sh" or die "$!";
	my $fig = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOverRegion_$context.pdf";
	my $cmd = "R --vanilla --slave --input $output --xlab $opts_sub->{RegionName} --output $fig < $FindBin::Bin/lib/Meth/OverRegion.R";
	print OUT "$cmd\n";
	close OUT;

	my $r_rep = `$cmd`;
        print "$class: $r_rep\n";
    }
}

sub calMeth{
   my ($class, $opts_sub) = @_;

   ## Process the --sample arguments 
   my $pro_sample = Meth::Sample -> new();
   #print "$opts_sub\t", keys %$opts_sub, "\n";
   $pro_sample -> processArgvSampleOverRegion($opts_sub);
   
   ## Start calculate methylation information for target context
   &generTab($class, $opts_sub);
}

sub generTab{
    my ($class, $opts_sub) = @_;
    print "Start calculate methylation information for target context\n" if !$opts_sub->{verbose};
    foreach my $context(@{$opts_sub->{context}}){
	print "$context\n" if !$opts_sub->{verbose};
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOverRegion_$context.txt";
        open OUT, "+>$output" or die "$!:$output";
	print OUT "sample_name\tregion\tbin_num\tC_number\tT_number\tMethylation_level\n";
	my %rec_meth_bin;
	&get_meth_info($class, $opts_sub, \%rec_meth_bin, $context);
	foreach my $keys(keys %rec_meth_bin){
	    my ($c_num, $t_num) = @{$rec_meth_bin{$keys}};
	    my $level = $c_num/($c_num + $t_num);
	    print OUT "$keys\t$c_num\t$t_num\t$level\n";
	}
    }
}

sub get_meth_info{
    my ($class, $opts_sub, $rec_meth_bin, $context) = @_;
    my ($bin_length, $bin_num, $min_len, $max_len, $flank) = ($opts_sub->{binLength}, $opts_sub->{binNumber}, $opts_sub->{minLength}, $opts_sub->{maxLength}, $opts_sub->{flank});
    foreach my $sam_info(@{$opts_sub->{sample_list}}){ ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name, $region) = split(/,/, $sam_info);
        open REGION, $region or die "$!:$region";
	my $flag = 0;
        while(my $line = <REGION>){
            chomp $line;
	    ++$flag;
	    print "." if $flag % 1000 == 0;
            my ($chr, $stt, $end, $name, $strand)=split(/\t/,$line);
	    $strand = $strand ? $strand: "+";  #if there is no column for strand
	    my $tabix = Bio::DB::HTS::Tabix->new( filename => $meth_file);
            my $stt_flank = $stt - $flank < 0 ? 1 : $stt - $flank + 2;
	    my $end_flank = $end + $flank -2 ;
	    #print "$chr:$stt_flank-$end_flank,$bin_length, $bin_num, $min_len, $max_len, $flank\n";
            my $iter = $tabix->query("$chr:$stt_flank-$end_flank");
            my ($tot_c_num, $tot_t_num) = (0, 0);
            while ( my $line = $iter->next) {
		my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
                if($context eq $tem_context || $context eq "CXX"){
		    next if ($c_num + $t_num < $opts_sub->{minDepth} || $c_num + $t_num > $opts_sub->{maxDepth});
		    &judge_bin($class,$rec_meth_bin, $sam_name, $stt,$end,$strand,$pos,$c_num,$t_num, $opts_sub);
        	}
	    }
	}
    }
}

sub judge_bin{
    my ($class, $rec_meth_bin, $sam_name, $stt,$end,$strand,$pos1,$c_num,$t_num, $opts_sub) = @_;
    my ($bin_length, $bin_num, $flank) = ($opts_sub->{binLength}, $opts_sub->{binNumber}, $opts_sub->{flank});
    my $unit = ($end-$stt+1)/($bin_num - 0.01);
    my $keys = 0;
    if($strand eq '+'){
        if($pos1 < $stt){
            $keys = $stt - $pos1 + 1 == $flank ? -int(($stt - $pos1 + 1)/$bin_length) +1 : -int(($stt - $pos1 + 1)/$bin_length);
            $keys = "$PROM\t$keys";
        }elsif($pos1>=$stt && $pos1<$end){
            $keys = int (($pos1 - $stt + 1) /$unit) + 1;
            $keys = "$BODY\t$keys";
        }else{
            $keys = $pos1 - $end + 1 == $flank ? int(($pos1 - $end + 1)/$bin_length) + $bin_num -2: int(($pos1-$end+1)/$bin_length) + $bin_num + 1;
            $keys="$DOWN\t$keys";
        }
    }else{
        if($pos1<=$stt){
            $keys = $stt - $pos1 + 1 == $flank ? int(($stt-$pos1+1)/$bin_length) + $bin_num -2 : int(($stt-$pos1+1)/$bin_length) + $bin_num + 1;
            $keys="$DOWN\t$keys";
        }elsif($pos1>$stt && $pos1<=$end){
            $keys=int (($end-$pos1+1)/$unit) + 1;
            $keys="$BODY\t$keys";
        }else{
            $keys = $pos1 - $end + 1 == $flank ? -int(($pos1-$end + 1)/$bin_length) + 1 : -int(($pos1 - $end + 1)/$bin_length);
            $keys="$PROM\t$keys";
        }
    }
    $keys="$sam_name\t$keys";
    ${$rec_meth_bin->{$keys}}[0] += $c_num;
    ${$rec_meth_bin->{$keys}}[1] += $t_num;
}

sub get_CT_num{
    my ($class, $meth_file, $chrom, $stt, $end, $context) = @_;
    my $tabix = Bio::DB::HTS::Tabix->new( filename => $meth_file);
    my $iter = $tabix->query("$chrom:$stt-$end");
    my ($tot_c_num, $tot_t_num) = (0, 0);
    while ( my $line = $iter->next) {
	#chrC    13      +       3       643     CG      CGG
        my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	if($context eq $tem_context){
	    $tot_c_num += $c_num;
	    $tot_t_num += $t_num;
	}
    }
    return ($tot_c_num, $tot_t_num);
}

1;


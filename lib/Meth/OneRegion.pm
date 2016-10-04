package Meth::OneRegion;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);
use Bio::DB::HTS::Tabix;

sub new{
    my $class     = shift;
    #print "Script:$FindBin::Bin\n";
    return bless {}, $class;  # built in bless function
}

sub drawMeth{
    my ($class, $opts_sub) = @_;
    foreach my $context(@{$opts_sub->{context}}){
	my $out = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOneRegion_$context.sh";
	open OUT, "+>$out" or die "$!: $out";
	my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOneRegion_$context.txt";
        my $fig = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOneRegion_$context.pdf";
	my $cmd = "R --vanilla --slave --input $output --output $fig  < $FindBin::Bin/lib/Meth/Coverage.R";
	print OUT "$cmd\n";
        my $r_rep = `$cmd`;
        print "$class: $r_rep\n";
    }
}

sub calMeth{
   my ($class, $opts_sub) = @_;

   ## Process the --sample arguments 
   my $pro_sample = Meth::Sample -> new();
   #print "$opts_sub\t", keys %$opts_sub, "\n";
   $pro_sample -> processArgvSampleOneRegion($opts_sub);
   
   ## Start calculate methylation information for target context
   &generTab($class, $opts_sub);
}

sub generTab{
    my ($class, $opts_sub) = @_;
    print "Start calculate methylation information for target context\n" if !$opts_sub->{verbose};
    foreach my $context(@{$opts_sub->{context}}){
	print "$context\n" if !$opts_sub->{verbose};
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethOneRegion_$context.txt";
        open OUT, "+>$output" or die "$!:$output";
	print OUT "Sample\tchr\tposition\tC_num\tT_num\tMethylationLevel\n";
        my ($chr, $stt, $end) = $opts_sub->{region} =~ /(.*):(\d+)-(\d+)/;
        foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
	    my ($meth_file, $sample_name ) = split(/,/, $sam_info);
            my $tabix = Bio::DB::HTS::Tabix->new( filename => $meth_file);
            my $iter = $tabix->query("$chr:$stt-$end");
            while ( my $line = $iter->next) {
                my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	        #print "$line\n";
                if($context eq $tem_context || $context eq "CXX"){
                    next if ($c_num + $t_num < $opts_sub->{minDepth} || $c_num + $t_num > $opts_sub->{maxDepth});
		    my $lev = $c_num/($c_num + $t_num);
		    print OUT "$sample_name\t$chr\t$pos\t$c_num\t$t_num\t$lev\n";
                }
            }
	}
    }
}

sub get_meth_info{
    my ($class, $opts_sub, $rec_meth_bin, $sample_list, $context) = @_;
    foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name, $region) = split(/,/, $sam_info);
	push @$sample_list, $sam_name;
        open REGION, $region or die "$!:$region";
	my $flag = 0;
        while(my $line = <REGION>){
            chomp $line;
	    ++$flag;
	    print "." if $flag % 1000 == 0;
            my ($chr, $stt, $end, $name) = split(/\s+/,$line);
	    my $id = $name ? $name : "$chr\_$stt\_$end";
	    my $level = &get_CT_num($class, $meth_file, $chr, $stt, $end, $context, $opts_sub);
	    $rec_meth_bin->{$id} -> {$sam_name} = $level;
	}
    }
}

sub get_CT_num{
    my ($class, $meth_file, $chrom, $stt, $end, $context, $opts_sub) = @_;
    my $tabix = Bio::DB::HTS::Tabix->new( filename => $meth_file);
    my $iter = $tabix->query("$chrom:$stt-$end");
    my ($tot_c_num, $tot_t_num) = (0, 0);
    my $total_cov_num = 0;
    while ( my $line = $iter->next) {
        my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	if($context eq $tem_context || $context eq "CXX"){
	    ++$total_cov_num;
	    next if ($c_num + $t_num < $opts_sub->{minDepth} || $c_num + $t_num > $opts_sub->{maxDepth});
	    $tot_c_num += $c_num;
	    $tot_t_num += $t_num;
	}
    }
    my $level = $tot_c_num / ($tot_c_num + $tot_t_num + 0.0000001);
    return $level;
}

1;


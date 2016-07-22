package Meth::Heatmap;

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
	my $out = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.sh";
	open OUT, "+>$out" or die "$!: $out";
	my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.txt";
        my $fig1 = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.pdf";
        my $fig2 = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHist_$context.pdf";
	my $cmd = "R --vanilla --slave --input $output --output1 $fig1 --output2 $fig2  --cluster_cols $opts_sub->{cluster_cols} --cluster_rows $opts_sub->{cluster_rows} < $FindBin::Bin/lib/Meth/Heatmap.R";
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
   $pro_sample -> processArgvSampleOverRegion($opts_sub);
   
   ## Start calculate methylation information for target context
   &generTab($class, $opts_sub);
}

sub generTab{
    my ($class, $opts_sub) = @_;
    print "Start calculate methylation information for target context\n" if !$opts_sub->{verbose};
    foreach my $context(@{$opts_sub->{context}}){
	print "$context\n" if !$opts_sub->{verbose};
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.txt";
        open OUT, "+>$output" or die "$!:$output";
	my %rec_meth_bin;
	my @sample_list;
	&get_meth_info($class, $opts_sub, \%rec_meth_bin, \@sample_list, $context);
	print OUT "\t", join("\t", @sample_list), "\n";
	foreach my $id(keys %rec_meth_bin){
	    print "$id\n";
	    my @level = @{$rec_meth_bin{$id}}{@sample_list};
	    #my @level = @rec_meth_bin{$id}{@sample_list};
	    my $level = join("\t", @level);
	    print OUT "$id\t$level\n";
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
	my %rec_region_id;
        while(my $line = <REGION>){
            chomp $line;
	    #print "$line\n";
	    ++$flag;
	    print "." if $flag % 1000 == 0;
            my ($chr, $stt, $end, $name) = split(/\s+/,$line);
	    my $id = $name ? $name : "$chr\_$stt\_$end";
	    if(!exists $rec_region_id{$id}){
		$rec_region_id{$id} = 0;
	    }else{
		$rec_region_id{$id} ++;
	        $id = $id.$rec_region_id{$id};
	    }
	    
	    my $level = &get_CT_num($class, $meth_file, $chr, $stt, $end, $context, $opts_sub);
	    $rec_meth_bin->{$id} -> {$sam_name} = $level;
	    print "$id\t$sam_name\n";
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
	#chrC    13      +       3       643     CG      CGG
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


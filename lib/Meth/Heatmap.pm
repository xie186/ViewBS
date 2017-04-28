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
    if($opts_sub->{merge}){
	&gener_cmd($class, $opts_sub, "mer");
    }else{
        foreach my $context(@{$opts_sub->{context}}){
	    &gener_cmd($class, $opts_sub, $context);
        }
    }
}

sub gener_cmd{
    my ($class, $opts_sub, $context) = @_;
    my $out = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.sh";
    open OUT, "+>$out" or die "$!: $out";
    my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.txt";
    my $fig1 = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.pdf";
    my $fig2 = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHist_$context.pdf";
    my $cmd = "R --vanilla --slave --input $output --output1 $fig1 --output2 $fig2  --cluster_cols $opts_sub->{cluster_cols} --cluster_rows $opts_sub->{cluster_rows}  --height $opts_sub->{height} --width $opts_sub->{width} --height2 $opts_sub->{height2} --width2 $opts_sub->{width2} --random_region $opts_sub->{random_region} < $FindBin::Bin/lib/Meth/Heatmap.R";
     print OUT "$cmd\n";
     my $r_rep = `$cmd`;
     print "$class: $r_rep\n";
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
    my %rec_meth_merge;
    foreach my $context(@{$opts_sub->{context}}){
	print "$context\n" if !$opts_sub->{verbose};
	my @sample_list;
	&get_meth_info($class, $opts_sub, \@sample_list, \%rec_meth_merge, $context);

        next if $opts_sub->{merge}; ### if --merge is true, then methylation level of different will be generated in one file rather than one file for each context.  
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_$context.txt";
        open OUT, "+>$output" or die "$!:$output";
	print OUT "\t", join("\t", @sample_list), "\n";
	foreach my $id(keys %rec_meth_merge){
	    #print "$id\n";
	    my @level = @{$rec_meth_merge{$id}->{$context}}{@sample_list};
	    my $level = join("\t", @level);
	    print OUT "$id\t$level\n";
	}
    }
    if($opts_sub->{merge}){ ## if --merge is true, then methylation level of different will be generated in one file rather than one file for each context.
        my $header = &gener_header_merge($class, $opts_sub);

	my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethHeatmap_mer.txt";
	open OUT, "+>$output" or die "$!:$output";
        print OUT "\t$header\n";
        foreach my $id(keys %rec_meth_merge){
	    ### 
            my $lev = &get_meth_lev($class, $opts_sub, $id, \%rec_meth_merge);
	    print OUT "$id\t$lev\n";
        }
        close OUT;
    }
}

sub get_meth_lev{
    my ($class, $opts_sub, $id, $rec_meth_merge) = @_;
    my @meth_lev;
    foreach my $context(@{$opts_sub->{context}}){
        foreach my $sam_info(@{$opts_sub->{sample_list}}){
            my ($meth_file, $sam_name, $region) = split(/,/, $sam_info);
            push @meth_lev, $rec_meth_merge->{$id}->{$context}->{$sam_name};
        }
    }
    return join("\t", @meth_lev);
}

sub gener_header_merge{
    my ($class, $opts_sub) = @_;
    my @header;
    foreach my $context(@{$opts_sub->{context}}){
	foreach my $sam_info(@{$opts_sub->{sample_list}}){
	    my ($meth_file, $sam_name, $region) = split(/,/, $sam_info);
	    push @header, "$sam_name-$context";
        }
    }
    return join("\t", @header);
}

sub get_meth_info{
    my ($class, $opts_sub, $sample_list, $rec_meth_merge, $context) = @_;
    my %rec_tabix;
    foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name, $region) = split(/,/, $sam_info);
        $rec_tabix{$sam_name} = Bio::DB::HTS::Tabix->new( filename => $meth_file) if !exists $rec_tabix{$sam_name};
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
	    if($stt !~ /^\d+$/ || $end !~ /^\d+$/){
	        print "Skip this line: $chr, $stt, $end\n";
		next;
            }
	    my $id = "$chr\_$stt\_$end";
	    if(!exists $rec_region_id{$id}){
		$rec_region_id{$id} = 0;
	    }else{
		$rec_region_id{$id} ++;
	        print "In the region file, there are regions with either duplcate regions or duplicate region names in the 4th column:$chr:$stt-$end.\n";
	        $id = $id.$rec_region_id{$id};
	    }
	    
	    my $level = &get_CT_num($class, $rec_tabix{$sam_name}, $chr, $stt, $end, $context, $opts_sub);
            $rec_meth_merge->{$id} -> {$context} -> {$sam_name} = $level;
	    #print "$id\t$sam_name\n";
	}
    }
}

sub get_CT_num{
    my ($class, $tabix, $chrom, $stt, $end, $context, $opts_sub) = @_;
    my $iter = $tabix->query("$chrom:$stt-$end");
    my ($tot_c_num, $tot_t_num, $tot_cover) = (0, 0, 0);
    my $total_cov_num = 0;
    while ( my $line = $iter->next) {
	#chrC    13      +       3       643     CG      CGG
        my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	if($context eq $tem_context || $context eq "CXX"){
	    ++$total_cov_num;
	    next if ($c_num + $t_num < $opts_sub->{minDepth} || $c_num + $t_num > $opts_sub->{maxDepth});
	    $tot_c_num += $c_num;
	    $tot_t_num += $t_num;
	    ++$tot_cover;
	}
    }
    my $level =  $tot_cover > 0 ? $tot_c_num / ($tot_c_num + $tot_t_num) : "NA";     
    return $level;
}

1;


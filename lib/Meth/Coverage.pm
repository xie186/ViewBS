package Meth::Coverage;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);
use Bio::DB::HTS::Tabix;
use Bio::SeqIO;

sub new{
    my $class     = shift;
    #print "Script:$FindBin::Bin\n";
    return bless {}, $class;  # built in bless function
}

sub drawMeth{
    my ($class, $opts_sub) = @_;
    my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}.tab";
    open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}.sh" or die "$!";
    my $fig = "$opts_sub->{outdir}/$opts_sub->{prefix}.pdf";
    #my $cmd = "R --vanilla --slave --input $output --height $opts_sub->{height} --width $opts_sub->{width} --output $fig < $FindBin::Bin/lib/Meth/Coverage.R";
    my $cmd = "Rscript $FindBin::Bin/lib/Meth/Coverage.R --input $output --height $opts_sub->{height} --width $opts_sub->{width} --output $fig";
    print OUT "$cmd\n";
    close OUT;
    my $r_rep = `$cmd`;
}

sub calMeth{
   my ($class, $opts_sub) = @_;

   ## Process the --sample arguments 
   my $pro_sample = Meth::Sample -> new();
   #print "$opts_sub\t", keys %$opts_sub, "\n";
   $pro_sample -> processArgvSampleCoverage($opts_sub);
   
   ## Start calculate methylation information for target context
   &generTab($class, $opts_sub);
}

sub generTab{
    my ($class, $opts_sub) = @_;
    print "Start reading the methylation file\n" if !$opts_sub->{verbose};

    my %rec_meth;
    # record read depth at differet coverage for CG context
    my %rec_meth_tot;
    my %rec_meth_context;
    my $max_depth = 0;
    my $min_depth = 10000;
     
    my @sample_list; 
    foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name) = split(/,/, $sam_info);
        push @sample_list, $sam_name;
	open METH, "gzip -cd $meth_file |" or die "$!: $meth_file\n";
	while(my $line = <METH>){
	    my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	    my $depth = $c_num + $t_num;
	    $rec_meth{$sam_name} -> {$tem_context} -> {$depth} ++;
	    $rec_meth_tot{$depth} ++ if $tem_context eq "CG";
	    $rec_meth_context{$tem_context} ++;
	    $min_depth = $depth if $depth < $min_depth;
	    $max_depth = $depth if $depth > $max_depth;
	}
	close METH;
    }

    if($min_depth ==0){
        print "Even cytosine sites with 0 read covered was outputted so we are going to use the total row numbers for CG (CHG/CHH) as Denominator\n";
        foreach my $temcontxt(keys %rec_meth_context){
            $opts_sub->{"ref_C"}->{$temcontxt} = $rec_meth_context{$temcontxt};
            #print "$temcontxt\t$rec_meth_context{$temcontxt}\n";
        }
    }else{
        print "Cytosine sites withour any reads seems not to be outputed so we are going to calculate the number of CG (CHG/CHH) as Denominator\n";
        &cal_CG_num($class, $opts_sub);
    }
    my $num_sam = keys %rec_meth;  # how many samples
    my ($max_depth_rep) = &determine_max_depth($class, $max_depth, $num_sam, \%rec_meth_tot, $opts_sub);   ## to detemine when should we stop
  
    open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}.tab" or die "$!";
    print OUT "Sample\tContext\tDepth\tPercentage\n";
    ###  @sample_list with order
    foreach my $sam_name(@sample_list){
	foreach my $tem_context(sort keys %rec_meth_context){
	    for(my $i = 1; $i <= $max_depth_rep; ++$i){
		my $num_cov = 0;
		for(my $j = $i; $j <= $max_depth; ++$j){
		    $num_cov += $rec_meth{$sam_name} -> {$tem_context} -> {$j} if exists $rec_meth{$sam_name} -> {$tem_context} -> {$j};
		}
                print "$sam_name\t$num_cov/$opts_sub->{ref_C}->{$tem_context}\n";
		my $perc = 100* $num_cov/$opts_sub->{"ref_C"}->{$tem_context};
		print OUT "$sam_name\t$tem_context\t$i\t$perc\n";
	    }
        }
    }
    close OUT;
}

#### Detemine the maximum depth
sub determine_max_depth{
    my ($class, $max_depth, $num_sam, $rec_meth_tot, $opts_sub) = @_;
    my $max_depth_rep;

    ## Start from 1 to MAX_DEPTH
    for(my $i = 1; $i <= $max_depth; ++$i){
        my $num_cov = 0;
        for(my $j = $i; $j <= $max_depth; ++$j){
            $num_cov += $$rec_meth_tot{$j} if exists $$rec_meth_tot{$j};
        }
        ## When the percentage of cytosines that can be covered is lower that 0.1. We stop there. 
        if($num_cov/($opts_sub->{"ref_C"}->{"CG"} * $num_sam) < 0.1){
            $max_depth_rep = $i;
            last;
        }
    }
    print "Maximum depth that will be calculated is $max_depth_rep\n";
    return $max_depth_rep;
}

### calculate the genomic 
sub cal_CG_num{
    my ($class, $opts_sub) = @_; 
    my $ref = $opts_sub->{reference};
    my $seq_in = Bio::SeqIO->new( -format => 'fasta',
                              -file   => $ref,
                             );
    
    print "$class: start to calculate CG, CHG and CHH number\n";
    print "ID: C_G_number\tCG_number\tCHG_number\tCHH_number\n";
    while ( my $seq = $seq_in->next_seq() ) {
        my $id = $seq->id;
	my $seq = $seq->seq;
	my $num_c_g = $seq =~ tr/CGcg/CGcg/; ### cal total C and G number
	my $num_cg = $seq =~ s/CG/CG/gi;	### calculate CG number
	$num_cg = $num_cg * 2;
	my $num_cag = $seq =~ s/CAG/CAG/gi;
	my $num_ctg = $seq =~ s/CTG/CTG/gi;
	my $num_ccg = $seq =~ s/CCG/CCG/gi;
	my $num_chg = 2* ($num_cag + $num_ctg + $num_ccg);
	my $num_chh = $num_c_g - $num_cg - $num_chg;
	print "$id:$num_c_g\t$num_cg\t$num_chg\t$num_chh\n";
	$opts_sub->{"ref_C"} -> {"C_G"} += $num_c_g;
        $opts_sub->{"ref_C"} -> {"CG"}  += $num_cg;
	$opts_sub->{"ref_C"} -> {"CHG"} += $num_chg;
	$opts_sub->{"ref_C"} -> {"CHH"} += $num_chh;
    }
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


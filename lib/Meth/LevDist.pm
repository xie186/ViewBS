package Meth::LevDist;

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
    my $cmd = "R --vanilla --slave --input $output --percentage TRUE --height $opts_sub->{height} --width $opts_sub->{width} --output $fig < $FindBin::Bin/lib/Meth/LevDist.R";
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

    my $BINNUM = int (1 / $opts_sub->{binMethLev});
    
    my %rec_meth;
    my %rec_meth_tot;
    my %rec_meth_context;
    if(!$opts_sub->{region}){ 
        foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
            my ($meth_file, $sam_name) = split(/,/, $sam_info);
    	    open METH, "zcat $meth_file |" or die "$!: $meth_file\n";
 	    while(my $line = <METH>){
	        my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
   	        my $depth = $c_num + $t_num;
	        next if ($depth < $opts_sub->{minDepth} || $depth > $opts_sub->{maxDepth});
	        my $lev = $c_num / $depth;
	        my $bin_num = ($lev ==1) ? $BINNUM -1 : int ( $lev / $opts_sub->{binMethLev}); 
	        $rec_meth{$sam_name}-> {$tem_context}-> {$bin_num} ++;
	        $rec_meth_tot{$sam_name} -> {$tem_context} ++;
	        $rec_meth_context{$tem_context} ++;
	    }
	    close METH;
        }
    }else{
	my @tabix;
	for(my $i = 0; $i < @{$opts_sub->{sample_list}}; ++$i){
	    my $sam_info = ${$opts_sub->{sample_list}}[$i];
	    my ($meth_file, $sam_name) = split(/,/, $sam_info);
	    my $tabix = Bio::DB::HTS::Tabix->new(filename => $meth_file);
            &get_CT_num($class, $opts_sub, $tabix, $sam_name, \%rec_meth, \%rec_meth_tot, \%rec_meth_context); 
           
        }
    }

    open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}.tab" or die "$!";
    print OUT "Sample\tContext\tMethLevBinMidPoint\tNumber\tPercentage\n";
    foreach my $sam_name(keys %rec_meth){
	foreach my $tem_context(sort keys %rec_meth_context){
	    for(my $i = 1; $i <= $BINNUM; ++$i){
		#print "$sam_name, $tem_context, $i\n";
		my $num = exists $rec_meth{$sam_name} -> {$tem_context} -> {$i-1} ? $rec_meth{$sam_name} -> {$tem_context} -> {$i-1} : 0;
		my $tot_num = $rec_meth_tot{$sam_name} -> {$tem_context};
		my $perc = 100* $num/$tot_num;
		my $bin = $opts_sub->{binMethLev} * $i - $opts_sub->{binMethLev}/2;
		print OUT "$sam_name\t$tem_context\t$bin\t$num\t$perc\n";
	    }
        }
    }
    close OUT;
}

sub get_CT_num{
    my ($class, $opts_sub, $tabix, $sam_name, $rec_meth, $rec_meth_tot, $rec_meth_context) = @_;
    my $BINNUM = int (1 / $opts_sub->{binMethLev});
    my ($num_qualify, $num_remove) = (0, 0);
    open REG, $opts_sub->{region} or die "$!";
    while(my $reg = <REG>){
	chomp $reg;
	my ($chrom, $stt, $end) = split(/\s+/, $reg);
        my ($tot_c, $tot_t) = (0, 0);
	my %rec_context;
	my %rec_lev;
	my %rec_number;
	my $iter = $tabix->query("$chrom:$stt-$end");
	while ( my $line = $iter->next) {
	    my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	    my $depth = $c_num + $t_num;
	    if($depth < $opts_sub->{minDepth} || $depth > $opts_sub->{maxDepth}){
	        $num_remove ++;
            }else{
		$num_qualify ++;
            }
            next if ($depth < $opts_sub->{minDepth} || $depth > $opts_sub->{maxDepth});
	    $rec_context{$tem_context} ++;
   
            if(!$opts_sub->{methodAverage}){  ## calculate weighted average methylation level
	        ${$rec_number{$tem_context}}[0] += $c_num;
	        ${$rec_number{$tem_context}}[1] += $c_num + $t_num;
            }else{  ## calculate average methylation level
                my $lev = $c_num / $depth;
		$rec_lev{$tem_context} += $lev;
            }
	}
	foreach my $tem_context(keys %rec_context){
	    my $lev = !$opts_sub->{methodAverage}  ? ${$rec_number{$tem_context}}[0] / ${$rec_number{$tem_context}}[1] : $rec_lev{$tem_context} / $rec_context{$tem_context};  ## weighted or normal
	    my $bin_num = ($lev ==1) ? $BINNUM -1 : int ( $lev / $opts_sub->{binMethLev});
	    $rec_meth->{$sam_name}-> {$tem_context}-> {$bin_num} ++;
	    $rec_meth_context->{$tem_context} ++;
	    $rec_meth_tot->{$sam_name} -> {$tem_context} ++;
	    print "Region\t$tem_context\t$reg\t$lev\t$sam_name\t$opts_sub->{methodAverage}\n";
	}
    }
    print "Number of sites used: $num_qualify; Not used: $num_remove\n";
}

1;


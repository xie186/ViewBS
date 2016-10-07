package Meth::Geno;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);
use Bio::DB::HTS::Tabix;

#use Meth::Sample;

my %rec_geno_len;

sub new{
    my $class     = shift;
    #print "Script:$FindBin::Bin\n";
    return bless {}, $class;  # built in bless function
}

sub drawMeth{
    my ($class, $opts_sub) = @_;
    foreach my $context(@{$opts_sub->{context}}){
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethGeno_$context.txt";
        my $cmd_out = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethGeno_$context.sh";
	my $fig = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethGeno_$context.pdf";
	my $cmd = "R --vanilla --slave --input $output --output $fig < $FindBin::Bin/lib/Meth/Geno.R";
	open OUT, "+>$cmd_out" or die "$!:$cmd_out";
        my $r_rep = `$cmd`;
	print OUT "$cmd\n";
	close OUT;
        print "$class: $r_rep\n";
    }
}

sub calMeth{
   my ($class, $opts_sub) = @_;
   &genomeLength($class, $opts_sub);

   ## Process the --sample arguments 
   my $pro_sample = Meth::Sample -> new();
   #print "$opts_sub\t", keys %$opts_sub, "\n";
   $pro_sample -> processArgvSampleGenome($opts_sub);
   
   ## Start calculate methylation information for target context
   &generTab($class, $opts_sub);
}

sub generTab{
    my ($class, $opts_sub) = @_;
    my ($win, $step) = ($opts_sub->{win}, $opts_sub->{step});
    print "Start calculate methylation information for target context\n" if !$opts_sub->{verbose};
    foreach my $context(@{$opts_sub->{context}}){
	print "$context\n" if !$opts_sub->{verbose};
        my $output = "$opts_sub->{outdir}/$opts_sub->{prefix}_MethGeno_$context.txt";
        open OUT, "+>$output" or die "$!:$output";
	print OUT "chr\tstt\tend\tsample_name\tC_number\tT_number\tMethylation_level\n";
	foreach(@{$opts_sub->{sample_list}}){
		my ($meth_file, $sam_name) = split(/,/, $_);
	        foreach my $chrom(sort keys %rec_geno_len){
		    for(my $i = 1; $i <= $rec_geno_len{$chrom}/$step-1; ++$i){
                         my ($stt, $end) = (($i-1) * $step + 1, ($i-1) * $step + $win);
                         if($end > $rec_geno_len{$chrom} && ($end - $rec_geno_len{$chrom})/$win < 0.5){
                            next;  #
                         }
		         my ($c_num, $t_num) = &get_CT_num($class, $meth_file, $chrom, $stt, $end, $context, $opts_sub);
                         my $level = $c_num/($c_num + $t_num + 0.000000001);
                         print OUT "$chrom\t$stt\t$end\t$sam_name\t$c_num\t$t_num\t$level\n";
                    }
	    }
        }
    }
}

sub get_CT_num{
    my ($class, $meth_file, $chrom, $stt, $end, $context, $opts_sub) = @_;
    my $tabix = Bio::DB::HTS::Tabix->new( filename => $meth_file);
    my $iter = $tabix->query("$chrom:$stt-$end");
    my ($tot_c_num, $tot_t_num) = (0, 0);
    while ( my $line = $iter->next) {
	#chrC    13      +       3       643     CG      CGG
        my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	if($context eq $tem_context || $context eq "CXX"){
	    next if ($c_num + $t_num < $opts_sub->{minDepth} || $c_num + $t_num > $opts_sub->{maxDepth});
	    $tot_c_num += $c_num;
	    $tot_t_num += $t_num;
	}
    }
    return ($tot_c_num, $tot_t_num);
}


sub genomeLength{
    my ($class, $opts_sub) = @_;
    my $genome_len = $opts_sub -> {genomeLength};
    open GENO, $genome_len or die "$!";
    print "Start reading the genome file:\n" if !$opts_sub->{verbose};
    while(my $line = <GENO>){
	chomp $line;
	my ($chr, $len) = split(/\s+/, $line);
	print "$chr\t$len\n";
	$rec_geno_len{$chr} = $len;
    }
    close GENO;
}
1;


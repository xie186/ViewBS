#!/usr/bin/perl
use strict;
use warnings;

die usage() unless @ARGV == 3;
my ($bsseek2_meth, $min_depth, $out) = @ARGV;

my $stat_unkown_context = 0;
my $tot_num = 0;

print "\nINFO: Start to read BS seeker2 CGmap file: \n";
open OUT, "+>$out" or die "$!: $out\n";
open METH, $bsseek2_meth or die "$!: $bsseek2_meth\n";
while(my $line = <METH>){
    chomp $line;
    
    #chr1    C       3001631 CG      CG      1.0     5       5
    my ($chrom, $CorG, $start, $context, $seq, $methylation_level, $num_c, $depth) = split(/\s+/, $line);
    #chr1    552     552     CHH:4   0.25    -
    #$context =~ s/CpG/CG/g;
    #my $num_c = int ($depth * $methylation_level + 0.5);
    my $num_t = $depth - $num_c;
    #<chromosome> <position> <strand> <count methylated> <count unmethylated> <C-context> <trinucleotide context>
    next if $depth < $min_depth;
    my $strand = $CorG eq "C" ? "+" : "-";
    print OUT "$chrom\t$start\t$strand\t$num_c\t$num_t\t$context\t$seq\n";

    ++$tot_num;
    if($context =~ /-/){
       $stat_unkown_context++; 
    }
}
close METH;
close OUT;

print "INFO: There are $stat_unkown_context sites (out of $tot_num) without explicit sequence context because the neighbouring sites are Ns.\n\n";

sub usage{
    my $die = <<OUT;

Usage:

    perl bsseeker2bismark.pl <BSseeker2 CGmap> <Minimum depth> <Output file>

OUT
}

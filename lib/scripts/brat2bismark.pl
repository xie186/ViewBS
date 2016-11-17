#!/usr/bin/perl
use strict;
use warnings;

die usage() unless @ARGV == 3;
my ($brat_meth, $min_depth, $out) = @ARGV;

open OUT, "+>$out" or die "$!: $out\n";
open METH, $brat_meth or die "$!: $brat_meth\n";
while(my $line = <METH>){
    chomp $line;
    my ($chrom, $start, $end, $total, $methylation_level, $strand) = split(/\t/, $line);
    my ($context, $depth) = split(/:/, $total);
    #chr1    552     552     CHH:4   0.25    -
    $context =~ s/CpG/CG/g;
    my $num_c = int ($depth * $methylation_level + 0.5);
    my $num_t = $depth - $num_c;
    #<chromosome> <position> <strand> <count methylated> <count unmethylated> <C-context> <trinucleotide context>
    next if $depth < $min_depth;
    print OUT "$chrom\t$start\t$strand\t$num_c\t$num_t\t$context\t$context\n";
}
close METH;
close OUT;

sub usage{
    my $die = <<OUT;

Usage:
    perl brat2bismark.pl <brat methlation output> <Minimum depth> <Output file>

OUT
}

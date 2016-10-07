package Meth::BisNonConvRate;

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
    my $cmd = "R --vanilla --slave --input $output --output $fig < $FindBin::Bin/lib/Meth/BisNonConvRate.R";
    print OUT "$cmd\n";
    close OUT;
    my $r_rep = `$cmd`;
}

my $TOTC_DEP = "Cytosine_num";
my $TOT_DEP  = "ToalCT_num";

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
    my %rec_meth_context;
    my %rec_meth_tot; 

    my @sample_list; 
    foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name) = split(/,/, $sam_info);
        push @sample_list, $sam_name;
	open METH, "zcat $meth_file |" or die "$!: $meth_file\n";
	while(my $line = <METH>){
	    my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	    my $depth = $c_num + $t_num;
	    next if ($depth < $opts_sub->{minDepth} || $depth > $opts_sub->{maxDepth});
	    $rec_meth{$sam_name} -> {$TOTC_DEP} += $c_num;
	    $rec_meth{$sam_name} -> {$TOT_DEP}  += $depth;
	}
	close METH;
    }

    open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}.tab" or die "$!";
    
    print OUT "Sample\tBisNonConvRate\n";
    foreach my $sam_name(keys %rec_meth){
	my $c_num = $rec_meth{$sam_name} -> {$TOTC_DEP};
	my $tot_num = $rec_meth{$sam_name}-> {$TOT_DEP};
	my $meth_lev = sprintf("%.3f", $c_num/$tot_num);   ## Non-conversion rate here
	print OUT "$sam_name\t$meth_lev\n";
    }
    close OUT;
}

1;


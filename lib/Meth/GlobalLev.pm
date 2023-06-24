package Meth::GlobalLev;

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
    #my $cmd = "R --vanilla --slave --input $output --height $opts_sub->{height} --width $opts_sub->{width} --output $fig < $FindBin::Bin/lib/Meth/GlobalLev.R";
    my $cmd = "Rscript $FindBin::Bin/lib/Meth/GlobalLev.R --input $output --height $opts_sub->{height} --width $opts_sub->{width} --output $fig";
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

    my %rec_meth;            #record total number of Cs and depth
    my %rec_meth_tot_lev;    #sum of the methylation level for all cytosines
    my %rec_meth_tot;        #Number of sites for each context.
    my %rec_meth_context;
    #print "@{$opts_sub->{sample_list}}\n"; 
    my @sample_list; 
    foreach my $sam_info(@{$opts_sub->{sample_list}}){   ## sample information: meth_file,sample,region
        my ($meth_file, $sam_name) = split(/,/, $sam_info);
        push @sample_list, $sam_name;
	open METH, "gzip -cd $meth_file |" or die "$!: $meth_file\n";
	while(my $line = <METH>){
	    my ($chr, $pos, $strand, $c_num, $t_num, $tem_context, $seq) = split(/\t/, $line);
	    my $depth = $c_num + $t_num;
	    next if ($depth < $opts_sub->{minDepth} || $depth > $opts_sub->{maxDepth});
	    #my $lev = $c_num / $depth;
	    #my $bin_num = ($lev ==1) ? $BINNUM -1 : int ( $lev / $opts_sub->{binMethLev}); 
	    $rec_meth{$sam_name}-> {$tem_context} -> {$TOTC_DEP} += $c_num;
	    $rec_meth{$sam_name}-> {$tem_context} -> {$TOT_DEP}  += $depth;
	    $rec_meth_tot{$sam_name} -> {$tem_context} ++;
	    $rec_meth_tot_lev{$sam_name} -> {$tem_context} += $c_num/$depth;   
	    $rec_meth_context{$tem_context} ++;
	}
	close METH;
    }

    open OUT, "+>$opts_sub->{outdir}/$opts_sub->{prefix}.tab" or die "$!";
    my @context = sort keys %rec_meth_context;
    for(@context){chomp $_};
    print OUT "Sample\t", join("\t", @context), "\n";
    ## keys %rec_meth ====> @sample_list 
    ## because we want to keep the order of the input samples
    foreach my $sam_name(@sample_list){
	my @meth_lev;
	foreach my $tem_context(sort keys %rec_meth_context){
	    my $c_num = $rec_meth{$sam_name}-> {$tem_context} -> {$TOTC_DEP};
	    my $tot_num = $rec_meth{$sam_name}-> {$tem_context} -> {$TOT_DEP};

	    my $total_sites_num = $rec_meth_tot{$sam_name} -> {$tem_context};
            my $total_meth_lev  = $rec_meth_tot_lev{$sam_name} -> {$tem_context};
	    my $meth_lev = $opts_sub->{methodAverage} ? sprintf("%.3f", $total_meth_lev/$total_sites_num) : sprintf("%.3f", $c_num/$tot_num);
	    push @meth_lev, $meth_lev;
        }
	print OUT "$sam_name\t", join("\t", @meth_lev), "\n";
    }
    close OUT;
}

1;


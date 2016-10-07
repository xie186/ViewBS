package SubCmd::MethCoverage;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);


use Meth::Coverage;

## class, $opts, $opts_sub
sub new{
    my $class     = shift;
    return bless {}, $class;  # built in bless function 
}

sub check_para_sub{

    my ($class, $opts_sub, $opts) = @_;

    if($opts->{help}){
        pod2usage(-exitval => 0, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_MethCoverage.txt");
        exit 0;
    }

    if(!&check_para($class, $opts_sub)){
        print "Please provide parameters\n";
        pod2usage(-exitval => 1, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_MethCoverage.txt");
        exit 0;
    }

    my $exit_code = 0;

    if(!@{$opts_sub->{"sample"}}){
	print "Please provide --sample!\n";
	++$exit_code; #exit 0;
    }
    print "Sample(s): @{$opts_sub->{"sample"}}\n";

    if(!$opts_sub->{"reference"}){
	print "Please provide --reference!\n"; ## Reference will be needed to calculate the total number of Cs, CGs, CHGs, and CHHs.
        ++$exit_code; #exit 0;
    }else{
	if(!-e $opts_sub->{"reference"}){
	    print "Reference file: $opts_sub->{reference} not exists. Please check!\n";
	    ++$exit_code; #exit 0;    
	}
	#print "xxx: $opts_sub->{reference}: xx\n";
	$opts_sub->{"reference"} = abs_path $opts_sub->{"reference"};
    }


    ## output directory  
    if(!$opts_sub->{"outdir"}){
        $opts_sub->{"outdir"} = abs_path "./";
    }else{
        if(-e  $opts_sub->{"outdir"} && !-d $opts_sub->{"outdir"}){
            print "File $opts_sub->{outdir} already exists. Please provide a new directory name.\n";
            ++$exit_code; #exit 0;
        }
        `mkdir $opts_sub->{"outdir"}` if !-d $opts_sub->{outdir};
        $opts_sub->{"outdir"} = abs_path $opts_sub->{"outdir"};
    }
    `mkdir $opts_sub->{"outdir"}` if !-d $opts_sub->{outdir};
    print "Output directory is: $opts_sub->{outdir}\n";
 
    ### 
    if(!$opts_sub->{"prefix"}){
        $opts_sub->{"prefix"} = "MethCoverage";
    }

#    if(!@{$opts_sub->{"context"}}){
#        push @{$opts_sub->{"context"}}, "CG";
#    }
#
#    if(!$opts_sub->{"minDepth"}){
#        $opts_sub->{"minDepth"} = 3;
#    }
#
#    if(!$opts_sub->{"maxDepth"}){
#        $opts_sub->{"maxDepth"} = 400;
#    }

 
    if($exit_code > 0){
        exit 0;
    }else{
	return "TRUE";
    }
}

sub check_para{
    my ($class, $opts) = @_;
    my $def = 0;
    my $num = 0;
    foreach(values %$opts){
        if(defined $_){
	    ## if one argument can be used multiple times. Even if you don't provide the argument, the value would be an reference to an array.
	    if(!/ARRAY/ || @{$_}){
		#print "Value\t@{$_}\n";
                $def ++;
	    }
        }
        ++$num;
    }
    if($def == 0){
        return 0;   ## No parameter was provide!
    }else{
        return 1;
    }
}


sub run_methCoverage{ 
    my ($class, $opts_sub) = @_;
    my $meth_geno = Meth::Coverage->new(); 
    $meth_geno -> calMeth($opts_sub);
    $meth_geno -> drawMeth($opts_sub);
}

1;

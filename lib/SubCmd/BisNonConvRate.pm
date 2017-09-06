package SubCmd::BisNonConvRate;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);

use Meth::BisNonConvRate;
use SubCmd::CommonArgument;
## class, $opts, $opts_sub
sub new{
    my $class     = shift;
    return bless {}, $class;  # built in bless function 
}

sub check_para_sub{

    my ($class, $opts_sub, $opts) = @_;

    if($opts->{help}){
        pod2usage(-exitval => 0, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_BisNonConvRate.txt");
        exit 0;
    }

    if(!&check_para($class, $opts_sub)){
        print "Please provide parameters\n";
        pod2usage(-exitval => 1, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_BisNonConvRate.txt");
        exit 0;
    }

    my $exit_code = 0;

    if(!@{$opts_sub->{"sample"}}){
	print "Please provide --sample!\n";
	++$exit_code; #exit 0;
    }
   
    #if(!@{$opts_sub->{"context"}}){
    #    #push @{$opts_sub->{"context"}}, "CXX";
    #    print "CXX\n";
    #	exit 1;
    #}

    if(!@{$opts_sub->{"chrom"}}){
        print "\nError: Please provide --chrom for the chromosome IDs that you want to use to estimate the non-conversion rate!\n\n";
        ++$exit_code; #exit 0;
    }
    #print "After sample: $exit_code\n";

    ### Common arguments
    my $cm_arg = SubCmd::CommonArgument -> new();
    my $exit_num_return = $cm_arg->common_argument($opts_sub, $exit_code);
    $exit_code += $exit_num_return;     
 
    if($exit_code > 0){
	print "Exit code is larger that 0. Please check the error message and help message\n";
	pod2usage(-exitval => 1, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_BisNonConvRate.txt");
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


sub run_bisNonConvRate{ 
    my ($class, $opts_sub) = @_;
    my $meth_geno = Meth::BisNonConvRate->new();
    print "Class name: $class\n"; 
    $meth_geno -> calMeth($opts_sub);
    $meth_geno -> drawMeth($opts_sub);
}

1;

package SubCmd::MethCoverage;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);


use Meth::Coverage;
use SubCmd::CommonArgument;
## class, $opts, $opts_sub
sub new{
    my $class     = shift;
    return bless {}, $class;  # built in bless function 
}

sub check_para_sub{

    my ($class, $opts_sub, $opts) = @_;
    #print "$opts_sub\t", join("\n", keys %$opts_sub), "\n";
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
    #print "xxx\tcc${$opts_sub->{sample}}[0]dd\n";
    if(!@{$opts_sub->{"sample"}} || length(${$opts_sub->{sample}}[0]) == 0){
        #print "xxx\t@{$opts_sub->{sample}}\n";
	print "Please provide --sample!\n";
	++$exit_code; #exit 0;
    }
    print "Sample(s): @{$opts_sub->{sample}}\n";

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

    #### Common arguments
    my $cm_arg = SubCmd::CommonArgument -> new();
    my $exit_num_return = $cm_arg -> common_argument($opts_sub, $exit_code);
    $exit_code += $exit_num_return;
    
 
    if($exit_code > 0){
	pod2usage(-exitval => 1, -verbose => 2, -input => "$FindBin::Bin/doc/pod4help_MethCoverage.txt");
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

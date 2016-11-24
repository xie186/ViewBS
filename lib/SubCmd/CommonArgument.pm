package SubCmd::CommonArgument;

use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);

use Meth::OverRegion;

## class, $opts, $opts_sub
sub new{
    my $class     = shift;
    return bless {}, $class;  # built in bless function 
}

sub common_argument{

    my ($class, $opts_sub, $opts) = @_;
   
    my $exit_code = 0;
    #### This is manly for MethOverRegion
    #flank regions
    if(!$opts_sub->{"flank"}){
        $opts_sub->{"flank"} = 2000;
    }
   
    if($opts_sub->{flank} % 1000 != 0){
	print "The flank region size should be able to divivied by 1000 with no remainder\n";
	++$exit_code; #exit 0;
    }
    
    # binLength
    if(!$opts_sub->{"binLength"}){
        $opts_sub->{"binLength"} = 100;
    }

    if($opts_sub->{flank} % $opts_sub->{"binLength"} != 0){
        print "The flank region size should be able to divivied by length of bin with no remainder\n";
        ++$exit_code; #exit 0;
    }
    
    # binNumber
    if(!$opts_sub->{"binNumber"}){
        $opts_sub->{"binNumber"} = 60;
    }
   
    # minLength
    if(!$opts_sub->{"minLength"}){
        $opts_sub->{"minLength"} = 300;
    }

    # maxLength
    if(!$opts_sub->{"maxLength"}){
        $opts_sub->{"maxLength"} = 5000000;
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
    
    if(!$opts_sub->{"prefix"}){
	my ($sub_cmd) = @{$opts->{"subcommand"}};
	$opts_sub->{prefix} = $sub_cmd;
	print "Default output prefix is used: $sub_cmd\n"; 
    }else{
	print "Output prefix: $opts_sub->{prefix}\n";
    }
    
    if(!@{$opts_sub->{"context"}}){
        push @{$opts_sub->{"context"}}, "CG";
    }

    if(!$opts_sub->{"minDepth"}){
        $opts_sub->{"minDepth"} = 5;
    }

    if(!$opts_sub->{"maxDepth"}){
        $opts_sub->{"maxDepth"} = 100000;
    }

    ##### lib/SubCmd/MethGeno.pm
    #window size
    if(!$opts_sub->{"win"}){
        $opts_sub->{"win"} = 500000;
    }
    ## step size
    if(!$opts_sub->{"step"}){
        $opts_sub->{"step"} = 500000;
    } 
   
    ##### lib/SubCmd/MethHeatmap.pm 
    if(!$opts_sub->{"cluster_rows"}){
        $opts_sub->{"cluster_rows"} = "TRUE";
    }else{
        $opts_sub->{"cluster_rows"} = uc $opts_sub->{"cluster_rows"};
        my $value = $opts_sub->{"cluster_rows"};
        if($value ne "TRUE" && $value ne "FALSE"){
            print "--cluster_rows should be either FALSE or TRUE. Please check\n";
            ++$exit_code;
        }
    }

    if(!$opts_sub->{"cluster_cols"}){
        $opts_sub->{"cluster_cols"} = "FALSE";
    }else{
        $opts_sub->{"cluster_cols"} = uc $opts_sub->{"cluster_cols"};
        my $value = $opts_sub->{"cluster_cols"};
        if($value ne "TRUE" && $value ne "FALSE"){
            print "--cluster_cols should be either FALSE or TRUE. Please check\n";
            ++$exit_code;
        }
    }

    # random_region 
    if(!$opts_sub->{random_region}){
        $opts_sub->{random_region} = 2000;
    }

    ##### For generating figures.  ######
   
    if(!$opts_sub->{width}){
        $opts_sub->{width} = 10;
    }
   
    if(!$opts_sub->{height}){
        $opts_sub->{height} = 10;
    }
  
    if(!$opts_sub->{width2}){
        $opts_sub->{width2} = 10;
    }

    if(!$opts_sub->{height2}){
        $opts_sub->{height2} = 10;
    }
   
    return $exit_code;
}

1;

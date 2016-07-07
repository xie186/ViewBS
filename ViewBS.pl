#!/usr/bin/perl -w

#################################################################
# This toolkits are developped to analyze and visualize bisulfite
# sequencing data.                                          
#################################################################

use strict;
use Getopt::Long::Subcommand;
use Pod::Usage;
use File::Basename;
use Cwd qw(abs_path);
use Benchmark;
my $t_stt = Benchmark->new;

#use FindBin;
#use lib "$FindBin::Bin";

$| = 1; # Do not buffer output 

#Self written libraries.
## Package written by Shanshan Huang
use lib dirname(abs_path $0) . '/lib';  #include seft written packages in @INC.
use SubCmd::MethGeno;
use Meth::Geno;
use Meth::Sample;
use SubCmd::MethOverRegion;
use SubCmd::MethHeatmap;
use Meth::Heatmap; 
my $main_path = dirname(abs_path $0);

my %opts;   ##

### Return hash structure, with these keys: success, subcommand (array of str) by GetOptions
my $resOpt = &processCMD();

&check_parameter();
my %opts_subcmd;
sub check_parameter{
     
    # check if there is a subcommand given.
    pod2usage(-exitval => 1, -verbose => 2, -input => "$main_path/doc/doc4main_help.txt") if (!exists $resOpt->{"subcommand"});

    #get the subcommand name success and subcommand
    my ($sub_cmd) = @{$resOpt->{"subcommand"}}; 
    print "Subcommand: $sub_cmd\n";

    if($sub_cmd eq "MethGeno"){
	#print "Keys: ", join("\t", keys %opts_onestop), "\n";
        #print "Vals: ", join("\t", values %opts_onestop), "\n";
	#&check_para_onestopsd();
	print "@{$opts_subcmd{sample}}\n";
	my $check_para = SubCmd::MethGeno -> new();
	#print %opts_onestop, "\n";
	my $status = $check_para -> check_para_sub(\%opts_subcmd, \%opts);
        if($status){
	    $check_para -> run_methGeno(\%opts_subcmd, \%opts);
        }
    }
    if($sub_cmd eq "MethOverRegion"){
	print "@{$opts_subcmd{sample}}\n";
	my $check_para = SubCmd::MethOverRegion -> new();
	my $status = $check_para -> check_para_sub(\%opts_subcmd, \%opts);
	$check_para -> run_methOverRegion(\%opts_subcmd, \%opts);
    }
    if($sub_cmd eq "MethHeatmap"){
        print "@{$opts_subcmd{sample}}\n";
        my $check_para = SubCmd::MethHeatmap -> new();
        my $status = $check_para -> check_para_sub(\%opts_subcmd, \%opts);
        $check_para -> run_methHeatmap(\%opts_subcmd, \%opts);
    } 
}

my $t_end = Benchmark->new;
my $td = timediff($t_end, $t_stt);
print "Running time:",timestr($td),"\n";

sub processCMD{
    $resOpt = GetOptions(   # Return hash structure, with these keys: success, subcommand (array of str)
        ## common options recognized by all subcommands
        options => {
	    'help|h|?'    => \$opts{help},
	    'version|v'   => \$opts{version},
	    'verbose'     => \$opts{verbose},
	    'outdir:s'    => \$opts_subcmd{outdir},
	    'minDepth:s'  => \$opts_subcmd{minDepth},
	    'maxDepth:s'  => \$opts_subcmd{maxDepth},
	    'context:s'   => \@{$opts_subcmd{context}},    # context
         },
         subcommands => {
             MethGeno => {       
                 summary => 'Plot methylation information across the chromsome.',
                 # subcommand-specific options
                 options => {
		    # mandatory arguments
        	    'genomeLength|g:s'       => \$opts_subcmd{genomeLength},
		    'sample:s'               => \@{$opts_subcmd{sample}},
		    'prefix:s'               => \$opts_subcmd{prefix}, 
		    # Optional arguments
		    'win:i'                  => \$opts_subcmd{win},
                    'step:i'                 => \$opts_subcmd{step},       
		    'minLength:s'            => \$opts_subcmd{minLength},  # cutoff for minimum length of chromosome.
		    'split:s'                => \$opts_subcmd{"split"},      # 
                 }
            },
	    MethOverRegion => {
		summary => 'Plot average distribution of methylation over your defined regions.',
		options => {
		    # mandatory arguments
		    'regions:s'              => \$opts_subcmd{region},
                    'sample:s'               => \@{$opts_subcmd{sample}},
                    'prefix:s'               => \$opts_subcmd{prefix},
		    # Optional arguments
		    'flank:i'                => \$opts_subcmd{flank},
		    'binLength:i'	     => \$opts_subcmd{binLength},
		    'binNumber:i'            => \$opts_subcmd{binNumber},
	            'minLength:i'	     => \$opts_subcmd{minLength},
		    'maxLength:i'            => \$opts_subcmd{maxLength},
		}
	    },
	    MethHeatmap => {
                summary => 'Generate heatmap for a given regions.',
                options => {
                    # mandatory arguments
                    'regions:s'              => \$opts_subcmd{region},
                    'sample:s'               => \@{$opts_subcmd{sample}},
                    'prefix:s'               => \$opts_subcmd{prefix},
                    # Optional arguments
                }
            }
        }
    );
}


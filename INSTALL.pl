#!/usr/bin/perl -w

# Shaojun Xie <xie186@purdue.edu>

use strict;
use File::Basename;
use Cwd qw(abs_path);

my @package_list=("Getopt::Long::Subcommand", "Bio::DB::HTS::Tabix", "Bio::SeqIO"); #, "HAHA::VIEWBS");


## Check Perl Version
##my $perl_v_help = `perl -v`;
#my ($perl_num, $perl_version, $perl_subv) = $perl_v_help =~ /v(\d+)\.(\d+)\.(\d+)/;
#print "$perl_num, $perl_version, $perl_subv\n";

if ($] >= 5.014004){
    print "PASSED: Perl version($] >= 5.14.4).\n";
}else{
    print "Error: Perl version needs to be above 5.14.4!\n";
    exit 1;
}

## Check htslib install
my $path = checkToolExists("tabix");
if($path){
    print "PASSED: tabix found in $path.\n";
}else{
    print "tabix not found:\n";
    print "Please install htslib (https://github.com/samtools/htslib) first. Then run this script again. If you already installed htslib, please make sure htslib is in your \$PATH\n";
    exit 1;
}

## Check cpanm 
my $dir = dirname(abs_path $0);
my $CPANM = "$dir/ext_tools/cpanm";
if(!-e $CPANM){
    print "Error: cpanm not found at $CPANM. Please check.\n";
    exit 1;
}else{
    print "PASSED: cpanm found at $CPANM\n";
}

my $cmd_chk_lib = qq(perl -e 'use local::lib;'  2>&1);
my $chk_lib = `$cmd_chk_lib`;
if(!$chk_lib){
    print "PASSED: Perl module (local::lib) installed.\n";
}else{
    print <<OUT;
Please run the following comand to install local::lib:

    $CPANM --local-lib=~/perl5 local::lib && eval \$(perl -I ~/perl5/lib/perl5/ -Mlocal::lib)

And the run `perl INSTALL.pl` again.
OUT
    exit 1;
}


foreach(@package_list){
    my $cmd_chk = qq(perl -e 'use $_;'  2>&1);
    my $check = `$cmd_chk`;
    if(!$check){
        print "PASSED: Perl module ($_) installed.\n";
    }else{
        print "Perl module ($_) not installed. Start to install using cpanm: \n";
        my $inst_check = `$CPANM $_`;
        if(!$inst_check){
            print "PASSED: Perl module ($_) installed.\n";
        }else{
            print "Installation of Perl module $_ failed. Please install manually\n";
            
            exit 1;
        }
    }
}


my $chk_R = checkToolExists("Rscript");
if($chk_R){
    print "PASSED: Rscript found in the \$PATH. \n";
}else{
    print "Please install R first. Then run this script again. If you already installed R, please make sure Rscript is in your \$PATH\n";
    exit 1;
}

my $R_package = "$dir/lib/scripts/install_R_packages.R";
my $base = `Rscript $R_package 2>&1`;
print "$base\n";

if($base =~ /succesfully/){
    print "All the dependencies were passed. Please go ahead to use ViewBS. Good luck!\n";
}

sub checkToolExists{
    my ($tool_name) = @_;
    my $exists = 0;
    for my $path ( split /:/, $ENV{PATH} ) {
        if ( -f "$path/$tool_name" && -x _ ) {
            #print "$tool_name found in $path\n";
            $exists = "$path";
            last;
        }
    }
    return $exists;
    #die "No $tool_name command available\n" unless ( $tool_path );
}

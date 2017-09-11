#!/usr/bin/env perl
# Shaojun Xie <xie186@purdue.edu>
use warnings;
use strict;

my @package_list=("Getopt::Long::Subcommand", "Bio::DB::HTS::Tabix", "Bio::SeqIO", "HAHA::VIEWBS");

my $xx = checkToolExists("tabix");
print "$xx\n";
exit;

## Check Perl Version
##my $perl_v_help = `perl -v`;
#my ($perl_num, $perl_version, $perl_subv) = $perl_v_help =~ /v(\d+)\.(\d+)\.(\d+)/;
#print "$perl_num, $perl_version, $perl_subv\n";

if ($] >= 5.014004){
    print "Perl version($] >= 5.14.4): PASSED\n";
}else{
    print "Error: Perl version needs to be above 5.14.4!\n";
    exit 1;
}

## Check htslib install
my $path = checkToolExists("tabix");
if($path){
    print "tabix found in $path: PASSSED\n";
}else{
    print "tabix not found:\n";
    print "Check whether git was installed.\n";
    my $chk_git = checkToolExists("git");
    if($chk_git){
       print "'git' found. Start to download htslib.\n";
       my $cmd = <<CMD;
git clone https://github.com/samtools/htslib.git
cd htslib
autoheader     # If using configure, generate the header template...
autoconf       # ...and configure script (or use autoreconf to do both)
./configure    # Optional, needed for choosing optional functionality
make
make --prefix ~/bin/ install 
CMD
       `git clone https://github.com/samtools/htslib.git`;
       
    }else{
        print "'git' not found. Please install git first.\n";
        exit;
    } 
}

foreach(@package_list){
    my $cmd_chk = qq(perl -e 'use $_;'  2>&1);
    my $check = `$cmd_chk`;
    if(!$check){
        print "Perl module ($_) installed. PASSED\n";
    }else{
        print "Perl module ($_) installed. FAILED\n";
        
    }
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

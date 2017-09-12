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
    print "Please install htslib. \n";
}

foreach(@package_list){
    my $cmd_chk = qq(perl -e 'use $_;'  2>&1);
    my $check = `$cmd_chk`;
    if(!$check){
        print "Perl module ($_) installed. PASSED\n";
    }else{
        print "Perl module ($_) not installed. installed. \n";
        
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

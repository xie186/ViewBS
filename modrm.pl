#!/usr/local/bin/perl -w
 
# Alan Burlison <Alan.Burlison@uk.sun.com>

use strict;
use IO::Dir;
use ExtUtils::Packlist;
use ExtUtils::Installed;

my @package_list=("Getopt::Long::Subcommand", "Bio::DB::HTS::Tabix", "Bio::SeqIO"); 
 
# Find all the installed packages
print("Finding all installed modules...\n");
my $installed = ExtUtils::Installed->new();
 
my %module_list;
foreach my $module (grep(!/^Perl$/, $installed->modules())) {
   my $version = $installed->version($module) || "???";
   $module_list{$module} = $version;
   #print("Found module $module Version $version\n");
}

foreach my $module(@package_list){
    if(exists $module_list{$module}){
	print("Found module $module Version $module_list{$module}\n");
    }else{
        print "Not found module $module\n";
    }
}

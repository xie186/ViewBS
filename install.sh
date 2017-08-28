#!/bin/sh

chmod 755 ext_tools/cpanm

### Getopt::Long::Subcommand
rep=`perl -e "use Getopt::Long::Subcommand"`
if [ -z $rep]; 
then
    ext_tools/cpanm Getopt::Long::Subcommand
else
    echo "Dectected: Getopt::Long::Subcommand"
fi

rep=`perl -e "use Bio::SeqIO"`
if [ -z $rep];
then
    ext_tools/cpanm Bio::SeqIO
else
    echo "Dectected: Bio::SeqIO"
fi

## With
if ! type "tabix" > /dev/null; then
   git clone https://github.com/samtools/htslib.git
   cd htslib
   autoheader     # If using configure, generate the header template...
   autoconf       # ...and configure script (or use autoreconf to do both)
   ./configure    # Optional, needed for choosing optional functionality
   make
   su -c "make install"
else 
   echo "Dectected: tabix"
fi 

rep=`perl -e "use Bio::DB::HTS::Tabix"`
if [ -z $rep];
then
    ext_tools/cpanm Bio::DB::HTS::Tabix
else 
    echo "Dectected: Bio::DB::HTS::Tabix"
fi

echo "Checking R packages";
R --vanilla --slave < lib/scripts/install_R_packages.R

chmod 755 ViewBS


#!/bin/sh

wget https://raw.githubusercontent.com/miyagawa/cpanminus/master/cpanm
chmod 755 cpanm
./cpanm Getopt::Long::Subcommand
./cpanm Bio::DB::HTS::Tabix


git clone https://github.com/samtools/htslib.git
cd htslib
utoheader     # If using configure, generate the header template...
autoconf       # ...and configure script (or use autoreconf to do both)
./configure    # Optional, needed for choosing optional functionality
make
make install

echo "Checking R packages";
R --vanilla --slave < lib/scripts/install_R_packages.R 

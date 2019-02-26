
## How to configure perl envrionment for this issue

```
perlbrew install perl==5.24.1 ## or perl==5.18.2
perlbrew use perl-5.24.1
perlbrew install-cpanm
cpanm Getopt::Long::Subcommand
cpanm Bio::DB::HTS::Tabix 
cpanm Bio::SeqIO
```

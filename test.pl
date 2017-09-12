print "$ENV{PATH}\n\n";
print "$ENV{CVSROOT}\n\n";
`export PATH=\$PATH:~/bin/`;
my $xx = `echo \$PATH`;
print "xx$xx\n";

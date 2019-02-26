perlbrew list-modules > environment_module_list.txt
perlbrew  available |grep -v '^i' |perl -e 'while(<>){print "perlbrew  --notest install $_";}' > chk_issue_install_perl.sh


zcat  ../ViewBS_testdata/testdata/bis_WT.tab.gz |head -10000 > test_WT.tab
bgzip test_WT.tab
tabix -p vcf test_WT.tab.gz

../ViewBS MethLevDist --sample test_WT.tab.gz,WT
rm MethLevDist.*



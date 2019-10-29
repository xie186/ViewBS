```
cd ViewBS_testdata/testdata
docker run -v ${PWD}:/data -w /data cd240cd8fcef ViewBS  MethOverRegion --region TAIR10_GFF3_genes_chr1.bed --sample bis_WT.tab.gz,WT --sample bis_cmt23.tab.gz,cmt23 --prefix bis_gene_chr1_sample --context CG --outdir MethOverRegion_Docker
```

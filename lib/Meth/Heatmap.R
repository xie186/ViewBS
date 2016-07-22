Args <- commandArgs();
cat("Useage: R --vanilla --slave --input --output1 heat.pdf --output2 hist.pdf < trans_DMS_heatmap.R", "\n");

clus_col = "FALSE"
clus_row = "TRUE"

cat (Args,"\n")
for (i in 1:length(Args)) {
        if (Args[i] == "--input")       cpg = Args[i+1]
        if (Args[i] == "--output1")     output1 = Args[i+1]
        if (Args[i] == "--output2")     output2 = Args[i+1]
	if (Args[i] == "--cluster_cols") clus_col = Args[i+1]
	if (Args[i] == "--cluster_rows") clus_row = Args[i+1]
    
}
print(1)
cat(clus_col, clus_row, "\n")
print(1)
   pdf(output1,height=5,width=3.5, onefile=FALSE)
   cc<-read.table(cpg,header=T)
   library(gplots)
   library(pheatmap)
   #library("RColorBrewer")
   x  <- as.matrix(cc)
   #heatmap.2(x, col=brewer.pal(11,"RdBu"), trace = "none", density.info=c("none"), labRow=FALSE, Colv = F, dendrogram = c("row"),margins=c(10,0))
   #heatmap.2(x, col= colorpanel(100,low="lightyellow",mid="darkred",high="black"), keysize=2, trace = "none", density.info=c("none"), labRow=FALSE, Colv = F, dendrogram = c("none"),margins=c(10,0))
   pheatmap(x, show_rownames = F, cluster_rows = clus_row, cluster_cols = clus_col)
   dev.off()

   library(reshape2);
   long <- melt(cc, measure=c(colnames(cc)),variable = "meth");
   library(ggplot2);
   p <- ggplot(long, aes(x=meth, y=value,col=meth)) +
        #geom_boxplot(outlier.shape=NA) +
        geom_boxplot() +
        theme(axis.text.x = element_text(angle = 90, hjust = 1)) +
        coord_cartesian(ylim = range(boxplot(long$value, plot=FALSE)$stats)*c(.9, 1.1))
   ggsave(output2, p)


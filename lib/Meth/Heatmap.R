Args <- commandArgs();

cat("Useage: R --vanilla --slave --input <> --height <heigtht> --width <width> --height2 <heigtht> --width2 <width> --output1 <heat.pdf> --output2 <hist.pdf> < trans_DMS_heatmap.R", "\n");

clus_col = "FALSE"
clus_row = "TRUE"

fig_height1 = 12.7
fig_width1  = 8.89
fig_height2 = 10
fig_width2  = 10

cat (Args,"\n")
for (i in 1:length(Args)) {
        if (Args[i] == "--input")        cpg      = Args[i+1]
        if (Args[i] == "--output1")      output1  = Args[i+1]
        if (Args[i] == "--output2")      output2  = Args[i+1]
	if (Args[i] == "--cluster_cols") clus_col = Args[i+1]
	if (Args[i] == "--cluster_rows") clus_row = Args[i+1]
	if (Args[i] == "--height")       fig_height1 = Args[i+1]
        if (Args[i] == "--width")        fig_width1  = Args[i+1]
        if (Args[i] == "--height2")      fig_height2 = Args[i+1]
        if (Args[i] == "--width2")       fig_width2  = Args[i+1]
}

fig_height1 <- as.numeric(fig_height1)
fig_width1  <- as.numeric(fig_width1)
fig_height2 <- as.numeric(fig_height2)
fig_width2  <- as.numeric(fig_width2)

cat(clus_col, clus_row, "\n")
clus_col = type.convert(clus_col, as.is=T)
clus_row = type.convert(clus_row, as.is=T)
print(1)
   pdf(output1,height= fig_height1/2.52, width= fig_width1/2.52, onefile=FALSE)
   cc<-read.table(cpg,header=T)

   cc <- cc[complete.cases(cc),]  ## will only select rows with complete data in all columns
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
   p <- ggplot(long, aes(x=meth, y=value)) 
        #geom_boxplot(outlier.shape=NA) +
   p <- p + geom_violin(aes(col=meth, fill=meth))
   p <- p+ geom_boxplot(width = 0.2, outlier.shape = 16, outlier.size = 0.01, col="gray", alpha = 0.7) + stat_boxplot(geom='errorbar', width=0.2, col="gray")
   p <- p + theme(axis.text.x = element_text(angle = 90, hjust = 1)) 
  
   #p <- p + stat_boxplot(geom='errorbar')
        #coord_cartesian(ylim = range(boxplot(long$value, plot=FALSE)$stats)*c(.9, 1.1)) +
        #stat_boxplot(geom='errorbar',coef=10)
   p <- p + ylab("Methylation level")
   p <- p + xlab("Sample")
   p <- p + theme(legend.title=element_blank()) ## no legend title
   ggsave(output2, p, width = fig_width2, height = fig_height2, units = "cm")


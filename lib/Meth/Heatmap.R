Args <- commandArgs();

cat("Useage: R --vanilla --slave --input <> --height <heigtht> --width <width> --height2 <heigtht> --width2 <width> --random_region <random_region> --output1 <heat.pdf> --output2 <hist.pdf> < trans_DMS_heatmap.R", "\n");

clus_col = "FALSE"
clus_row = "TRUE"

RANDOM_NUM = 2000

fig_height1 = 12.7
fig_width1  = 8.89
fig_height2 = 10
fig_width2  = 10

cat (Args,"\n")
for (i in 1:length(Args)) {
    if (Args[i] == "--input")         cpg      = Args[i+1]
    if (Args[i] == "--output1")       output1  = Args[i+1]
    if (Args[i] == "--output2")       output2  = Args[i+1]
    if (Args[i] == "--cluster_cols")  clus_col = Args[i+1]
    if (Args[i] == "--cluster_rows")  clus_row = Args[i+1]
    if (Args[i] == "--height")        fig_height1 = as.numeric(Args[i+1])
    if (Args[i] == "--width")         fig_width1  = as.numeric(Args[i+1])
    if (Args[i] == "--height2")       fig_height2 = as.numeric(Args[i+1])
    if (Args[i] == "--width2")        fig_width2  = as.numeric(Args[i+1])
    if (Args[i] == "--random_region") RANDOM_NUM  = as.numeric(Args[i+1])
}

cat(clus_col, clus_row, "\n")
clus_col = type.convert(clus_col, as.is=T)
clus_row = type.convert(clus_row, as.is=T)

pdf(output1,height= fig_height1/2.54, width= fig_width1/2.54, onefile=FALSE)
cc<-read.table(cpg,header=T)

cc <- cc[complete.cases(cc),]  ## will only select rows with complete data in all columns
library(pheatmap)
#library("RColorBrewer")
x  <- as.matrix(cc)

### If the give region regions are too many, it may cause the error: "Error: cannot allocate vector of size". 
# Error messages beginning cannot allocate vector of size indicate a failure to obtain memory, either because the size exceeded the address-space limit for a process or, more likely, because the system was unable to provide the memory. 
# If this case, ViewBS will catch the error and randomly select RANDOM_NUM regions from the given list.
result <- 
tryCatch({ 
   pheatmap(x, show_rownames = F, cluster_rows = clus_row, cluster_cols = clus_col)
   }, error = function(err) {
	x <- x[sample(1:nrow(x), RANDOM_NUM),]
	pheatmap(x, show_rownames = F, cluster_rows = clus_row, cluster_cols = clus_col)
   }
)

dev.off()

#### Generate violin-boxplots for the given regions.
library(reshape2);
long <- melt(cc, measure=c(colnames(cc)),variable = "meth");
library(ggplot2);
p <- ggplot(long, aes(x=meth, y=value)) 
        #geom_boxplot(outlier.shape=NA) +
p <- p + geom_violin(aes(col=meth, fill=meth), scale = "width")
p <- p + geom_boxplot(width = 0.2, outlier.shape = 16, outlier.size = 0.01, col="gray", alpha = 0.7) 
p <- p + stat_boxplot(geom='errorbar', width=0.2, col="gray")
p <- p + theme(axis.text.x = element_text(angle = 90, hjust = 1)) 
  
p <- p + ylab("Methylation level")
p <- p + xlab("Sample")
p <- p + theme(legend.title=element_blank()) ## no legend title
ggsave(output2, p, width = fig_width2, height = fig_height2, units = "cm")

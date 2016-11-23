Args <- commandArgs();

cat("Usage: R --vanilla --slave --height <fig height> --width <fig width> --adjustXaxis <X> --input <input tab> --xlab <Gene>  --output <output> < OverRegion.R", "\n")

cat(Args, "\n")

xlab = "Gene"
legend_title = "Sample name"

for(i in 1:length(Args)){
    if(Args[i] == "--input")            meth = Args[i+1]
    if(Args[i] == "--tts")              tss  = Args[i+1]
    if(Args[i] == "--xlab")             xlab  = Args[i+1]
    if(Args[i] == "--output")           fig = Args[i+1]
    if(Args[i] == "--height")           fig_height = as.numeric(Args[i+1])
    if(Args[i] == "--width")            fig_width  = as.numeric(Args[i+1])
    if(Args[i] == "--adjustXaxis")      adjustXaxis  = as.numeric(Args[i+1])
}

library(ggplot2)
tab<-read.table(meth,head=T);
tab <- tab[order(tab[,3]),]

p=ggplot(tab, aes(x=bin_num, y=Methylation_level, group=sample_name, col=sample_name)) +
    geom_line() + xlab(xlab)
    
min <- min(tab$bin_num)  ## by default the lowest value is -19
max <- abs(max(tab$bin_num))
    #p = p + scale_x_continuous(breaks=c(min(tab$Pos), 0.5,max(tab$Pos)+min(tab$Pos), max(tab$Pos)), labels=c(abs(min(tab$Pos))/10, "TSS", "TTS", abs(min(tab$Pos))/10))

    #p = p + scale_x_continuous(breaks=c(min/2, (max + min)/2, max + min/2), labels=c("Upstream", xlab, "Downstream"))
#p = p + scale_x_continuous(breaks=c(min/2, (max + min)/2, max + min/2), labels=c("Upstream", xlab, "Downstream"))
#p = p + theme(axis.text.x = element_blank())

flank = paste( -(min -1)/adjustXaxis, "kb", sep = " ")

p = p + scale_x_continuous(breaks=c(min, max), labels=c(flank, flank));

p <- p + theme(legend.title=element_blank()) ## no legend title
#p = p + scale_fill_continuous(guide = guide_legend(title = legend_title)) # title text

## 1 means the first bin in the gene body, max + min + 1 means the last bean in the gene body.

p = p + geom_vline(xintercept = c(1, max + min - 1), linetype = "dashed")
p = p + expand_limits(y=0)
p = p + ylab("Methylation level")
ggsave(fig, p, height = fig_height, width = fig_width, unit="cm")

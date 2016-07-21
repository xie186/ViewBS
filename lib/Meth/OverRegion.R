Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --xlab <Gene>  --output <output> < *R", "\n")

cat(Args, "\n")

xlab = "Gene"

for(i in 1:length(Args)){
    if(Args[i] == "--input")   meth = Args[i+1]
    if(Args[i] == "--tts")     tss  = Args[i+1]
    if(Args[i] == "--xlab")     xlab  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}


library(ggplot2)
tab<-read.table(meth,head=T);
tab <- tab[order(tab[,3]),]

p=ggplot(tab, aes(x=bin_num, y=Methylation_level, group=sample_name, col=sample_name)) +
    geom_line() + xlab(xlab)
    
min <- min(tab$bin_num) -1
max <- abs(max(tab$bin_num))
    #p = p + scale_x_continuous(breaks=c(min(tab$Pos), 0.5,max(tab$Pos)+min(tab$Pos), max(tab$Pos)), labels=c(abs(min(tab$Pos))/10, "TSS", "TTS", abs(min(tab$Pos))/10))

    #p = p + scale_x_continuous(breaks=c(min/2, (max + min)/2, max + min/2), labels=c("Upstream", xlab, "Downstream"))
#p = p + scale_x_continuous(breaks=c(min/2, (max + min)/2, max + min/2), labels=c("Upstream", xlab, "Downstream"))
#p = p + theme(axis.text.x = element_blank())

flank = paste( -min/10, "kb", sep = " ")

p = p + scale_x_continuous(breaks=c(min, max), labels=c(flank, flank));
p = p + geom_vline(xintercept = c(0.5, max + min + 1.5), linetype = "dashed")
ggsave(fig, p)


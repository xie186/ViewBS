methGeno <- function(meth, out){

    library(ggplot2)
    tab <-read.table(meth, header=T);
    p <- ggplot(tab, aes(x=(stt+end)/1000000, y=Methylation_level, group=sample_name,col = sample_name));
    p <- p + geom_line(size = 0.5)+facet_wrap(~chr)+xlab("Chromosome coordinate")+ylab("Methylation level")
    #p <- p + geom_line(size = 0.5, alpha=0.4)+facet_wrap(~chr)+xlab("Chromosome coordinate")+ylab("Methylation level")
    p <- p+ theme(legend.position = "top")
    if(length(levels(tab$chr)) <=8){
        ggsave(out, p, width = 20, height = 20)
    }else{
        p <- ggplot(tab, aes(x=(stt+end)/1000000, y=Methylation_level, group=sample_name,col = sample_name));
        p <- p + geom_line(size = 0.5)+facet_wrap(~chr,ncol = 2)+xlab("Chromosome coordinate")+ylab("Methylation level")
        p <- p+ theme(legend.position = "top")
        ggsave(out, p, width = 20, height = 20*length(levels(tab$chr))/3, units = "cm")
    }
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}

methGeno(input, fig);

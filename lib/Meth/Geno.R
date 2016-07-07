methGeno <- function(meth, out){

    library(ggplot2)
    tab <-read.table(meth, header=T);
    p <- ggplot(tab, aes(x=(stt+end)/1000000, y=Methylation_level, group=sample_name,col = sample_name)); 
    p <- p + geom_line(size = 1.5)+facet_wrap(~chr)+xlab("Chromosome coordinate")+ylab("Methylation level")
    ggsave(out, p)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}

methGeno(input, fig);

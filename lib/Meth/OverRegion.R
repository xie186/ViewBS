methGeno <- function(meth, out){

    library(ggplot2)
    tab<-read.table(meth,head=T);

    p=ggplot(tab, aes(x=bin_num, y=Methylation_level, group=sample_name, col=sample_name))
	     +geom_line()
    p = p + scale_x_continuous(breaks=c(0.5,61.5), labels=c("TSS", "TTS"))
     
    ggsave(out, p)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --flank <flanking regions> --tts <bin number> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--tts")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}

methGeno(input, fig);

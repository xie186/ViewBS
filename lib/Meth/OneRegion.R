methGeno <- function(meth, out){

     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
 
     p<-ggplot(tab,aes(x=position,xend=position,y=0,yend=MethylationLevel, col=Sample))+
	geom_segment() + facet_grid(Sample ~.)+
	ylab("Methylation Level") + 
	xlab(paste(levels(tab[,2]))) +
        theme(legend.position="none")

      #par("din") : the device dimensions in inches,
      #par("fin") : the current figure dimensions in inches,
      #par("pin") : the current plot region dimensions in inches,
      #par("fig") : NDC coordinates for the figure region,
      #par("plt") : NDC coordinates for the plot region,
      ggsave(out, p, height=length(levels(tab[,1]))*par("din")[2]/6)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}
options(warn=-1)

suppressWarnings(methGeno(input, fig))

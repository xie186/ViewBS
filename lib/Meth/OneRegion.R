methGeno <- function(meth, region, out){

     tab <-read.table(meth, head = T, sep = "\t")
     Sample = unique(tab[,1])
     tab[,1] = as.factor(tab[,1])
     levels(tab[,1]) = Sample
     #print(tab$Sample)

     library(ggplot2)
 
     p <- ggplot(tab,aes(x=position,xend=position,y=0,yend=MethylationLevel, col=Sample))+
	geom_segment() + facet_grid(Sample ~.)+
	ylab("Methylation Level") + 
	xlab(paste(levels(tab[,2]))) +
        theme(legend.position="none")
      
      #http://docs.ggplot2.org/0.9.3/annotate.html
      coor <- as.numeric(unlist(strsplit(region,split="-")))
      p <- p + annotate("rect", xmin = coor[1], xmax = coor[2], ymin = 0, ymax = 1,alpha = .2)
      #par("din") : the device dimensions in inches,
      #par("fin") : the current figure dimensions in inches,
      #par("pin") : the current plot region dimensions in inches,
      #par("fig") : NDC coordinates for the figure region,
      #par("plt") : NDC coordinates for the plot region,
      ggsave(out, p, height=length(levels(tab[,1]))*par("din")[2]/6)
      #https://stackoverflow.com/questions/5577221/how-can-i-load-an-object-into-a-variable-name-that-i-specify-from-an-r-data-file
      out_rds = paste(out, ".rds", sep="")
      saveRDS(p, out_rds)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --region <chr:stt-end> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input = Args[i+1]
    if(Args[i] == "--region")   region = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}
options(warn=-1)

suppressWarnings(methGeno(input, region, fig))


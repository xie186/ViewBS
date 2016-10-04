methGeno <- function(meth, out){
     print(meth)
     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
     #Sample  Context Depth   Percentage
     p <- ggplot(tab, aes(x=Depth, y=Percentage, group = Context, col=Context)) +
	         geom_line() +
	         facet_wrap("Sample");
     ggsave(out, p)
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




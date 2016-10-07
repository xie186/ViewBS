methGeno <- function(meth, out){
     print(meth)
     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
     #Sample  Context MethylationLevel        Number  Percentage
     #library(reshape2)
     #tab <- melt(tab, id.vars="Sample")
     p <- ggplot(tab, aes(x=Sample, y=BisNonConvRate, fill=Sample)) +
	   geom_bar(stat="identity",position="dodge") +
           ylab("Non conversion rate") +
	   xlab("Sample")

     ggsave(out, p)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
}

#this shows the correct way to suppress warnings is suppressWarnings() instead of options(warn = -1) alone; similarly, you should use suppressMessages() to suppress messages
#https://gist.github.com/yihui/6656584

options(warn=-1)

suppressWarnings(methGeno(input, fig))




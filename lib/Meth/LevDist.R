methGeno <- function(meth, out){
     print(meth)
     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
     #Sample  Context MethylationLevel        Number  Percentage

     p <- ggplot(tab, aes(x=MethLevBinMidPoint, y=Percentage, fill=Context)) +
               geom_bar(stat="identity") +
               facet_grid( Context ~ Sample, scales = "free") + 
	       theme(legend.position="none")
     #p <- ggplot(tab, aes(x=MethylationLevel, y=Percentage, group = Context, col=Context)) +
     #	         geom_line() +
     #	         facet_wrap("Sample");
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




methGeno <- function(meth, out, fig_height, fig_width){
     print(meth)
     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
     #Sample  Context MethylationLevel        Number  Percentage
     library(reshape2)
     tab <- melt(tab, id.vars="Sample")
     ### Define the order of the samples
     tab$Sample = factor(tab$Sample, levels = unique(tab$Sample))
     p <- ggplot(tab, aes(x=variable, y=value, group=Sample,fill=Sample)) +
	   geom_bar(stat="identity",position="dodge") +
           ylab("Methylation level") +
	   xlab("Context")

    ggsave(out, p, height = fig_height, width = fig_width, unit="cm")
    #https://stackoverflow.com/questions/5577221/how-can-i-load-an-object-into-a-variable-name-that-i-specify-from-an-r-data-file
    out_rds = paste(input, ".rds", sep="")
    saveRDS(p, out_rds)
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
    if(Args[i] == "--height")  fig_height = as.numeric(Args[i+1])
    if(Args[i] == "--width")   fig_width  = as.numeric(Args[i+1])
}

#this shows the correct way to suppress warnings is suppressWarnings() instead of options(warn = -1) alone; similarly, you should use suppressMessages() to suppress messages
#https://gist.github.com/yihui/6656584

options(warn=-1)

suppressWarnings(methGeno(input, fig, fig_height, fig_width))




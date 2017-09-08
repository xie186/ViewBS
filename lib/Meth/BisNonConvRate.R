Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --height <heigtht> --width <width> --output <output> < *R", "\n")

cat(Args, "\n")
fig_height = 10
fig_width  = 10
for(i in 1:length(Args)){
    if(Args[i] == "--input")      input  = Args[i+1]
    if(Args[i] == "--output")     fig = Args[i+1]
    if (Args[i] == "--height")    fig_height = as.numeric(Args[i+1])
    if (Args[i] == "--width")     fig_width  = as.numeric(Args[i+1])
}

#this shows the correct way to suppress warnings is suppressWarnings() instead of options(warn = -1) alone; similarly, you should use suppressMessages() to suppress messages
#https://gist.github.com/yihui/6656584
#options(warn=-1)
#suppressWarnings(methGeno(input, fig))

tab <-read.table(input, head = T, sep = "\t")
library(ggplot2)
#Sample\tBisNonConvRate\tC_number\tTotal_Depth\tContext
#library(reshape2)
#tab <- melt(tab, id.vars="Sample")

p <- ggplot(tab, aes(x=Sample, y=BisNonConvRate, fill=Sample)) +
           geom_bar(stat="identity",position="dodge") +
           facet_wrap("Context") +
           theme(axis.text.x=element_text(angle=45, hjust=1)) +
           ylab("Non conversion rate (%)") +
           xlab("Sample")

ggsave(fig, p,height = fig_height, width = fig_width, unit =  "cm")


#https://stackoverflow.com/questions/5577221/how-can-i-load-an-object-into-a-variable-name-that-i-specify-from-an-r-data-file
out_rds = paste(input, ".rds", sep="")
saveRDS(p, out_rds)

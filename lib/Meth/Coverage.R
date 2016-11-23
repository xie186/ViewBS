Args <- commandArgs();

cat("Usage: R --vanilla --slave --height <fig height> --width <fig width> --input <input tab> --output <output> < *R", "\n")

cat(Args, "\n")

fig_height = 10
fig_width  = 10

for(i in 1:length(Args)){
    if(Args[i] == "--input")   input      = Args[i+1]
    if(Args[i] == "--output")  fig        = Args[i+1]
    if(Args[i] == "--height")  fig_height = as.numeric(Args[i+1])
    if(Args[i] == "--width")   fig_width  = as.numeric(Args[i+1])
}

#options(warn=-1)
#suppressWarnings(methGeno(input, fig))
print(input)
tab <-read.table(input, head = T, sep = "\t")

library(ggplot2)
#Sample  Context Depth   Percentage
p <- ggplot(tab, aes(x=Depth, y=Percentage, group = Context, col=Context)) 
p <- p + geom_line()
p <- p + facet_wrap("Sample")

ggsave(fig, p, height=fig_height, width = fig_width, unit = "cm")

methGeno <- function(meth, out, percentage){
     print(meth)
     tab <-read.table(meth, head = T, sep = "\t")
     library(ggplot2)
     #Sample  Context MethylationLevel        Number  Percentage

     
     if(isTRUE(percentage)){
         p <- ggplot(tab, aes(x=MethLevBinMidPoint, y=Percentage, fill=Context)) +
               geom_bar(stat="identity") +
               facet_grid( Context ~ Sample, scales = "free") + 
	       theme(legend.position="none") +
	       xlab("Methylation level") +
	       ylab("Percentage (%)")
         ggsave(out, p)
     }else{
	p <- ggplot(tab, aes(x=MethLevBinMidPoint, y=Number, fill=Context))
        p <- p + geom_bar(stat="identity") 
        p <- p + facet_grid( Context ~ Sample, scales = "free") 
        p <- p + theme(legend.position="none") 
        p <- p + xlab("Methylation level") 
        p <- p + ylab("Number")
         ggsave(out, p)
     }
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --percentage --output <output> < *R", "\n")

cat(Args, "\n")

percentage = "TRUE"
for(i in 1:length(Args)){
    if(Args[i] == "--input")       input  = Args[i+1]
    if(Args[i] == "--output")      fig = Args[i+1]
    if(Args[i] == "--percentage")  percentage = Args[i+1]
}

percentage = type.convert(percentage, as.is=T)

#this shows the correct way to suppress warnings is suppressWarnings() instead of options(warn = -1) alone; similarly, you should use suppressMessages() to suppress messages
#https://gist.github.com/yihui/6656584

options(warn=-1)

suppressWarnings(methGeno(input, fig, percentage))

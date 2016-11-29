methGeno <- function(meth, out, fig_height, fig_width){

    library(ggplot2)
    tab <-read.table(meth, header=T);
    p <- ggplot(tab, aes(x=(stt+end)/1000000, y=Methylation_level, group=sample_name,col = sample_name));
    p <- p + geom_line()+facet_wrap(~chr)+xlab("Chromosome")+ylab("Methylation level")
    #p <- p + geom_line(size = 0.5, alpha=0.4)+facet_wrap(~chr)+xlab("Chromosome coordinate")+ylab("Methylation level")
    
    #p <- p + scale_fill_continuous(guide = guide_legend(title = "Sample")) # title text
    #p <- p + scale_fill_continuous(guide = guide_legend(title = NULL))
    
    p <- p + theme(legend.position = "top")
    p <- p + theme(legend.title=element_blank())
    if(length(levels(tab$chr)) <=8){
        ggsave(out, p, width = fig_width, height = fig_height, unit = "cm")
    }else{
        p <- ggplot(tab, aes(x=(stt+end)/1000000, y=Methylation_level, group=sample_name,col = sample_name));
        p <- p + geom_line()+facet_wrap(~chr,ncol = 2)+xlab("Chromosome")+ylab("Methylation level")
	#p <- p + scale_fill_continuous(guide = guide_legend(title = "Sample")) # title text
	p <- p + scale_fill_continuous(guide = guide_legend(title = NULL))
  	#p <- p+ theme(legend.position = "top")
	p <- p + theme(legend.title=element_blank())
        ggsave(out, p, width = fig_width, height = 7*length(levels(tab$chr))/3, units = "cm")
    }
}

Args <- commandArgs();

cat("Usage: R --vanilla --slave --input <input tab> --height <heigtht> --width <width> --output <output> < *R", "\n")

cat(Args, "\n")

fig_height = 10
fig_width  = 10
for(i in 1:length(Args)){
    if(Args[i] == "--input")   input  = Args[i+1]
    if(Args[i] == "--output")  fig = Args[i+1]
    if (Args[i] == "--height")    fig_height = as.numeric(Args[i+1])
    if (Args[i] == "--width")     fig_width  = as.numeric(Args[i+1])
}

methGeno(input, fig, fig_height, fig_width);

methGeno <- function(meth, out, fig_height, fig_width){
    unit_Mb = 2000000
    library(ggplot2)
    tab <-read.table(meth, header=T);
    p <- ggplot(tab, aes(x=(stt+end)/unit_Mb, y=Methylation_level, group=sample_name,col = sample_name));
    p <- p + geom_line()+facet_wrap(~chr)+xlab("Chromosome (Mb)")+ylab("Methylation level")
    #p <- p + geom_line(size = 0.5, alpha=0.4)+facet_wrap(~chr)+xlab("Chromosome coordinate")+ylab("Methylation level")
    
    #p <- p + scale_fill_continuous(guide = guide_legend(title = "Sample")) # title text
    #p <- p + scale_fill_continuous(guide = guide_legend(title = NULL))
    
    p <- p + theme(legend.position = "top")
    p <- p + theme(legend.title=element_blank())
    if(length(levels(tab$chr)) <=8){
        ggsave(out, p, width = fig_width, height = fig_height, unit = "cm")
    }else{
        p <- ggplot(tab, aes(x=(stt+end)/unit_Mb, y=Methylation_level, group=sample_name,col = sample_name));
        p <- p + geom_line()+facet_wrap(~chr,ncol = 2)+xlab("Chromosome (Mb)")+ylab("Methylation level")
	#p <- p + scale_fill_continuous(guide = guide_legend(title = "Sample")) # title text
	p <- p + scale_fill_continuous(guide = guide_legend(title = NULL))
  	#p <- p+ theme(legend.position = "top")
	p <- p + theme(legend.title=element_blank())
        ggsave(out, p, width = fig_width, height = 7*length(levels(tab$chr))/3, units = "cm")
    }
    #https://stackoverflow.com/questions/5577221/how-can-i-load-an-object-into-a-variable-name-that-i-specify-from-an-r-data-file
    out_rds = paste(input, ".rds", sep="")
    saveRDS(p, out_rds)
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

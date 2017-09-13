Args <- commandArgs();

args = 0
help = 0
for (i in 1:length(Args)) {
    if (Args[i] == "--args")   args = args +1
    if (Args[i] == "-h" || Args[i] == "--help")   help = help +1
}

if(args == 0 || (help > 0 && args > 0)){
    #cat("Usage: Rscript mer_fig.R --input <fig1.rds,fig2.rds> --labels <A,B,C,D>", "\n")
    cat("
USAGE
    Usage: Rscript mer_fig.R --input <fig1.rds,fig2.rds> --labels <A,B,C,D> [options]

DESCRIPTION
    mer_fig.R is developed to merge figures into on graph.

Options
    -help | -h
            Prints the help message and exits.

    --input [required]
           - RDS files. <fig1.rds,fig2.rds...>

    --labels [optional]
           - Labesl for each figure. Default: <A,B,C,D...> 

    --output [optional]
           - Output files for the graph. Default: cowplot_mer_fig.pdf
    
    --ncol [optional]
           - Number of columns on the graph. 

    --base_height [optional]
           - The height (in inches) of each sub-plot

    --base_aspect_ratio [optional]
           -  The aspect ratio of each sub-plot. Default: 1.6
 
")
    stop("Please check the help information!")
}

fig_height = 12.7
fig_width  = 8.89
num_col = 2
output = "cowplot_mer_fig.pdf"
asp_ratio = 1.6
cat (Args,"\n")
for (i in 1:length(Args)) {
    if (Args[i] == "--input")        input       = Args[i+1]
    if (Args[i] == "--labels")       label       = Args[i+1]
    if (Args[i] == "--ncol")         num_col     = as.numeric(Args[i+1])
    if (Args[i] == "--output")       output  = Args[i+1]
    if (Args[i] == "--base_height")        fig_height = as.numeric(Args[i+1])
    #if (Args[i] == "--width")         fig_width  = as.numeric(Args[i+1])
    if (Args[i] == "--base_aspect_ratio")  asp_ratio  = as.numeric(Args[i+1]) #base_aspect_ratio
}


labs <- c()
rds_files <- unlist(strsplit(input, split=","))
if(exists("label")){
    labs <- unlist(strsplit(label, split=",")) 
}else{
    labs <- LETTERS[1:length(rds_files)]
}

if(length(rds_files) != length(labs)){
    stop("")
}
#pl <- vector("list", length = length(rds_files))
pl <-list()
for(i in 1:length(rds_files)){
    i <- i  
    p <- readRDS(rds_files[i])
    cat("Plot class: ", class(p), "\n")
    pl[[i]] <- p
}

library(cowplot)
plot2by2 <- plot_grid(plotlist=pl,
                      labels=labs, ncol = num_col)
num_col<- as.numeric(num_col)
save_plot(output, plot2by2,
          #ncol = num_col, # we're saving a grid plot of 2 columns
          #nrow = 2, # and 2 rows
          base_height = fig_height/2.54, 
          # each individual subplot should have an aspect ratio of 1.
          base_aspect_ratio = asp_ratio
          )

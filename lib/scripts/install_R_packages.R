cat("Loading R libraries\n")

if(!suppressMessages(require(ggplot2, warn.conflicts=F))){
    install.packages("ggplot2", dep=T)
    if(!suppressMessages(require(ggplot2, warn.conflicts=F))) {
        stop('Loading package ggplot2 failed! Please install manually.\n')
    }
}

cat(".")

if(!suppressMessages(require(reshape2, warn.conflicts=F))){
    install.packages("reshape2", dep=T)
    if(!suppressMessages(require(reshape2, warn.conflicts=F))) {
        stop('Loading package reshape2 failed!Please install manually.\n')
    }
}

cat(".")

if(!suppressMessages(require(pheatmap, warn.conflicts=F))){
    install.packages("pheatmap", dep=T)
    if(!suppressMessages(require(pheatmap, warn.conflicts=F))) {
        stop('Loading package pheatmap failed! Please install manually.\n')
    }
}

cat(".")
# install.packages("cowplot")
if(!suppressMessages(require(cowplot, warn.conflicts=F))){
    install.packages("cowplot", dep=T)
    if(!suppressMessages(require(cowplot, warn.conflicts=F))) {
        stop('Loading package cowplot failed! Please install manually.\n')
    }
}

## Test
#cat(".")
#if(!suppressMessages(require(xxxTest, warn.conflicts=F))){
#    install.packages("xxxTest", dep=T)
#    if(!suppressMessages(require(xxxTest, warn.conflicts=F))) {
#        stop('Loading package xxxTest failed! Please install manually.\n')
#    }
#}
cat(".Done succesfully\n")


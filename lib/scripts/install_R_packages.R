cat("Loading R libraries\n")

if(!suppressMessages(require(ggplot2, warn.conflicts=F))){
    install.packages("ggplot2", dep=T)
    if(!suppressMessages(require(ggplot2, warn.conflicts=F))) {
        stop('Loading package ggplot2 failed!')
    }
}

cat(".")

if(!suppressMessages(require(reshape2, warn.conflicts=F))){
    install.packages("reshape2", dep=T)
    if(!suppressMessages(require(reshape2, warn.conflicts=F))) {
        stop('Loading package reshape2 failed!')
    }
}

cat(".")

if(!suppressMessages(require(pheatmap, warn.conflicts=F))){
    install.packages("pheatmap", dep=T)
    if(!suppressMessages(require(pheatmap, warn.conflicts=F))) {
        stop('Loading package pheatmap failed!')
    }
}

cat(".Done\n")


cat("Loading R libraries")

if(!suppressMessages(require(ShortRead, warn.conflicts=F))) {
    source("http://bioconductor.org/biocLite.R")
    biocLite(ShortRead)
    if(!suppressMessages(require(ShortRead, warn.conflicts=F))) {
        stop('Loading package ShortRead failed!')
    }
}

install.packages("ggplot2", dep=T)
install.packages("pheatmap", dep=T)
install.packages("reshape2", dep=T)


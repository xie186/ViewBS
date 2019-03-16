FROM biocontainers/biocontainers:latest
MAINTAINER Shaojun Xie <xie186@purdue.edu>
LABEL    software="ViewBS" \ 
    container="ViewBS" \ 
    about.summary="ViewBS - a powerful toolkit for visualization of high-throughput bisulfite sequencing data" \ 
    about.home="https://github.com/xie186/ViewBS" \ 
    software.version="0.1.8" \ 
    version="1" \ 
    #about.copyright="" \ 
    about.license="GPL-3.0" \ 
    #about.license_file="/usr/share/doc/ariba/copyright" \ 
    #extra.binaries="/usr/bin/ariba" \ 
    about.tags="biology::nucleic-acids, field::biology, field::biology:bioinformatics,:perl, interface::commandline, role::program,:application, use::analysing" 

################## BEGIN INSTALLATION ######################

# Change user to root
USER root

RUN apt-get clean all && \
    apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y  \
        r-base r-base-dev \
        perl-doc libxml-libxml-perl libgd-perl \
        libcurl4-openssl-dev &&   \
        apt-get clean && \
        apt-get purge && \
        rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*


### Install htslib
ENV ZIP=htslib-1.9.tar.bz2
ENV URL=https://github.com/samtools/htslib/releases/download/1.9/
ENV FOLDER=htslib-1.9
ENV DST=/tmp

RUN wget $URL/$ZIP -O $DST/$ZIP && \
    tar xvf $DST/$ZIP -C $DST && \
    rm $DST/$ZIP && \
    cd $DST/$FOLDER && \
    autoheader  && \
    autoconf  && \
    ./configure --disable-bz2 --disable-lzma && \
    make && \
    make install && \
    cd / && \
    rm -rf $DST/$FOLDER

### Install perl packages using cpanm
RUN curl -L http://cpanmin.us | perl - App::cpanminus
RUN cpanm Getopt::Long::Subcommand Bio::DB::HTS::Tabix Bio::SeqIO

## Install R packges
RUN Rscript -e "install.packages('ggplot2', repos = \"http://cran.us.r-project.org\")"
RUN Rscript -e "install.packages('cowplot', repos = \"http://cran.us.r-project.org\")"
RUN Rscript -e "install.packages('pheatmap', repos = \"http://cran.us.r-project.org\")"
RUN Rscript -e "install.packages('reshape2', repos = \"http://cran.us.r-project.org\")"


USER biodocker

################## INSTALLATION ######################

#ENV ZIP=ViewBS_v0.1.8.zip
#ENV URL=https://github.com/xie186/ViewBS/archive/
#ENV FOLDER=ViewBS-ViewBS_v0.1.8
#ENV DST=/home/biodocker/bin

#RUN wget $URL/$ZIP -O $DST/$ZIP && \
#    unzip $DST/$ZIP -d $DST && \
#    rm $DST/$ZIP && \
#    mv $DST/$FOLDER/* $DST && \
#    rmdir $DST/$FOLDER

# Install ViewBS
# reference: https://github.com/andreirozanski/rozanskide-trim_galore/blob/master/Dockerfile

RUN cd /home/biodocker/ && git clone https://github.com/xie186/ViewBS.git
ENV PATH /home/biodocker/ViewBS/:$PATH 

# CHANGE WORKDIR TO /DATA
#WORKDIR /tools
WORKDIR /home/biodocker/

# DEFINE DEFAULT COMMAND
CMD ["ViewBS"]

##################### INSTALLATION END #####################

# File Author / Maintainer
MAINTAINER Shaojun Xie at Bioinformatics Core in Purdue University <xie186@purdue.edu>


package Meth::Sample;
use strict;
use warnings;
use File::Basename;
use FindBin;
use Pod::Usage;
use Cwd qw(abs_path);

my %rec_geno_len;
my $SAMPLELIST = "sample_list";
sub new{
    my $class     = shift;
    return bless {}, $class;  # built in bless function
}

sub processArgvSampleCoverage{
   my ($class, $opts_sub) = @_;
   my @sample = @{$opts_sub->{sample}};
   if(join("", @sample) !~ /file:/){
        foreach my $sam(@sample){
             my ($meth_file, $sam_name) = split(/,/, $sam);
             push @{$opts_sub -> {$SAMPLELIST}}, $sam;
        }
   }else{
        if(@sample > 0){
            print "Only sample should be provided if you use a TEXT file to provide the sample information.\n";
            exit 0;
        }else{
            $sample[0] =~s/file://;
            open SAM, $sample[0] or die "$!";
            while( my $line = <SAM>){
                next if /#/;
                print "$line";
                chomp $line;
                my ($meth_file, $sam_name) = split(/\s+/, $line);
                push @{$opts_sub -> {$SAMPLELIST}}, "$meth_file,$sam_name";
            }
        }
   }
}

sub processArgvSampleGenome{
   my ($class, $opts_sub) = @_;
   my @sample = @{$opts_sub->{sample}};
   if(join("", @sample) !~ /file:/){
	foreach my $sam(@sample){
	     my ($meth_file, $sam_name) = split(/,/, $sam);
	     push @{$opts_sub -> {$SAMPLELIST}}, $sam;
        }
   }else{
	if(@sample > 0){
	    print "Only sample should be provided if you use a TEXT file to provide the sample information.\n";
	    exit 0;
        }else{
	    $sample[0] =~s/file://;
	    open SAM, $sample[0] or die "$!";
	    while( my $line = <SAM>){
		next if /#/;
		print "$line";
		chomp $line;
		my ($meth_file, $sam_name) = split(/\s+/, $line);
		push @{$opts_sub -> {$SAMPLELIST}}, "$meth_file,$sam_name";
            }
	}
   }  
}

sub processArgvSampleOverRegion{
   my ($class, $opts_sub) = @_;
   my @sample = @{$opts_sub->{sample}};
   if(join("", @sample) !~ /file:/){
	my $region = $opts_sub -> {region};
        foreach my $sam(@sample){
             my ($meth_file, $sam_name) = split(/,/, $sam);
             push @{$opts_sub -> {$SAMPLELIST}}, "$sam,$region";
        }
   }else{
        if(@sample > 0){
            print "Only sample should be provided if you use a TEXT file to provide the sample information.\n";
            exit 0;
        }else{
            $sample[0] =~s/file://;
            open SAM, $sample[0] or die "$!";
            while( my $line = <SAM>){
                next if /#/;
                print "$line";
                chomp $line;
                my ($meth_file, $legend, $tem_region) = split(/\s+/, $line);
		my $region = ($tem_region) ? $tem_region: $opts_sub -> {region};
                push @{$opts_sub -> {$SAMPLELIST}}, "$meth_file,$legend,$region";
            }
        }
   }
}

sub processArgvSampleOneRegion{
   my ($class, $opts_sub) = @_;
   my @sample = @{$opts_sub->{sample}};
   if(join("", @sample) !~ /file:/){
        my $region = $opts_sub -> {region};
        foreach my $sam(@sample){
             my ($meth_file, $sam_name) = split(/,/, $sam);
             push @{$opts_sub -> {$SAMPLELIST}}, "$sam,$region";
        }
   }else{
        if(@sample > 0){
            print "Only sample should be provided if you use a TEXT file to provide the sample information.\n";
            exit 0;
        }else{
            $sample[0] =~s/file://;
            open SAM, $sample[0] or die "$!";
            while( my $line = <SAM>){
                next if /#/;
                print "$line";
                chomp $line;
                my ($meth_file, $legend) = split(/\s+/, $line);
                my $region = $opts_sub -> {region};
                push @{$opts_sub -> {$SAMPLELIST}}, "$meth_file,$legend,$region";
            }
        }
   }
}

1;


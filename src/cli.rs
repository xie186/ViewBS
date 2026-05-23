use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use viewbs::api::{
    BisNonConvRateArgs, ConvertArgs, ConvertGffArgs, DepthFilter, GlobalMethLevArgs,
    MergeFiguresArgs, MethCoverageArgs, MethGenoArgs, MethHeatmapArgs, MethLevDistArgs,
    MethOneRegionArgs, MethOverRegionArgs, PlotFormat, PlotOptions,
};
use viewbs::io::samples::parse_sample_args;
use viewbs::meth::Context;
use viewbs::{
    bis_non_conv_rate, convert_brat, convert_bsseeker, convert_gff, global_meth_lev, merge_figures,
    meth_coverage, meth_geno, meth_heatmap, meth_lev_dist, meth_one_region, meth_over_region,
    ViewBsError,
};

#[derive(Debug, Parser)]
#[command(name = "ViewBS")]
#[command(version)]
#[command(about = "Tools for exploring and visualizing bisulfite sequencing data")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        name = "MethCoverage",
        about = "Generate coverage report for BS-seq data"
    )]
    MethCoverage(MethCoverageCli),

    #[command(name = "BisNonConvRate", about = "Estimate non-conversion rate")]
    BisNonConvRate(BisNonConvRateCli),

    #[command(
        name = "GlobalMethLev",
        about = "Generate global methylation level report"
    )]
    GlobalMethLev(GlobalMethLevCli),

    #[command(
        name = "MethLevDist",
        about = "Generate methylation level distribution"
    )]
    MethLevDist(MethLevDistCli),

    #[command(
        name = "MethGeno",
        about = "Generate methylation information across chromosomes"
    )]
    MethGeno(MethGenoCli),

    #[command(
        name = "MethHeatmap",
        about = "Generate methylation heatmap for regions"
    )]
    MethHeatmap(MethHeatmapCli),

    #[command(
        name = "MethOverRegion",
        about = "Plot average methylation over regions"
    )]
    MethOverRegion(MethOverRegionCli),

    #[command(
        name = "MethOneRegion",
        about = "Extract and plot methylation for one region"
    )]
    MethOneRegion(MethOneRegionCli),

    #[command(
        name = "merge-figures",
        about = "Merge SVG plot artifacts into one figure grid"
    )]
    MergeFigures(MergeFiguresCli),

    #[command(
        name = "bsseeker2bismark",
        visible_alias = "bsseeker2bismark.pl",
        about = "Compatibility alias for `convert bsseeker`"
    )]
    LegacyBsseeker(ConvertFileCli),

    #[command(
        name = "brat2bismark",
        visible_alias = "brat2bismark.pl",
        about = "Compatibility alias for `convert brat`"
    )]
    LegacyBrat(ConvertFileCli),

    #[command(
        name = "gff2tab",
        visible_alias = "gff2tab.pl",
        about = "Compatibility alias for `convert gff`"
    )]
    LegacyGff(ConvertGffCli),

    #[command(
        name = "mer_fig",
        visible_alias = "mer_fig.R",
        about = "Compatibility alias for `merge-figures`"
    )]
    LegacyMergeFigures(MergeFiguresCli),

    #[command(
        name = "convert",
        about = "Convert methylation outputs to ViewBS input"
    )]
    Convert(ConvertCli),
}

#[derive(Debug, Parser)]
struct GlobalMethLevCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "GlobalMethLev")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long = "methodAverage", default_value_t = false)]
    method_average: bool,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethCoverageCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long)]
    reference: PathBuf,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethCoverage")]
    prefix: String,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct BisNonConvRateCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long)]
    chrom: String,

    #[arg(long = "context")]
    context: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "BisNonConvRate")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethLevDistCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long = "regions")]
    #[arg(alias = "region")]
    regions: Option<PathBuf>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethLevDist")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long = "binMethLev", default_value_t = 0.1)]
    bin_meth_lev: f64,

    #[arg(long = "methodAverage", default_value_t = false)]
    method_average: bool,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethOneRegionCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long)]
    region: String,

    #[arg(long = "context")]
    context: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethOneRegion")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long, default_value_t = 300)]
    flank: u64,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethGenoCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long = "genomeLength")]
    genome_length: PathBuf,

    #[arg(long = "context")]
    context: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethGeno")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long, default_value_t = 500_000)]
    win: u64,

    #[arg(long, default_value_t = 500_000)]
    step: u64,

    #[arg(long = "minLength", default_value_t = 300)]
    min_length: u64,

    #[arg(long = "maxChromNumber", default_value_t = 60)]
    max_chrom_number: usize,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethHeatmapCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long = "regions")]
    #[arg(alias = "region")]
    region: Option<PathBuf>,

    #[arg(long = "context")]
    context: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethHeatmap")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long, default_value_t = false)]
    merge: bool,

    #[arg(long = "cluster_rows", default_value = "TRUE")]
    cluster_rows: String,

    #[arg(long = "cluster_cols", default_value = "FALSE")]
    cluster_cols: String,

    #[arg(long = "random_region", default_value_t = 2_000)]
    random_region: usize,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long = "height2", default_value_t = 10.0)]
    height2: f64,

    #[arg(long = "width2", default_value_t = 10.0)]
    width2: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct MethOverRegionCli {
    #[arg(long = "sample", required = true)]
    sample: Vec<String>,

    #[arg(long = "region")]
    region: Option<PathBuf>,

    #[arg(long = "context")]
    context: Vec<String>,

    #[arg(long, default_value = ".")]
    outdir: PathBuf,

    #[arg(long, default_value = "MethOverRegion")]
    prefix: String,

    #[arg(long, alias = "minDepth", default_value_t = 5)]
    min_depth: u64,

    #[arg(long, alias = "maxDepth", default_value_t = 1_000_000)]
    max_depth: u64,

    #[arg(long, default_value_t = 2_000)]
    flank: u64,

    #[arg(long = "binLength", default_value_t = 100)]
    bin_length: u64,

    #[arg(long = "binNumber", default_value_t = 60)]
    bin_number: u64,

    #[arg(long = "minLength", default_value_t = 300)]
    min_length: u64,

    #[arg(long = "maxLength", default_value_t = 5_000_000)]
    max_length: u64,

    #[arg(long = "regionName", default_value = "Gene")]
    region_name: String,

    #[arg(long, default_value_t = 10.0)]
    height: f64,

    #[arg(long, default_value_t = 10.0)]
    width: f64,

    #[arg(long, value_enum, default_value_t = CliPlotFormat::Pdf)]
    plot_format: CliPlotFormat,

    #[arg(long, default_value_t = false)]
    keep_svg: bool,

    #[arg(long, default_value_t = false)]
    no_plot: bool,
}

#[derive(Debug, Parser)]
struct ConvertCli {
    #[command(subcommand)]
    command: ConvertCommands,
}

#[derive(Debug, Subcommand)]
enum ConvertCommands {
    #[command(name = "bsseeker", about = "Convert BSseeker2 CGmap output")]
    Bsseeker(ConvertFileCli),

    #[command(name = "brat", about = "Convert BRAT methylation output")]
    Brat(ConvertFileCli),

    #[command(name = "gff", about = "Convert GFF/GTF features to a region table")]
    Gff(ConvertGffCli),
}

#[derive(Debug, Parser)]
struct ConvertFileCli {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,

    #[arg(long, alias = "minDepth", default_value_t = 1)]
    min_depth: u64,
}

#[derive(Debug, Parser)]
struct ConvertGffCli {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,

    #[arg(long = "feature")]
    feature: Vec<String>,

    #[arg(long = "id-attribute", default_value = "ID")]
    id_attribute: String,
}

#[derive(Debug, Parser)]
struct MergeFiguresCli {
    #[arg(long, required = true, value_delimiter = ',')]
    input: Vec<PathBuf>,

    #[arg(long, value_delimiter = ',')]
    labels: Vec<String>,

    #[arg(long, default_value = "cowplot_mer_fig.pdf")]
    output: PathBuf,

    #[arg(long, default_value_t = 2)]
    ncol: usize,

    #[arg(long = "base-height", alias = "base_height", default_value_t = 12.7)]
    base_height: f64,

    #[arg(
        long = "base-aspect-ratio",
        alias = "base_aspect_ratio",
        alias = "aspect_ratio",
        default_value_t = 1.6
    )]
    base_aspect_ratio: f64,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPlotFormat {
    Pdf,
    Svg,
    Png,
}

impl From<CliPlotFormat> for PlotFormat {
    fn from(value: CliPlotFormat) -> Self {
        match value {
            CliPlotFormat::Pdf => PlotFormat::Pdf,
            CliPlotFormat::Svg => PlotFormat::Svg,
            CliPlotFormat::Png => PlotFormat::Png,
        }
    }
}

fn parse_bool_flag(value: &str) -> std::result::Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" => Ok(true),
        "false" | "f" | "0" | "no" | "n" => Ok(false),
        _ => Err("expected TRUE or FALSE".to_string()),
    }
}

impl GlobalMethLevCli {
    fn into_args(self) -> viewbs::Result<GlobalMethLevArgs> {
        Ok(GlobalMethLevArgs {
            samples: parse_sample_args(&self.sample)?,
            outdir: self.outdir,
            prefix: self.prefix,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            method_average: self.method_average,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethCoverageCli {
    fn into_args(self) -> viewbs::Result<MethCoverageArgs> {
        Ok(MethCoverageArgs {
            samples: parse_sample_args(&self.sample)?,
            reference: self.reference,
            outdir: self.outdir,
            prefix: self.prefix,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl BisNonConvRateCli {
    fn into_args(self) -> viewbs::Result<BisNonConvRateArgs> {
        let contexts = if self.context.is_empty() {
            vec![Context::Cxx]
        } else {
            self.context
                .iter()
                .map(|value| Context::from(value.as_str()))
                .collect()
        };
        Ok(BisNonConvRateArgs {
            samples: parse_sample_args(&self.sample)?,
            outdir: self.outdir,
            prefix: self.prefix,
            chrom: self.chrom,
            contexts,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethLevDistCli {
    fn into_args(self) -> viewbs::Result<MethLevDistArgs> {
        Ok(MethLevDistArgs {
            samples: parse_sample_args(&self.sample)?,
            regions: self.regions,
            outdir: self.outdir,
            prefix: self.prefix,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            bin_meth_lev: self.bin_meth_lev,
            method_average: self.method_average,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethOneRegionCli {
    fn into_args(self) -> viewbs::Result<MethOneRegionArgs> {
        let contexts = if self.context.is_empty() {
            vec![Context::Cg]
        } else {
            self.context
                .iter()
                .map(|value| Context::from(value.as_str()))
                .collect()
        };
        Ok(MethOneRegionArgs {
            samples: parse_sample_args(&self.sample)?,
            outdir: self.outdir,
            prefix: self.prefix,
            region: self.region,
            flank: self.flank,
            contexts,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethGenoCli {
    fn into_args(self) -> viewbs::Result<MethGenoArgs> {
        let contexts = if self.context.is_empty() {
            vec![Context::Cg]
        } else {
            self.context
                .iter()
                .map(|value| Context::from(value.as_str()))
                .collect()
        };
        Ok(MethGenoArgs {
            samples: parse_sample_args(&self.sample)?,
            genome_length: self.genome_length,
            outdir: self.outdir,
            prefix: self.prefix,
            contexts,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            win: self.win,
            step: self.step,
            min_length: self.min_length,
            max_chrom_number: self.max_chrom_number,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethHeatmapCli {
    fn into_args(self) -> viewbs::Result<MethHeatmapArgs> {
        let contexts = if self.context.is_empty() {
            vec![Context::Cg]
        } else {
            self.context
                .iter()
                .map(|value| Context::from(value.as_str()))
                .collect()
        };
        Ok(MethHeatmapArgs {
            samples: parse_sample_args(&self.sample)?,
            region: self.region,
            outdir: self.outdir,
            prefix: self.prefix,
            contexts,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            merge: self.merge,
            cluster_rows: parse_bool_flag(&self.cluster_rows)
                .map_err(|message| ViewBsError::invalid_input("--cluster_rows", message))?,
            cluster_cols: parse_bool_flag(&self.cluster_cols)
                .map_err(|message| ViewBsError::invalid_input("--cluster_cols", message))?,
            random_region: self.random_region,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
            distribution_plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width2,
                height_cm: self.height2,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl MethOverRegionCli {
    fn into_args(self) -> viewbs::Result<MethOverRegionArgs> {
        let contexts = if self.context.is_empty() {
            vec![Context::Cg]
        } else {
            self.context
                .iter()
                .map(|value| Context::from(value.as_str()))
                .collect()
        };
        Ok(MethOverRegionArgs {
            samples: parse_sample_args(&self.sample)?,
            region: self.region,
            outdir: self.outdir,
            prefix: self.prefix,
            contexts,
            depth: DepthFilter {
                min: self.min_depth,
                max: self.max_depth,
            },
            flank: self.flank,
            bin_length: self.bin_length,
            bin_number: self.bin_number,
            min_length: self.min_length,
            max_length: self.max_length,
            region_name: self.region_name,
            plot: PlotOptions {
                enabled: !self.no_plot,
                format: self.plot_format.into(),
                width_cm: self.width,
                height_cm: self.height,
                keep_svg: self.keep_svg,
            },
        })
    }
}

impl ConvertFileCli {
    fn into_args(self) -> ConvertArgs {
        ConvertArgs {
            input: self.input,
            output: self.output,
            min_depth: self.min_depth,
        }
    }
}

impl ConvertGffCli {
    fn into_args(self) -> ConvertGffArgs {
        ConvertGffArgs {
            input: self.input,
            output: self.output,
            features: self.feature,
            id_attribute: self.id_attribute,
        }
    }
}

impl MergeFiguresCli {
    fn into_args(self) -> MergeFiguresArgs {
        MergeFiguresArgs {
            inputs: self.input,
            output: self.output,
            labels: self.labels,
            ncol: self.ncol,
            base_height_cm: self.base_height,
            base_aspect_ratio: self.base_aspect_ratio,
        }
    }
}

pub fn run() -> viewbs::Result<()> {
    match Cli::parse().command {
        Commands::MethCoverage(cli_args) => {
            let output = meth_coverage(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::BisNonConvRate(cli_args) => {
            let output = bis_non_conv_rate(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::GlobalMethLev(cli_args) => {
            let output = global_meth_lev(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MethLevDist(cli_args) => {
            let output = meth_lev_dist(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MethGeno(cli_args) => {
            let output = meth_geno(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MethHeatmap(cli_args) => {
            let output = meth_heatmap(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MethOverRegion(cli_args) => {
            let output = meth_over_region(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MethOneRegion(cli_args) => {
            let output = meth_one_region(cli_args.into_args()?, None, None)?;
            print_output(output);
            Ok(())
        }
        Commands::MergeFigures(cli_args) => {
            let output = merge_figures(cli_args.into_args())?;
            print_output(output);
            Ok(())
        }
        Commands::LegacyBsseeker(cli_args) => {
            let output = convert_bsseeker(cli_args.into_args())?;
            print_output(output);
            Ok(())
        }
        Commands::LegacyBrat(cli_args) => {
            let output = convert_brat(cli_args.into_args())?;
            print_output(output);
            Ok(())
        }
        Commands::LegacyGff(cli_args) => {
            let output = convert_gff(cli_args.into_args())?;
            print_output(output);
            Ok(())
        }
        Commands::LegacyMergeFigures(cli_args) => {
            let output = merge_figures(cli_args.into_args())?;
            print_output(output);
            Ok(())
        }
        Commands::Convert(cli_args) => {
            let output = match cli_args.command {
                ConvertCommands::Bsseeker(convert_args) => {
                    convert_bsseeker(convert_args.into_args())?
                }
                ConvertCommands::Brat(convert_args) => convert_brat(convert_args.into_args())?,
                ConvertCommands::Gff(convert_args) => convert_gff(convert_args.into_args())?,
            };
            print_output(output);
            Ok(())
        }
    }
}

fn print_output(output: viewbs::CommandOutput) {
    for table in output.tables {
        println!("Table: {}", table.path.display());
    }
    for plot in output.plots {
        println!("Plot: {}", plot.path.display());
    }
}

pub fn exit_code(error: &ViewBsError) -> i32 {
    match error {
        ViewBsError::Cancelled => 130,
        ViewBsError::InvalidInput { .. } | ViewBsError::ParseError { .. } => 2,
        ViewBsError::Io { .. }
        | ViewBsError::Csv { .. }
        | ViewBsError::IndexedQueryError { .. }
        | ViewBsError::PlotError { .. }
        | ViewBsError::NotImplemented { .. } => 1,
    }
}

//! Public Rust API for ViewBS.
//!
//! The library API is the preferred integration surface for Rust desktop tools. It exposes the
//! same command behavior as the `ViewBS` binary without requiring a child process, stdout parsing,
//! or any Perl/R runtime.
//!
//! # Streaming command example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use viewbs::{
//!     global_meth_lev, DepthFilter, GlobalMethLevArgs, PlotOptions, SampleSpec,
//! };
//!
//! # fn main() -> viewbs::Result<()> {
//! let output = global_meth_lev(
//!     GlobalMethLevArgs {
//!         samples: vec![SampleSpec {
//!             path: PathBuf::from("sample.tab.gz"),
//!             name: "WT".to_string(),
//!             region_path: None,
//!         }],
//!         outdir: PathBuf::from("results"),
//!         prefix: "global".to_string(),
//!         depth: DepthFilter { min: 5, max: 1_000_000 },
//!         method_average: false,
//!         plot: PlotOptions::default(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! for table in output.tables {
//!     eprintln!("wrote {}", table.path.display());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Indexed-region command example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use viewbs::{
//!     meth_heatmap, DepthFilter, MethHeatmapArgs, PlotOptions, SampleSpec,
//! };
//! use viewbs::meth::Context;
//!
//! # fn main() -> viewbs::Result<()> {
//! let output = meth_heatmap(
//!     MethHeatmapArgs {
//!         samples: vec![SampleSpec {
//!             path: PathBuf::from("sample.tab.gz"),
//!             name: "WT".to_string(),
//!             region_path: None,
//!         }],
//!         region: Some(PathBuf::from("genes.bed")),
//!         outdir: PathBuf::from("results"),
//!         prefix: "heatmap".to_string(),
//!         contexts: vec![Context::Cg],
//!         depth: DepthFilter { min: 5, max: 1_000_000 },
//!         merge: false,
//!         cluster_rows: true,
//!         cluster_cols: false,
//!         random_region: 2_000,
//!         plot: PlotOptions::default(),
//!         distribution_plot: PlotOptions::default(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! assert_eq!(output.summary.command, "MethHeatmap");
//! # Ok(())
//! # }
//! ```
//!
//! # Progress and cancellation
//!
//! ```no_run
//! use std::path::PathBuf;
//! use viewbs::{
//!     meth_geno, CancellationToken, DepthFilter, MethGenoArgs, PlotOptions, ProgressEvent,
//!     ProgressReporter, SampleSpec,
//! };
//! use viewbs::meth::Context;
//!
//! struct Logger;
//!
//! impl ProgressReporter for Logger {
//!     fn report(&self, event: ProgressEvent) {
//!         eprintln!("{} {} {}", event.command, event.phase, event.processed);
//!     }
//! }
//!
//! # fn main() -> viewbs::Result<()> {
//! let cancel = CancellationToken::default();
//! let progress = Logger;
//! let output = meth_geno(
//!     MethGenoArgs {
//!         samples: vec![SampleSpec {
//!             path: PathBuf::from("sample.tab.gz"),
//!             name: "WT".to_string(),
//!             region_path: None,
//!         }],
//!         genome_length: PathBuf::from("genome.fai"),
//!         outdir: PathBuf::from("results"),
//!         prefix: "geno".to_string(),
//!         contexts: vec![Context::Cg],
//!         depth: DepthFilter { min: 5, max: 1_000_000 },
//!         win: 500_000,
//!         step: 500_000,
//!         min_length: 300,
//!         max_chrom_number: 60,
//!         plot: PlotOptions::default(),
//!     },
//!     Some(&progress),
//!     Some(&cancel),
//! )?;
//!
//! assert_eq!(output.summary.command, "MethGeno");
//! # Ok(())
//! # }
//! ```
//!
//! # All analysis commands
//!
//! ```no_run
//! use std::path::PathBuf;
//! use viewbs::{
//!     bis_non_conv_rate, meth_coverage, meth_lev_dist, meth_one_region, meth_over_region,
//!     BisNonConvRateArgs, DepthFilter, MethCoverageArgs, MethLevDistArgs, MethOneRegionArgs,
//!     MethOverRegionArgs, PlotOptions, SampleSpec,
//! };
//! use viewbs::meth::Context;
//!
//! # fn main() -> viewbs::Result<()> {
//! let sample = SampleSpec {
//!     path: PathBuf::from("sample.tab.gz"),
//!     name: "WT".to_string(),
//!     region_path: None,
//! };
//! let depth = DepthFilter { min: 5, max: 1_000_000 };
//! let no_plot = PlotOptions { enabled: false, ..PlotOptions::default() };
//!
//! bis_non_conv_rate(
//!     BisNonConvRateArgs {
//!         samples: vec![sample.clone()],
//!         outdir: PathBuf::from("results"),
//!         prefix: "bis".to_string(),
//!         chrom: "chrC".to_string(),
//!         contexts: vec![Context::Cxx],
//!         depth,
//!         plot: no_plot.clone(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! meth_coverage(
//!     MethCoverageArgs {
//!         samples: vec![sample.clone()],
//!         reference: PathBuf::from("reference.fa"),
//!         outdir: PathBuf::from("results"),
//!         prefix: "coverage".to_string(),
//!         plot: no_plot.clone(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! meth_lev_dist(
//!     MethLevDistArgs {
//!         samples: vec![sample.clone()],
//!         regions: None,
//!         outdir: PathBuf::from("results"),
//!         prefix: "dist".to_string(),
//!         depth,
//!         bin_meth_lev: 0.1,
//!         method_average: false,
//!         plot: no_plot.clone(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! meth_one_region(
//!     MethOneRegionArgs {
//!         samples: vec![sample.clone()],
//!         outdir: PathBuf::from("results"),
//!         prefix: "one".to_string(),
//!         region: "chr2:1006-1010".to_string(),
//!         flank: 300,
//!         contexts: vec![Context::Cg],
//!         depth,
//!         plot: no_plot.clone(),
//!     },
//!     None,
//!     None,
//! )?;
//!
//! meth_over_region(
//!     MethOverRegionArgs {
//!         samples: vec![sample],
//!         region: Some(PathBuf::from("genes.bed")),
//!         outdir: PathBuf::from("results"),
//!         prefix: "over".to_string(),
//!         contexts: vec![Context::Cg],
//!         depth,
//!         flank: 2_000,
//!         bin_length: 100,
//!         bin_number: 60,
//!         min_length: 300,
//!         max_length: 5_000_000,
//!         region_name: "Gene".to_string(),
//!         plot: no_plot,
//!     },
//!     None,
//!     None,
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! # Converter and figure-composition commands
//!
//! ```no_run
//! use std::path::PathBuf;
//! use viewbs::{
//!     convert_brat, convert_bsseeker, convert_gff, merge_figures, ConvertArgs, ConvertGffArgs,
//!     MergeFiguresArgs,
//! };
//!
//! # fn main() -> viewbs::Result<()> {
//! convert_bsseeker(ConvertArgs {
//!     input: PathBuf::from("sample.cgmap"),
//!     output: PathBuf::from("sample.bismark.tab"),
//!     min_depth: 1,
//! })?;
//!
//! convert_brat(ConvertArgs {
//!     input: PathBuf::from("brat.tsv"),
//!     output: PathBuf::from("brat.bismark.tab"),
//!     min_depth: 1,
//! })?;
//!
//! convert_gff(ConvertGffArgs {
//!     input: PathBuf::from("genes.gff3"),
//!     output: PathBuf::from("genes.bed"),
//!     features: vec!["gene".to_string()],
//!     id_attribute: "ID".to_string(),
//! })?;
//!
//! merge_figures(MergeFiguresArgs {
//!     inputs: vec![PathBuf::from("first.svg"), PathBuf::from("second.svg")],
//!     output: PathBuf::from("merged.pdf"),
//!     labels: vec!["A".to_string(), "B".to_string()],
//!     ncol: 2,
//!     base_height_cm: 12.7,
//!     base_aspect_ratio: 1.6,
//! })?;
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod commands;
pub mod errors;
pub mod io;
pub mod meth;
pub mod output;
pub mod plot;

pub use api::{
    BisNonConvRateArgs, CancellationToken, CommandOutput, CommandSummary, ConvertArgs,
    ConvertGffArgs, DepthFilter, GlobalMethLevArgs, LogLevel, LogMessage, MergeFiguresArgs,
    MethCoverageArgs, MethGenoArgs, MethHeatmapArgs, MethLevDistArgs, MethOneRegionArgs,
    MethOverRegionArgs, OutputFile, OutputKind, PlotFormat, PlotOptions, ProgressEvent,
    ProgressReporter, SampleSpec,
};
pub use commands::{
    bis_non_conv_rate, convert_brat, convert_bsseeker, convert_gff, global_meth_lev, merge_figures,
    meth_coverage, meth_geno, meth_heatmap, meth_lev_dist, meth_one_region, meth_over_region,
};
pub use errors::{Result, ViewBsError};

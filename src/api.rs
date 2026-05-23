use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::meth::Context;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SampleSpec {
    pub path: PathBuf,
    pub name: String,
    pub region_path: Option<PathBuf>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct DepthFilter {
    pub min: u64,
    pub max: u64,
}

impl DepthFilter {
    pub fn contains(self, depth: u64) -> bool {
        depth >= self.min && depth <= self.max
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum PlotFormat {
    Pdf,
    Svg,
    Png,
}

impl PlotFormat {
    pub fn extension(self) -> &'static str {
        match self {
            PlotFormat::Pdf => "pdf",
            PlotFormat::Svg => "svg",
            PlotFormat::Png => "png",
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PlotOptions {
    pub enabled: bool,
    pub format: PlotFormat,
    pub width_cm: f64,
    pub height_cm: f64,
    pub keep_svg: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            format: PlotFormat::Pdf,
            width_cm: 10.0,
            height_cm: 10.0,
            keep_svg: false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ConvertArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    pub min_depth: u64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ConvertGffArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    pub features: Vec<String>,
    pub id_attribute: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MergeFiguresArgs {
    pub inputs: Vec<PathBuf>,
    pub output: PathBuf,
    pub labels: Vec<String>,
    pub ncol: usize,
    pub base_height_cm: f64,
    pub base_aspect_ratio: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct GlobalMethLevArgs {
    pub samples: Vec<SampleSpec>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub depth: DepthFilter,
    pub method_average: bool,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BisNonConvRateArgs {
    pub samples: Vec<SampleSpec>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub chrom: String,
    pub contexts: Vec<Context>,
    pub depth: DepthFilter,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethLevDistArgs {
    pub samples: Vec<SampleSpec>,
    pub regions: Option<PathBuf>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub depth: DepthFilter,
    pub bin_meth_lev: f64,
    pub method_average: bool,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethCoverageArgs {
    pub samples: Vec<SampleSpec>,
    pub reference: PathBuf,
    pub outdir: PathBuf,
    pub prefix: String,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethOneRegionArgs {
    pub samples: Vec<SampleSpec>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub region: String,
    pub flank: u64,
    pub contexts: Vec<Context>,
    pub depth: DepthFilter,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethGenoArgs {
    pub samples: Vec<SampleSpec>,
    pub genome_length: PathBuf,
    pub outdir: PathBuf,
    pub prefix: String,
    pub contexts: Vec<Context>,
    pub depth: DepthFilter,
    pub win: u64,
    pub step: u64,
    pub min_length: u64,
    pub max_chrom_number: usize,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethHeatmapArgs {
    pub samples: Vec<SampleSpec>,
    pub region: Option<PathBuf>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub contexts: Vec<Context>,
    pub depth: DepthFilter,
    pub merge: bool,
    pub cluster_rows: bool,
    pub cluster_cols: bool,
    pub random_region: usize,
    pub plot: PlotOptions,
    pub distribution_plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethOverRegionArgs {
    pub samples: Vec<SampleSpec>,
    pub region: Option<PathBuf>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub contexts: Vec<Context>,
    pub depth: DepthFilter,
    pub flank: u64,
    pub bin_length: u64,
    pub bin_number: u64,
    pub min_length: u64,
    pub max_length: u64,
    pub region_name: String,
    pub plot: PlotOptions,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub tables: Vec<OutputFile>,
    pub plots: Vec<OutputFile>,
    pub logs: Vec<LogMessage>,
    pub summary: CommandSummary,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CommandSummary {
    pub command: String,
    pub samples: usize,
    pub records_read: u64,
    pub records_used: u64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct OutputFile {
    pub kind: OutputKind,
    pub path: PathBuf,
    pub format: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum OutputKind {
    Table,
    Plot,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub command: String,
    pub phase: String,
    pub sample: Option<String>,
    pub processed: u64,
    pub total: Option<u64>,
    pub message: String,
}

pub trait ProgressReporter: Send + Sync {
    fn report(&self, event: ProgressEvent);
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

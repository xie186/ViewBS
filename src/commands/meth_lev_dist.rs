use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethLevDistArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::methyl_report::MethylReportReader;
use crate::io::regions::{read_region_file, GenomicRegion};
use crate::io::tabix::IndexedMethylReader;
use crate::meth::Context;

type BinCounts = HashMap<String, HashMap<Context, Vec<u64>>>;
type Totals = HashMap<String, HashMap<Context, u64>>;

#[derive(Clone, Copy, Debug, Default)]
struct RegionStats {
    methylated: u64,
    depth: u64,
    sites: u64,
    sum_levels: f64,
}

impl RegionStats {
    fn add(&mut self, methylated: u64, depth: u64) {
        self.methylated += methylated;
        self.depth += depth;
        self.sites += 1;
        if depth > 0 {
            self.sum_levels += methylated as f64 / depth as f64;
        }
    }

    fn level(self, method_average: bool) -> Option<f64> {
        if self.sites == 0 {
            None
        } else if method_average {
            Some(self.sum_levels / self.sites as f64)
        } else if self.depth == 0 {
            None
        } else {
            Some(self.methylated as f64 / self.depth as f64)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MethLevDistRow {
    pub sample: String,
    pub context: Context,
    pub midpoint: f64,
    pub number: u64,
    pub percentage: f64,
}

#[derive(Debug)]
pub(crate) struct MethLevDistTable {
    pub rows: Vec<MethLevDistRow>,
    pub path: PathBuf,
}

pub fn meth_lev_dist(
    args: MethLevDistArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethLevDist requires at least one sample",
        ));
    }
    if !(args.bin_meth_lev > 0.0 && args.bin_meth_lev <= 1.0) {
        return Err(ViewBsError::invalid_input(
            "<binMethLev>",
            "MethLevDist requires 0 < --binMethLev <= 1",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;

    let bin_count = (1.0 / args.bin_meth_lev).floor() as usize;
    if bin_count == 0 {
        return Err(ViewBsError::invalid_input(
            "<binMethLev>",
            "bin count must be greater than zero",
        ));
    }

    let mut state = DistributionState::new();
    if let Some(region_path) = &args.regions {
        let regions = read_region_file(region_path)?;
        collect_region_distribution(&args, &regions, bin_count, &mut state, progress, cancel)?;
    } else {
        collect_genome_distribution(&args, bin_count, &mut state, progress, cancel)?;
    }

    let contexts = state.contexts.into_iter().collect::<Vec<_>>();
    let table_path = args.outdir.join(format!("{}.tab", args.prefix));
    let table = build_table(
        table_path,
        &state.sample_order,
        &contexts,
        &state.counts,
        &state.totals,
        args.bin_meth_lev,
        bin_count,
    )?;
    write_table(&table)?;

    let mut plots = Vec::new();
    if args.plot.enabled {
        if let Some(plot_file) = crate::plot::meth_lev_dist::write_plot(&table, &args.plot)? {
            plots.push(plot_file);
        }
    }

    Ok(CommandOutput {
        tables: vec![OutputFile {
            kind: OutputKind::Table,
            path: table.path,
            format: "tab".to_string(),
        }],
        plots,
        logs: Vec::new(),
        summary: CommandSummary {
            command: "MethLevDist".to_string(),
            samples: args.samples.len(),
            records_read: state.records_read,
            records_used: state.records_used,
        },
    })
}

struct DistributionState {
    sample_order: Vec<String>,
    contexts: BTreeSet<Context>,
    counts: BinCounts,
    totals: Totals,
    records_read: u64,
    records_used: u64,
}

impl DistributionState {
    fn new() -> Self {
        Self {
            sample_order: Vec::new(),
            contexts: BTreeSet::new(),
            counts: HashMap::new(),
            totals: HashMap::new(),
            records_read: 0,
            records_used: 0,
        }
    }
}

fn collect_genome_distribution(
    args: &MethLevDistArgs,
    bin_count: usize,
    state: &mut DistributionState,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<()> {
    let mut contexts = BTreeSet::new();

    for sample in &args.samples {
        check_cancel(cancel)?;
        state.sample_order.push(sample.name.clone());
        report(
            progress,
            "MethLevDist",
            "read",
            Some(sample.name.clone()),
            state.records_read,
            None,
            format!("reading {}", sample.path.display()),
        );

        for record in MethylReportReader::from_path(&sample.path)? {
            check_cancel(cancel)?;
            let record = record?;
            state.records_read += 1;
            let depth = record.depth();
            if !args.depth.contains(depth) || depth == 0 {
                continue;
            }
            state.records_used += 1;
            let level = record.methylated as f64 / depth as f64;
            let bin = level_to_bin(level, args.bin_meth_lev, bin_count);
            let context = record.context;
            contexts.insert(context.clone());
            let by_context = state.counts.entry(sample.name.clone()).or_default();
            let bins = by_context
                .entry(context.clone())
                .or_insert_with(|| vec![0; bin_count]);
            bins[bin] += 1;
            *state
                .totals
                .entry(sample.name.clone())
                .or_default()
                .entry(context)
                .or_default() += 1;
        }
    }
    state.contexts.extend(contexts);
    Ok(())
}

fn collect_region_distribution(
    args: &MethLevDistArgs,
    regions: &[GenomicRegion],
    bin_count: usize,
    state: &mut DistributionState,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<()> {
    for sample in &args.samples {
        check_cancel(cancel)?;
        state.sample_order.push(sample.name.clone());
        let mut methyl_reader = IndexedMethylReader::from_path(&sample.path)?;
        for region in regions {
            check_cancel(cancel)?;
            let query_region = region.to_tabix_region();
            report(
                progress,
                "MethLevDist",
                "query",
                Some(sample.name.clone()),
                state.records_read,
                None,
                format!("querying {} in {}", query_region, sample.path.display()),
            );
            let mut stats_by_context: HashMap<Context, RegionStats> = HashMap::new();
            for record in methyl_reader.query(&query_region)? {
                state.records_read += 1;
                let depth = record.depth();
                if !args.depth.contains(depth) || depth == 0 {
                    continue;
                }
                state.records_used += 1;
                stats_by_context
                    .entry(record.context)
                    .or_default()
                    .add(record.methylated, depth);
            }

            for (context, stats) in stats_by_context {
                let Some(level) = stats.level(args.method_average) else {
                    continue;
                };
                state.contexts.insert(context.clone());
                let bin = level_to_bin(level, args.bin_meth_lev, bin_count);
                let bins = state
                    .counts
                    .entry(sample.name.clone())
                    .or_default()
                    .entry(context.clone())
                    .or_insert_with(|| vec![0; bin_count]);
                bins[bin] += 1;
                *state
                    .totals
                    .entry(sample.name.clone())
                    .or_default()
                    .entry(context)
                    .or_default() += 1;
            }
        }
    }
    Ok(())
}

fn level_to_bin(level: f64, bin_meth_lev: f64, bin_count: usize) -> usize {
    if level == 1.0 {
        bin_count - 1
    } else {
        ((level / bin_meth_lev).floor() as usize).min(bin_count - 1)
    }
}

fn build_table(
    path: PathBuf,
    sample_order: &[String],
    contexts: &[Context],
    counts: &BinCounts,
    totals: &Totals,
    bin_meth_lev: f64,
    bin_count: usize,
) -> Result<MethLevDistTable> {
    let mut rows = Vec::new();
    for sample in sample_order {
        for context in contexts {
            let total = totals
                .get(sample)
                .and_then(|by_context| by_context.get(context))
                .copied()
                .unwrap_or_default();
            for i in 0..bin_count {
                let number = counts
                    .get(sample)
                    .and_then(|by_context| by_context.get(context))
                    .and_then(|bins| bins.get(i))
                    .copied()
                    .unwrap_or_default();
                let midpoint = bin_meth_lev * (i + 1) as f64 - bin_meth_lev / 2.0;
                let percentage = if total == 0 {
                    0.0
                } else {
                    100.0 * number as f64 / total as f64
                };
                rows.push(MethLevDistRow {
                    sample: sample.clone(),
                    context: context.clone(),
                    midpoint,
                    number,
                    percentage,
                });
            }
        }
    }
    Ok(MethLevDistTable { rows, path })
}

fn write_table(table: &MethLevDistTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record([
            "Sample",
            "Context",
            "MethLevBinMidPoint",
            "Number",
            "Percentage",
        ])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        writer
            .write_record([
                row.sample.as_str(),
                row.context.as_str(),
                &row.midpoint.to_string(),
                &row.number.to_string(),
                &row.percentage.to_string(),
            ])
            .map_err(|source| ViewBsError::csv(&table.path, source))?;
    }
    writer
        .flush()
        .map_err(|source| ViewBsError::io(&table.path, source))
}

fn check_cancel(cancel: Option<&CancellationToken>) -> Result<()> {
    if cancel.is_some_and(CancellationToken::is_cancelled) {
        Err(ViewBsError::Cancelled)
    } else {
        Ok(())
    }
}

fn report(
    progress: Option<&dyn ProgressReporter>,
    command: &str,
    phase: &str,
    sample: Option<String>,
    processed: u64,
    total: Option<u64>,
    message: String,
) {
    if let Some(progress) = progress {
        progress.report(ProgressEvent {
            command: command.to_string(),
            phase: phase.to_string(),
            sample,
            processed,
            total,
            message,
        });
    }
}

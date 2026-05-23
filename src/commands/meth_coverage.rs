use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethCoverageArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::fasta::{count_reference_contexts, ensure_reference_exists};
use crate::io::methyl_report::MethylReportReader;
use crate::meth::Context;

type DepthCounts = HashMap<String, HashMap<Context, HashMap<u64, u64>>>;

#[derive(Debug, Clone)]
pub(crate) struct CoverageRow {
    pub sample: String,
    pub context: Context,
    pub depth: u64,
    pub percentage: f64,
}

#[derive(Debug)]
pub(crate) struct CoverageTable {
    pub rows: Vec<CoverageRow>,
    pub path: PathBuf,
}

pub fn meth_coverage(
    args: MethCoverageArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethCoverage requires at least one sample",
        ));
    }
    let reference = ensure_reference_exists(&args.reference)?;
    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;

    let mut sample_order = Vec::new();
    let mut contexts = BTreeSet::new();
    let mut depth_counts: DepthCounts = HashMap::new();
    let mut all_cg_depth_counts: HashMap<u64, u64> = HashMap::new();
    let mut observed_context_counts: HashMap<Context, u64> = HashMap::new();
    let mut min_depth_seen: Option<u64> = None;
    let mut max_depth = 0_u64;
    let mut records_read = 0_u64;

    for sample in &args.samples {
        check_cancel(cancel)?;
        sample_order.push(sample.name.clone());
        report(
            progress,
            "MethCoverage",
            "read",
            Some(sample.name.clone()),
            records_read,
            None,
            format!("reading {}", sample.path.display()),
        );

        for record in MethylReportReader::from_path(&sample.path)? {
            check_cancel(cancel)?;
            let record = record?;
            records_read += 1;
            let depth = record.depth();
            min_depth_seen = Some(min_depth_seen.map_or(depth, |min_depth| min_depth.min(depth)));
            max_depth = max_depth.max(depth);
            contexts.insert(record.context.clone());
            *observed_context_counts
                .entry(record.context.clone())
                .or_default() += 1;
            *depth_counts
                .entry(sample.name.clone())
                .or_default()
                .entry(record.context.clone())
                .or_default()
                .entry(depth)
                .or_default() += 1;
            if record.context == Context::Cg {
                *all_cg_depth_counts.entry(depth).or_default() += 1;
            }
        }
    }

    let ref_counts = if min_depth_seen == Some(0) {
        observed_context_counts
    } else {
        count_reference_contexts(&reference)?.as_context_map()
    };
    let max_depth_report = determine_max_depth(
        max_depth,
        args.samples.len() as u64,
        &all_cg_depth_counts,
        ref_counts.get(&Context::Cg).copied().unwrap_or_default(),
    );

    let contexts = contexts.into_iter().collect::<Vec<_>>();
    let table_path = args.outdir.join(format!("{}.tab", args.prefix));
    let table = build_table(
        table_path,
        &sample_order,
        &contexts,
        &depth_counts,
        &ref_counts,
        max_depth,
        max_depth_report,
    )?;
    write_table(&table)?;

    let mut plots = Vec::new();
    if args.plot.enabled {
        if let Some(plot_file) = crate::plot::meth_coverage::write_plot(&table, &args.plot)? {
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
            command: "MethCoverage".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used: records_read,
        },
    })
}

fn determine_max_depth(
    max_depth: u64,
    sample_count: u64,
    all_cg_depth_counts: &HashMap<u64, u64>,
    reference_cg_count: u64,
) -> u64 {
    if max_depth == 0 || sample_count == 0 || reference_cg_count == 0 {
        return max_depth;
    }

    for depth in 1..=max_depth {
        let covered = cumulative_covered(all_cg_depth_counts, depth, max_depth);
        let fraction = covered as f64 / (reference_cg_count * sample_count) as f64;
        if fraction < 0.1 {
            return depth;
        }
    }
    max_depth
}

fn build_table(
    path: PathBuf,
    sample_order: &[String],
    contexts: &[Context],
    depth_counts: &DepthCounts,
    ref_counts: &HashMap<Context, u64>,
    max_depth: u64,
    max_depth_report: u64,
) -> Result<CoverageTable> {
    let mut rows = Vec::new();
    for sample in sample_order {
        for context in contexts {
            let denominator = ref_counts.get(context).copied().unwrap_or_default();
            for depth in 1..=max_depth_report {
                let covered = depth_counts
                    .get(sample)
                    .and_then(|by_context| by_context.get(context))
                    .map(|counts| cumulative_covered(counts, depth, max_depth))
                    .unwrap_or_default();
                let percentage = if denominator == 0 {
                    0.0
                } else {
                    100.0 * covered as f64 / denominator as f64
                };
                rows.push(CoverageRow {
                    sample: sample.clone(),
                    context: context.clone(),
                    depth,
                    percentage,
                });
            }
        }
    }
    Ok(CoverageTable { rows, path })
}

fn write_table(table: &CoverageTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record(["Sample", "Context", "Depth", "Percentage"])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        writer
            .write_record([
                row.sample.as_str(),
                row.context.as_str(),
                &row.depth.to_string(),
                &row.percentage.to_string(),
            ])
            .map_err(|source| ViewBsError::csv(&table.path, source))?;
    }

    writer
        .flush()
        .map_err(|source| ViewBsError::io(&table.path, source))
}

fn cumulative_covered(counts: &HashMap<u64, u64>, min_depth: u64, max_depth: u64) -> u64 {
    (min_depth..=max_depth)
        .map(|depth| counts.get(&depth).copied().unwrap_or_default())
        .sum()
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

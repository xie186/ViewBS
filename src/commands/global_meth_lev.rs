use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, GlobalMethLevArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::methyl_report::MethylReportReader;
use crate::meth::Context;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ContextStats {
    methylated: u64,
    depth: u64,
    sites: u64,
    sum_levels: f64,
}

impl ContextStats {
    fn add(&mut self, methylated: u64, depth: u64) {
        self.methylated += methylated;
        self.depth += depth;
        self.sites += 1;
        if depth > 0 {
            self.sum_levels += methylated as f64 / depth as f64;
        }
    }

    fn level(self, method_average: bool) -> f64 {
        if method_average {
            if self.sites == 0 {
                0.0
            } else {
                self.sum_levels / self.sites as f64
            }
        } else if self.depth == 0 {
            0.0
        } else {
            self.methylated as f64 / self.depth as f64
        }
    }
}

#[derive(Debug)]
pub(crate) struct GlobalMethLevTable {
    pub sample_order: Vec<String>,
    pub contexts: Vec<Context>,
    pub values: HashMap<String, HashMap<Context, f64>>,
    pub path: PathBuf,
}

pub fn global_meth_lev(
    args: GlobalMethLevArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "GlobalMethLev requires at least one sample",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;

    let mut sample_order = Vec::new();
    let mut context_order = BTreeSet::new();
    let mut stats_by_sample: HashMap<String, HashMap<Context, ContextStats>> = HashMap::new();
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;

    for sample in &args.samples {
        check_cancel(cancel)?;
        sample_order.push(sample.name.clone());
        report(
            progress,
            "GlobalMethLev",
            "read",
            Some(sample.name.clone()),
            records_read,
            None,
            format!("reading {}", sample.path.display()),
        );

        let mut reader = MethylReportReader::from_path(&sample.path)?;
        for record in &mut reader {
            check_cancel(cancel)?;
            let record = record?;
            records_read += 1;
            let depth = record.depth();
            if !args.depth.contains(depth) {
                continue;
            }
            records_used += 1;
            context_order.insert(record.context.clone());
            stats_by_sample
                .entry(sample.name.clone())
                .or_default()
                .entry(record.context)
                .or_default()
                .add(record.methylated, depth);
        }
    }

    let contexts = context_order.into_iter().collect::<Vec<_>>();
    let values = stats_by_sample
        .into_iter()
        .map(|(sample, stats)| {
            let levels = stats
                .into_iter()
                .map(|(context, stats)| (context, stats.level(args.method_average)))
                .collect::<HashMap<_, _>>();
            (sample, levels)
        })
        .collect::<HashMap<_, _>>();

    let table_path = args.outdir.join(format!("{}.tab", args.prefix));
    let table = GlobalMethLevTable {
        sample_order,
        contexts,
        values,
        path: table_path,
    };
    write_table(&table)?;

    let mut plots = Vec::new();
    if args.plot.enabled {
        if let Some(plot_file) = crate::plot::global_meth_lev::write_plot(&table, &args.plot)? {
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
            command: "GlobalMethLev".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used,
        },
    })
}

fn write_table(table: &GlobalMethLevTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    let mut header = vec!["Sample".to_string()];
    header.extend(
        table
            .contexts
            .iter()
            .map(|context| context.as_str().to_string()),
    );
    writer
        .write_record(header)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for sample in &table.sample_order {
        let mut row = vec![sample.clone()];
        for context in &table.contexts {
            let value = table
                .values
                .get(sample)
                .and_then(|contexts| contexts.get(context))
                .copied()
                .unwrap_or(0.0);
            row.push(format!("{value:.3}"));
        }
        writer
            .write_record(row)
            .map_err(|source| ViewBsError::csv(&table.path, source))?;
    }
    writer
        .flush()
        .map_err(|source| ViewBsError::io(&table.path, source))?;
    Ok(())
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

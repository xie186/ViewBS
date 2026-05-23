use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::api::{
    BisNonConvRateArgs, CancellationToken, CommandOutput, CommandSummary, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::methyl_report::MethylReportReader;
use crate::meth::Context;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BisNonConvRateValue {
    pub methylated: u64,
    pub depth: u64,
}

impl BisNonConvRateValue {
    fn add(&mut self, methylated: u64, depth: u64) {
        self.methylated += methylated;
        self.depth += depth;
    }

    pub(crate) fn rate(self) -> f64 {
        if self.depth == 0 {
            0.0
        } else {
            self.methylated as f64 / self.depth as f64
        }
    }
}

#[derive(Debug)]
pub(crate) struct BisNonConvRateTable {
    pub sample_order: Vec<String>,
    pub contexts: Vec<Context>,
    pub values: HashMap<String, HashMap<Context, BisNonConvRateValue>>,
    pub path: PathBuf,
}

pub fn bis_non_conv_rate(
    args: BisNonConvRateArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "BisNonConvRate requires at least one sample",
        ));
    }
    if args.chrom.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<chrom>",
            "BisNonConvRate requires --chrom",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;

    let contexts = if args.contexts.is_empty() {
        vec![Context::Cxx]
    } else {
        args.contexts.clone()
    };
    let all_contexts = contexts.len() == 1 && contexts[0] == Context::Cxx;
    let requested = contexts.iter().cloned().collect::<HashSet<_>>();
    let mut sample_order = Vec::new();
    let mut totals: HashMap<String, HashMap<Context, BisNonConvRateValue>> = HashMap::new();
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;

    for sample in &args.samples {
        check_cancel(cancel)?;
        sample_order.push(sample.name.clone());
        report(
            progress,
            "BisNonConvRate",
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

            if record.chrom != args.chrom {
                continue;
            }
            let depth = record.depth();
            if !args.depth.contains(depth) {
                continue;
            }

            let context = if all_contexts {
                Context::Cxx
            } else {
                record.context
            };
            if !all_contexts && !requested.contains(&context) {
                continue;
            }

            records_used += 1;
            totals
                .entry(sample.name.clone())
                .or_default()
                .entry(context)
                .or_default()
                .add(record.methylated, depth);
        }
    }

    let table_path = args.outdir.join(format!("{}.tab", args.prefix));
    let table = BisNonConvRateTable {
        sample_order,
        contexts,
        values: totals,
        path: table_path,
    };
    write_table(&table)?;

    let mut plots = Vec::new();
    if args.plot.enabled {
        if let Some(plot_file) = crate::plot::bis_non_conv_rate::write_plot(&table, &args.plot)? {
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
            command: "BisNonConvRate".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used,
        },
    })
}

fn write_table(table: &BisNonConvRateTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record([
            "Sample",
            "BisNonConvRate",
            "C_number",
            "Total_Depth",
            "Context",
        ])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for context in &table.contexts {
        for sample in &table.sample_order {
            let value = table
                .values
                .get(sample)
                .and_then(|by_context| by_context.get(context))
                .copied()
                .unwrap_or_default();
            writer
                .write_record([
                    sample.as_str(),
                    &format!("{:.5}", value.rate()),
                    &value.methylated.to_string(),
                    &value.depth.to_string(),
                    context.as_str(),
                ])
                .map_err(|source| ViewBsError::csv(&table.path, source))?;
        }
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

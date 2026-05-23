use std::fs;
use std::path::PathBuf;

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethOneRegionArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::regions::GenomicRegion;
use crate::io::tabix::IndexedMethylReader;
use crate::meth::Context;

#[derive(Debug, Clone)]
pub(crate) struct MethOneRegionRow {
    pub sample: String,
    pub chrom: String,
    pub position: u64,
    pub methylated: u64,
    pub unmethylated: u64,
    pub level: f64,
}

#[derive(Debug)]
pub(crate) struct MethOneRegionTable {
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub context: Context,
    pub rows: Vec<MethOneRegionRow>,
    pub path: PathBuf,
}

pub fn meth_one_region(
    args: MethOneRegionArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethOneRegion requires at least one sample",
        ));
    }
    let region = args
        .region
        .parse::<GenomicRegion>()
        .map_err(|message| ViewBsError::invalid_input("<region>", message))?
        .with_flank(args.flank);
    let query_region = region.to_tabix_region();
    let contexts = if args.contexts.is_empty() {
        vec![Context::Cg]
    } else {
        args.contexts.clone()
    };

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;

    let mut tables = Vec::new();
    let mut plots = Vec::new();
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;

    for context in &contexts {
        check_cancel(cancel)?;
        let table_path = args.outdir.join(format!(
            "{}_MethOneRegion_{}.txt",
            args.prefix,
            context.as_str()
        ));
        let mut table = MethOneRegionTable {
            context: context.clone(),
            rows: Vec::new(),
            path: table_path,
        };

        for sample in &args.samples {
            check_cancel(cancel)?;
            let mut methyl_reader = IndexedMethylReader::from_path(&sample.path)?;
            report(
                progress,
                "MethOneRegion",
                "query",
                Some(sample.name.clone()),
                records_read,
                None,
                format!("querying {} in {}", query_region, sample.path.display()),
            );

            for record in methyl_reader.query(&query_region)? {
                check_cancel(cancel)?;
                records_read += 1;
                if !matches_context(context, &record.context) {
                    continue;
                }
                let depth = record.depth();
                if !args.depth.contains(depth) || depth == 0 {
                    continue;
                }
                records_used += 1;
                table.rows.push(MethOneRegionRow {
                    sample: sample.name.clone(),
                    chrom: record.chrom,
                    position: record.pos,
                    methylated: record.methylated,
                    unmethylated: record.unmethylated,
                    level: record.methylated as f64 / depth as f64,
                });
            }
        }

        write_table(&table)?;
        if args.plot.enabled {
            if let Some(plot_file) = crate::plot::meth_one_region::write_plot(&table, &args.plot)? {
                plots.push(plot_file);
            }
        }
        tables.push(OutputFile {
            kind: OutputKind::Table,
            path: table.path,
            format: "txt".to_string(),
        });
    }

    Ok(CommandOutput {
        tables,
        plots,
        logs: Vec::new(),
        summary: CommandSummary {
            command: "MethOneRegion".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used,
        },
    })
}

fn write_table(table: &MethOneRegionTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record([
            "Sample",
            "chr",
            "position",
            "C_num",
            "T_num",
            "MethylationLevel",
        ])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        writer
            .write_record([
                row.sample.as_str(),
                row.chrom.as_str(),
                &row.position.to_string(),
                &row.methylated.to_string(),
                &row.unmethylated.to_string(),
                &row.level.to_string(),
            ])
            .map_err(|source| ViewBsError::csv(&table.path, source))?;
    }
    writer
        .flush()
        .map_err(|source| ViewBsError::io(&table.path, source))
}

fn matches_context(requested: &Context, observed: &Context) -> bool {
    requested == &Context::Cxx || requested == observed
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

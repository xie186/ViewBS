use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethGenoArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter,
};
use crate::errors::{Result, ViewBsError};
use crate::io::tabix::IndexedMethylReader;
use crate::meth::Context;

#[derive(Debug, Clone)]
pub(crate) struct MethGenoRow {
    pub chrom: String,
    pub start: u64,
    pub end: u64,
    pub sample: String,
    pub methylated: u64,
    pub unmethylated: u64,
    pub level: f64,
}

#[derive(Debug)]
pub(crate) struct MethGenoTable {
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub context: Context,
    pub rows: Vec<MethGenoRow>,
    pub path: PathBuf,
}

#[derive(Debug)]
struct ChromLength {
    chrom: String,
    length: u64,
}

pub fn meth_geno(
    args: MethGenoArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethGeno requires at least one sample",
        ));
    }
    if args.win == 0 {
        return Err(ViewBsError::invalid_input(
            "<win>",
            "MethGeno requires --win > 0",
        ));
    }
    if args.step == 0 {
        return Err(ViewBsError::invalid_input(
            "<step>",
            "MethGeno requires --step > 0",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;
    let chrom_lengths =
        read_genome_lengths(&args.genome_length, args.min_length, args.max_chrom_number)?;
    let contexts = if args.contexts.is_empty() {
        vec![Context::Cg]
    } else {
        args.contexts.clone()
    };

    let mut tables = Vec::new();
    let mut plots = Vec::new();
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;

    for context in &contexts {
        check_cancel(cancel)?;
        let table_path =
            args.outdir
                .join(format!("{}_MethGeno_{}.txt", args.prefix, context.as_str()));
        let mut table = MethGenoTable {
            context: context.clone(),
            rows: Vec::new(),
            path: table_path,
        };

        for sample in &args.samples {
            let mut methyl_reader = IndexedMethylReader::from_path(&sample.path)?;
            for chrom_length in &chrom_lengths {
                for (start, end) in windows(chrom_length.length, args.win, args.step) {
                    check_cancel(cancel)?;
                    let region = format!("{}:{}-{}", chrom_length.chrom, start, end);
                    report(
                        progress,
                        "MethGeno",
                        "query",
                        Some(sample.name.clone()),
                        records_read,
                        None,
                        format!("querying {} in {}", region, sample.path.display()),
                    );
                    let mut methylated = 0_u64;
                    let mut unmethylated = 0_u64;
                    for record in methyl_reader.query(&region)? {
                        records_read += 1;
                        if !matches_context(context, &record.context) {
                            continue;
                        }
                        let depth = record.depth();
                        if !args.depth.contains(depth) {
                            continue;
                        }
                        records_used += 1;
                        methylated += record.methylated;
                        unmethylated += record.unmethylated;
                    }
                    let level = if methylated + unmethylated == 0 {
                        0.0
                    } else {
                        methylated as f64 / ((methylated + unmethylated) as f64 + 0.000000001)
                    };
                    table.rows.push(MethGenoRow {
                        chrom: chrom_length.chrom.clone(),
                        start,
                        end,
                        sample: sample.name.clone(),
                        methylated,
                        unmethylated,
                        level,
                    });
                }
            }
        }

        write_table(&table)?;
        if args.plot.enabled {
            if let Some(plot_file) = crate::plot::meth_geno::write_plot(&table, &args.plot)? {
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
            command: "MethGeno".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used,
        },
    })
}

fn read_genome_lengths(
    path: &Path,
    min_length: u64,
    max_chrom_number: usize,
) -> Result<Vec<ChromLength>> {
    let file = File::open(path).map_err(|source| ViewBsError::io(path, source))?;
    let reader = BufReader::new(file);
    let mut lengths = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| ViewBsError::io(path, source))?;
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 2 {
            return Err(ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                "expected chromosome and length columns",
            ));
        }
        let length = columns[1].parse::<u64>().map_err(|source| {
            ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                format!("invalid chromosome length `{}`: {source}", columns[1]),
            )
        })?;
        if length >= min_length {
            lengths.push(ChromLength {
                chrom: columns[0].to_string(),
                length,
            });
        }
    }

    if lengths.len() > max_chrom_number {
        return Err(ViewBsError::invalid_input(
            path,
            format!(
                "there are too many chromosomes ({}) with length >= minLength {}; maxChromNumber is {}",
                lengths.len(),
                min_length,
                max_chrom_number
            ),
        ));
    }

    lengths.sort_by(|a, b| a.chrom.cmp(&b.chrom));
    Ok(lengths)
}

fn windows(chrom_length: u64, win: u64, step: u64) -> impl Iterator<Item = (u64, u64)> {
    let limit = chrom_length as f64 / step as f64 - 1.0;
    let mut i = 1_u64;
    std::iter::from_fn(move || loop {
        if (i as f64) > limit {
            return None;
        }
        let start = (i - 1) * step + 1;
        let end = (i - 1) * step + win;
        i += 1;
        if end > chrom_length && (end - chrom_length) as f64 / (win as f64) < 0.5 {
            continue;
        }
        return Some((start, end));
    })
}

fn write_table(table: &MethGenoTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record([
            "chr",
            "stt",
            "end",
            "sample_name",
            "C_number",
            "T_number",
            "Methylation_level",
        ])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        writer
            .write_record([
                row.chrom.as_str(),
                &row.start.to_string(),
                &row.end.to_string(),
                row.sample.as_str(),
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

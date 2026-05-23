use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethOverRegionArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter, SampleSpec,
};
use crate::errors::{Result, ViewBsError};
use crate::io::regions::{read_region_records, RegionRecord};
use crate::io::tabix::IndexedMethylReader;
use crate::meth::Context;

const UPSTREAM: &str = "Upstream";
const BODY: &str = "Body";
const DOWNSTREAM: &str = "Downstream";

#[derive(Debug, Clone)]
pub(crate) struct MethOverRegionRow {
    pub sample: String,
    pub region: String,
    pub bin: i64,
    pub methylated: u64,
    pub unmethylated: u64,
    pub level: f64,
}

#[derive(Debug)]
pub(crate) struct MethOverRegionTable {
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub context: Context,
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub region_name: String,
    pub rows: Vec<MethOverRegionRow>,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Default)]
struct Totals {
    methylated: u64,
    unmethylated: u64,
}

impl Totals {
    fn add(&mut self, methylated: u64, unmethylated: u64) {
        self.methylated += methylated;
        self.unmethylated += unmethylated;
    }

    fn level(self) -> f64 {
        self.methylated as f64 / (self.methylated + self.unmethylated) as f64
    }
}

pub fn meth_over_region(
    args: MethOverRegionArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethOverRegion requires at least one sample",
        ));
    }
    if args.bin_length == 0 {
        return Err(ViewBsError::invalid_input(
            "<binLength>",
            "MethOverRegion requires --binLength > 0",
        ));
    }
    if args.bin_number == 0 {
        return Err(ViewBsError::invalid_input(
            "<binNumber>",
            "MethOverRegion requires --binNumber > 0",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;
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
        let table_path = args.outdir.join(format!(
            "{}_MethOverRegion_{}.txt",
            args.prefix,
            context.as_str()
        ));
        let mut table = MethOverRegionTable {
            context: context.clone(),
            region_name: args.region_name.clone(),
            rows: Vec::new(),
            path: table_path,
        };
        let mut totals_by_key = BTreeMap::<String, Totals>::new();

        for sample in &args.samples {
            check_cancel(cancel)?;
            let region_path = region_path_for_sample(sample, &args)?;
            let regions = read_region_records(region_path)?;
            let mut methyl_reader = IndexedMethylReader::from_path(&sample.path)?;
            for region in regions {
                let region_len = region.len();
                if region_len < args.min_length || region_len > args.max_length {
                    continue;
                }
                check_cancel(cancel)?;
                let query_region = query_region(&region, args.flank);
                report(
                    progress,
                    "MethOverRegion",
                    "query",
                    Some(sample.name.clone()),
                    records_read,
                    None,
                    format!("querying {} in {}", query_region, sample.path.display()),
                );
                for record in methyl_reader.query(&query_region)? {
                    records_read += 1;
                    if !matches_context(context, &record.context) {
                        continue;
                    }
                    let depth = record.depth();
                    if !args.depth.contains(depth) || depth == 0 {
                        continue;
                    }
                    records_used += 1;
                    let (segment, bin) = judge_bin(
                        &region,
                        record.pos,
                        args.bin_length,
                        args.bin_number,
                        args.flank,
                    );
                    let key = format!("{}\t{}\t{}", sample.name, segment, bin);
                    totals_by_key
                        .entry(key)
                        .or_default()
                        .add(record.methylated, record.unmethylated);
                }
            }
        }

        for (key, totals) in totals_by_key {
            let mut columns = key.split('\t');
            let sample = columns.next().unwrap_or_default().to_string();
            let region = columns.next().unwrap_or_default().to_string();
            let bin = columns
                .next()
                .unwrap_or_default()
                .parse::<i64>()
                .unwrap_or_default();
            table.rows.push(MethOverRegionRow {
                sample,
                region,
                bin,
                methylated: totals.methylated,
                unmethylated: totals.unmethylated,
                level: totals.level(),
            });
        }

        write_table(&table)?;
        if args.plot.enabled {
            if let Some(plot_file) = crate::plot::meth_over_region::write_plot(&table, &args.plot)?
            {
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
            command: "MethOverRegion".to_string(),
            samples: args.samples.len(),
            records_read,
            records_used,
        },
    })
}

fn region_path_for_sample<'a>(
    sample: &'a SampleSpec,
    args: &'a MethOverRegionArgs,
) -> Result<&'a std::path::Path> {
    sample
        .region_path
        .as_deref()
        .or(args.region.as_deref())
        .ok_or_else(|| {
            ViewBsError::invalid_input(
                "<region>",
                "MethOverRegion requires --region unless each sample has a region file",
            )
        })
}

fn query_region(region: &RegionRecord, flank: u64) -> String {
    let start = if region.start <= flank {
        1
    } else {
        region.start - flank + 2
    };
    let end = region
        .end
        .saturating_add(flank)
        .saturating_sub(2)
        .max(start);
    format!("{}:{}-{}", region.chrom, start, end)
}

fn judge_bin(
    region: &RegionRecord,
    pos: u64,
    bin_length: u64,
    bin_number: u64,
    flank: u64,
) -> (&'static str, i64) {
    let unit = (region.end - region.start + 1) as f64 / (bin_number as f64 - 0.01);
    if region.strand == '-' {
        judge_minus(region, pos, bin_length, bin_number, flank, unit)
    } else {
        judge_plus(region, pos, bin_length, bin_number, flank, unit)
    }
}

fn judge_plus(
    region: &RegionRecord,
    pos: u64,
    bin_length: u64,
    bin_number: u64,
    flank: u64,
    unit: f64,
) -> (&'static str, i64) {
    if pos < region.start {
        let flank_len = region.start - pos;
        let bin = if flank_len == flank {
            -(flank_len as i64 / bin_length as i64) + 1
        } else {
            -(flank_len as i64 / bin_length as i64)
        };
        (UPSTREAM, bin)
    } else if pos >= region.start && pos < region.end {
        let bin = ((pos - region.start + 1) as f64 / unit).floor() as i64 + 1;
        (BODY, bin)
    } else {
        let flank_len = pos - region.end;
        let bin = if flank_len == flank {
            flank_len as i64 / bin_length as i64 + bin_number as i64 - 1
        } else {
            flank_len as i64 / bin_length as i64 + bin_number as i64 + 1
        };
        (DOWNSTREAM, bin)
    }
}

fn judge_minus(
    region: &RegionRecord,
    pos: u64,
    bin_length: u64,
    bin_number: u64,
    flank: u64,
    unit: f64,
) -> (&'static str, i64) {
    if pos <= region.start {
        let flank_len = region.start - pos;
        let bin = if flank_len == flank {
            flank_len as i64 / bin_length as i64 + bin_number as i64 - 1
        } else {
            flank_len as i64 / bin_length as i64 + bin_number as i64 + 1
        };
        (DOWNSTREAM, bin)
    } else if pos > region.start && pos <= region.end {
        let bin = ((region.end - pos + 1) as f64 / unit).floor() as i64 + 1;
        (BODY, bin)
    } else {
        let flank_len = pos - region.end;
        let bin = if flank_len == flank {
            -(flank_len as i64 / bin_length as i64) + 1
        } else {
            -(flank_len as i64 / bin_length as i64)
        };
        (UPSTREAM, bin)
    }
}

fn write_table(table: &MethOverRegionTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;
    writer
        .write_record([
            "sample_name",
            "region",
            "bin_num",
            "C_number",
            "T_number",
            "Methylation_level",
        ])
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        writer
            .write_record([
                row.sample.as_str(),
                row.region.as_str(),
                &row.bin.to_string(),
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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::api::{
    CancellationToken, CommandOutput, CommandSummary, MethHeatmapArgs, OutputFile, OutputKind,
    ProgressEvent, ProgressReporter, SampleSpec,
};
use crate::errors::{Result, ViewBsError};
use crate::io::regions::{read_region_records, GenomicRegion};
use crate::io::tabix::IndexedMethylReader;
use crate::meth::Context;

#[derive(Debug, Clone)]
pub(crate) struct MethHeatmapRow {
    pub id: String,
    pub values: Vec<Option<f64>>,
}

#[derive(Debug)]
pub(crate) struct MethHeatmapTable {
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub label: String,
    pub columns: Vec<String>,
    pub rows: Vec<MethHeatmapRow>,
    pub path: PathBuf,
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub cluster_rows: bool,
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub cluster_cols: bool,
    #[cfg_attr(not(feature = "plots"), allow(dead_code))]
    pub random_region: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct RegionStats {
    methylated: u64,
    unmethylated: u64,
    used_sites: u64,
    read_records: u64,
}

impl RegionStats {
    fn add(&mut self, methylated: u64, unmethylated: u64) {
        self.methylated += methylated;
        self.unmethylated += unmethylated;
        self.used_sites += 1;
    }

    fn level(self) -> Option<f64> {
        if self.used_sites == 0 {
            None
        } else {
            Some(self.methylated as f64 / (self.methylated + self.unmethylated) as f64)
        }
    }
}

#[derive(Debug)]
struct RegionInput {
    id: String,
    region: GenomicRegion,
}

pub fn meth_heatmap(
    args: MethHeatmapArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput> {
    if args.samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "MethHeatmap requires at least one sample",
        ));
    }
    if args.random_region == 0 {
        return Err(ViewBsError::invalid_input(
            "<random_region>",
            "MethHeatmap requires --random_region > 0",
        ));
    }

    fs::create_dir_all(&args.outdir).map_err(|source| ViewBsError::io(&args.outdir, source))?;
    let contexts = if args.contexts.is_empty() {
        vec![Context::Cg]
    } else {
        args.contexts.clone()
    };

    let mut state = RunState {
        progress,
        cancel,
        records_read: 0,
        records_used: 0,
    };
    let mut table_outputs = Vec::new();
    let mut plots = Vec::new();

    if args.merge {
        let table = build_merged_table(&args, &contexts, &mut state)?;
        write_table(&table)?;
        if args.plot.enabled || args.distribution_plot.enabled {
            plots.extend(crate::plot::meth_heatmap::write_plots(
                &table,
                &args.plot,
                &args.distribution_plot,
            )?);
        }
        table_outputs.push(OutputFile {
            kind: OutputKind::Table,
            path: table.path,
            format: "txt".to_string(),
        });
    } else {
        for context in &contexts {
            state.check_cancel()?;
            let table = build_context_table(&args, context, &mut state)?;
            write_table(&table)?;
            if args.plot.enabled || args.distribution_plot.enabled {
                plots.extend(crate::plot::meth_heatmap::write_plots(
                    &table,
                    &args.plot,
                    &args.distribution_plot,
                )?);
            }
            table_outputs.push(OutputFile {
                kind: OutputKind::Table,
                path: table.path,
                format: "txt".to_string(),
            });
        }
    }

    Ok(CommandOutput {
        tables: table_outputs,
        plots,
        logs: Vec::new(),
        summary: CommandSummary {
            command: "MethHeatmap".to_string(),
            samples: args.samples.len(),
            records_read: state.records_read,
            records_used: state.records_used,
        },
    })
}

struct RunState<'a> {
    progress: Option<&'a dyn ProgressReporter>,
    cancel: Option<&'a CancellationToken>,
    records_read: u64,
    records_used: u64,
}

impl RunState<'_> {
    fn check_cancel(&self) -> Result<()> {
        if self.cancel.is_some_and(CancellationToken::is_cancelled) {
            Err(ViewBsError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn report(
        &self,
        phase: &str,
        sample: Option<String>,
        processed: u64,
        total: Option<u64>,
        message: String,
    ) {
        if let Some(progress) = self.progress {
            progress.report(ProgressEvent {
                command: "MethHeatmap".to_string(),
                phase: phase.to_string(),
                sample,
                processed,
                total,
                message,
            });
        }
    }
}

fn build_context_table(
    args: &MethHeatmapArgs,
    context: &Context,
    state: &mut RunState<'_>,
) -> Result<MethHeatmapTable> {
    let sample_names = args
        .samples
        .iter()
        .map(|sample| sample.name.clone())
        .collect::<Vec<_>>();
    let mut matrix = MatrixBuilder::new(sample_names.clone());

    for (sample_index, sample) in args.samples.iter().enumerate() {
        collect_sample_context(args, sample, sample_index, context, &mut matrix, state)?;
    }

    Ok(MethHeatmapTable {
        label: context.as_str().to_string(),
        columns: sample_names,
        rows: matrix.into_rows(),
        path: args.outdir.join(format!(
            "{}_MethHeatmap_{}.txt",
            args.prefix,
            context.as_str()
        )),
        cluster_rows: args.cluster_rows,
        cluster_cols: args.cluster_cols,
        random_region: args.random_region,
    })
}

fn build_merged_table(
    args: &MethHeatmapArgs,
    contexts: &[Context],
    state: &mut RunState<'_>,
) -> Result<MethHeatmapTable> {
    let mut columns = Vec::new();
    for context in contexts {
        for sample in &args.samples {
            columns.push(format!("{}-{}", sample.name, context.as_str()));
        }
    }
    let mut matrix = MatrixBuilder::new(columns.clone());

    for (context_index, context) in contexts.iter().enumerate() {
        for (sample_index, sample) in args.samples.iter().enumerate() {
            let column_index = context_index * args.samples.len() + sample_index;
            collect_sample_context(args, sample, column_index, context, &mut matrix, state)?;
        }
    }

    Ok(MethHeatmapTable {
        label: "mer".to_string(),
        columns,
        rows: matrix.into_rows(),
        path: args
            .outdir
            .join(format!("{}_MethHeatmap_mer.txt", args.prefix)),
        cluster_rows: args.cluster_rows,
        cluster_cols: args.cluster_cols,
        random_region: args.random_region,
    })
}

fn collect_sample_context(
    args: &MethHeatmapArgs,
    sample: &SampleSpec,
    column_index: usize,
    context: &Context,
    matrix: &mut MatrixBuilder,
    state: &mut RunState<'_>,
) -> Result<()> {
    state.check_cancel()?;
    let region_path = region_path_for_sample(sample, args)?;
    let regions = read_region_inputs(region_path)?;
    let mut methyl_reader = IndexedMethylReader::from_path(&sample.path)?;

    for (region_index, region) in regions.iter().enumerate() {
        state.check_cancel()?;
        let stats = query_region_stats(&mut methyl_reader, &region.region, context, args)?;
        state.records_read += stats.read_records;
        state.records_used += stats.used_sites;
        matrix.set(&region.id, column_index, stats.level());
        state.report(
            "query",
            Some(sample.name.clone()),
            region_index as u64 + 1,
            Some(regions.len() as u64),
            format!(
                "queried {} in {}",
                region.region.to_tabix_region(),
                sample.path.display()
            ),
        );
    }

    Ok(())
}

fn query_region_stats(
    methyl_reader: &mut IndexedMethylReader,
    region: &GenomicRegion,
    context: &Context,
    args: &MethHeatmapArgs,
) -> Result<RegionStats> {
    let mut stats = RegionStats::default();
    for record in methyl_reader.query(&region.to_tabix_region())? {
        stats.read_records += 1;
        if !matches_context(context, &record.context) {
            continue;
        }
        let depth = record.depth();
        if !args.depth.contains(depth) || depth == 0 {
            continue;
        }
        stats.add(record.methylated, record.unmethylated);
    }
    Ok(stats)
}

fn read_region_inputs(path: &Path) -> Result<Vec<RegionInput>> {
    let mut duplicate_counts = BTreeMap::<String, usize>::new();
    let mut inputs = Vec::new();

    for record in read_region_records(path)? {
        let base_id = format!("{}_{}_{}", record.chrom, record.start, record.end);
        let count = duplicate_counts.entry(base_id.clone()).or_insert(0);
        let id = if *count == 0 {
            base_id.clone()
        } else {
            format!("{base_id}{count}")
        };
        *count += 1;
        inputs.push(RegionInput {
            id,
            region: record.to_genomic_region(),
        });
    }

    if inputs.is_empty() {
        return Err(ViewBsError::invalid_input(
            path,
            "MethHeatmap region file did not contain any usable regions",
        ));
    }

    Ok(inputs)
}

fn region_path_for_sample<'a>(
    sample: &'a SampleSpec,
    args: &'a MethHeatmapArgs,
) -> Result<&'a Path> {
    sample
        .region_path
        .as_deref()
        .or(args.region.as_deref())
        .ok_or_else(|| {
            ViewBsError::invalid_input(
                "<region>",
                "MethHeatmap requires --region unless each sample has a region file",
            )
        })
}

struct MatrixBuilder {
    width: usize,
    order: Vec<String>,
    values: BTreeMap<String, Vec<Option<f64>>>,
}

impl MatrixBuilder {
    fn new(columns: Vec<String>) -> Self {
        Self {
            width: columns.len(),
            order: Vec::new(),
            values: BTreeMap::new(),
        }
    }

    fn set(&mut self, id: &str, column_index: usize, value: Option<f64>) {
        if !self.values.contains_key(id) {
            self.order.push(id.to_string());
        }
        let row = self
            .values
            .entry(id.to_string())
            .or_insert_with(|| vec![None; self.width]);
        if let Some(cell) = row.get_mut(column_index) {
            *cell = value;
        }
    }

    fn into_rows(self) -> Vec<MethHeatmapRow> {
        self.order
            .into_iter()
            .filter_map(|id| {
                self.values
                    .get(&id)
                    .cloned()
                    .map(|values| MethHeatmapRow { id, values })
            })
            .collect()
    }
}

fn write_table(table: &MethHeatmapTable) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(&table.path)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    let mut header = Vec::with_capacity(table.columns.len() + 1);
    header.push(String::new());
    header.extend(table.columns.iter().cloned());
    writer
        .write_record(header)
        .map_err(|source| ViewBsError::csv(&table.path, source))?;

    for row in &table.rows {
        let mut record = Vec::with_capacity(row.values.len() + 1);
        record.push(row.id.clone());
        record.extend(row.values.iter().map(|value| match value {
            Some(value) => value.to_string(),
            None => "NA".to_string(),
        }));
        writer
            .write_record(record)
            .map_err(|source| ViewBsError::csv(&table.path, source))?;
    }
    writer
        .flush()
        .map_err(|source| ViewBsError::io(&table.path, source))
}

fn matches_context(expected: &Context, observed: &Context) -> bool {
    expected == observed || matches!(expected, Context::Cxx)
}

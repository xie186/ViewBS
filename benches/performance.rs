use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::{tempdir, TempDir};
use viewbs::io::methyl_report::MethylReportReader;
use viewbs::io::tabix::IndexedMethylReader;
use viewbs::meth::Context;
use viewbs::{
    meth_geno, meth_heatmap, CommandOutput, DepthFilter, MethGenoArgs, MethHeatmapArgs, PlotFormat,
    PlotOptions, SampleSpec,
};

#[derive(Debug, Clone)]
struct ExternalBenchmarkDataset {
    methyl_report: PathBuf,
    regions: PathBuf,
    sample_name: String,
}

fn indexed_fixture() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz")
}

fn external_benchmark_dataset() -> Option<ExternalBenchmarkDataset> {
    let root = env::var_os("VIEWBS_BENCH_DATA_DIR").map(PathBuf::from)?;
    let methyl_report = root.join("sample.tab.gz");
    let tabix_index = root.join("sample.tab.gz.tbi");
    let regions = root.join("regions.bed");
    let missing = [
        (&methyl_report, "sample.tab.gz"),
        (&tabix_index, "sample.tab.gz.tbi"),
        (&regions, "regions.bed"),
    ]
    .into_iter()
    .filter_map(|(path, label)| (!path.exists()).then_some(label))
    .collect::<Vec<_>>();

    if !missing.is_empty() {
        panic!(
            "VIEWBS_BENCH_DATA_DIR={} is missing required file(s): {}",
            root.display(),
            missing.join(", ")
        );
    }

    Some(ExternalBenchmarkDataset {
        methyl_report,
        regions,
        sample_name: env::var("VIEWBS_BENCH_SAMPLE_NAME").unwrap_or_else(|_| "external".into()),
    })
}

fn no_plot() -> PlotOptions {
    PlotOptions {
        enabled: false,
        ..PlotOptions::default()
    }
}

fn svg_plot(width_cm: f64, height_cm: f64) -> PlotOptions {
    PlotOptions {
        enabled: true,
        format: PlotFormat::Svg,
        width_cm,
        height_cm,
        ..PlotOptions::default()
    }
}

fn write_small_regions(dir: &Path, prefix: &str) -> PathBuf {
    let regions = dir.join(format!("{prefix}_regions.bed"));
    fs::write(
        &regions,
        "chr2\t1006\t1010\tgene1\nchr2\t1012\t1015\tgene2\n",
    )
    .unwrap();
    regions
}

fn heatmap_args(
    tmp: &TempDir,
    prefix: &str,
    plot: PlotOptions,
    distribution_plot: PlotOptions,
) -> MethHeatmapArgs {
    let external = external_benchmark_dataset();
    let (sample_path, sample_name, regions) = if let Some(dataset) = external {
        (dataset.methyl_report, dataset.sample_name, dataset.regions)
    } else {
        (
            indexed_fixture(),
            "WT".to_string(),
            write_small_regions(tmp.path(), prefix),
        )
    };

    MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: sample_path,
            name: sample_name,
            region_path: None,
        }],
        region: Some(regions),
        outdir: tmp.path().to_path_buf(),
        prefix: prefix.to_string(),
        contexts: vec![Context::Cg],
        depth: DepthFilter { min: 1, max: 100 },
        merge: false,
        cluster_rows: false,
        cluster_cols: false,
        random_region: 2_000,
        plot,
        distribution_plot,
    }
}

fn command_output_size_bytes(output: &CommandOutput) -> u64 {
    output
        .tables
        .iter()
        .chain(output.plots.iter())
        .map(|file| fs::metadata(&file.path).unwrap().len())
        .sum()
}

fn heatmap_table_cells(output: &CommandOutput) -> usize {
    output
        .tables
        .iter()
        .map(|file| {
            fs::read_to_string(&file.path)
                .unwrap()
                .lines()
                .skip(1)
                .map(|line| line.split('\t').count().saturating_sub(1))
                .sum::<usize>()
        })
        .sum()
}

fn bench_whole_file_scan(c: &mut Criterion) {
    let fixture = indexed_fixture();

    c.bench_function("whole_file_scan_methyl_report", |b| {
        b.iter(|| {
            let mut records = 0_u64;
            let mut total_depth = 0_u64;
            for record in MethylReportReader::from_path(&fixture).unwrap() {
                let record = record.unwrap();
                records += 1;
                total_depth += record.depth();
            }
            black_box((records, total_depth))
        });
    });
}

fn bench_tabix_query(c: &mut Criterion) {
    let fixture = indexed_fixture();

    c.bench_function("tabix_query_small_region", |b| {
        let mut reader = IndexedMethylReader::from_path(&fixture).unwrap();
        b.iter(|| {
            let records = reader.query("chr2:1006-1010").unwrap();
            black_box(records.len())
        });
    });
}

fn bench_meth_geno_windows(c: &mut Criterion) {
    let tmp = tempdir().unwrap();
    let genome_length = tmp.path().join("genome.fai");
    fs::write(&genome_length, "chr2\t1020\n").unwrap();
    let args = MethGenoArgs {
        samples: vec![SampleSpec {
            path: indexed_fixture(),
            name: "WT".to_string(),
            region_path: None,
        }],
        genome_length,
        outdir: tmp.path().to_path_buf(),
        prefix: "geno".to_string(),
        contexts: vec![Context::Cg],
        depth: DepthFilter { min: 1, max: 100 },
        win: 10,
        step: 10,
        min_length: 1,
        max_chrom_number: 60,
        plot: no_plot(),
    };

    c.bench_function("meth_geno_window_runtime", |b| {
        b.iter(|| {
            let output = meth_geno(args.clone(), None, None).unwrap();
            black_box(output.summary.records_read)
        });
    });
}

fn bench_meth_heatmap_regions(c: &mut Criterion) {
    let tmp = tempdir().unwrap();
    let args = heatmap_args(&tmp, "heat", no_plot(), no_plot());

    c.bench_function("meth_heatmap_region_runtime", |b| {
        b.iter(|| {
            let output = meth_heatmap(args.clone(), None, None).unwrap();
            black_box((output.summary.records_read, output.summary.records_used))
        });
    });
}

fn bench_meth_heatmap_memory_proxy(c: &mut Criterion) {
    let tmp = tempdir().unwrap();
    let args = heatmap_args(&tmp, "heat_memory", no_plot(), no_plot());

    c.bench_function("meth_heatmap_memory_proxy_cells", |b| {
        b.iter(|| {
            let output = meth_heatmap(args.clone(), None, None).unwrap();
            black_box(heatmap_table_cells(&output))
        });
    });
}

fn bench_meth_heatmap_plot_generation(c: &mut Criterion) {
    let tmp = tempdir().unwrap();
    let args = heatmap_args(&tmp, "heat_plot", svg_plot(12.0, 10.0), svg_plot(12.0, 8.0));

    c.bench_function("meth_heatmap_plot_generation_time", |b| {
        b.iter(|| {
            let output = meth_heatmap(args.clone(), None, None).unwrap();
            black_box(output.plots.len())
        });
    });
}

fn bench_meth_heatmap_output_size(c: &mut Criterion) {
    let tmp = tempdir().unwrap();
    let args = heatmap_args(&tmp, "heat_size", svg_plot(12.0, 10.0), svg_plot(12.0, 8.0));

    c.bench_function("meth_heatmap_output_size_bytes", |b| {
        b.iter(|| {
            let output = meth_heatmap(args.clone(), None, None).unwrap();
            black_box(command_output_size_bytes(&output))
        });
    });
}

criterion_group!(
    performance,
    bench_whole_file_scan,
    bench_tabix_query,
    bench_meth_geno_windows,
    bench_meth_heatmap_regions,
    bench_meth_heatmap_memory_proxy,
    bench_meth_heatmap_plot_generation,
    bench_meth_heatmap_output_size
);
criterion_main!(performance);

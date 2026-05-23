use std::fs;

use tempfile::tempdir;
use viewbs::api::{
    BisNonConvRateArgs, DepthFilter, GlobalMethLevArgs, MethCoverageArgs, MethGenoArgs,
    MethHeatmapArgs, MethLevDistArgs, MethOneRegionArgs, MethOverRegionArgs, PlotOptions,
    SampleSpec,
};
use viewbs::meth::Context;
use viewbs::{
    bis_non_conv_rate, global_meth_lev, meth_coverage, meth_geno, meth_heatmap, meth_lev_dist,
    meth_one_region, meth_over_region,
};

fn golden_root(name: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("testdata/golden/legacy")
        .join(name)
}

fn indexed_sample() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz")
}

#[test]
fn global_meth_lev_matches_legacy_golden_fixture() {
    let root = golden_root("global_meth_lev");
    let input = root.join("sample.tab");
    let expected = fs::read_to_string(root.join("expected.tab")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("GlobalMethLev"));

    let tmp = tempdir().unwrap();
    let result = global_meth_lev(
        GlobalMethLevArgs {
            samples: vec![SampleSpec {
                path: input,
                name: "WT".to_string(),
                region_path: None,
            }],
            outdir: tmp.path().to_path_buf(),
            prefix: "global".to_string(),
            depth: DepthFilter { min: 1, max: 100 },
            method_average: false,
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "GlobalMethLev");
    let actual = fs::read_to_string(tmp.path().join("global.tab")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn bis_non_conv_rate_matches_legacy_golden_fixture() {
    let root = golden_root("bis_non_conv_rate");
    let input = root.join("sample.tab");
    let expected = fs::read_to_string(root.join("expected.tab")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("BisNonConvRate"));

    let tmp = tempdir().unwrap();
    let result = bis_non_conv_rate(
        BisNonConvRateArgs {
            samples: vec![SampleSpec {
                path: input,
                name: "WT".to_string(),
                region_path: None,
            }],
            outdir: tmp.path().to_path_buf(),
            prefix: "bis".to_string(),
            chrom: "chrC".to_string(),
            contexts: vec![Context::Cxx],
            depth: DepthFilter { min: 1, max: 100 },
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "BisNonConvRate");
    let actual = fs::read_to_string(tmp.path().join("bis.tab")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_coverage_matches_legacy_golden_fixture() {
    let root = golden_root("meth_coverage");
    let input = root.join("sample.tab");
    let reference = root.join("reference.fa");
    let expected = fs::read_to_string(root.join("expected.tab")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethCoverage"));

    let tmp = tempdir().unwrap();
    let result = meth_coverage(
        MethCoverageArgs {
            samples: vec![SampleSpec {
                path: input,
                name: "WT".to_string(),
                region_path: None,
            }],
            reference,
            outdir: tmp.path().to_path_buf(),
            prefix: "coverage".to_string(),
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethCoverage");
    let actual = fs::read_to_string(tmp.path().join("coverage.tab")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_lev_dist_matches_legacy_golden_fixture() {
    let root = golden_root("meth_lev_dist");
    let input = root.join("sample.tab");
    let expected = fs::read_to_string(root.join("expected.tab")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethLevDist"));

    let tmp = tempdir().unwrap();
    let result = meth_lev_dist(
        MethLevDistArgs {
            samples: vec![SampleSpec {
                path: input,
                name: "WT".to_string(),
                region_path: None,
            }],
            regions: None,
            outdir: tmp.path().to_path_buf(),
            prefix: "dist".to_string(),
            depth: DepthFilter { min: 1, max: 100 },
            bin_meth_lev: 0.5,
            method_average: false,
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethLevDist");
    let actual = fs::read_to_string(tmp.path().join("dist.tab")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_one_region_matches_legacy_golden_fixture() {
    let root = golden_root("meth_one_region");
    let expected = fs::read_to_string(root.join("expected.txt")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethOneRegion"));

    let tmp = tempdir().unwrap();
    let result = meth_one_region(
        MethOneRegionArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
                name: "WT".to_string(),
                region_path: None,
            }],
            outdir: tmp.path().to_path_buf(),
            prefix: "one".to_string(),
            region: "chr2:1006-1010".to_string(),
            flank: 0,
            contexts: vec![Context::Cg],
            depth: DepthFilter { min: 1, max: 100 },
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethOneRegion");
    let actual = fs::read_to_string(tmp.path().join("one_MethOneRegion_CG.txt")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_lev_dist_regions_matches_legacy_golden_fixture() {
    let root = golden_root("meth_lev_dist_regions");
    let regions = root.join("regions.bed");
    let expected = fs::read_to_string(root.join("expected.tab")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethLevDist"));

    let tmp = tempdir().unwrap();
    let result = meth_lev_dist(
        MethLevDistArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
                name: "WT".to_string(),
                region_path: None,
            }],
            regions: Some(regions),
            outdir: tmp.path().to_path_buf(),
            prefix: "dist_regions".to_string(),
            depth: DepthFilter { min: 1, max: 100 },
            bin_meth_lev: 0.5,
            method_average: false,
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethLevDist");
    let actual = fs::read_to_string(tmp.path().join("dist_regions.tab")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_over_region_matches_legacy_golden_fixture() {
    let root = golden_root("meth_over_region");
    let regions = root.join("regions.bed");
    let expected = fs::read_to_string(root.join("expected.txt")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethOverRegion"));

    let tmp = tempdir().unwrap();
    let result = meth_over_region(
        MethOverRegionArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
                name: "WT".to_string(),
                region_path: None,
            }],
            region: Some(regions),
            outdir: tmp.path().to_path_buf(),
            prefix: "over".to_string(),
            contexts: vec![Context::Cg],
            depth: DepthFilter { min: 1, max: 100 },
            flank: 10,
            bin_length: 5,
            bin_number: 5,
            min_length: 1,
            max_length: 1_000_000,
            region_name: "Gene".to_string(),
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethOverRegion");
    let actual = fs::read_to_string(tmp.path().join("over_MethOverRegion_CG.txt")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_heatmap_matches_legacy_golden_fixture() {
    let root = golden_root("meth_heatmap");
    let regions = root.join("regions.bed");
    let expected = fs::read_to_string(root.join("expected_CG.txt")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethHeatmap"));

    let tmp = tempdir().unwrap();
    let result = meth_heatmap(
        MethHeatmapArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
                name: "WT".to_string(),
                region_path: None,
            }],
            region: Some(regions),
            outdir: tmp.path().to_path_buf(),
            prefix: "heat".to_string(),
            contexts: vec![Context::Cg],
            depth: DepthFilter { min: 1, max: 100 },
            merge: false,
            cluster_rows: true,
            cluster_cols: false,
            random_region: 2_000,
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
            distribution_plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethHeatmap");
    let actual = fs::read_to_string(tmp.path().join("heat_MethHeatmap_CG.txt")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_heatmap_merge_matches_legacy_golden_fixture() {
    let root = golden_root("meth_heatmap_merge");
    let regions = root.join("regions.bed");
    let expected = fs::read_to_string(root.join("expected_mer.txt")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethHeatmap"));
    assert!(command.contains("--merge"));

    let tmp = tempdir().unwrap();
    let result = meth_heatmap(
        MethHeatmapArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
                name: "WT".to_string(),
                region_path: None,
            }],
            region: Some(regions),
            outdir: tmp.path().to_path_buf(),
            prefix: "heat".to_string(),
            contexts: vec![Context::Cg, Context::Chg],
            depth: DepthFilter { min: 1, max: 100 },
            merge: true,
            cluster_rows: true,
            cluster_cols: false,
            random_region: 2_000,
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
            distribution_plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethHeatmap");
    let actual = fs::read_to_string(tmp.path().join("heat_MethHeatmap_mer.txt")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn meth_geno_matches_legacy_golden_fixture() {
    let root = golden_root("meth_geno");
    let genome_length = root.join("genome.fai");
    let expected = fs::read_to_string(root.join("expected_CG.txt")).unwrap();
    let command = fs::read_to_string(root.join("command.txt")).unwrap();
    assert!(command.contains("MethGeno"));

    let tmp = tempdir().unwrap();
    let result = meth_geno(
        MethGenoArgs {
            samples: vec![SampleSpec {
                path: indexed_sample(),
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
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(result.summary.command, "MethGeno");
    let actual = fs::read_to_string(tmp.path().join("geno_MethGeno_CG.txt")).unwrap();
    assert_eq!(actual, expected);
}

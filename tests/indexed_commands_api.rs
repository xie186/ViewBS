use std::fs;

use tempfile::tempdir;
#[cfg(feature = "plots")]
use viewbs::api::PlotFormat;
use viewbs::api::{
    DepthFilter, MethGenoArgs, MethHeatmapArgs, MethLevDistArgs, MethOneRegionArgs,
    MethOverRegionArgs, PlotOptions, SampleSpec,
};
use viewbs::meth::Context;
use viewbs::{meth_geno, meth_heatmap, meth_lev_dist, meth_one_region, meth_over_region};

#[test]
fn meth_one_region_queries_tabix_and_writes_context_table() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");

    let args = MethOneRegionArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_one_region(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 4);
    assert_eq!(output.summary.records_used, 4);
    assert_eq!(output.tables.len(), 1);

    let table = fs::read_to_string(tmp.path().join("one_MethOneRegion_CG.txt")).unwrap();
    assert_eq!(
        table,
        concat!(
            "Sample\tchr\tposition\tC_num\tT_num\tMethylationLevel\n",
            "WT\tchr2\t1006\t2\t31\t0.06060606060606061\n",
            "WT\tchr2\t1007\t6\t36\t0.14285714285714285\n",
            "WT\tchr2\t1009\t3\t30\t0.09090909090909091\n",
            "WT\tchr2\t1010\t8\t34\t0.19047619047619047\n",
        )
    );
}

#[cfg(feature = "plots")]
#[test]
fn meth_one_region_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");

    let args = MethOneRegionArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_one_region(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(
        output.plots[0].path,
        tmp.path().join("one_MethOneRegion_CG.svg")
    );

    let svg = fs::read_to_string(tmp.path().join("one_MethOneRegion_CG.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("MethOneRegion CG"));
    assert!(svg.contains("WT"));
}

#[test]
fn meth_geno_queries_tabix_windows_and_writes_context_table() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let genome_length = tmp.path().join("genome.fai");
    fs::write(&genome_length, "chr2\t1020\n").unwrap();

    let args = MethGenoArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_geno(args, None, None).unwrap();
    assert_eq!(output.tables.len(), 1);

    let table = fs::read_to_string(tmp.path().join("geno_MethGeno_CG.txt")).unwrap();
    assert!(
        table.starts_with("chr\tstt\tend\tsample_name\tC_number\tT_number\tMethylation_level\n")
    );

    let target = table
        .lines()
        .find(|line| line.starts_with("chr2\t1001\t1010\tWT\t"))
        .expect("missing 1001-1010 window");
    let columns = target.split('\t').collect::<Vec<_>>();
    assert_eq!(columns[4], "19");
    assert_eq!(columns[5], "131");
    let level = columns[6].parse::<f64>().unwrap();
    assert!((level - 19.0 / (150.0 + 0.000000001)).abs() < 1e-12);
}

#[cfg(feature = "plots")]
#[test]
fn meth_geno_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let genome_length = tmp.path().join("genome.fai");
    fs::write(&genome_length, "chr2\t1020\n").unwrap();

    let args = MethGenoArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_geno(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(
        output.plots[0].path,
        tmp.path().join("geno_MethGeno_CG.svg")
    );

    let svg = fs::read_to_string(tmp.path().join("geno_MethGeno_CG.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("MethGeno CG"));
    assert!(svg.contains("WT-chr2"));
}

#[test]
fn meth_lev_dist_regions_bins_region_weighted_methylation_levels() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\n").unwrap();

    let args = MethLevDistArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_lev_dist(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 4);
    assert_eq!(output.summary.records_used, 4);

    let table = fs::read_to_string(tmp.path().join("dist_regions.tab")).unwrap();
    assert_eq!(
        table,
        concat!(
            "Sample\tContext\tMethLevBinMidPoint\tNumber\tPercentage\n",
            "WT\tCG\t0.25\t1\t100\n",
            "WT\tCG\t0.75\t0\t0\n",
        )
    );
}

#[cfg(feature = "plots")]
#[test]
fn meth_lev_dist_regions_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\n").unwrap();

    let args = MethLevDistArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_lev_dist(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(output.plots[0].path, tmp.path().join("dist_regions.svg"));

    let svg = fs::read_to_string(tmp.path().join("dist_regions.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Methylation level distribution"));
    assert!(svg.contains("WT-CG"));
}

#[test]
fn meth_over_region_bins_context_records_over_region_body_and_flank() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\tgene1\t+\n").unwrap();

    let args = MethOverRegionArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_over_region(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 10);
    assert_eq!(output.summary.records_used, 4);

    let table = fs::read_to_string(tmp.path().join("over_MethOverRegion_CG.txt")).unwrap();
    assert_eq!(
        table,
        concat!(
            "sample_name\tregion\tbin_num\tC_number\tT_number\tMethylation_level\n",
            "WT\tBody\t1\t2\t31\t0.06060606060606061\n",
            "WT\tBody\t2\t6\t36\t0.14285714285714285\n",
            "WT\tBody\t4\t3\t30\t0.09090909090909091\n",
            "WT\tDownstream\t6\t8\t34\t0.19047619047619047\n",
        )
    );
}

#[cfg(feature = "plots")]
#[test]
fn meth_over_region_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\tgene1\t+\n").unwrap();

    let args = MethOverRegionArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_over_region(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(
        output.plots[0].path,
        tmp.path().join("over_MethOverRegion_CG.svg")
    );

    let svg = fs::read_to_string(tmp.path().join("over_MethOverRegion_CG.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("MethOverRegion CG"));
    assert!(svg.contains("WT"));
}

#[test]
fn meth_heatmap_writes_per_context_region_matrix() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(
        &regions,
        "chr2\t1006\t1010\tgene1\nchr2\t1012\t1015\tgene2\n",
    )
    .unwrap();

    let args = MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_heatmap(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 7);
    assert_eq!(output.summary.records_used, 4);
    assert_eq!(output.tables.len(), 1);

    let table = fs::read_to_string(tmp.path().join("heat_MethHeatmap_CG.txt")).unwrap();
    assert_eq!(
        table,
        concat!(
            "\tWT\n",
            "chr2_1006_1010\t0.12666666666666668\n",
            "chr2_1012_1015\tNA\n",
        )
    );
}

#[test]
fn meth_heatmap_merge_writes_context_sample_columns() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1015\tgene1\n").unwrap();

    let args = MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: fixture,
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
    };

    let output = meth_heatmap(args, None, None).unwrap();
    assert_eq!(output.tables.len(), 1);

    let table = fs::read_to_string(tmp.path().join("heat_MethHeatmap_mer.txt")).unwrap();
    assert_eq!(
        table,
        concat!(
            "\tWT-CG\tWT-CHG\n",
            "chr2_1006_1015\t0.12666666666666668\t0.02666666666666667\n",
        )
    );
}

#[cfg(feature = "plots")]
#[test]
fn meth_heatmap_writes_svg_heatmap_and_distribution_plots() {
    let tmp = tempdir().unwrap();
    let fixture = std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz");
    let regions = tmp.path().join("regions.bed");
    fs::write(
        &regions,
        "chr2\t1006\t1010\tgene1\nchr2\t1012\t1015\tgene2\n",
    )
    .unwrap();

    let args = MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: fixture,
            name: "WT".to_string(),
            region_path: None,
        }],
        region: Some(regions),
        outdir: tmp.path().to_path_buf(),
        prefix: "heat".to_string(),
        contexts: vec![Context::Cg],
        depth: DepthFilter { min: 1, max: 100 },
        merge: false,
        cluster_rows: false,
        cluster_cols: false,
        random_region: 2_000,
        plot: PlotOptions {
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
        distribution_plot: PlotOptions {
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_heatmap(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 2);
    assert_eq!(
        output.plots[0].path,
        tmp.path().join("heat_MethHeatmap_CG.svg")
    );
    assert_eq!(
        output.plots[1].path,
        tmp.path().join("heat_MethHist_CG.svg")
    );

    let heatmap_svg = fs::read_to_string(tmp.path().join("heat_MethHeatmap_CG.svg")).unwrap();
    assert!(heatmap_svg.contains("<svg"));
    assert!(heatmap_svg.contains("MethHeatmap CG"));
    assert!(heatmap_svg.contains("WT"));

    let hist_svg = fs::read_to_string(tmp.path().join("heat_MethHist_CG.svg")).unwrap();
    assert!(hist_svg.contains("<svg"));
    assert!(hist_svg.contains("MethHist CG"));
    assert!(hist_svg.contains("WT"));
}

#![cfg(feature = "cli")]

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use viewbs::meth::Context;
use viewbs::{DepthFilter, MethHeatmapArgs, PlotOptions, SampleSpec, ViewBsError};

fn viewbs_stdout(args: &[&str]) -> String {
    let output = Command::new(cargo_bin("ViewBS"))
        .args(args)
        .output()
        .expect("failed to run ViewBS");
    assert!(
        output.status.success(),
        "expected success\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn top_level_help_snapshot_includes_legacy_helper_aliases() {
    let help = viewbs_stdout(&["--help"]);

    insta::assert_snapshot!(help, @r###"
Tools for exploring and visualizing bisulfite sequencing data

Usage: ViewBS <COMMAND>

Commands:
  MethCoverage      Generate coverage report for BS-seq data
  BisNonConvRate    Estimate non-conversion rate
  GlobalMethLev     Generate global methylation level report
  MethLevDist       Generate methylation level distribution
  MethGeno          Generate methylation information across chromosomes
  MethHeatmap       Generate methylation heatmap for regions
  MethOverRegion    Plot average methylation over regions
  MethOneRegion     Extract and plot methylation for one region
  merge-figures     Merge SVG plot artifacts into one figure grid
  bsseeker2bismark  Compatibility alias for `convert bsseeker` [aliases: bsseeker2bismark.pl]
  brat2bismark      Compatibility alias for `convert brat` [aliases: brat2bismark.pl]
  gff2tab           Compatibility alias for `convert gff` [aliases: gff2tab.pl]
  mer_fig           Compatibility alias for `merge-figures` [aliases: mer_fig.R]
  convert           Convert methylation outputs to ViewBS input
  help              Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
"###);
}

#[test]
fn structured_error_message_snapshot_is_stable() {
    let error = ViewBsError::parse_error(
        PathBuf::from("sample.tab"),
        Some(7),
        "invalid depth `x`: invalid digit found in string",
    );

    insta::assert_snapshot!(
        error.to_string(),
        @"parse error in sample.tab at line Some(7): invalid depth `x`: invalid digit found in string"
    );
}

#[cfg(feature = "serde")]
#[test]
fn json_config_snapshot_is_stable() {
    let args = MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: PathBuf::from("sample.tab.gz"),
            name: "WT".to_string(),
            region_path: None,
        }],
        region: Some(PathBuf::from("genes.bed")),
        outdir: PathBuf::from("results"),
        prefix: "heatmap".to_string(),
        contexts: vec![Context::Cg],
        depth: DepthFilter { min: 5, max: 100 },
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
    let json = serde_json::to_string_pretty(&args).unwrap();

    insta::assert_snapshot!(json, @r###"
{
  "samples": [
    {
      "path": "sample.tab.gz",
      "name": "WT",
      "region_path": null
    }
  ],
  "region": "genes.bed",
  "outdir": "results",
  "prefix": "heatmap",
  "contexts": [
    "Cg"
  ],
  "depth": {
    "min": 5,
    "max": 100
  },
  "merge": false,
  "cluster_rows": true,
  "cluster_cols": false,
  "random_region": 2000,
  "plot": {
    "enabled": false,
    "format": "Pdf",
    "width_cm": 10.0,
    "height_cm": 10.0,
    "keep_svg": false
  },
  "distribution_plot": {
    "enabled": false,
    "format": "Pdf",
    "width_cm": 10.0,
    "height_cm": 10.0,
    "keep_svg": false
  }
}
"###);
}

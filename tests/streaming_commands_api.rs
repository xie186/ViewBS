use std::fs;

use tempfile::tempdir;
use viewbs::api::MethCoverageArgs;
use viewbs::api::{BisNonConvRateArgs, DepthFilter, MethLevDistArgs, PlotOptions, SampleSpec};
#[cfg(feature = "plots")]
use viewbs::api::{GlobalMethLevArgs, PlotFormat};
#[cfg(feature = "plots")]
use viewbs::global_meth_lev;
use viewbs::io::fasta::count_sequence_contexts;
use viewbs::meth::Context;
#[cfg(feature = "plots")]
use viewbs::ViewBsError;
use viewbs::{bis_non_conv_rate, meth_coverage, meth_lev_dist};

#[cfg(feature = "plots")]
fn svg_numeric_attribute(svg: &str, attribute: &str) -> f64 {
    let needle = format!("{attribute}=\"");
    let start = svg.find(&needle).expect("missing SVG attribute") + needle.len();
    let end = start + svg[start..].find('"').expect("unterminated SVG attribute");
    svg[start..end]
        .strip_suffix("px")
        .unwrap_or(&svg[start..end])
        .parse()
        .expect("SVG attribute must be numeric")
}

#[cfg(feature = "plots")]
fn assert_svg_dimensions(svg: &str, width_px: f64, height_px: f64) {
    let width = svg_numeric_attribute(svg, "width");
    let height = svg_numeric_attribute(svg, "height");
    assert!(
        (width - width_px).abs() < 1e-6,
        "expected SVG width {width_px}, got {width}"
    );
    assert!(
        (height - height_px).abs() < 1e-6,
        "expected SVG height {height_px}, got {height}"
    );
}

#[cfg(feature = "plots")]
fn assert_svg_has_no_invalid_numeric_or_placeholder_values(svg: &str) {
    for banned in ["NaN", "nan", "inf", "Inf", "placeholder", "TODO"] {
        assert!(
            !svg.contains(banned),
            "SVG must not contain invalid numeric or placeholder value `{banned}`"
        );
    }
}

#[test]
fn reference_context_counting_matches_legacy_formula() {
    let counts = count_sequence_contexts("ACGCAGCA");
    assert_eq!(counts.c_or_g, 5);
    assert_eq!(counts.cg, 2);
    assert_eq!(counts.chg, 2);
    assert_eq!(counts.chh, 1);
}

#[test]
fn meth_coverage_writes_expected_reverse_cumulative_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    let reference = tmp.path().join("reference.fa");
    fs::write(&reference, ">chr1\nACGCAGCA\n").unwrap();
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t1\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t0\tCG\tCGA\n",
            "chr1\t3\t+\t1\t1\tCHG\tCAG\n",
            "chr1\t4\t+\t0\t1\tCHH\tCAA\n",
        ),
    )
    .unwrap();

    let args = MethCoverageArgs {
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
    };

    let output = meth_coverage(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 4);
    assert_eq!(output.summary.records_used, 4);

    let table = fs::read_to_string(tmp.path().join("coverage.tab")).unwrap();
    assert_eq!(
        table,
        concat!(
            "Sample\tContext\tDepth\tPercentage\n",
            "WT\tCG\t1\t100\n",
            "WT\tCG\t2\t50\n",
            "WT\tCHG\t1\t50\n",
            "WT\tCHG\t2\t50\n",
            "WT\tCHH\t1\t100\n",
            "WT\tCHH\t2\t0\n",
        )
    );
}

#[test]
fn bis_non_conv_rate_writes_expected_cxx_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chrC\t1\t+\t2\t8\tCG\tCGA\n",
            "chrC\t2\t+\t1\t9\tCHG\tCAG\n",
            "chr1\t1\t+\t9\t1\tCG\tCGA\n",
        ),
    )
    .unwrap();

    let args = BisNonConvRateArgs {
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
    };

    let output = bis_non_conv_rate(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 3);
    assert_eq!(output.summary.records_used, 2);

    let table = fs::read_to_string(tmp.path().join("bis.tab")).unwrap();
    assert_eq!(
        table,
        "Sample\tBisNonConvRate\tC_number\tTotal_Depth\tContext\nWT\t0.15000\t3\t20\tCXX\n"
    );
}

#[cfg(feature = "plots")]
#[test]
fn bis_non_conv_rate_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chrC\t1\t+\t2\t8\tCG\tCGA\n",
            "chrC\t2\t+\t1\t9\tCHG\tCAG\n",
            "chr1\t1\t+\t9\t1\tCG\tCGA\n",
        ),
    )
    .unwrap();

    let args = BisNonConvRateArgs {
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = bis_non_conv_rate(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(output.plots[0].path, tmp.path().join("bis.svg"));

    let svg = fs::read_to_string(tmp.path().join("bis.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Bisulfite non-conversion rate"));
    assert!(svg.contains("WT"));
    assert!(svg.contains("CXX"));
}

#[cfg(feature = "plots")]
#[test]
fn global_meth_lev_writes_png_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t3\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t3\tCHG\tCAG\n",
            "chr1\t3\t+\t0\t4\tCHH\tCAA\n",
        ),
    )
    .unwrap();

    let output = global_meth_lev(
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
                enabled: true,
                format: PlotFormat::Png,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(output.plots.len(), 1);
    assert_eq!(output.plots[0].format, "png");
    assert_eq!(output.plots[0].path, tmp.path().join("global.png"));

    let png = fs::read(tmp.path().join("global.png")).unwrap();
    assert!(png.len() > 8);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[cfg(feature = "plots")]
#[test]
fn global_meth_lev_writes_pdf_and_keeps_svg_when_requested() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t3\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t3\tCHG\tCAG\n",
            "chr1\t3\t+\t0\t4\tCHH\tCAA\n",
        ),
    )
    .unwrap();

    let output = global_meth_lev(
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
                enabled: true,
                format: PlotFormat::Pdf,
                keep_svg: true,
                width_cm: 5.08,
                height_cm: 2.54,
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(output.plots.len(), 1);
    assert_eq!(output.plots[0].format, "pdf");
    assert_eq!(output.plots[0].path, tmp.path().join("global.pdf"));

    let pdf = fs::read(tmp.path().join("global.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));

    let svg = fs::read_to_string(tmp.path().join("global.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert_svg_dimensions(&svg, 192.0, 96.0);
    assert!(svg.contains("Global methylation level"));
    assert!(svg.contains("WT"));
    assert_svg_has_no_invalid_numeric_or_placeholder_values(&svg);
}

#[cfg(feature = "plots")]
#[test]
fn global_meth_lev_rejects_non_finite_plot_dimensions_before_rendering() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(&input, "chr1\t1\t+\t3\t1\tCG\tCGA\n").unwrap();

    let error = global_meth_lev(
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
                enabled: true,
                format: PlotFormat::Svg,
                width_cm: f64::NAN,
                height_cm: 10.0,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap_err();

    match error {
        ViewBsError::InvalidInput { path, message } => {
            assert_eq!(path.to_string_lossy(), "<plot>");
            assert!(message.contains("finite positive"));
        }
        other => panic!("expected invalid plot options, got {other:?}"),
    }

    assert!(!tmp.path().join("global.svg").exists());
}

#[test]
fn meth_lev_dist_writes_expected_genome_wide_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t3\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t3\tCG\tCGA\n",
            "chr1\t3\t+\t0\t4\tCHH\tCAA\n",
        ),
    )
    .unwrap();

    let args = MethLevDistArgs {
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
    };

    let output = meth_lev_dist(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 3);
    assert_eq!(output.summary.records_used, 3);

    let table = fs::read_to_string(tmp.path().join("dist.tab")).unwrap();
    assert_eq!(
        table,
        concat!(
            "Sample\tContext\tMethLevBinMidPoint\tNumber\tPercentage\n",
            "WT\tCG\t0.25\t1\t50\n",
            "WT\tCG\t0.75\t1\t50\n",
            "WT\tCHH\t0.25\t1\t100\n",
            "WT\tCHH\t0.75\t0\t0\n",
        )
    );
}

#[cfg(feature = "plots")]
#[test]
fn meth_lev_dist_writes_svg_plot_when_enabled() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t3\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t3\tCG\tCGA\n",
            "chr1\t3\t+\t0\t4\tCHH\tCAA\n",
        ),
    )
    .unwrap();

    let args = MethLevDistArgs {
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
            enabled: true,
            format: PlotFormat::Svg,
            ..PlotOptions::default()
        },
    };

    let output = meth_lev_dist(args, None, None).unwrap();
    assert_eq!(output.plots.len(), 1);
    assert_eq!(output.plots[0].path, tmp.path().join("dist.svg"));

    let svg = fs::read_to_string(tmp.path().join("dist.svg")).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Methylation level distribution"));
    assert!(svg.contains("WT-CG"));
    assert!(svg.contains("WT-CHH"));
}

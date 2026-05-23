#![cfg(feature = "cli")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use tempfile::{tempdir, TempDir};

fn viewbs() -> Command {
    Command::new(cargo_bin("ViewBS"))
}

fn assert_success(mut command: Command) -> String {
    let output = command.output().expect("failed to run ViewBS");
    assert!(
        output.status.success(),
        "expected success\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

fn assert_failure(mut command: Command) -> String {
    let output = command.output().expect("failed to run ViewBS");
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stderr).expect("stderr should be UTF-8")
}

fn write_streaming_fixture(tmp: &TempDir) -> PathBuf {
    let input = tmp.path().join("sample.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t1\t+\t3\t1\tCG\tCGA\n",
            "chr1\t2\t+\t1\t3\tCHG\tCAG\n",
            "chr1\t3\t+\t0\t4\tCHH\tCAA\n",
            "chrC\t1\t+\t2\t8\tCG\tCGA\n",
            "chrC\t2\t+\t1\t9\tCHG\tCAG\n",
        ),
    )
    .unwrap();
    input
}

fn indexed_fixture() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz")
}

fn command_outdir(tmp: &TempDir, name: &str) -> PathBuf {
    tmp.path().join(name)
}

fn write_svg_figure(path: &std::path::Path, title: &str, fill: &str) {
    fs::write(
        path,
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80"><rect width="120" height="80" fill="{fill}"/><text x="12" y="42">{title}</text></svg>"#
        ),
    )
    .unwrap();
}

#[test]
fn cli_help_and_version_work() {
    let help = assert_success({
        let mut cmd = viewbs();
        cmd.arg("--help");
        cmd
    });
    assert!(help.contains("ViewBS"));
    assert!(help.contains("MethHeatmap"));

    let version = assert_success({
        let mut cmd = viewbs();
        cmd.arg("--version");
        cmd
    });
    assert!(version.starts_with("ViewBS "));
}

#[test]
fn cli_reports_missing_required_arguments() {
    let stderr = assert_failure({
        let mut cmd = viewbs();
        cmd.arg("GlobalMethLev");
        cmd
    });
    assert!(stderr.contains("--sample"));
}

#[test]
fn cli_reports_bad_input_file_errors() {
    let tmp = tempdir().unwrap();
    let outdir = command_outdir(&tmp, "bad_input");
    let stderr = assert_failure({
        let mut cmd = viewbs();
        cmd.args([
            "GlobalMethLev",
            "--sample",
            "missing.tab,WT",
            "--outdir",
            outdir.to_str().unwrap(),
            "--prefix",
            "bad",
            "--no-plot",
        ]);
        cmd
    });
    assert!(stderr.contains("I/O error"));
    assert!(stderr.contains("missing.tab"));
}

#[test]
fn cli_runs_streaming_subcommands_with_legacy_depth_flags() {
    let tmp = tempdir().unwrap();
    let input = write_streaming_fixture(&tmp);
    let sample = format!("{},WT", input.display());

    let global_outdir = command_outdir(&tmp, "global");
    let global_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "GlobalMethLev",
            "--sample",
            sample.as_str(),
            "--outdir",
            global_outdir.to_str().unwrap(),
            "--prefix",
            "global",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--no-plot",
        ]);
        cmd
    });
    assert!(global_stdout.contains("global.tab"));
    assert!(global_outdir.join("global.tab").exists());

    let reference = tmp.path().join("reference.fa");
    fs::write(&reference, ">chr1\nACGCAGCA\n").unwrap();
    let coverage_outdir = command_outdir(&tmp, "coverage");
    let coverage_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethCoverage",
            "--sample",
            sample.as_str(),
            "--reference",
            reference.to_str().unwrap(),
            "--outdir",
            coverage_outdir.to_str().unwrap(),
            "--prefix",
            "coverage",
            "--no-plot",
        ]);
        cmd
    });
    assert!(coverage_stdout.contains("coverage.tab"));
    assert!(coverage_outdir.join("coverage.tab").exists());

    let conv_outdir = command_outdir(&tmp, "conversion");
    let conv_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "BisNonConvRate",
            "--sample",
            sample.as_str(),
            "--chrom",
            "chrC",
            "--outdir",
            conv_outdir.to_str().unwrap(),
            "--prefix",
            "conversion",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--no-plot",
        ]);
        cmd
    });
    assert!(conv_stdout.contains("conversion.tab"));
    assert!(conv_outdir.join("conversion.tab").exists());

    let dist_outdir = command_outdir(&tmp, "dist");
    let dist_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethLevDist",
            "--sample",
            sample.as_str(),
            "--outdir",
            dist_outdir.to_str().unwrap(),
            "--prefix",
            "dist",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--binMethLev",
            "0.5",
            "--no-plot",
        ]);
        cmd
    });
    assert!(dist_stdout.contains("dist.tab"));
    assert!(dist_outdir.join("dist.tab").exists());
}

#[test]
fn cli_runs_indexed_subcommands_with_legacy_depth_flags() {
    let tmp = tempdir().unwrap();
    let fixture = indexed_fixture();
    let sample = format!("{},WT", fixture.display());
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\tgene1\t+\n").unwrap();

    let one_outdir = command_outdir(&tmp, "one");
    let one_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethOneRegion",
            "--sample",
            sample.as_str(),
            "--region",
            "chr2:1006-1010",
            "--outdir",
            one_outdir.to_str().unwrap(),
            "--prefix",
            "one",
            "--context",
            "CG",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--no-plot",
        ]);
        cmd
    });
    assert!(one_stdout.contains("one_MethOneRegion_CG.txt"));
    assert!(one_outdir.join("one_MethOneRegion_CG.txt").exists());

    let heat_outdir = command_outdir(&tmp, "heat");
    let heat_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethHeatmap",
            "--sample",
            sample.as_str(),
            "--region",
            regions.to_str().unwrap(),
            "--outdir",
            heat_outdir.to_str().unwrap(),
            "--prefix",
            "heat",
            "--context",
            "CG",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--cluster_rows",
            "FALSE",
            "--cluster_cols",
            "FALSE",
            "--no-plot",
        ]);
        cmd
    });
    assert!(heat_stdout.contains("heat_MethHeatmap_CG.txt"));
    assert!(heat_outdir.join("heat_MethHeatmap_CG.txt").exists());

    let over_outdir = command_outdir(&tmp, "over");
    let over_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethOverRegion",
            "--sample",
            sample.as_str(),
            "--region",
            regions.to_str().unwrap(),
            "--outdir",
            over_outdir.to_str().unwrap(),
            "--prefix",
            "over",
            "--context",
            "CG",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--flank",
            "10",
            "--binLength",
            "5",
            "--binNumber",
            "5",
            "--minLength",
            "1",
            "--no-plot",
        ]);
        cmd
    });
    assert!(over_stdout.contains("over_MethOverRegion_CG.txt"));
    assert!(over_outdir.join("over_MethOverRegion_CG.txt").exists());

    let genome_length = tmp.path().join("genome.fai");
    fs::write(&genome_length, "chr2\t1020\n").unwrap();
    let geno_outdir = command_outdir(&tmp, "geno");
    let geno_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "MethGeno",
            "--sample",
            sample.as_str(),
            "--genomeLength",
            genome_length.to_str().unwrap(),
            "--outdir",
            geno_outdir.to_str().unwrap(),
            "--prefix",
            "geno",
            "--context",
            "CG",
            "--minDepth",
            "1",
            "--maxDepth",
            "100",
            "--win",
            "10",
            "--step",
            "10",
            "--minLength",
            "1",
            "--no-plot",
        ]);
        cmd
    });
    assert!(geno_stdout.contains("geno_MethGeno_CG.txt"));
    assert!(geno_outdir.join("geno_MethGeno_CG.txt").exists());
}

#[test]
fn cli_converts_bsseeker_cgmap_to_bismark_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("bsseeker.cgmap");
    let output = tmp.path().join("bsseeker.tab");
    fs::write(
        &input,
        concat!(
            "chr1\tC\t3001631\tCG\tCG\t1.0\t5\t5\n",
            "chr1\tG\t3001632\tCHG\tCAG\t0.5\t2\t4\n",
            "chr1\tC\t3001633\t-\tCNN\t0.0\t0\t2\n",
        ),
    )
    .unwrap();

    let stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "convert",
            "bsseeker",
            "--input",
            input.to_str().unwrap(),
            "--min-depth",
            "3",
            "--output",
            output.to_str().unwrap(),
        ]);
        cmd
    });

    assert!(stdout.contains("Table:"));
    let table = fs::read_to_string(output).unwrap();
    assert_eq!(
        table,
        concat!(
            "chr1\t3001631\t+\t5\t0\tCG\tCG\n",
            "chr1\t3001632\t-\t2\t2\tCHG\tCAG\n",
        )
    );
}

#[test]
fn cli_converts_brat_output_to_bismark_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("brat.tsv");
    let output = tmp.path().join("brat.tab");
    fs::write(
        &input,
        concat!(
            "chr1\t552\t552\tCpG:4\t0.25\t-\n",
            "chr1\t553\t553\tCHH:2\t0.75\t+\n",
            "chr1\t554\t554\tCHG:1\t1.00\t+\n",
        ),
    )
    .unwrap();

    let stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "convert",
            "brat",
            "--input",
            input.to_str().unwrap(),
            "--min-depth",
            "2",
            "--output",
            output.to_str().unwrap(),
        ]);
        cmd
    });

    assert!(stdout.contains("Table:"));
    let table = fs::read_to_string(output).unwrap();
    assert_eq!(
        table,
        concat!(
            "chr1\t552\t-\t1\t3\tCG\tCG\n",
            "chr1\t553\t+\t2\t0\tCHH\tCHH\n",
        )
    );
}

#[test]
fn cli_converts_gff_features_to_region_table() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("genes.gff3");
    let output = tmp.path().join("genes.tab");
    fs::write(
        &input,
        concat!(
            "##gff-version 3\n",
            "chr1\tTAIR10\tgene\t10\t20\t.\t+\t.\tID=AT1G00010;Name=GeneA\n",
            "chr1\tTAIR10\tmRNA\t10\t20\t.\t+\t.\tID=AT1G00010.1;Parent=AT1G00010\n",
            "chr2\tTAIR10\tgene\t30\t40\t.\t-\t.\tName=GeneB\n",
            "chr3\tTAIR10\texon\t50\t60\t.\t+\t.\tID=exon1\n",
        ),
    )
    .unwrap();

    let stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "convert",
            "gff",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--feature",
            "gene",
        ]);
        cmd
    });

    assert!(stdout.contains("Table:"));
    let table = fs::read_to_string(output).unwrap();
    assert_eq!(
        table,
        concat!("chr1\t10\t20\tAT1G00010\t+\n", "chr2\t30\t40\tGeneB\t-\n",)
    );
}

#[test]
fn cli_merges_svg_figures_into_labeled_grid() {
    let tmp = tempdir().unwrap();
    let first = tmp.path().join("first.svg");
    let second = tmp.path().join("second.svg");
    let output = tmp.path().join("merged.svg");
    write_svg_figure(&first, "first plot", "#dceeff");
    write_svg_figure(&second, "second plot", "#ffe8d6");
    let inputs = format!("{},{}", first.display(), second.display());

    let stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "merge-figures",
            "--input",
            inputs.as_str(),
            "--labels",
            "Bis,Global",
            "--output",
            output.to_str().unwrap(),
            "--ncol",
            "2",
            "--base-height",
            "2.54",
            "--base-aspect-ratio",
            "1.0",
        ]);
        cmd
    });

    assert!(stdout.contains("Plot:"));
    let svg = fs::read_to_string(output).unwrap();
    assert!(svg.contains("data-viewbs-command=\"merge-figures\""));
    assert!(svg.contains("Bis"));
    assert!(svg.contains("Global"));
    assert!(svg.contains("data:image/svg+xml;base64,"));
}

#[test]
fn cli_accepts_legacy_helper_script_aliases() {
    let tmp = tempdir().unwrap();

    let bsseeker_input = tmp.path().join("bsseeker.cgmap");
    let bsseeker_output = tmp.path().join("bsseeker.tab");
    fs::write(&bsseeker_input, "chr1\tC\t3001631\tCG\tCG\t1.0\t5\t5\n").unwrap();
    let bsseeker_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "bsseeker2bismark.pl",
            "--input",
            bsseeker_input.to_str().unwrap(),
            "--output",
            bsseeker_output.to_str().unwrap(),
        ]);
        cmd
    });
    assert!(bsseeker_stdout.contains("Table:"));
    assert_eq!(
        fs::read_to_string(bsseeker_output).unwrap(),
        "chr1\t3001631\t+\t5\t0\tCG\tCG\n"
    );

    let brat_input = tmp.path().join("brat.tsv");
    let brat_output = tmp.path().join("brat.tab");
    fs::write(&brat_input, "chr1\t552\t552\tCpG:4\t0.25\t-\n").unwrap();
    let brat_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "brat2bismark.pl",
            "--input",
            brat_input.to_str().unwrap(),
            "--output",
            brat_output.to_str().unwrap(),
        ]);
        cmd
    });
    assert!(brat_stdout.contains("Table:"));
    assert_eq!(
        fs::read_to_string(brat_output).unwrap(),
        "chr1\t552\t-\t1\t3\tCG\tCG\n"
    );

    let gff_input = tmp.path().join("genes.gff3");
    let gff_output = tmp.path().join("genes.tab");
    fs::write(
        &gff_input,
        "chr1\tTAIR10\tgene\t10\t20\t.\t+\t.\tID=AT1G00010\n",
    )
    .unwrap();
    let gff_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "gff2tab.pl",
            "--input",
            gff_input.to_str().unwrap(),
            "--output",
            gff_output.to_str().unwrap(),
            "--feature",
            "gene",
        ]);
        cmd
    });
    assert!(gff_stdout.contains("Table:"));
    assert_eq!(
        fs::read_to_string(gff_output).unwrap(),
        "chr1\t10\t20\tAT1G00010\t+\n"
    );

    let first = tmp.path().join("first.svg");
    let second = tmp.path().join("second.svg");
    let merged = tmp.path().join("merged.svg");
    write_svg_figure(&first, "first plot", "#dceeff");
    write_svg_figure(&second, "second plot", "#ffe8d6");
    let inputs = format!("{},{}", first.display(), second.display());
    let merge_stdout = assert_success({
        let mut cmd = viewbs();
        cmd.args([
            "mer_fig.R",
            "--input",
            inputs.as_str(),
            "--output",
            merged.to_str().unwrap(),
            "--ncol",
            "2",
        ]);
        cmd
    });
    assert!(merge_stdout.contains("Plot:"));
    assert!(fs::read_to_string(merged)
        .unwrap()
        .contains("data-viewbs-command=\"merge-figures\""));
}

#![cfg(feature = "plots")]

use std::fs;

use tempfile::tempdir;
use viewbs::{merge_figures, MergeFiguresArgs, OutputKind};

fn write_svg(path: &std::path::Path, label: &str) {
    fs::write(
        path,
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="60" viewBox="0 0 80 60"><rect width="80" height="60" fill="white"/><text x="8" y="32">{label}</text></svg>"#
        ),
    )
    .unwrap();
}

#[test]
fn merge_figures_api_writes_pdf_grid() {
    let tmp = tempdir().unwrap();
    let first = tmp.path().join("first.svg");
    let second = tmp.path().join("second.svg");
    let output = tmp.path().join("merged.pdf");
    write_svg(&first, "first");
    write_svg(&second, "second");

    let result = merge_figures(MergeFiguresArgs {
        inputs: vec![first, second],
        output: output.clone(),
        labels: Vec::new(),
        ncol: 1,
        base_height_cm: 2.54,
        base_aspect_ratio: 1.0,
    })
    .unwrap();

    assert_eq!(result.summary.command, "MergeFigures");
    assert_eq!(result.summary.records_read, 2);
    assert_eq!(result.summary.records_used, 2);
    assert_eq!(result.plots.len(), 1);
    assert!(matches!(result.plots[0].kind, OutputKind::Plot));
    assert_eq!(result.plots[0].path, output);
    assert_eq!(result.plots[0].format, "pdf");
    let pdf = fs::read(result.plots[0].path.clone()).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

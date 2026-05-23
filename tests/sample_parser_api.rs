use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use viewbs::io::samples::parse_sample_args;

#[test]
fn sample_file_preserves_optional_region_paths() {
    let tmp = tempdir().unwrap();
    let sample_list = tmp.path().join("samples.tsv");
    fs::write(
        &sample_list,
        concat!(
            "# methylation_file sample_name optional_region_file\n",
            "wt.tab.gz\tWT\twt_regions.bed\n",
            "mut.tab.gz\tmutant\n",
        ),
    )
    .unwrap();

    let samples = parse_sample_args(&[format!("file:{}", sample_list.display())]).unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].path, PathBuf::from("wt.tab.gz"));
    assert_eq!(samples[0].name, "WT");
    assert_eq!(
        samples[0].region_path,
        Some(PathBuf::from("wt_regions.bed"))
    );
    assert_eq!(samples[1].path, PathBuf::from("mut.tab.gz"));
    assert_eq!(samples[1].name, "mutant");
    assert_eq!(samples[1].region_path, None);
}

#[test]
fn sample_file_mode_rejects_mixed_inline_samples() {
    let error = parse_sample_args(&[
        "file:samples.tsv".to_string(),
        "other.tab.gz,other".to_string(),
    ])
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("only one --sample value is allowed"));
}

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tempfile::tempdir;
use viewbs::meth::Context;
use viewbs::{
    meth_heatmap, CancellationToken, DepthFilter, MethHeatmapArgs, PlotOptions, ProgressEvent,
    ProgressReporter, SampleSpec, ViewBsError,
};

#[derive(Default)]
struct RecordingProgress {
    events: Mutex<Vec<ProgressEvent>>,
}

impl RecordingProgress {
    fn events(&self) -> Vec<ProgressEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl ProgressReporter for RecordingProgress {
    fn report(&self, event: ProgressEvent) {
        self.events.lock().unwrap().push(event);
    }
}

fn indexed_sample() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("data/test_data/test_WT.tab.gz")
}

fn heatmap_args(outdir: &Path, region: PathBuf, prefix: &str) -> MethHeatmapArgs {
    MethHeatmapArgs {
        samples: vec![SampleSpec {
            path: indexed_sample(),
            name: "WT".to_string(),
            region_path: None,
        }],
        region: Some(region),
        outdir: outdir.to_path_buf(),
        prefix: prefix.to_string(),
        contexts: vec![Context::Cg],
        depth: DepthFilter { min: 1, max: 100 },
        merge: false,
        cluster_rows: false,
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
    }
}

#[test]
fn desktop_api_reports_meth_heatmap_progress_by_region() {
    let tmp = tempdir().unwrap();
    let regions = tmp.path().join("regions.bed");
    fs::write(
        &regions,
        "chr2\t1006\t1010\tgene1\nchr2\t1012\t1015\tgene2\n",
    )
    .unwrap();
    let progress = RecordingProgress::default();

    let output = meth_heatmap(
        heatmap_args(tmp.path(), regions, "heat"),
        Some(&progress),
        None,
    )
    .unwrap();

    assert_eq!(output.summary.command, "MethHeatmap");
    let events = progress.events();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events
            .iter()
            .map(|event| event.processed)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );

    for event in &events {
        assert_eq!(event.command, "MethHeatmap");
        assert_eq!(event.phase, "query");
        assert_eq!(event.sample.as_deref(), Some("WT"));
        assert_eq!(event.total, Some(2));
    }
    assert!(events[0].message.contains("chr2:1006-1010"));
    assert!(events[1].message.contains("chr2:1012-1015"));
}

#[cfg(feature = "serde")]
#[test]
fn desktop_api_command_args_round_trip_through_json() {
    let args = heatmap_args(
        Path::new("analysis-out"),
        PathBuf::from("regions.bed"),
        "heat",
    );

    let json = serde_json::to_string(&args).unwrap();
    let decoded: MethHeatmapArgs = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.prefix, "heat");
    assert_eq!(decoded.outdir, PathBuf::from("analysis-out"));
    assert_eq!(decoded.region, Some(PathBuf::from("regions.bed")));
    assert_eq!(decoded.samples[0].name, "WT");
    assert_eq!(decoded.contexts, vec![Context::Cg]);
    assert!(!decoded.plot.enabled);
    assert!(!decoded.distribution_plot.enabled);
}

#[test]
fn desktop_api_cancels_meth_heatmap_with_typed_error() {
    let tmp = tempdir().unwrap();
    let regions = tmp.path().join("regions.bed");
    fs::write(&regions, "chr2\t1006\t1010\tgene1\n").unwrap();
    let cancel = CancellationToken::default();
    cancel.cancel();

    let error = meth_heatmap(
        heatmap_args(tmp.path(), regions, "cancelled"),
        None,
        Some(&cancel),
    )
    .unwrap_err();

    assert!(matches!(error, ViewBsError::Cancelled));
    assert!(!tmp.path().join("cancelled_MethHeatmap_CG.txt").exists());
}

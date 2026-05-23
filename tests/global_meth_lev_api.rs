use std::fs;

use tempfile::tempdir;
use viewbs::api::{DepthFilter, GlobalMethLevArgs, PlotOptions, SampleSpec};
use viewbs::global_meth_lev;

#[test]
fn global_meth_lev_writes_expected_table() {
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

    let args = GlobalMethLevArgs {
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
    };

    let output = global_meth_lev(args, None, None).unwrap();
    assert_eq!(output.summary.records_read, 3);
    assert_eq!(output.summary.records_used, 3);

    let table = fs::read_to_string(tmp.path().join("global.tab")).unwrap();
    assert_eq!(table, "Sample\tCG\tCHG\tCHH\nWT\t0.750\t0.250\t0.000\n");
}

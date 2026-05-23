# ViewBS Benchmarks

Tier 7 benchmarks track runtime, heatmap scaling, plot generation, and output size. They are intended for release checks and scheduled monitoring, not for gating every small pull request.

Run the default small-fixture suite:

```bash
cargo bench --bench performance --all-features
```

For large realistic data, keep the files outside the crates.io package and point the benchmark harness at them:

```bash
VIEWBS_BENCH_DATA_DIR=/path/to/viewbs-benchdata cargo bench --bench performance --all-features
```

The external directory must contain:

```text
sample.tab.gz
sample.tab.gz.tbi
regions.bed
```

`sample.tab.gz` should be a BGZF methylation table with a Tabix index. `regions.bed` should contain enough region-heavy intervals to exercise `MethHeatmap` at release scale. Keep this data in `/benchdata/`, object storage, or another private benchmark-data checkout; do not add it to the crate include list.

The `meth_heatmap_memory_proxy_cells` benchmark reports the number of heatmap matrix cells written after each run. It is a memory proxy for regression tracking, not a direct RSS measurement. For release candidates, pair it with OS-level memory tools such as `/usr/bin/time -v` on Linux.

Useful Tier 7 signals:

- `whole_file_scan_methyl_report`: parser throughput.
- `tabix_query_small_region`: indexed query throughput.
- `meth_geno_window_runtime`: genomic-window command runtime.
- `meth_heatmap_region_runtime`: region-heavy heatmap runtime.
- `meth_heatmap_memory_proxy_cells`: heatmap matrix size proxy.
- `meth_heatmap_plot_generation_time`: Kuva heatmap and distribution SVG generation path.
- `meth_heatmap_output_size_bytes`: table and plot artifact size.

use std::fs;

fn assert_absent(haystack: &str, banned: &[&str]) {
    let normalized = haystack.to_ascii_lowercase();
    for value in banned {
        assert!(
            !normalized.contains(value),
            "packaging metadata should not contain legacy runtime dependency `{value}`"
        );
    }
}

#[test]
fn conda_recipe_builds_the_rust_binary_without_legacy_runtimes() {
    let recipe = fs::read_to_string("conda/meta.yaml").unwrap();

    assert!(recipe.contains("package:"));
    assert!(recipe.contains("name: viewbs"));
    assert!(recipe.contains("compiler('rust')"));
    assert!(recipe.contains("cargo install --locked --path"));
    assert!(recipe.contains("ViewBS --version"));
    assert!(recipe.contains("ViewBS --help"));
    assert!(!recipe.contains("git_rev: v0.1.7"));
    assert_absent(
        &recipe,
        &[
            "perl",
            "r-base",
            "rscript",
            "htslib",
            "bio::db::hts",
            "ggplot2",
            "cowplot",
            "pheatmap",
        ],
    );
}

#[test]
fn dockerfile_uses_a_rust_builder_and_minimal_runtime() {
    let dockerfile = fs::read_to_string("ViewBSdocker/Dockerfile").unwrap();

    assert!(dockerfile.contains("FROM rust:"));
    assert!(dockerfile.contains("cargo build --release --all-features"));
    assert!(dockerfile.contains("COPY --from=builder"));
    assert!(dockerfile.contains("/usr/local/bin/ViewBS"));
    assert!(dockerfile.contains("ENTRYPOINT [\"ViewBS\"]"));
    assert_absent(
        &dockerfile,
        &[
            "biocontainers",
            "perl",
            "r-base",
            "rscript",
            "htslib",
            "cpanm",
            "bio::db::hts",
            "ggplot2",
            "cowplot",
            "pheatmap",
        ],
    );
}

#[test]
fn release_workflow_tests_downloaded_archives_with_real_commands() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(workflow.contains("Test archive contents"));
    assert!(workflow.contains("Get-FileHash -Algorithm SHA256"));
    assert!(workflow.contains("checksum verification failed"));
    assert!(workflow.contains("GlobalMethLev"));
    assert!(workflow.contains("MethOneRegion"));
    assert!(workflow.contains("--plot-format"));
    assert!(workflow.contains("test_WT.tab.gz"));
    assert!(workflow.contains("test_WT.tab.gz.tbi"));
    assert!(workflow.contains("unexpected legacy runtime dependency"));
    assert_absent(&workflow, &["rscript", "cpanm", "bio::db::hts"]);
}

#[test]
fn release_archives_include_user_facing_release_notes() {
    let changelog = fs::read_to_string("CHANGELOG.md").unwrap();
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(changelog.contains("# Changelog"));
    assert!(changelog.contains("0.2.0-alpha.0"));
    assert!(changelog.contains("Rust"));
    assert!(manifest.contains("/CHANGELOG.md"));
    assert!(workflow.contains("Copy-Item \"CHANGELOG.md\""));
}

#[test]
fn cargo_package_does_not_ship_generated_snapshot_artifacts() {
    let gitignore = fs::read_to_string(".gitignore").unwrap();

    assert!(gitignore.contains("*.pending-snap"));
    for entry in fs::read_dir("tests").unwrap() {
        let path = entry.unwrap().path();
        assert!(
            !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".pending-snap")),
            "generated snapshot artifact must not be kept in tests/: {}",
            path.display()
        );
    }
}

#[test]
fn tier7_benchmarks_are_documented_and_can_use_external_data() {
    let benchmarks = fs::read_to_string("benches/performance.rs").unwrap();
    let docs = fs::read_to_string("docs/benchmarks.md").unwrap();
    let workflow = fs::read_to_string(".github/workflows/benchmarks.yml").unwrap();
    let gitignore = fs::read_to_string(".gitignore").unwrap();

    assert!(benchmarks.contains("VIEWBS_BENCH_DATA_DIR"));
    assert!(benchmarks.contains("external_benchmark_dataset"));
    assert!(benchmarks.contains("meth_heatmap_memory_proxy_cells"));
    assert!(benchmarks.contains("meth_heatmap_plot_generation_time"));
    assert!(benchmarks.contains("meth_heatmap_output_size_bytes"));

    assert!(docs.contains("VIEWBS_BENCH_DATA_DIR"));
    assert!(docs.contains("large realistic data"));
    assert!(docs.contains("outside the crates.io package"));
    assert!(docs.contains("memory proxy"));

    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("cargo bench --bench performance --all-features"));
    assert!(workflow.contains("VIEWBS_BENCH_DATA_DIR"));

    assert!(gitignore.contains("/benchdata/"));
}

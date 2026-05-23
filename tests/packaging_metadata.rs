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
fn root_install_docs_are_rust_release_focused() {
    let install = fs::read_to_string("INSTALL").unwrap();
    let environment = fs::read_to_string("environment.yaml").unwrap();

    assert!(install.contains("ViewBS Rust Installation"));
    assert!(install.contains("release archives"));
    assert!(install.contains("cargo install --locked --path ."));
    assert!(install.contains("conda/meta.yaml"));
    assert!(environment.contains("name: viewbs-rust-dev"));
    assert!(environment.contains("rust"));
    assert!(environment.contains("pkg-config"));
    assert_absent(
        &format!("{install}\n{environment}"),
        &[
            "install.pl",
            "rscript",
            "r-base",
            "perl",
            "htslib",
            "bio::db::hts",
            "cpanm",
        ],
    );
}

#[test]
fn conda_upload_docs_and_script_are_rust_release_focused() {
    let docs = fs::read_to_string("conda/conda_upload.md").unwrap();
    let script = fs::read_to_string("conda/conda_upload.sh").unwrap();
    let combined = format!("{docs}\n{script}");

    assert!(docs.contains("# ViewBS Bioconda Release Upload"));
    assert!(docs.contains("conda/meta.yaml"));
    assert!(docs.contains("conda build conda"));
    assert!(docs.contains("ViewBS --version"));
    assert!(docs.contains("ViewBS --help"));
    assert!(docs.contains("GlobalMethLev"));
    assert!(script.contains("set -euo pipefail"));
    assert!(script.contains("conda build"));
    assert!(script.contains("conda run"));
    assert!(script.contains("upload \"$package_path\""));
    assert!(script.contains("anaconda -t \"$ANACONDA_API_TOKEN\""));
    assert!(script.contains("ANACONDA_API_TOKEN"));
    assert!(script.contains("--plot-format svg"));
    assert!(script.contains("conda_global.tab"));
    assert!(script.contains("conda_global.svg"));
    assert_absent(
        &combined,
        &[
            "travis",
            ".travis",
            "cdp",
            "miniconda2",
            "travis_os_name",
            "conda_upload_token",
            "version=`date",
            "python testing",
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
fn legacy_perl_ci_and_backup_docker_docs_are_not_kept() {
    assert!(
        !std::path::Path::new(".travis.yml").exists(),
        "Rust rewrite uses GitHub Actions; the stale Perl Travis config should not remain"
    );
    assert!(
        !std::path::Path::new("ViewBSdocker/README_bak.md").exists(),
        "Docker docs should keep one Rust-focused README, not a stale backup file"
    );
}

#[test]
fn legacy_perl_launcher_is_not_kept_as_the_root_viewbs_entrypoint() {
    assert!(
        !std::path::Path::new("ViewBS").exists(),
        "The Rust rewrite owns the root ViewBS binary name; preserve the Perl launcher under legacy/"
    );

    let legacy = fs::read_to_string("legacy/ViewBS.pl").unwrap();
    assert!(legacy.starts_with("#!/usr/bin/env perl"));
    assert!(legacy.contains("use Getopt::Long::Subcommand"));
    assert!(legacy.contains("basename($main_path) eq \"legacy\""));
    assert!(legacy.contains("dirname($main_path)"));
}

#[test]
fn release_workflow_tests_downloaded_archives_with_real_commands() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(workflow.contains("branches:"));
    assert!(workflow.contains("viewbs-rs"));
    assert!(workflow.contains("macos-15-intel"));
    assert!(
        !workflow.contains("macos-13"),
        "macOS 13 hosted runners are stale; use a current Intel macOS label"
    );
    assert!(workflow.contains("Test archive contents"));
    assert!(workflow.contains("Get-FileHash -Algorithm SHA256"));
    assert!(workflow.contains("checksum verification failed"));
    assert!(workflow.contains("GlobalMethLev"));
    assert!(workflow.contains("MethOneRegion"));
    assert!(workflow.contains("--plot-format"));
    assert!(workflow.contains("test_WT.tab.gz"));
    assert!(workflow.contains("test_WT.tab.gz.tbi"));
    assert!(workflow.contains("unexpected legacy runtime dependency"));
    assert!(
        !workflow.contains("(if (\"${{ runner.os }}\""),
        "PowerShell if statements are not expressions; assign the binary name before Join-Path"
    );
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
    assert!(workflow.contains("Copy-Item \"INSTALL\""));
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
    let record_script = fs::read_to_string("ci/record_tier7_baseline.sh").unwrap();
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
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
    assert!(docs.contains("ci/record_tier7_baseline.sh"));
    assert!(docs.contains("release-candidate baseline"));

    assert!(record_script.contains("VIEWBS_BENCH_DATA_DIR"));
    assert!(record_script.contains("bench --bench performance --all-features"));
    assert!(record_script.contains("/usr/bin/time -v"));
    assert!(record_script.contains("target/criterion"));
    assert!(record_script.contains("baseline.md"));

    assert!(manifest.contains("/ci/record_tier7_baseline.sh"));

    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("cargo bench --bench performance --all-features"));
    assert!(workflow.contains("VIEWBS_BENCH_DATA_DIR"));

    assert!(gitignore.contains("/benchdata/"));
}

#[test]
fn packaging_validation_workflow_runs_docker_and_conda_smoke_tests() {
    let workflow = fs::read_to_string(".github/workflows/packaging.yml").unwrap();

    assert!(workflow.contains("name: Packaging"));
    assert!(workflow.contains("viewbs-rs"));
    assert!(workflow.contains("docker build"));
    assert!(workflow.contains("docker run"));
    assert!(workflow.contains("ViewBSdocker/Dockerfile"));
    assert!(workflow.contains("conda-incubator/setup-miniconda"));
    assert!(workflow.contains("Build and smoke-test Conda package"));
    assert!(workflow.contains("conda/conda_upload.sh"));
    assert!(workflow.contains("VIEWBS_CONDA_UPLOAD: \"0\""));
    assert!(workflow.contains("GlobalMethLev"));
    assert!(workflow.contains("--plot-format svg"));
    assert_absent(&workflow, &["rscript", "perl", "htslib"]);
}

#[test]
fn readme_documents_rust_plot_artifacts_instead_of_legacy_rds_workflow() {
    let readme = fs::read_to_string("README.md").unwrap();
    let normalized = readme.to_ascii_lowercase();

    assert!(readme.contains("ViewBS merge-figures"));
    assert!(readme.contains("SVG"));
    assert!(readme.contains("PDF"));
    assert!(readme.contains("PNG"));
    assert!(normalized.contains("no longer writes `.rds`"));
    assert!(normalized.contains("pure rust"));
    assert!(normalized.contains("without r"));
    assert_absent(
        &readme,
        &["rscript", "readrds", "cowplot", ".tab.rds", "fig1.rds"],
    );
}

#[test]
fn readme_documents_rust_converter_commands_instead_of_legacy_scripts() {
    let readme = fs::read_to_string("README.md").unwrap();

    assert!(readme.contains("ViewBS convert bsseeker"));
    assert!(readme.contains("ViewBS convert brat"));
    assert!(readme.contains("ViewBS convert gff"));
    assert!(readme.contains("bsseeker2bismark.pl"));
    assert!(readme.contains("brat2bismark.pl"));
    assert!(readme.contains("gff2tab.pl"));
    assert_absent(
        &readme,
        &[
            "lib/scripts",
            "scripts to convert",
            "use the r script in viewbs",
        ],
    );
}

#[test]
fn readme_command_examples_describe_native_plot_outputs() {
    let readme = fs::read_to_string("README.md").unwrap();
    let normalized = readme.to_ascii_lowercase();

    assert!(readme.contains("ViewBS MethLevDist"));
    assert!(normalized.contains("plot artifact"));
    assert!(normalized.contains("generated directly by viewbs"));
    assert_absent(
        &readme,
        &[
            "viewbs.pl",
            "shell script which can re-generate",
            "re-generate the figure",
        ],
    );
}

#[test]
fn readme_presents_release_archives_as_primary_distribution() {
    let readme = fs::read_to_string("README.md").unwrap();
    let release_archive_pos = readme
        .find("### Installation from release archives")
        .expect("README should document release archive installation");
    let conda_pos = readme
        .find("### Installation via `conda`")
        .expect("README should keep Conda installation as a fallback");
    let docker_pos = readme
        .find("### Installation with `Docker`")
        .expect("README should keep Docker installation as a fallback");

    assert!(release_archive_pos < conda_pos);
    assert!(release_archive_pos < docker_pos);
    assert!(readme.contains("release archives are the primary distribution"));
    assert!(readme.contains("Fallback package channels"));
    assert_absent(&readme, &["installation via `conda` [recommended]"]);
}

#[test]
fn readme_documents_legacy_differences_for_the_rust_rewrite() {
    let readme = fs::read_to_string("README.md").unwrap();
    let normalized = readme.to_ascii_lowercase();

    assert!(readme.contains("### Legacy differences"));
    assert!(normalized.contains("command names and input formats remain compatible"));
    assert!(normalized.contains("intentional differences"));
    assert!(normalized.contains("serialized `.rds`"));
    assert!(normalized.contains("runtime dependencies"));
    assert!(normalized.contains("helper-script names"));
}

#[test]
fn release_validation_checklist_covers_remaining_release_gates() {
    let docs = fs::read_to_string("docs/release.md").unwrap();
    let normalized = docs.to_ascii_lowercase();

    assert!(docs.contains("# ViewBS Release Validation"));
    assert!(docs.contains("GitHub Actions matrices"));
    assert!(docs.contains("pushes to `viewbs-rs`"));
    assert!(docs.contains("default branch"));
    assert!(docs.contains("VIEWBS_BENCH_DATA_DIR"));
    assert!(docs.contains("Bioconda"));
    assert!(docs.contains("Docker"));
    assert!(docs.contains("macOS"));
    assert!(docs.contains("Windows"));
    assert!(docs.contains("Current 0.2.0-alpha.0 package decision"));
    assert!(normalized.contains("package publication is deferred"));
    assert!(docs.contains("Current 0.2.0-alpha.0 decision"));
    assert!(normalized.contains("unsigned development artifacts"));
    assert!(docs.contains("SHA256 checksums"));
    assert!(normalized.contains("do not tag"));
    assert!(normalized.contains("release-candidate tier 7 baseline"));
    assert!(normalized.contains("code signing decision"));
}

#[test]
fn session_handoff_doc_records_resume_context_and_external_gates() {
    let handoff = fs::read_to_string("SESSION_HANDOFF.md").unwrap();

    assert!(handoff.contains("# ViewBS Rust Rewrite Handoff"));
    assert!(handoff.contains("Use $superpowers to implement plan.md"));
    assert!(handoff.contains("viewbs-rs"));
    assert!(handoff.contains("git bundle create"));
    assert!(handoff.contains("release archives now stage `INSTALL`"));
    assert!(handoff.contains("VIEWBS_BENCH_DATA_DIR"));
    assert!(handoff.contains("Bioconda"));
    assert!(handoff.contains("Docker"));
    assert!(handoff.contains("code signing"));
}

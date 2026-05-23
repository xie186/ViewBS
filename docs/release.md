# ViewBS Release Validation

Use this checklist for each Rust rewrite release candidate. Do not tag a public release until every required gate below has current evidence.

## 1. GitHub Actions matrices

Run the CI workflow on the release branch or release candidate commit:

```bash
gh workflow run ci.yml --ref viewbs-rs
```

Required evidence:

- Tier 0 format, lint, compile passes on Linux.
- Feature-matrix tests pass for default, no-default-features, and all-features.
- Tier 6 smoke tests pass on Linux, macOS, and Windows.

Run the release workflow as a dry-run artifact build before pushing a tag. The workflow runs automatically on pushes to `viewbs-rs`, or it can be started manually once the workflow exists on the repository default branch:

```bash
gh workflow run release.yml --ref viewbs-rs
```

Required evidence:

- Linux, macOS Intel, macOS Apple Silicon, and Windows archives build.
- Each archive passes checksum verification.
- Extracted archive smoke tests pass with `ViewBS --version`, `ViewBS --help`, `GlobalMethLev`, and `MethOneRegion`.
- Linux dependency check does not report Perl, R, htslib, or local build-path dependencies.

## 2. Tier 7 benchmark baseline

Create or refresh an external benchmark-data directory containing:

```text
sample.tab.gz
sample.tab.gz.tbi
regions.bed
```

Run Criterion with the external data:

```bash
VIEWBS_BENCH_DATA_DIR=/path/to/viewbs-benchdata cargo bench --bench performance --all-features
```

Record the release-candidate Tier 7 baseline from `target/criterion` and include:

- parser throughput;
- indexed query throughput;
- `MethGeno` window runtime;
- `MethHeatmap` region runtime;
- `MethHeatmap` memory proxy cells;
- heatmap plot generation time;
- output artifact size.

Pair the heatmap memory proxy with OS-level memory evidence for release candidates, such as `/usr/bin/time -v` on Linux.

## 3. Bioconda and Docker validation

The `Packaging` workflow runs Docker and Conda smoke tests automatically on pushes to `viewbs-rs`, and can also be started manually after the workflow exists on the repository default branch.

Bioconda:

- Build the recipe from `conda/meta.yaml`.
- Confirm the package installs the Rust `ViewBS` binary.
- Run `ViewBS --version` and `ViewBS --help` from a clean Conda environment.
- Run one small table command and one plot command from the package.
- Confirm the package does not require R, Perl, htslib, or external plotting runtimes.

Docker:

- Build `ViewBSdocker/Dockerfile` from the repository root.
- Run `ViewBS --version` and `ViewBS --help` inside the image.
- Run one mounted-volume table command and one plot command.
- Confirm the runtime image does not install R, Perl, htslib, or external plotting runtimes.

## 4. macOS and Windows signing decision

Record the code signing decision for every release candidate:

- macOS: unsigned development archive, signed binary, or signed and notarized archive.
- Windows: unsigned development archive or Authenticode-signed `.exe`.

Current 0.2.0-alpha.0 decision:

- macOS and Windows archives are unsigned development artifacts.
- GitHub release notes must label these artifacts as unsigned and mention expected first-run warnings.
- SHA256 checksums must remain attached for every release archive.
- Revisit signed macOS notarization and Windows Authenticode signing before a stable public release.

If the release artifacts are unsigned:

- label them clearly as unsigned;
- document expected first-run warnings;
- keep SHA256 checksums attached to the GitHub release.

If signing is enabled:

- verify signed artifacts still pass the release archive smoke tests;
- verify macOS notarization behavior on a clean machine;
- verify Windows `.exe` launch behavior from a normal shell.

## Final Release Gate

Before tagging:

- all GitHub Actions matrices have current passing runs for the exact commit;
- a release-candidate Tier 7 baseline has been recorded;
- Bioconda and Docker package validation has passed or the package release is explicitly deferred;
- the macOS and Windows code signing decision is recorded;
- `CHANGELOG.md` includes the release version and notable Rust rewrite changes.

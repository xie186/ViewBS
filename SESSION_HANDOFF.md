# ViewBS Rust Rewrite Handoff

## Objective

Continue the active goal:

```text
Use $superpowers to implement plan.md
```

The canonical plan is `plan.md`. Keep using the Superpowers workflow when continuing the implementation.

## Repository State

- Repository path on the source server: `/data2/github/ViewBS`
- Active branch: `viewbs-rs`
- Current local commit: `f779923 Implement Rust rewrite scaffold`
- Current remote observed in this checkout: `origin/viewbs-rs` points at `f779923`
- Main remote: `https://github.com/xie186/ViewBS.git`

Current uncommitted work in this checkout:

- `README.md`: Rust rewrite documentation updates for standalone release-archive installation as the primary path, Conda/Docker fallback channels, replacement plot artifacts, `ViewBS merge-figures`, converter subcommands, compatibility aliases, native plot-output wording in command examples, and a legacy differences section.
- `plan.md`: implementation-status updates for README migration notes.
- `tests/packaging_metadata.rs`: metadata tests guarding the README against legacy `.rds`/Rscript plotting instructions, old helper-script paths, stale `ViewBS.pl` examples, legacy figure-regeneration shell-script wording, fallback package channels being presented ahead of release archives, missing legacy differences documentation, and missing release validation documentation.
- `docs/release.md`: release-candidate validation checklist for the remaining external gates.
- `.github/workflows/release.yml`: release archives now stage `INSTALL` alongside `README.md`, `license.txt`, `CHANGELOG.md`, and smoke-test example data. The workflow also runs dry-run artifact builds on `viewbs-rs` branch pushes, while GitHub release publication remains tag-only.
- `conda/conda_upload.md` and `conda/conda_upload.sh`: Rust-focused Bioconda build, smoke-test, and opt-in upload flow replacing stale Travis-era instructions.
- `.travis.yml` and `ViewBSdocker/README_bak.md`: removed obsolete Perl-era CI and stale backup Docker documentation.
- `legacy/ViewBS.pl`: moved from the repository root so the root `ViewBS` name is reserved for the Rust binary.
- `INSTALL`, `environment.yaml`, and `INSTALL.pl`: root install guidance now points to Rust release/source/package paths; stale dependency installer was removed.
- `SESSION_HANDOFF.md`: this handoff document.

If cloning on a new server:

```bash
git clone --branch viewbs-rs https://github.com/xie186/ViewBS.git ViewBS
cd ViewBS
```

If GitHub authentication is not available, create and transfer a bundle from this checkout:

```bash
cd /data2/github/ViewBS
git bundle create ../ViewBS-viewbs-rs.bundle viewbs-rs
```

Then on the new server:

```bash
git clone ViewBS-viewbs-rs.bundle ViewBS
cd ViewBS
git switch viewbs-rs
```

## Implemented Rust Rewrite Work

The current branch contains:

- Rust crate `viewbs` and binary `ViewBS`.
- Library API for desktop integration with typed args, outputs, errors, progress, and cancellation.
- CLI wrapper preserving legacy command names and option aliases.
- Rust implementations for the legacy analysis commands on small fixtures.
- Converter subcommands:
  - `ViewBS convert bsseeker`
  - `ViewBS convert brat`
  - `ViewBS convert gff`
  - `ViewBS merge-figures`
- Legacy helper aliases:
  - `bsseeker2bismark.pl`
  - `brat2bismark.pl`
  - `gff2tab.pl`
  - `mer_fig.R`
- Plain/gzip/BGZF input handling, sample files, BED-like regions, one-region strings, Tabix `.tbi`, and CSI `.csi`.
- Kuva-based SVG/PDF/PNG plotting, with plot-dimension validation.
- Golden parity tests, CLI tests, desktop API tests, snapshot tests, plot smoke tests, packaging metadata tests, and Criterion benchmark compilation.
- GitHub Actions workflows for CI, release artifacts, and scheduled/manual benchmarks.
- Rust-focused Bioconda and Docker metadata.
- `CHANGELOG.md`, `docs/coordinates.md`, and `docs/benchmarks.md`.

## Last Local Verification

After the current handoff update, these commands passed on the source server:

```bash
CARGO_HOME=/tmp/viewbs-cargo RUSTUP_HOME=/tmp/viewbs-rustup /tmp/viewbs-cargo/bin/cargo fmt --check
CARGO_HOME=/tmp/viewbs-cargo RUSTUP_HOME=/tmp/viewbs-rustup /tmp/viewbs-cargo/bin/cargo test --test packaging_metadata --all-features
CARGO_HOME=/tmp/viewbs-cargo RUSTUP_HOME=/tmp/viewbs-rustup /tmp/viewbs-cargo/bin/cargo test --all-features
CARGO_HOME=/tmp/viewbs-cargo RUSTUP_HOME=/tmp/viewbs-rustup /tmp/viewbs-cargo/bin/cargo test --no-default-features
CARGO_HOME=/tmp/viewbs-cargo RUSTUP_HOME=/tmp/viewbs-rustup /tmp/viewbs-cargo/bin/cargo clippy --all-targets --all-features -- -D warnings
```

The source session also checked formatting-only diffs and generated snapshot artifacts after the handoff update.

On a normal Rust installation, the same commands can usually be run as `cargo ...` without the explicit `CARGO_HOME` and `RUSTUP_HOME` prefixes.

## Known Remaining Work

`plan.md` currently lists release work that still needs external validation:

- Run GitHub Actions matrices on Linux, macOS, and Windows before tagging a release.
- Populate external large performance/memory datasets for `VIEWBS_BENCH_DATA_DIR` and record Tier 7 baselines.
- Validate and publish the updated Bioconda and Docker packages from release artifacts.
- Decide whether macOS and Windows artifacts need code signing before public distribution.

Recent post-handoff progress:

- `README.md` now documents SVG/PDF/PNG plot artifacts, states that `.rds` figure objects are no longer written, and shows `ViewBS merge-figures` as the replacement for the legacy R helper workflow.
- `tests/packaging_metadata.rs` includes a metadata test to prevent reintroducing README instructions for the old `Rscript`/serialized-figure workflow.
- `README.md` now documents `ViewBS convert bsseeker`, `ViewBS convert brat`, `ViewBS convert gff`, and the compatibility aliases `bsseeker2bismark.pl`, `brat2bismark.pl`, and `gff2tab.pl`.
- `tests/packaging_metadata.rs` includes a metadata test to prevent reintroducing README instructions that send users to old helper-script paths.
- `README.md` command examples now describe plot artifacts generated directly by ViewBS and use `ViewBS MethLevDist` instead of `ViewBS.pl MethLevDist`.
- `tests/packaging_metadata.rs` includes a metadata test to prevent reintroducing stale `ViewBS.pl` examples or old figure-regeneration shell-script wording.
- `README.md` now presents release archives before Conda/Docker and labels Conda/Docker as fallback package channels.
- `tests/packaging_metadata.rs` includes a metadata test to keep release archives documented as the primary distribution path.
- `README.md` now includes a `Legacy differences` section under the Rust rewrite migration notes.
- `tests/packaging_metadata.rs` includes a metadata test to keep the Rust rewrite legacy differences documented.
- `docs/release.md` now documents the GitHub Actions matrix, external Tier 7 benchmark, Bioconda/Docker validation, and macOS/Windows signing gates for release candidates.
- `tests/packaging_metadata.rs` includes a metadata test to keep that release validation checklist present.
- `.github/workflows/release.yml` now stages `INSTALL` in release archives so copied release bundles include user-facing install notes.
- `tests/packaging_metadata.rs` includes a metadata test to keep release archives staging `INSTALL`.
- `.github/workflows/release.yml` now runs dry-run artifact builds on `viewbs-rs` pushes because the manual dispatch endpoint is unavailable until the workflow exists on the default branch.
- `tests/packaging_metadata.rs` includes a metadata test to keep the release dry-run trigger available on the rewrite branch.
- `.github/workflows/release.yml` now avoids using `if` as a PowerShell expression in the Windows archive extraction smoke test.
- `tests/packaging_metadata.rs` includes a metadata test for that Windows release-workflow syntax issue.
- `.github/workflows/release.yml` now uses `macos-15-intel` for the x86_64 macOS release artifact because the old `macos-13` label no longer starts reliably on GitHub-hosted runners.
- `tests/packaging_metadata.rs` includes a metadata test to keep stale macOS 13 release runners out.
- `conda/conda_upload.md` and `conda/conda_upload.sh` now describe/build/smoke-test the Rust Conda package and require explicit upload credentials/flags.
- `tests/packaging_metadata.rs` includes a metadata test to prevent the Conda upload path from regressing to Travis-era package publishing instructions.
- `.travis.yml` and `ViewBSdocker/README_bak.md` were removed so the Rust rewrite does not retain stale Perl-era CI or duplicate Docker docs.
- `tests/packaging_metadata.rs` includes a metadata test to keep those obsolete files out.
- The old root `ViewBS` Perl launcher was moved to `legacy/ViewBS.pl`.
- `tests/packaging_metadata.rs` includes a metadata test to keep the legacy launcher out of the root `ViewBS` path.
- `INSTALL` and `environment.yaml` now describe Rust installation/development paths, and `INSTALL.pl` was removed.
- `tests/packaging_metadata.rs` includes a metadata test to prevent root install files from reintroducing Perl/R/htslib dependency setup.

Suggested next work:

- Continue auditing README and release docs against `plan.md`, especially any remaining legacy wording around command examples, standalone distribution, and release readiness.
- If changing behavior, keep using TDD: add the smallest failing test first, then implement and verify.

## Notes For The Next Session

- Do not mark the active goal complete until the full `plan.md` scope is audited and verified.
- Keep edits scoped and use existing code/test patterns.
- Use `rg` for searches.
- Use `apply_patch` for manual edits.
- Do not commit or push without checking `git status -sb` and confirming the intended scope.
- `target/`, `/benchdata/`, and `*.pending-snap` are ignored.
- If push fails, check GitHub authentication. In the previous source session, HTTPS push originally failed without credentials, but the current checkout now observes `origin/viewbs-rs` at `f779923`.

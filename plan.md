# Plan: Rewrite ViewBS in Pure Rust

## Goal

Rewrite ViewBS as a pure Rust command-line tool while preserving the existing user-facing behavior, command names, input formats, and tabular outputs.

The rewritten binary should still install and run as `ViewBS`. A temporary development repository or branch can be called `ViewBS-rs`, but the final user-facing tool name should remain `ViewBS`.

In this plan, "pure Rust" primarily means the user should receive a standalone executable for their operating system. Users should not need to install Perl, R, htslib, Conda, Docker, or extra plotting tools just to run ViewBS.

Standalone distribution means:

- Publish prebuilt executables for Linux, macOS, and Windows.
- Keep runtime dependencies bundled into the executable whenever practical.
- Avoid dependencies that require users to install system libraries manually.
- Avoid shelling out to `gzip`, `tabix`, `bgzip`, `Rscript`, or plotting commands.
- Prefer Rust crates that make cross-platform packaging simple.
- Keep the tool usable from a copied release archive, not only from a managed Conda environment.

Implementation preferences:

- No Perl runtime.
- No R runtime or `Rscript`.
- Avoid C htslib bindings as required runtime dependencies.
- Use existing Rust crates wherever they already solve a generic problem.
- Only write custom code for ViewBS-specific methylation logic and plot layout glue.

## Current Tool Surface

The current top-level executable is `ViewBS`, with these subcommands:

- `MethCoverage`
- `BisNonConvRate`
- `GlobalMethLev`
- `MethLevDist`
- `MethGeno`
- `MethHeatmap`
- `MethOverRegion`
- `MethOneRegion`

Important existing files:

- CLI dispatch: `ViewBS`
- Shared defaults: `lib/SubCmd/CommonArgument.pm`
- Sample parsing: `lib/Meth/Sample.pm`
- Current computation modules: `lib/Meth/*.pm`
- Current plot scripts: `lib/Meth/*.R`
- Current help text: `doc/pod4help_*.txt`
- User examples and expected outputs: `README.md`

## High-Level Strategy

Do not rewrite everything at once. Build a Rust implementation behind the same command surface and validate command outputs against the existing Perl/R implementation.

Recommended sequence:

1. Freeze current behavior with golden tests.
2. Design the reusable Rust library API.
3. Build Rust CLI and shared input readers on top of the library.
4. Port pure streaming commands first.
5. Port indexed region-query commands.
6. Replace R plotting with Rust plotting.
7. Package through Cargo, crates.io, Bioconda, and Docker.
8. Release as a major version after parity is proven.

## Implementation Status

Status as of 2026-05-23:

- Rust crate scaffold is in place with a library crate named `viewbs` and a binary named `ViewBS`.
- The public library API exposes typed argument/output/error/progress/cancellation types for desktop tool integration.
- All legacy analysis commands are represented in Rust and are covered by library/API, CLI, and golden parity tests on small fixtures.
- Converter commands are represented as `ViewBS convert bsseeker`, `ViewBS convert brat`, `ViewBS convert gff`, and `ViewBS merge-figures`.
- Legacy helper names are accepted as CLI compatibility aliases: `bsseeker2bismark.pl`, `brat2bismark.pl`, `gff2tab.pl`, and `mer_fig.R`.
- Methylation input parsing supports plain/gzip/BGZF-style reads, sample files, BED-like regions, one-region strings, Tabix `.tbi`, and CSI `.csi`.
- Plotting is implemented through the `kuva` Rust library with SVG, PDF, and PNG output support; no R or `Rscript` runtime is required for plots.
- Test tiers are wired into local tests and GitHub Actions, including feature-matrix tests, cross-platform smoke tests, release artifact builds, plot smoke tests, golden parity tests, and Criterion benchmark compilation.
- Snapshot tests cover top-level help text, structured error messages, and a small JSON config example.
- Cargo packaging is constrained to Rust crate files, docs, tests, benchmarks, and required fixtures so crates.io distribution does not ship the legacy Perl/R implementation.
- Bioconda and Docker metadata now build/use the Rust binary without Perl, R, htslib, or external plotting runtimes.
- Release artifact tests verify archive checksums, run `ViewBS --version` and `ViewBS --help`, run a small `GlobalMethLev` plot smoke command, run an indexed `MethOneRegion` smoke command, and check Linux release binary dependencies for legacy runtime libraries.
- Release archives and Cargo packages include `CHANGELOG.md` with Rust rewrite release notes.
- Tier 7 Criterion benchmarks cover parser throughput, indexed queries, `MethGeno`, `MethHeatmap` runtime, a heatmap memory-size proxy, heatmap plot generation, and output artifact size. The benchmark harness can use `VIEWBS_BENCH_DATA_DIR` for large external data that stays outside the crates.io package.
- A scheduled/manual GitHub Actions benchmark workflow runs the Tier 7 suite and uploads Criterion reports.
- README migration notes document that `.rds` figure objects were removed, SVG/PDF/PNG are the replacement plot artifacts, `ViewBS merge-figures` replaces the legacy R helper workflow, and the Rust binary runs without R, Perl, htslib, or external plotting tools.
- README input-preparation notes document the Rust converter subcommands and legacy helper aliases instead of sending users to old helper-script paths.
- README command examples now use the `ViewBS` binary name and describe plot artifacts generated directly by ViewBS instead of legacy regeneration shell scripts.
- README installation notes now present standalone release archives as the primary distribution and Conda/Docker as fallback package channels.
- README migration notes include a dedicated legacy differences section covering removed `.rds` objects, direct PDF/SVG/PNG plots, `ViewBS merge-figures`, removed runtime dependencies, and helper-name compatibility aliases.
- `docs/release.md` documents the remaining release validation gates for GitHub Actions matrices, external Tier 7 benchmark data, Bioconda/Docker validation, and macOS/Windows signing decisions.
- Release archive staging now includes `INSTALL` so copied bundles carry user-facing install notes.
- The release artifact workflow now runs as a dry-run archive build on `viewbs-rs` branch pushes, while publishing remains restricted to version tags.
- The release artifact workflow uses Windows-compatible PowerShell in archive extraction smoke tests.
- The release artifact workflow uses the current `macos-15-intel` runner label for x86_64 macOS artifacts instead of the retired macOS 13 runner.
- `conda/conda_upload.md` and `conda/conda_upload.sh` now document and automate Rust package build, smoke-test, and opt-in Anaconda upload flow without Travis-era upload logic.
- The Conda smoke script now runs a real `GlobalMethLev` table-and-SVG plot command from the built package.
- A packaging GitHub Actions workflow now builds and smoke-tests the Docker image and Conda package on `viewbs-rs` branch pushes.
- The 0.2.0-alpha.0 macOS and Windows signing decision is recorded as unsigned development artifacts with checksums and first-run warning notes.
- Obsolete Perl-era Travis CI configuration and stale backup Docker README were removed; CI is now represented by GitHub Actions.
- The legacy Perl launcher was moved from the repository root to `legacy/ViewBS.pl` so the root `ViewBS` name belongs to the Rust binary.
- Root install files now document Rust release archives, Cargo source builds, package-maintainer paths, and a small Rust development Conda environment instead of Perl/R dependency installation.

Remaining release work:

- Populate external large realistic performance/memory datasets for `VIEWBS_BENCH_DATA_DIR` and record release-candidate Tier 7 baselines.
- Publish or explicitly defer the updated Bioconda and Docker packages from release artifacts.
- Revisit macOS notarization and Windows Authenticode signing before a stable public release.

## Repository Strategy

Recommended approach:

- Keep this repository as the canonical project.
- Create a long-lived branch named `rust-rewrite`, or a temporary repo named `ViewBS-rs`.
- Make the Cargo binary name `ViewBS` from the beginning.
- Keep the old Perl version available as `legacy/ViewBS.pl` or on a maintenance branch until Rust parity is proven.

Avoid creating a new final tool name unless the Rust version intentionally breaks old examples, output formats, or scientific behavior.

## Rust Project Layout

Suggested structure:

```text
Cargo.toml
src/
  lib.rs
  main.rs
  api.rs
  cli.rs
  errors.rs
  io/
    methyl_report.rs
    samples.rs
    regions.rs
    fasta.rs
    tabix.rs
  meth/
    aggregate.rs
    context.rs
    depth.rs
    record.rs
  commands/
    bis_non_conv_rate.rs
    global_meth_lev.rs
    meth_coverage.rs
    meth_geno.rs
    meth_heatmap.rs
    meth_lev_dist.rs
    meth_one_region.rs
    meth_over_region.rs
  plot/
    mod.rs
    bar.rs
    line.rs
    heatmap.rs
    violin.rs
    layout.rs
  output/
    tables.rs
    paths.rs
tests/
  cli_parity.rs
  golden/
testdata/
  small/
```

Keep command modules thin. Shared methylation parsing, interval query, aggregation, and plot code should live outside command modules. The CLI should be a wrapper around the public library API, not the owner of the core logic.

`Cargo.toml` should expose both a library crate and a binary:

```toml
[lib]
name = "viewbs"
path = "src/lib.rs"

[[bin]]
name = "ViewBS"
path = "src/main.rs"
```

## Library API for Desktop Tool Integration

ViewBS should be published as both:

- a command-line binary named `ViewBS`;
- a Rust library crate named `viewbs` for use inside another desktop tool.

The desktop tool should not call the CLI binary unless it deliberately wants process isolation. It should call the Rust library directly.

### Library Design Goals

- Stable typed API for all analysis commands.
- No direct writes to stdout or stderr from library code.
- No process exits or panics for normal user/data errors.
- No hidden global state.
- Clear progress reporting for long-running jobs.
- Cancellation support for desktop UI responsiveness.
- Deterministic output tables and plot artifacts.
- API usable from GUI runtimes such as Tauri, egui, iced, Slint, or a custom desktop application.

### Public API Shape

Expose one function per command plus shared configuration types:

```rust
pub fn global_meth_lev(
    args: GlobalMethLevArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput>;

pub fn meth_heatmap(
    args: MethHeatmapArgs,
    progress: Option<&dyn ProgressReporter>,
    cancel: Option<&CancellationToken>,
) -> Result<CommandOutput>;
```

Each command argument type should be serializable:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMethLevArgs {
    pub samples: Vec<SampleSpec>,
    pub outdir: PathBuf,
    pub prefix: String,
    pub depth: DepthFilter,
    pub method_average: bool,
    pub plot: PlotOptions,
}
```

Use `serde` for configuration structs so a desktop app can save, load, and pass jobs as JSON.

### Command Output Type

Return structured output rather than printing paths:

```rust
pub struct CommandOutput {
    pub tables: Vec<OutputFile>,
    pub plots: Vec<OutputFile>,
    pub logs: Vec<LogMessage>,
    pub summary: CommandSummary,
}

pub struct OutputFile {
    pub kind: OutputKind,
    pub path: PathBuf,
    pub format: String,
}
```

This lets a desktop tool display generated tables and plots immediately after a command finishes.

### Progress and Cancellation

Long-running commands such as `MethHeatmap`, `MethGeno`, and `MethOverRegion` need progress callbacks.

Recommended trait:

```rust
pub trait ProgressReporter: Send + Sync {
    fn report(&self, event: ProgressEvent);
}
```

Progress events should include:

- command name;
- current phase;
- sample name if applicable;
- records or regions processed;
- total records or regions when known;
- human-readable status message.

Use a lightweight local cancellation token, such as a wrapper around `Arc<AtomicBool>`. Do not depend on Tokio cancellation types in the core library. If the desktop app cancels a job, the library should stop at the next safe checkpoint and return a cancellation error with any partial output clearly marked.

### Error Handling

Library errors should be typed and user-displayable:

```rust
pub enum ViewBsError {
    InvalidInput { path: PathBuf, message: String },
    ParseError { path: PathBuf, line: Option<u64>, message: String },
    IndexedQueryError { path: PathBuf, region: String, message: String },
    PlotError { message: String },
    Cancelled,
}
```

The CLI can convert these errors into stderr messages and exit codes. The desktop app can render the same errors in dialogs or job logs.

### API Stability

Use feature gates to keep the library practical for different consumers:

- `default = ["plots"]`
- `plots`: enables Rust plot generation.
- `serde`: enables config serialization.
- `parallel`: enables `rayon`.
- `cli`: enables CLI-only dependencies if any are introduced.

Avoid exposing low-level implementation details from `noodles`, `ndarray`, or plotting crates in public types unless necessary. This keeps the public API stable even if internals change.

### Threading Model

The library should be synchronous at the core. A desktop tool can run work on its own background thread or async task.

Do not require a specific async runtime such as Tokio in the core API. If async wrappers are useful later, add them behind an optional feature.

## Core Data Model

Create typed structs early:

```rust
struct MethylRecord {
    chrom: String,
    pos: u64,
    strand: char,
    methylated: u64,
    unmethylated: u64,
    context: Context,
    trinuc: String,
}

enum Context {
    Cg,
    Chg,
    Chh,
    Cxx,
    Other(String),
}

struct SampleSpec {
    path: PathBuf,
    name: String,
    region_path: Option<PathBuf>,
}

struct DepthFilter {
    min: u64,
    max: u64,
}

struct Region {
    chrom: String,
    start: u64,
    end: u64,
    name: Option<String>,
    strand: Option<char>,
}
```

Design rule: all command logic should consume these typed structures, not raw split strings.

## Recommended Rust Crates

Use existing crates for standard problems.

| Area | Recommended crate | Why |
| --- | --- | --- |
| CLI | `clap` with `derive` | Mature Rust CLI parser with subcommand support. |
| Errors | `anyhow`, `thiserror` | Clear command errors and typed internal errors. |
| Serialization | `serde`, `serde_json` | Save/load command configs and exchange jobs with desktop tools. |
| TSV parsing and writing | `csv` with tab delimiter | Fast, configurable delimited reader/writer. |
| Gzip streams | `flate2` default Rust backend | Pure Rust decompression through `miniz_oxide` by default. |
| BGZF and indexed files | `noodles-bgzf`, `noodles-tabix`, `noodles-csi` | Pure Rust bioinformatics file handling. |
| FASTA and FAI | `noodles-fasta` | FASTA reader and FAI support. |
| BED | `noodles-bed` | BED parser for region files. |
| GFF3 | `noodles-gff` | Useful for replacing `gff2tab.pl` later. |
| Parallelism | `rayon` | Sample-level and region-level parallelism. |
| Matrices | `ndarray` | Heatmap matrices and clustering inputs. |
| Hierarchical clustering | `kodama` | Fast agglomerative clustering and dendrogram support. |
| Plotting | `kuva` first, `plotters` fallback only if needed | Scientific plotting in Rust with SVG output and optional PNG/PDF backends. |
| SVG to PDF | `kuva` `pdf` feature, backed by `svg2pdf` | Prefer `kuva::render_to_pdf`; use direct `svg2pdf` only for custom composed SVGs. |
| Color maps | `colorous` | Viridis and other scientific color maps. |
| KDE for violin plots | `kuva` violin/KDE support | Avoid hand-rolling density estimates unless a ViewBS-specific overlay requires it. |
| CLI tests | `assert_cmd`, `tempfile` | Run `ViewBS` in integration tests and inspect outputs. |
| Snapshot tests | `insta` | Stable snapshots for help text and small output tables. |
| Benchmarks | `criterion` | Track performance against old Perl implementation. |

Avoid by default:

- `rust-htslib`: useful and mature, but it binds to htslib and is not pure Rust.
- `plotly` or `charming` for core plots: good APIs, but rendering/export paths depend on JavaScript, browser automation, or web runtimes.
- `polars`: powerful, but likely unnecessary for streaming TSV methylation calculations.

## Plotting Recommendation

### Plot Requirements

Current R scripts generate these plot types:

| Command | Current R plot | Rust requirement |
| --- | --- | --- |
| `GlobalMethLev` | grouped bar chart | grouped bar chart by context and sample |
| `BisNonConvRate` | bar chart faceted by context | bar chart plus per-context panels |
| `MethCoverage` | reverse cumulative coverage line plot, faceted by sample | line plot panels |
| `MethLevDist` | binned bar charts faceted by context and sample | histogram/bar grid |
| `MethGeno` | line plot faceted by chromosome | multi-panel genomic line plot |
| `MethOverRegion` | average methylation metaplot line chart | line plot with vertical markers |
| `MethHeatmap` | clustered heatmap plus violin-boxplot | heatmap, clustering, violin and box overlay |
| `MethOneRegion` | per-sample segment/lollipop plot | per-sample methylation segments |
| `mer_fig.R` | cowplot-style figure grid | subplot or SVG/PDF composition |

### Primary Backend: `kuva`

Use `kuva` as the plotting backend for ViewBS.

Reasons:

- It is a Rust scientific plotting library.
- It targets bioinformatics use cases.
- It supports SVG output by default.
- It has optional PNG and PDF backends through feature flags.
- It supports the core ViewBS plot needs: line, bar, histogram, box, violin, and heatmap-style plots.
- It provides both a library API and a CLI, but ViewBS should use the library API directly.

Recommended dependency:

```toml
kuva = { version = "0.2", features = ["full"] }
```

Use narrower feature flags if release size matters:

- default/SVG only for minimal builds;
- `png` for PNG output;
- `pdf` for PDF output;
- `full` for PNG plus PDF.

Risks:

- `kuva` is newer than low-level drawing libraries such as `plotters`.
- Some ViewBS-specific layouts may need small wrapper code around `kuva`.
- Clustered heatmap dendrograms and cowplot-style figure composition should be verified in a spike.

Mitigation:

- Put all plotting behind a local `PlotBackend` trait.
- Do a short spike rendering one example for each ViewBS plot.
- If `kuva` blocks any required figure, first extend ViewBS-side layout code around `kuva`.
- Use `plotters` only as a fallback for a specific missing plot or layout feature.

### Fallback: `plotters`

Use `plotters` only if `kuva` cannot meet a specific production need.

Reasons:

- Mature and widely used.
- Pure Rust drawing library.
- Supports SVG and bitmap backends.
- Supports line series, histograms, and boxplots.

Tradeoff:

- More custom code will be needed for faceting, violin plots, heatmaps, dendrograms, and cowplot-style layouts.

### PDF Output

Current ViewBS examples produce PDF. Keep PDF support for compatibility using `kuva`'s `pdf` feature.

Recommended flow:

1. Build `kuva` plots through the ViewBS `PlotBackend`.
2. Render SVG with `kuva::render_to_svg` for testing and optional user output.
3. Render PDF with `kuva::render_to_pdf` for the default legacy-compatible output.
4. Render PNG with `kuva::render_to_png` when requested.

This keeps plotting inside the Rust process and avoids R, Cairo, LaTeX, browser automation, and external command-line converters.

Also support `--plot-format svg` and `--plot-format png`.

Recommended default:

- Default to `pdf` for backward compatibility.
- Also write `svg` when `--keep-svg` is enabled, because SVG is easier to inspect and test.
- Use embedded fonts for standalone SVGs when practical, so users can open plots on systems without matching fonts.

### Heatmap and Clustering

For `MethHeatmap`:

- Store matrix as `ndarray::Array2<f64>` or `Vec<Vec<f64>>`.
- Use `kodama` for row and column hierarchical clustering.
- Use `kuva` heatmap rendering if output quality is acceptable.
- Use `colorous::VIRIDIS` or another perceptually uniform palette for methylation levels.
- Preserve current `--cluster_rows`, `--cluster_cols`, and `--random_region` behavior.

Important behavior to preserve:

- Drop rows with incomplete values for heatmap plotting, matching current R `complete.cases`.
- If the matrix is too large, sample up to `--random_region` rows.
- Write the full table even if the plotted heatmap uses a sampled subset.

### Violin and Box Plot

For `MethHeatmap` histogram/distribution output:

- Prefer `kuva` violin and box plot support.
- If a combined violin-plus-box overlay is not directly supported, compose it in ViewBS using multiple `kuva` plot layers or a small custom wrapper.

Do not hand-roll KDE unless `kuva` cannot support the needed ViewBS distribution plot after testing.

## Command Porting Plan

### Phase 0: Golden Test Data

- [ ] Collect one small input set per command.
- [ ] Run existing ViewBS and save all `.tab` and `.txt` outputs.
- [ ] Save command stdout/stderr if users depend on it.
- [ ] Decide numeric tolerance for floating-point outputs.
- [ ] Add a `testdata/golden/legacy/` directory.
- [ ] Document exact legacy command used for each expected output.

Priority outputs:

- Data tables first.
- Plot file existence and dimensions second.
- Pixel-perfect plot matching is not required.

### Phase 1: Rust CLI Skeleton

- [ ] Create Cargo project.
- [ ] Binary name: `ViewBS`.
- [ ] Add `clap` subcommands matching legacy names exactly.
- [ ] Support legacy option aliases and spelling where possible.
- [ ] Implement `--help` and `--version`.
- [ ] Preserve defaults from `lib/SubCmd/CommonArgument.pm`.
- [ ] Add integration tests for command parsing.

Compatibility note:

- Keep legacy subcommand capitalization: `MethCoverage`, `GlobalMethLev`, etc.
- Optionally add lowercase aliases later, but do not remove old names.

### Phase 2: Shared Input Readers

- [ ] Implement methylation report parser:
  - columns: chromosome, position, strand, methylated count, unmethylated count, context, trinucleotide context.
  - support plain TSV, `.gz`, and BGZF.
  - validate numeric fields and report line numbers on errors.
- [ ] Implement sample parser:
  - repeated `--sample file.gz,name`
  - `--sample file:sample_info.txt`
  - optional per-sample region file for commands that support it.
- [ ] Implement region parser:
  - BED-like region files.
  - one-region strings like `chr5:19499001-19499600`.
  - preserve legacy coordinate behavior first.
- [ ] Implement indexed methylation reader:
  - `.tbi` with `noodles-tabix`.
  - `.csi` with `noodles-csi`.
  - one reusable reader per sample where possible.
- [ ] Implement FASTA/FAI reader for `MethCoverage` and `MethGeno`.

Important: current README documents both `.tbi` and `.csi`, so the Rust version should support both.

### Phase 3: Streaming Commands

Port commands that can work by scanning each methylation file from start to end.

#### `GlobalMethLev`

- [ ] Sum methylated C and total depth by sample and context.
- [ ] Preserve weighted average default.
- [ ] Preserve `--methodAverage`.
- [ ] Preserve output table:
  - `Sample`
  - contexts as columns
- [ ] Add grouped bar plot.

#### `BisNonConvRate`

- [ ] Filter records by `--chrom`.
- [ ] Default context to `CXX`.
- [ ] Preserve `CXX` as all cytosines.
- [ ] Preserve output table:
  - `Sample`
  - `BisNonConvRate`
  - `C_number`
  - `Total_Depth`
  - `Context`
- [ ] Add context-faceted bar plot.

#### `MethCoverage`

- [ ] Count cytosine depths by sample and context.
- [ ] If zero-depth cytosines are present, use file row counts as denominators.
- [ ] Otherwise calculate reference context counts from FASTA.
- [ ] Preserve reverse cumulative percentage calculation.
- [ ] Preserve output table:
  - `Sample`
  - `Context`
  - `Depth`
  - `Percentage`
- [ ] Add line plot panels by sample.

#### `MethLevDist` Without `--region`

- [ ] Bin methylation levels by `--binMethLev`.
- [ ] Preserve bin midpoint behavior.
- [ ] Preserve output table:
  - `Sample`
  - `Context`
  - `MethLevBinMidPoint`
  - `Number`
  - `Percentage`
- [ ] Add faceted bar plot.

### Phase 4: Indexed Region Commands

Port commands that require genomic interval queries.

#### `MethOneRegion`

- [ ] Parse `chr:start-end`.
- [ ] Apply `--flank`.
- [ ] Query each sample with Tabix/CSI.
- [ ] Preserve context and depth filters.
- [ ] Preserve output table:
  - `Sample`
  - `chr`
  - `position`
  - `C_num`
  - `T_num`
  - `MethylationLevel`
- [ ] Add per-sample segment/lollipop plot.

#### `MethHeatmap`

- [ ] Parse region file.
- [ ] Query each region for each sample and context.
- [ ] Compute weighted methylation level.
- [ ] Preserve `NA` for missing regions.
- [ ] Preserve `--merge` behavior.
- [ ] Write one matrix per context or one merged matrix.
- [ ] Add clustered heatmap.
- [ ] Add violin plus box plot.

#### `MethOverRegion`

- [ ] Parse region file with optional strand.
- [ ] Preserve upstream/body/downstream bins.
- [ ] Preserve `--binLength`, `--binNumber`, `--flank`, `--minLength`, `--maxLength`.
- [ ] Preserve strand-aware binning.
- [ ] Preserve output table:
  - `sample_name`
  - `region`
  - `bin_num`
  - `C_number`
  - `T_number`
  - `Methylation_level`
- [ ] Add metaplot line chart with vertical body boundary markers.

#### `MethGeno`

- [ ] Read chromosome lengths from `.fai` or equivalent genome length file.
- [ ] Preserve `--win`, `--step`, `--minLength`, `--maxChromNumber`.
- [ ] Query windows per chromosome and sample.
- [ ] Reuse indexed readers rather than reopening per window.
- [ ] Preserve output table:
  - `chr`
  - `stt`
  - `end`
  - `sample_name`
  - `C_number`
  - `T_number`
  - `Methylation_level`
- [ ] Add chromosome-faceted line plot.

### Phase 5: Conversion Scripts

Port helper scripts after core commands:

- [ ] `lib/scripts/bsseeker2bismark.pl`
- [ ] `lib/scripts/brat2bismark.pl`
- [ ] `lib/scripts/gff2tab.pl`
- [ ] `lib/scripts/mer_fig.R`

Suggested Rust subcommands:

- `ViewBS convert bsseeker`
- `ViewBS convert brat`
- `ViewBS convert gff`
- `ViewBS merge-figures`

Keep the old script names as wrapper aliases if users rely on them.

## Output Compatibility

Default behavior should write:

- legacy `.tab` or `.txt` table output;
- plot output, defaulting to PDF;
- no `.rds` files.

Replacement for `.rds`:

- Use SVG/PDF outputs for figure reuse.
- Optionally write a small `.plot.json` manifest containing plot metadata and source table path.

Do not change column names, ordering, or rounding until after parity tests pass.

## Coordinates and Compatibility Risk

The old code mixes BED-like files, one-based Tabix regions, and command-specific start/end adjustments. This is the highest scientific correctness risk.

Plan:

- [ ] Write tests for all coordinate conversions.
- [ ] Preserve legacy behavior by default.
- [ ] Add explicit documentation for accepted coordinate systems.
- [ ] Only add cleaner coordinate modes behind explicit flags, not as silent default changes.

## Performance Plan

Expected Rust wins:

- Streaming parsers avoid shell pipelines.
- Indexed readers are reused instead of reopened repeatedly.
- Sample-level parallelism with `rayon`.
- Region-level parallelism for heatmap and metaplot commands.
- Lower memory use for streaming commands.

Guidelines:

- Stream whole-genome commands.
- Keep one indexed reader per sample per worker.
- Avoid storing all methylation records unless needed.
- Use `f64` for calculations initially to match Perl/R behavior.
- Consider `f32` only for large heatmap plotting after parity is stable.

Benchmarks:

- [ ] Whole-file scan throughput.
- [ ] Tabix region query throughput.
- [ ] `MethHeatmap` memory use.
- [ ] `MethGeno` window query runtime.

## Testing Strategy

Use explicit test tiers so local development, CI, and release validation stay fast and predictable.

| Tier | Scope | Runs When | Purpose |
| --- | --- | --- | --- |
| Tier 0 | Formatting, linting, compile checks | every commit and PR | Catch simple mistakes quickly. |
| Tier 1 | Unit tests | every commit and PR | Validate parsers, context handling, depth filters, coordinate conversion, binning, and numerical helpers. |
| Tier 2 | Library API integration tests | every commit and PR | Call `viewbs` directly without invoking the CLI, matching how a Rust desktop app would use the library. |
| Tier 3 | CLI integration tests | every commit and PR | Run `ViewBS` with `assert_cmd` and verify args, exit codes, output files, and errors. |
| Tier 4 | Golden parity tests | every PR touching command logic | Compare Rust `.tab` and `.txt` outputs against legacy ViewBS outputs. |
| Tier 5 | Plot smoke tests | every PR touching plotting code | Verify SVG/PDF/PNG files are valid, non-empty, correctly sized, and include expected labels. |
| Tier 6 | Cross-platform tests | nightly and before release | Build and run the small test suite on Linux, macOS, and Windows. |
| Tier 7 | Performance and memory benchmarks | nightly or manual release check | Track runtime and memory for large files and region-heavy commands. |
| Tier 8 | Release artifact tests | every release candidate | Test downloaded archives, not just local build outputs. |

### Tier 0: Format, Lint, Compile

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo check --all-targets --all-features`
- `cargo deny` or equivalent supply-chain/license audit if adopted.

### Tier 1: Unit Tests

Cover small deterministic pieces:

- methylation report parsing;
- sample file parsing;
- context parsing, including `CG`, `CHG`, `CHH`, and `CXX`;
- depth filtering;
- FASTA cytosine context counting;
- BED and one-region parsing;
- coordinate conversion;
- strand-aware binning for `MethOverRegion`;
- bin midpoint calculation for `MethLevDist`;
- missing-value handling for heatmaps.

These tests should not touch large files or require external binaries.

### Tier 2: Library API Integration Tests

Test the public `viewbs` crate directly:

- construct typed command args;
- run commands without invoking `ViewBS`;
- capture `CommandOutput`;
- verify returned table and plot paths;
- verify progress callbacks are emitted for long commands;
- verify cancellation returns `ViewBsError::Cancelled`;
- verify errors are structured and user-displayable.

This tier protects the desktop tool integration.

### Tier 3: CLI Integration Tests

Use `assert_cmd` and `tempfile`:

- `ViewBS --help`;
- `ViewBS --version`;
- one minimal successful command per subcommand;
- missing required argument errors;
- bad input file errors;
- output directory creation;
- legacy subcommand capitalization.

### Tier 4: Golden Parity Tests

Golden tests compare Rust output to legacy ViewBS output.

For each command:

- store a tiny input fixture;
- store the exact legacy command line;
- store expected `.tab` or `.txt` outputs;
- compare column order, row order, and numeric values;
- apply numeric tolerance only where legacy output is not fixed-width formatted.

Golden parity gates command rewrites. Plot appearance does not need pixel-perfect legacy parity.

### Tier 5: Plot Smoke Tests

For plot outputs:

- file exists;
- valid SVG/PDF/PNG;
- non-empty;
- expected dimensions;
- expected axis labels and legend text;
- no `NaN`, `inf`, or placeholder text in SVG;
- generated without R or any external plotting command.

Use small fixture data first. Add larger plot fixtures only for heatmap scaling behavior.

### Tier 6: Cross-Platform Tests

Run a small test suite on:

- Linux x86_64;
- macOS x86_64 or ARM64;
- Windows x86_64.

At minimum:

- `ViewBS --version`;
- `ViewBS --help`;
- one streaming command;
- one indexed region command;
- one plot-generating command.

This tier is important because standalone executables are a primary distribution goal.

### Tier 7: Performance and Memory Benchmarks

Use `criterion` for focused benchmarks and separate end-to-end timing scripts for realistic data.

Track:

- whole-file scan throughput;
- Tabix/CSI query throughput;
- `MethHeatmap` memory use;
- `MethGeno` window query runtime;
- plot generation time for large heatmaps;
- output size and compression where relevant.

Benchmarks should detect regressions, not block every small PR.

### Tier 8: Release Artifact Tests

Before publishing:

- download the generated release archive;
- unpack it into a clean temp directory;
- run `ViewBS --version`;
- run `ViewBS --help`;
- run the small fixture command set;
- verify no required Perl, R, htslib, or local build-path dependency;
- verify checksums;
- verify Windows `.exe` launches from a normal shell;
- verify macOS artifact behavior after signing/notarization if used.

### Snapshot Tests

Use `insta` for:

- help text;
- structured error messages;
- small JSON config examples;
- optional plot metadata manifests.

Floating-point policy:

- Use exact string comparison only where legacy output is formatted with fixed decimals.
- Use tolerance comparison for unrounded tables.
- Document tolerances in tests.

## Packaging

Cargo:

- Build with stable Rust.
- Publish a library crate named `viewbs`.
- Publish a binary named `ViewBS` from the same package.
- Produce a single static-ish binary where practical.
- Publish source and release binaries.

crates.io:

- Publish the reusable library API for desktop tools.
- Include complete rustdoc examples for each command.
- Use semantic versioning for public API changes.
- Keep unstable or experimental internals private or behind feature flags.

Bioconda:

- Replace Perl/R runtime dependencies with the Rust binary package.
- Keep `htslib` only if needed for external tools, not for ViewBS itself.

Docker:

- Use a small runtime image.
- No R or Perl needed for the Rust implementation.

## Standalone Executable Distribution

Primary distribution should be prebuilt release archives that users can download, unpack, and run directly.

Recommended initial targets:

| OS | Target | Artifact |
| --- | --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-musl` or `x86_64-unknown-linux-gnu` | `ViewBS-linux-x86_64.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-musl` or `aarch64-unknown-linux-gnu` | `ViewBS-linux-aarch64.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `ViewBS-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `ViewBS-macos-aarch64.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `ViewBS-windows-x86_64.zip` |

Recommended release contents:

```text
ViewBS
README.md
LICENSE
CHANGELOG.md
examples/
```

For Windows, the binary will be `ViewBS.exe`.

### Build Approach

Use GitHub Actions or another CI system with an OS matrix:

- Linux runner for Linux builds.
- macOS runner for macOS builds, especially if code signing or notarization is added.
- Windows runner for Windows MSVC builds.

Cross-compilation can help, but the safest release process is to build and test each major OS on that OS. Use cross-compilation mostly for additional Linux targets or automation convenience.

Useful tools:

- `cross` for container-based Linux cross-compilation and cross-testing.
- `cargo-zigbuild` for simpler cross-linking where it fits.
- Plain `cargo build --release --target <target>` on native CI runners for the main release artifacts.

### Linux Linking Choice

Prefer `x86_64-unknown-linux-musl` for a portable standalone Linux binary if all selected crates build cleanly with musl.

If musl causes problems with plotting, compression, or file-format crates:

- ship `x86_64-unknown-linux-gnu`;
- document the minimum supported glibc version;
- keep Conda and Docker packages as fallback distribution channels.

### macOS Release Notes

Build both Intel and Apple Silicon binaries. A universal macOS binary is optional; separate archives are simpler and easier to debug.

If distributing outside developer workflows:

- sign binaries with an Apple Developer ID when available;
- notarize release artifacts if users will download them through a browser;
- keep unsigned development builds clearly labeled.

### Windows Release Notes

Use the MSVC target for the main Windows release. Package as `.zip` with `ViewBS.exe`.

If Windows security warnings become a user-support issue:

- sign releases with an Authenticode certificate;
- include checksums for all release artifacts.

### Release Artifact Quality Checks

For every target:

- [ ] `ViewBS --version` works.
- [ ] `ViewBS --help` works.
- [ ] A small `GlobalMethLev` example runs.
- [ ] A small indexed-region command runs if fixture indexes are available.
- [ ] Plot generation works without external R or plotting tools.
- [ ] Binary has no unexpected dynamic dependency on Perl, R, htslib, or local build paths.
- [ ] SHA256 checksum is published.

### Desktop App Bundling

The desktop tool has two viable integration modes:

- Link to the `viewbs` Rust library directly.
- Bundle the `ViewBS` executable and run it as a child process.

Prefer direct library integration when the desktop app is also Rust-based. Use bundled executable mode when process isolation, plugin-style updates, or non-Rust desktop technology makes that simpler.

## Release Plan

Suggested versioning:

- `0.2.0-alpha`: Rust library skeleton, CLI wrapper, and one command.
- `0.2.0-beta`: all tables match legacy for test data.
- `0.2.0-rc1`: plots implemented and package builds complete.
- `0.2.0`: Rust release if output parity is acceptable.
- `1.0.0`: after broader user testing and documentation cleanup.

Migration notes:

- Explain that command names and inputs are preserved.
- Document removed `.rds` output and replacement plot artifacts. Completed in README migration notes.
- Document pure Rust dependency changes.
- Replace legacy figure regeneration script wording in README examples. Completed in README command-output notes.
- Present release archives as the primary installation path, with Conda and Docker as fallback channels. Completed in README installation notes.
- Include a `legacy differences` section for any intentional behavior changes. Completed in README migration notes.
- Document the release-candidate validation checklist. Completed in `docs/release.md`; external validation gates remain open until current evidence is collected.
- Replace stale Bioconda upload instructions. Completed in `conda/conda_upload.md` and `conda/conda_upload.sh`; actual package publication still requires release-candidate validation.
- Remove stale Perl-era CI and backup Docker docs. Completed by dropping `.travis.yml` and `ViewBSdocker/README_bak.md`.
- Move the legacy Perl launcher out of the root `ViewBS` path. Completed by moving it to `legacy/ViewBS.pl`.
- Replace stale root installation files. Completed by rewriting `INSTALL`, replacing `environment.yaml`, and removing `INSTALL.pl`.

## `kuva` Plot Library Spike Checklist

Before committing fully to `kuva`, implement a two-day spike:

- [ ] grouped bar plot for `GlobalMethLev`;
- [ ] line panels for `MethCoverage`;
- [ ] chromosome panels for `MethGeno`;
- [ ] heatmap for `MethHeatmap`;
- [ ] violin plus box plot for `MethHeatmap`;
- [ ] SVG output;
- [ ] PDF output through `kuva::render_to_pdf`;
- [ ] PNG output through `kuva::render_to_png`;
- [ ] subplot layout for replacing `mer_fig.R`.
- [ ] embedded-font SVG output if users need portable SVG files.

Acceptance criteria:

- Output is publication-usable.
- All required labels and legends are possible.
- Plots scale to realistic sample counts.
- Figure size options in centimeters can be mapped to inches/pixels.
- No non-Rust runtime dependency is introduced.
- `ViewBS` does not shell out to the `kuva` CLI; it uses the `kuva` library.

If the spike fails for only one figure type, use `plotters` only for that missing figure. If the spike fails broadly, keep `kuva` documented as the preferred candidate but reassess the plotting backend before implementation.

## Initial Development Milestones

1. `cargo new viewbs-rs --lib`
2. Add `src/main.rs` as a thin CLI wrapper around `viewbs`.
3. Define public argument, output, progress, cancellation, and error types.
4. Implement `ViewBS --help`, `ViewBS --version`, and all subcommand parsers.
5. Implement methylation report streaming reader.
6. Port `GlobalMethLev` as a library function first, then expose it through the CLI.
7. Add golden test comparing legacy `GlobalMethLev`.
8. Add first Rust plot for `GlobalMethLev`.
9. Add direct library integration tests for `GlobalMethLev`.
10. Port `BisNonConvRate`, `MethCoverage`, and genome-wide `MethLevDist`.
11. Implement Tabix/CSI indexed reader.
12. Port `MethOneRegion`.
13. Port `MethHeatmap`, `MethOverRegion`, and `MethGeno`.
14. Port converter scripts.
15. Package, publish library docs, and benchmark.

## References

Rust crate documentation checked for this plan:

- `clap`: https://docs.rs/clap/latest/clap/_derive/
- `serde`: https://docs.rs/serde/latest/serde/
- `serde_json`: https://docs.rs/serde_json/latest/serde_json/
- `csv`: https://docs.rs/csv/latest/csv/
- `flate2`: https://docs.rs/flate2/latest/flate2/
- `rayon`: https://docs.rs/rayon/latest/rayon/
- `noodles`: https://docs.rs/noodles/latest/noodles/
- `noodles-tabix`: https://docs.rs/noodles-tabix/latest/noodles_tabix/
- `noodles-fasta`: https://docs.rs/noodles/latest/noodles/fasta/
- `noodles-bed`: https://docs.rs/noodles-bed/latest/noodles_bed/
- `noodles-gff`: https://docs.rs/noodles-gff/latest/noodles_gff/
- `ndarray`: https://docs.rs/ndarray/latest/ndarray/
- `kodama`: https://docs.rs/kodama/latest/kodama/
- `kuva`: https://docs.rs/kuva/latest/kuva/
- `kuva` repository: https://github.com/Psy-Fer/kuva
- `kuva` documentation: https://psy-fer.github.io/kuva/
- `plotters`: https://docs.rs/plotters/latest/plotters/
- `plotters-svg`: https://docs.rs/plotters-svg/latest/plotters_svg/
- `svg2pdf`: https://docs.rs/svg2pdf/latest/svg2pdf/
- `colorous`: https://docs.rs/colorous/latest/colorous/
- `assert_cmd`: https://docs.rs/assert_cmd/latest/assert_cmd/
- `insta`: https://docs.rs/insta/latest/insta/
- `criterion`: https://docs.rs/criterion/latest/criterion/
- Rust platform support: https://doc.rust-lang.org/rustc/platform-support.html
- `cross`: https://github.com/cross-rs/cross
- `cargo-zigbuild`: https://github.com/rust-cross/cargo-zigbuild

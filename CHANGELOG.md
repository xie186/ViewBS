# Changelog

All notable ViewBS changes are tracked here.

## 0.2.0-alpha.0

- Started the Rust rewrite as a library crate named `viewbs` and a command-line binary named `ViewBS`.
- Added Rust implementations for the legacy analysis command surface with table parity tests against small legacy fixtures.
- Added `kuva`-based plot generation for SVG, PDF, and PNG outputs without R or external plotting tools.
- Added Tabix `.tbi` and CSI `.csi` indexed methylation query support through pure Rust crates.
- Added typed library APIs, progress reporting, cancellation, and serde-enabled configuration structs for desktop tool integration.
- Added Rust converter and figure-composition commands, plus compatibility aliases for the legacy helper script names.
- Added CI, release archive, packaging, golden parity, plot smoke, snapshot, and performance benchmark coverage.

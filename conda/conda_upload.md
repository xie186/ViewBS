# ViewBS Bioconda Release Upload

Use this checklist when validating or uploading the Rust `viewbs` Conda package. The recipe lives in `conda/meta.yaml` and builds the Rust binary with Cargo.

## Prerequisites

Install the Conda packaging tools in a packaging environment:

```bash
conda install -c conda-forge conda-build anaconda-client
```

Confirm the Rust package metadata is ready before building:

```bash
cargo package --allow-dirty --no-verify --list
```

## Build The Package

From the repository root:

```bash
conda build conda --output-folder target/conda-bld
```

The recipe should compile `ViewBS` from the current checkout with:

```bash
cargo install --locked --path . --root "${PREFIX}" --features cli,plots,serde
```

## Validate The Package

Install the built package into a clean environment and run the package smoke tests:

```bash
conda create -y -n viewbs-conda-test target/conda-bld/*/viewbs-*.tar.bz2
conda run -n viewbs-conda-test ViewBS --version
conda run -n viewbs-conda-test ViewBS --help
conda run -n viewbs-conda-test ViewBS GlobalMethLev --help
conda run -n viewbs-conda-test ViewBS GlobalMethLev --sample data/test_data/test_WT.tab.gz,WT --outdir target/conda-smoke --prefix conda_global --minDepth 1 --maxDepth 100 --plot-format svg
conda env remove -y -n viewbs-conda-test
```

The helper script performs the same table-and-SVG smoke command and checks `conda_global.tab` and `conda_global.svg`. Do not upload until the package has been validated against the current release candidate.

## Upload

Set the upload token only in the shell or CI secret store:

```bash
export ANACONDA_API_TOKEN=...
export VIEWBS_ANACONDA_USER=xie186
```

Use the helper script for the full build, smoke-test, and optional upload flow:

```bash
VIEWBS_CONDA_UPLOAD=1 ./conda/conda_upload.sh
```

Optional settings:

- `VIEWBS_CONDA_BLD_PATH`: output directory for built packages, default `target/conda-bld`.
- `VIEWBS_CONDA_LABEL`: optional Anaconda label such as `rc`.
- `VIEWBS_CONDA_UPLOAD`: set to `1` to upload after validation; otherwise the script only builds and smoke-tests.

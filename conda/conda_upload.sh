#!/usr/bin/env bash
set -euo pipefail

recipe_dir="${1:-conda}"
build_dir="${VIEWBS_CONDA_BLD_PATH:-target/conda-bld}"
env_name="${VIEWBS_CONDA_TEST_ENV:-viewbs-conda-smoke-$$}"
upload="${VIEWBS_CONDA_UPLOAD:-0}"
anaconda_user="${VIEWBS_ANACONDA_USER:-xie186}"
label="${VIEWBS_CONDA_LABEL:-}"

cleanup() {
  conda env remove -y -n "$env_name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

mkdir -p "$build_dir"
conda build "$recipe_dir" --output-folder "$build_dir"

package_path="$(
  find "$build_dir" -type f \( -name 'viewbs-*.conda' -o -name 'viewbs-*.tar.bz2' \) \
    | sort \
    | tail -n 1
)"

if [[ -z "$package_path" ]]; then
  echo "No built viewbs Conda package found under $build_dir" >&2
  exit 1
fi

conda create -y -n "$env_name" "$package_path"
conda run -n "$env_name" ViewBS --version
conda run -n "$env_name" ViewBS --help >/dev/null
conda run -n "$env_name" ViewBS GlobalMethLev --help >/dev/null

smoke_dir="${VIEWBS_CONDA_SMOKE_DIR:-target/conda-smoke-$env_name}"
rm -rf "$smoke_dir"
mkdir -p "$smoke_dir"
conda run -n "$env_name" ViewBS GlobalMethLev \
  --sample data/test_data/test_WT.tab.gz,WT \
  --outdir "$smoke_dir" \
  --prefix conda_global \
  --minDepth 1 \
  --maxDepth 100 \
  --plot-format svg >/dev/null
test -s "$smoke_dir/conda_global.tab"
test -s "$smoke_dir/conda_global.svg"

if [[ "$upload" == "1" ]]; then
  if [[ -z "${ANACONDA_API_TOKEN:-}" ]]; then
    echo "ANACONDA_API_TOKEN is required when VIEWBS_CONDA_UPLOAD=1" >&2
    exit 2
  fi

  upload_args=(upload "$package_path" --user "$anaconda_user")
  if [[ -n "$label" ]]; then
    upload_args+=(-l "$label")
  fi

  anaconda -t "$ANACONDA_API_TOKEN" "${upload_args[@]}"
else
  echo "Built and smoke-tested $package_path"
  echo "Set VIEWBS_CONDA_UPLOAD=1 and ANACONDA_API_TOKEN to upload."
fi

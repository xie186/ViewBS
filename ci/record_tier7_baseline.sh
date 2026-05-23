#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${VIEWBS_BENCH_DATA_DIR:-}" ]]; then
  echo "VIEWBS_BENCH_DATA_DIR must point at external benchmark data" >&2
  exit 2
fi

bench_root="$VIEWBS_BENCH_DATA_DIR"
required_files=(
  "$bench_root/sample.tab.gz"
  "$bench_root/sample.tab.gz.tbi"
  "$bench_root/regions.bed"
)

for path in "${required_files[@]}"; do
  if [[ ! -s "$path" ]]; then
    echo "Missing required benchmark file: $path" >&2
    exit 2
  fi
done

cargo_bin="${CARGO:-cargo}"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
commit="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
outdir="${VIEWBS_TIER7_BASELINE_DIR:-target/tier7-baselines/$commit-$timestamp}"
mkdir -p "$outdir"

baseline="$outdir/baseline.md"

{
  echo "# ViewBS Tier 7 Baseline"
  echo
  echo "- Commit: $commit"
  echo "- Timestamp UTC: $timestamp"
  echo "- Benchmark data: $bench_root"
  echo "- Cargo: $("$cargo_bin" --version)"
  echo "- Rustc: $(rustc --version)"
  echo "- Host: $(uname -a)"
  echo
  echo "## Benchmark Data Manifest"
  echo
  echo "| File | Bytes | SHA256 |"
  echo "| --- | ---: | --- |"
  for path in "${required_files[@]}"; do
    size="$(wc -c <"$path" | tr -d ' ')"
    hash="$(sha256sum "$path" | awk '{print $1}')"
    echo "| $(basename "$path") | $size | $hash |"
  done
  echo
  echo "## Command"
  echo
  echo '```bash'
  echo "VIEWBS_BENCH_DATA_DIR=$bench_root $cargo_bin bench --bench performance --all-features"
  echo '```'
  echo
  echo "## Outputs"
  echo
  echo "- Criterion report: \`$outdir/criterion\`"
  echo "- OS memory/time report: \`$outdir/time.txt\`"
} >"$baseline"

rm -rf "$outdir/criterion"

if [[ -x /usr/bin/time ]]; then
  VIEWBS_BENCH_DATA_DIR="$bench_root" \
    /usr/bin/time -v -o "$outdir/time.txt" \
    "$cargo_bin" bench --bench performance --all-features
else
  {
    echo "/usr/bin/time -v is unavailable on this host."
    echo "Run this script on Linux for release-candidate RSS evidence."
  } >"$outdir/time.txt"
  VIEWBS_BENCH_DATA_DIR="$bench_root" "$cargo_bin" bench --bench performance --all-features
fi

cp -R target/criterion "$outdir/criterion"

echo "Wrote Tier 7 release-candidate baseline to $outdir"

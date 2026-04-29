#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0

mkdir -p "$tmpdir/w30-source-diff"
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role baseline \
  --out "$tmpdir/w30-source-diff/baseline.wav" \
  --duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$tmpdir/w30-source-diff/candidate.wav" \
  --duration-seconds 0.5 \
  --source "$tmpdir/source.wav" \
  --source-duration-seconds 0.25
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --baseline "$tmpdir/w30-source-diff/baseline.metrics.md" \
  --candidate "$tmpdir/w30-source-diff/candidate.metrics.md" \
  --report "$tmpdir/w30-source-diff/comparison.md" \
  --min-rms-delta 0.001 \
  --min-sum-delta 1.0 \
  --max-active-samples-delta 200000 \
  --max-peak-delta 1.0 \
  --max-rms-delta 1.0 \
  --max-sum-delta 1000.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/w30-source-diff/manifest.json"

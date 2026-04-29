#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0

mkdir -p "$tmpdir/w30-preview"
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role baseline \
  --out "$tmpdir/w30-preview/baseline.wav" \
  --duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$tmpdir/w30-preview/candidate.wav" \
  --duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --baseline "$tmpdir/w30-preview/baseline.metrics.md" \
  --candidate "$tmpdir/w30-preview/candidate.metrics.md" \
  --report "$tmpdir/w30-preview/comparison.md"
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/w30-preview/manifest.json"

cargo run -p riotbox-audio --bin lane_recipe_pack -- \
  --output-dir "$tmpdir/lane-recipe" \
  --duration-seconds 2.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/lane-recipe/manifest.json"

cargo run -p riotbox-audio --bin feral_before_after_pack -- \
  --source "$tmpdir/source.wav" \
  --output-dir "$tmpdir/feral-before-after" \
  --duration-seconds 1.0 \
  --source-window-seconds 0.5
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/feral-before-after/manifest.json"

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$tmpdir/source.wav" \
  --output-dir "$tmpdir/feral-grid" \
  --bars 2 \
  --source-window-seconds 0.5
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/feral-grid/manifest.json"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0
cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$tmpdir/source.wav" \
  --output-dir "$tmpdir/feral-grid" \
  --bars 2 \
  --source-window-seconds 0.5
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/feral-grid/manifest.json"

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson \
  --manifest "$tmpdir/feral-grid/manifest.json" \
  --require-evidence

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson \
  --manifest "$tmpdir/feral-grid/manifest.json" \
  --output "$tmpdir/observer-audio-summary.json" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .schema_version == 1
    and .control_path.present == true
    and .output_path.present == true
    and (.output_path.issues | length == 0)' \
  "$tmpdir/observer-audio-summary.json"
python3 scripts/validate_observer_audio_summary_json.py \
  "$tmpdir/observer-audio-summary.json"

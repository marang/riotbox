#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

bars="${RIOTBOX_FULL_GRID_EXPORT_BARS:-4}"
if ! [[ "$bars" =~ ^[0-9]+$ ]] || (( bars < 2 )); then
  echo "RIOTBOX_FULL_GRID_EXPORT_BARS must be an integer >= 2" >&2
  exit 1
fi

source_seconds="${RIOTBOX_FULL_GRID_EXPORT_SOURCE_SECONDS:-8.0}"
source_window_seconds="${RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS:-1.0}"
python3 - "$source_seconds" "$source_window_seconds" <<'PY'
import math
import sys

try:
    source_seconds = float(sys.argv[1])
    source_window_seconds = float(sys.argv[2])
except ValueError:
    raise SystemExit("full-grid export durations must be finite numbers")
if not math.isfinite(source_seconds) or source_seconds <= 0.0:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_SECONDS must be a positive finite number")
if not math.isfinite(source_window_seconds) or source_window_seconds <= 0.0:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS must be a positive finite number")
if source_window_seconds > source_seconds:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS must not exceed source seconds")
PY

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_a="$tmpdir/source-a.wav"
source_b="$tmpdir/source-b.wav"
run_a="$tmpdir/run-a"
run_b="$tmpdir/run-b"
proof="$tmpdir/product-export-proof.json"

python3 scripts/write_synthetic_break_wav.py "$source_a" "$source_seconds"
python3 scripts/write_synthetic_break_wav.py "$source_b" "$source_seconds"

source_hash_a="$(sha256sum "$source_a" | awk '{print $1}')"
source_hash_b="$(sha256sum "$source_b" | awk '{print $1}')"
if [[ "$source_hash_a" != "$source_hash_b" ]]; then
  echo "deterministic full-grid source generation drifted: $source_hash_a != $source_hash_b" >&2
  exit 1
fi

render_grid_export() {
  local source_wav="$1"
  local run_dir="$2"
  cargo run -p riotbox-audio --bin feral_grid_pack -- \
    --source "$source_wav" \
    --output-dir "$run_dir" \
    --bars "$bars" \
    --source-window-seconds "$source_window_seconds"
  python3 scripts/validate_listening_manifest_json.py \
    --require-existing-artifacts \
    "$run_dir/manifest.json"
  jq -e \
    '.pack_id == "feral-grid-demo"
      and .result == "pass"
      and .feral_scorecard.readiness == "ready"
      and .feral_scorecard.source_backed == true
      and .feral_scorecard.fallback_like == false
      and (.feral_scorecard.lane_gestures | index("mc202 question/answer")) == null
      and .metrics.full_grid_mix.signal.rms > 0.000001
      and .metrics.full_grid_mix.low_band.rms > 0.000001
      and .metrics.tr909_beat_fill.signal.rms > 0.000001
      and .metrics.mc202_bass_pressure.applied == true
      and .metrics.mc202_bass_pressure_stem.signal.rms > 0.000001
      and .metrics.w30_feral_source_chop.signal.rms > 0.000001
      and (.metrics | has("mc202_question_answer_delta") | not)' \
    "$run_dir/manifest.json"
}

render_grid_export "$source_a" "$run_a"
render_grid_export "$source_b" "$run_b"

python3 scripts/validate_product_export_reproducibility.py \
  --write-proof "$proof" \
  "$run_a/manifest.json" \
  "$run_b/manifest.json"

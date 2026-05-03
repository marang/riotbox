#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

repetitions="${RIOTBOX_STAGE_STYLE_STABILITY_REPETITIONS:-2}"
if ! [[ "$repetitions" =~ ^[0-9]+$ ]] || (( repetitions < 2 )); then
  echo "RIOTBOX_STAGE_STYLE_STABILITY_REPETITIONS must be an integer >= 2" >&2
  exit 1
fi

bars="${RIOTBOX_STAGE_STYLE_STABILITY_BARS:-4}"
if ! [[ "$bars" =~ ^[0-9]+$ ]] || (( bars < 2 )); then
  echo "RIOTBOX_STAGE_STYLE_STABILITY_BARS must be an integer >= 2" >&2
  exit 1
fi

source_seconds="${RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_SECONDS:-8.0}"
source_window_seconds="${RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_WINDOW_SECONDS:-1.0}"
python3 - "$source_seconds" "$source_window_seconds" <<'PY'
import math
import sys

try:
    source_seconds = float(sys.argv[1])
    source_window_seconds = float(sys.argv[2])
except ValueError as exc:
    raise SystemExit("stage-style stability durations must be finite numbers")
if not math.isfinite(source_seconds) or source_seconds <= 0.0:
    raise SystemExit("RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_SECONDS must be a positive finite number")
if not math.isfinite(source_window_seconds) or source_window_seconds <= 0.0:
    raise SystemExit("RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_WINDOW_SECONDS must be a positive finite number")
if source_window_seconds > source_seconds:
    raise SystemExit("RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_WINDOW_SECONDS must not exceed source seconds")
PY

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

expected_mix_hash=""

echo "stage-style stability config: repetitions=$repetitions bars=$bars source_seconds=$source_seconds source_window_seconds=$source_window_seconds"

for run in $(seq 1 "$repetitions"); do
  run_dir="$tmpdir/stage-style-stability-$run"
  mkdir -p "$run_dir"
  source_wav="$run_dir/source.wav"
  observer_fixture="$run_dir/events.ndjson"
  pack_dir="$run_dir/feral-grid"
  summary="$run_dir/observer-audio-summary.json"

  python3 scripts/write_synthetic_break_wav.py "$source_wav" "$source_seconds"
  cargo run -p riotbox-app --bin user_session_observer_probe -- \
    --probe stage-style-restore-diversity \
    --observer "$observer_fixture"
  python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
  jq -s -e \
    'length >= 24
      and .[0].event == "observer_started"
      and .[0].launch.probe == "stage-style-restore-diversity"
      and all(.[]; has("snapshot"))
      and all(.[]; .snapshot.transport | type == "object")
      and all(.[]; .snapshot.queue | type == "object")
      and all(.[]; .snapshot.runtime | type == "object")
      and all(.[]; .snapshot.recovery | type == "object")
      and any(.[]; .event == "key_outcome" and .key == "c" and .outcome == "queue_capture_bar")
      and any(.[]; .event == "key_outcome" and .key == "w" and .outcome == "queue_w30_trigger_pad")
      and any(.[]; .event == "key_outcome" and .key == "f" and .outcome == "queue_tr909_fill")
      and any(.[]; .event == "key_outcome" and .key == "g" and .outcome == "queue_mc202_generate_follower")
      and any(.[]; .event == "transport_commit" and .snapshot.queue.session_log_count >= 12)' \
    "$observer_fixture"

  cargo run -p riotbox-audio --bin feral_grid_pack -- \
    --source "$source_wav" \
    --output-dir "$pack_dir" \
    --bars "$bars" \
    --source-window-seconds "$source_window_seconds"
  python3 scripts/validate_listening_manifest_json.py \
    --require-existing-artifacts \
    "$pack_dir/manifest.json"

  cargo run -p riotbox-app --bin observer_audio_correlate -- \
    --observer "$observer_fixture" \
    --manifest "$pack_dir/manifest.json" \
    --output "$summary" \
    --json \
    --require-evidence
  jq -e \
    '.schema == "riotbox.observer_audio_summary.v1"
      and .schema_version == 1
      and .control_path.present == true
      and .control_path.commit_count >= 12
      and (.control_path.commit_boundaries | index("Phrase")) != null
      and (.control_path.commit_boundaries | index("Bar")) != null
      and (.control_path.commit_boundaries | index("Beat")) != null
      and .output_path.present == true
      and (.output_path.issues | length == 0)
      and .output_path.metrics.full_mix_rms > 0.000001
      and .output_path.metrics.full_mix_low_band_rms > 0.000001' \
    "$summary"
  python3 scripts/validate_observer_audio_summary_json.py "$summary"

  mix_hash="$(sha256sum "$pack_dir/03_riotbox_grid_feral_mix.wav" | awk '{print $1}')"
  if [[ -z "$expected_mix_hash" ]]; then
    expected_mix_hash="$mix_hash"
  elif [[ "$mix_hash" != "$expected_mix_hash" ]]; then
    echo "stage-style stability smoke produced a non-deterministic mix hash on run $run" >&2
    echo "expected: $expected_mix_hash" >&2
    echo "actual:   $mix_hash" >&2
    exit 1
  fi

  echo "stage-style stability run $run/$repetitions ok: $mix_hash"
done

echo "stage-style stability smoke ok across $repetitions repeated runs: $expected_mix_hash"

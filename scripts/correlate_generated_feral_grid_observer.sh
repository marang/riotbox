#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="$tmpdir/feral-grid-observer/events.ndjson"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0
cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe feral-grid-jam \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 6
    and .[0].event == "observer_started"
    and .[0].launch.probe == "feral-grid-jam"
    and all(.[]; has("snapshot"))
    and all(.[]; .snapshot.transport | type == "object")
    and all(.[]; .snapshot.queue | type == "object")
    and all(.[]; .snapshot.runtime | type == "object")
    and all(.[]; .snapshot.recovery | type == "object")
    and all(.[]; .snapshot.source_timing.source_id == "src-feral-grid-probe")
    and all(.[]; .snapshot.source_timing.quality == "medium")
    and all(.[]; .snapshot.source_timing.degraded_policy == "cautious")
    and all(.[]; .snapshot.source_timing.primary_warning_code == "phrase_uncertain")
    and any(.[]; .event == "key_outcome" and .key == "f" and .outcome == "queue_tr909_fill")
    and any(.[]; .event == "key_outcome" and .key == "g" and .outcome == "queue_mc202_generate_follower")' \
  "$observer_fixture"

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$tmpdir/source.wav" \
  --output-dir "$tmpdir/feral-grid" \
  --bars 2 \
  --source-window-seconds 0.5
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/feral-grid/manifest.json"

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$observer_fixture" \
  --manifest "$tmpdir/feral-grid/manifest.json" \
  --require-evidence

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$observer_fixture" \
  --manifest "$tmpdir/feral-grid/manifest.json" \
  --output "$tmpdir/observer-audio-summary.json" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .schema_version == 1
    and .control_path.present == true
    and (.control_path.key_outcomes | index("f -> queue_tr909_fill")) != null
    and (.control_path.key_outcomes | index("g -> queue_mc202_generate_follower")) != null
    and .control_path.commit_count >= 2
    and (.control_path.commit_boundaries | index("Bar")) != null
    and (.control_path.commit_boundaries | index("Phrase")) != null
    and .control_path.observer_source_timing.source_id == "src-feral-grid-probe"
    and .control_path.observer_source_timing.quality == "medium"
    and .control_path.observer_source_timing.degraded_policy == "cautious"
    and .control_path.observer_source_timing.primary_warning_code == "phrase_uncertain"
    and .output_path.present == true
    and (.output_path.issues | length == 0)' \
  "$tmpdir/observer-audio-summary.json"
python3 scripts/validate_observer_audio_summary_json.py \
  "$tmpdir/observer-audio-summary.json"

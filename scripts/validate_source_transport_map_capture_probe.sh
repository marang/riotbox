#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

probe_dir="$tmpdir/source-transport-map-capture"
mkdir -p "$probe_dir"
observer_fixture="$probe_dir/events.ndjson"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 8.0
cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe source-transport-map-capture \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 12
    and .[0].event == "observer_started"
    and .[0].launch.probe == "source-transport-map-capture"
    and .[0].snapshot.runtime.source_monitor_mode == "source"
    and .[0].snapshot.runtime.source_monitor_audio_route == "source_only"
    and .[0].snapshot.source_map.capture_range_available == false
    and .[0].snapshot.source_map.trust_label == "needs confirm"
    and any(.[]; .event == "key_outcome"
      and .key == "C"
      and .outcome == "confirm_source_timing_grid"
      and .snapshot.source_timing.grid_confirmed == true
      and .snapshot.source_map.mode == "bar grid"
      and .snapshot.source_map.capture_range_available == true)
    and any(.[]; .event == "key_outcome"
      and .key == "Right"
      and .outcome == "navigate_source_map_next_bar"
      and (.status | startswith("source map moved to"))
      and .snapshot.transport.position_beats == 4)
    and any(.[]; .event == "transport_commit"
      and (.snapshot.queue.recent_history | map(.command) | index("capture.bar_group")) != null
      and .snapshot.capture.source_window_available == true
      and .snapshot.source_map.capture_range_available == true)
    and any(.[]; .event == "key_outcome" and .key == "o" and .outcome == "queue_w30_audition")
    and any(.[]; .event == "key_outcome" and .key == "p" and .outcome == "promote_last_capture")
    and any(.[]; .event == "key_outcome" and .key == "w" and .outcome == "queue_w30_trigger_pad")
    and any(.[]; .event == "transport_commit"
      and (.snapshot.queue.recent_history | map(.command) | index("w30.trigger_pad")) != null
      and .snapshot.runtime.w30_preview_mode == "live_recall"
      and (.snapshot.runtime.w30_preview_target | contains("cap-01")))' \
  "$observer_fixture"

cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role baseline \
  --out "$probe_dir/baseline.wav" \
  --duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$probe_dir/candidate.wav" \
  --duration-seconds 0.5 \
  --source "$tmpdir/source.wav" \
  --source-start-seconds 3.75 \
  --source-duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --baseline "$probe_dir/baseline.metrics.md" \
  --candidate "$probe_dir/candidate.metrics.md" \
  --report "$probe_dir/comparison.md" \
  --min-rms-delta 0.001 \
  --min-sum-delta 1.0 \
  --max-active-samples-delta 200000 \
  --max-peak-delta 1.0 \
  --max-rms-delta 1.0 \
  --max-sum-delta 1000.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$probe_dir/manifest.json"

summary="$probe_dir/observer-audio-summary.json"
cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$observer_fixture" \
  --manifest "$probe_dir/manifest.json" \
  --output "$summary" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .control_path.present == true
    and (.control_path.key_outcomes | index("space -> toggle_transport")) != null
    and (.control_path.key_outcomes | index("C -> confirm_source_timing_grid")) != null
    and (.control_path.key_outcomes | index("Right -> navigate_source_map_next_bar")) != null
    and (.control_path.key_outcomes | index("c -> queue_capture_bar")) != null
    and (.control_path.key_outcomes | index("o -> queue_w30_audition")) != null
    and (.control_path.key_outcomes | index("p -> promote_last_capture")) != null
    and (.control_path.key_outcomes | index("w -> queue_w30_trigger_pad")) != null
    and .control_path.commit_count >= 6
    and (.control_path.commit_boundaries | index("Immediate")) != null
    and (.control_path.commit_boundaries | index("Bar")) != null
    and (.control_path.commit_boundaries | index("Beat")) != null
    and .control_path.observer_source_timing.source_id == "src-source-transport-map-capture"
    and .control_path.observer_source_timing.degraded_policy == "manual_confirm"
    and .control_path.observer_source_timing.grid_use == "manual_confirm_only"
    and .output_path.present == true
    and (.output_path.issues | length == 0)
    and .output_path.metrics.w30_candidate_rms > 0.000001
    and .output_path.metrics.w30_rms_delta > 0.000001' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

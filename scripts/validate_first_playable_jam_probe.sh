#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

probe_dir="$tmpdir/first-playable-jam"
mkdir -p "$probe_dir"
observer_fixture="$probe_dir/events.ndjson"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0
cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe first-playable-jam \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 10
    and .[0].event == "observer_started"
    and .[0].launch.probe == "first-playable-jam"
    and all(.[]; has("snapshot"))
    and all(.[]; .snapshot.transport | type == "object")
    and all(.[]; .snapshot.queue | type == "object")
    and all(.[]; .snapshot.runtime | type == "object")
    and all(.[]; .snapshot.recovery | type == "object")
    and any(.[]; .event == "key_outcome" and .key == "c" and .snapshot.queue.pending_count >= 1)
    and any(.[]; .event == "transport_commit" and .snapshot.queue.session_log_count >= 1)
    and any(.[]; .event == "key_outcome" and .key == "w" and .outcome == "queue_w30_trigger_pad")' \
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
  --source-duration-seconds 0.25
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
    and (.control_path.key_outcomes | index("c -> queue_capture_bar")) != null
    and (.control_path.key_outcomes | index("o -> queue_w30_audition")) != null
    and (.control_path.key_outcomes | index("p -> promote_last_capture")) != null
    and (.control_path.key_outcomes | index("w -> queue_w30_trigger_pad")) != null
    and .control_path.commit_count >= 4
    and (.control_path.commit_boundaries | index("Phrase")) != null
    and (.control_path.commit_boundaries | index("Bar")) != null
    and (.control_path.commit_boundaries | index("Beat")) != null
    and .output_path.present == true
    and (.output_path.issues | length == 0)
    and .output_path.metrics.w30_candidate_rms > 0.000001
    and .output_path.metrics.w30_candidate_active_sample_ratio > 0.000001
    and .output_path.metrics.w30_rms_delta > 0.000001' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

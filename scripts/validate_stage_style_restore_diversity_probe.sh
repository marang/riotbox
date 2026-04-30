#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

probe_dir="$tmpdir/stage-style-restore-diversity"
mkdir -p "$probe_dir"
observer_fixture="$probe_dir/events.ndjson"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 8.0
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
    and any(.[]; .event == "key_outcome" and .key == "o" and .outcome == "queue_w30_audition")
    and any(.[]; .event == "key_outcome" and .key == "p" and .outcome == "promote_last_capture")
    and any(.[]; .event == "key_outcome" and .key == "w" and .outcome == "queue_w30_trigger_pad")
    and any(.[]; .event == "key_outcome" and .key == "f" and .outcome == "queue_tr909_fill")
    and any(.[]; .event == "key_outcome" and .key == "d" and .outcome == "queue_tr909_reinforce")
    and any(.[]; .event == "key_outcome" and .key == "k" and .outcome == "queue_tr909_scene_lock")
    and any(.[]; .event == "key_outcome" and .key == "x" and .outcome == "queue_tr909_release")
    and any(.[]; .event == "key_outcome" and .key == "g" and .outcome == "queue_mc202_generate_follower")
    and any(.[]; .event == "key_outcome" and .key == "a" and .outcome == "queue_mc202_generate_answer")
    and any(.[]; .event == "key_outcome" and .key == "P" and .outcome == "queue_mc202_generate_pressure")
    and any(.[]; .event == "key_outcome" and .key == "I" and .outcome == "queue_mc202_generate_instigator")
    and any(.[]; .event == "key_outcome" and .key == "G" and .outcome == "queue_mc202_mutate_phrase")
    and any(.[]; .event == "transport_commit" and .snapshot.queue.session_log_count >= 12)' \
  "$observer_fixture"

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$tmpdir/source.wav" \
  --output-dir "$probe_dir/feral-grid" \
  --bars 4 \
  --source-window-seconds 1.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$probe_dir/feral-grid/manifest.json"

summary="$probe_dir/observer-audio-summary.json"
cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$observer_fixture" \
  --manifest "$probe_dir/feral-grid/manifest.json" \
  --output "$summary" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .schema_version == 1
    and .control_path.present == true
    and (.control_path.key_outcomes | index("space -> toggle_transport")) != null
    and (.control_path.key_outcomes | index("c -> queue_capture_bar")) != null
    and (.control_path.key_outcomes | index("o -> queue_w30_audition")) != null
    and (.control_path.key_outcomes | index("p -> promote_last_capture")) != null
    and (.control_path.key_outcomes | index("w -> queue_w30_trigger_pad")) != null
    and (.control_path.key_outcomes | index("f -> queue_tr909_fill")) != null
    and (.control_path.key_outcomes | index("d -> queue_tr909_reinforce")) != null
    and (.control_path.key_outcomes | index("k -> queue_tr909_scene_lock")) != null
    and (.control_path.key_outcomes | index("x -> queue_tr909_release")) != null
    and (.control_path.key_outcomes | index("g -> queue_mc202_generate_follower")) != null
    and (.control_path.key_outcomes | index("a -> queue_mc202_generate_answer")) != null
    and (.control_path.key_outcomes | index("P -> queue_mc202_generate_pressure")) != null
    and (.control_path.key_outcomes | index("I -> queue_mc202_generate_instigator")) != null
    and (.control_path.key_outcomes | index("G -> queue_mc202_mutate_phrase")) != null
    and .control_path.commit_count >= 12
    and (.control_path.commit_boundaries | index("Phrase")) != null
    and (.control_path.commit_boundaries | index("Bar")) != null
    and (.control_path.commit_boundaries | index("Beat")) != null
    and .output_path.present == true
    and (.output_path.issues | length == 0)
    and .output_path.metrics.full_mix_rms > 0.000001
    and .output_path.metrics.full_mix_low_band_rms > 0.000001
    and .output_path.metrics.mc202_question_answer_delta_rms > 0.000001' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

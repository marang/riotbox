#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="$tmpdir/recipe2-mc202/events.ndjson"
pack_dir="$tmpdir/lane-recipe"
summary="$tmpdir/observer-audio-summary.json"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe recipe2-mc202 \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 9
    and .[0].event == "observer_started"
    and .[0].opt_in == true
    and .[0].capture_context == "headless_probe"
    and .[0].raw_audio_recording == false
    and .[0].realtime_callback_io == false
    and .[0].launch.mode == "ingest"
    and .[0].launch.probe == "recipe2-mc202"
    and all(.[]; has("snapshot"))
    and all(.[]; .snapshot.transport | type == "object")
    and all(.[]; .snapshot.queue | type == "object")
    and all(.[]; .snapshot.runtime | type == "object")
    and all(.[]; .snapshot.recovery | type == "object")
    and any(.[]; .event == "audio_runtime" and .status == "started" and .snapshot.runtime.audio_status == "running")
    and any(.[]; .event == "key_outcome" and .key == "g" and .snapshot.queue.pending_count >= 1)
    and any(.[]; .event == "transport_commit" and .snapshot.queue.session_log_count >= 1)' \
  "$observer_fixture"

cargo run -p riotbox-audio --bin lane_recipe_pack -- \
  --output-dir "$pack_dir" \
  --duration-seconds 2.0
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
    and .control_path.present == true
    and (.control_path.key_outcomes | index("space -> toggle_transport")) != null
    and (.control_path.key_outcomes | index("g -> queue_mc202_generate_follower")) != null
    and (.control_path.key_outcomes | index("a -> queue_mc202_generate_answer")) != null
    and (.control_path.key_outcomes | index("P -> queue_mc202_generate_pressure")) != null
    and (.control_path.key_outcomes | index("I -> queue_mc202_generate_instigator")) != null
    and (.control_path.key_outcomes | index("G -> queue_mc202_mutate_phrase")) != null
    and (.control_path.key_outcomes | index("> -> raise_mc202_touch")) != null
    and .control_path.commit_count >= 5
    and (.control_path.commit_boundaries | index("Phrase")) != null
    and .output_path.present == true
    and (.output_path.issues | length == 0)' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

jq -e \
  '.pack_id == "lane-recipe-listening-pack"
    and .result == "pass"
    and ([.cases[].id] | index("mc202-follower-to-answer")) != null
    and ([.cases[].id] | index("mc202-touch-low-to-high")) != null
    and ([.cases[].id] | index("mc202-follower-to-pressure")) != null
    and ([.cases[].id] | index("mc202-follower-to-instigator")) != null
    and ([.cases[].id] | index("mc202-follower-to-mutated-drive")) != null
    and ([.cases[].id] | index("mc202-neutral-to-lift-contour")) != null
    and ([.cases[].id] | index("mc202-direct-to-hook-response")) != null
    and all(.cases[] | select(.id | startswith("mc202-")); .result == "pass")
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.candidate.rms > 0.000001)
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.signal_delta.rms >= .thresholds.min_signal_delta_rms)
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.mc202_phrase_grid.passed == true)
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.mc202_phrase_grid.starts_on_phrase_boundary == true)
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.mc202_phrase_grid.hit_ratio >= 0.95)
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.mc202_phrase_grid.candidate_onset_count > 0)' \
  "$pack_dir/manifest.json"

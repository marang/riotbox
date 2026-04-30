#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="crates/riotbox-app/tests/fixtures/recipe2_mc202_probe/events.ndjson"
pack_dir="$tmpdir/lane-recipe"
summary="$tmpdir/observer-audio-summary.json"

python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"

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
    and (.control_path.key_outcomes | index("space -> transport started")) != null
    and (.control_path.key_outcomes | index("g -> follower queued")) != null
    and (.control_path.key_outcomes | index("a -> answer queued")) != null
    and (.control_path.key_outcomes | index("P -> pressure queued")) != null
    and (.control_path.key_outcomes | index("I -> instigator queued")) != null
    and (.control_path.key_outcomes | index("G -> mutate queued")) != null
    and (.control_path.key_outcomes | index("> -> touch raised")) != null
    and .control_path.commit_count >= 5
    and (.control_path.commit_boundaries | index("NextBar")) != null
    and (.control_path.commit_boundaries | index("NextPhrase")) != null
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
    and all(.cases[] | select(.id | startswith("mc202-")); .metrics.signal_delta.rms >= .thresholds.min_signal_delta_rms)' \
  "$pack_dir/manifest.json"

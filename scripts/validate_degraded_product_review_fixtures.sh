#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

source_wav="$(realpath crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_beat08/03_riotbox_grid_feral_mix.wav)"
source_sha="$(sha256sum "$source_wav" | cut -d ' ' -f1)"
graph="$tmp/source-graph.json"
session="$tmp/session.json"
observer="$tmp/events.ndjson"

jq -n \
  --arg source "$source_wav" \
  --arg hash "sha256:$source_sha" \
  '{source:{source_id:"src-review-fixture",path:$source,content_hash:$hash}}' \
  >"$graph"

jq -n \
  --arg source "$source_wav" \
  --arg hash "sha256:$source_sha" \
  --arg graph "$graph" \
  '{
    source_refs:[{source_id:"src-review-fixture",path_hint:$source,content_hash:$hash}],
    source_graph_refs:[{source_id:"src-review-fixture",external_path:$graph}],
    runtime_state:{source_timing:{confirmed_grid:null}},
    action_log:{actions:[],commit_records:[]}
  }' \
  >"$session"

jq -c \
  --arg source "$source_wav" \
  '.launch.source = $source
   | .snapshot.runtime = {
       audio_status:"running",
       audio_callback_count:12,
       source_monitor_mode:"source",
       source_monitor_audio_route:"source_only",
       tr909_mode:"idle",
       mc202_mode:"idle",
       mc202_routing:"silent",
       w30_resample_tap_mode:"idle"
     }
   | .snapshot.transport.is_playing = false
   | .snapshot.queue.pending_count = 0
   | .snapshot.queue.session_log_count = 0
   | .snapshot.source_timing.source_id = "src-review-fixture"' \
  crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson \
  >"$observer"

python3 scripts/degraded_product_review.py pack \
  --ticket RIOTBOX-FIXTURE \
  --output "$tmp/review" \
  --source-family bad_timing \
  --outcome degraded \
  --source "$source_wav" \
  --source-graph "$graph" \
  --session "$session" \
  --observer "$observer" \
  --reason "Fixture calibration keeps ambiguous timing out of bar-locked product output." \
  --evidence-role fixture_calibration

python3 scripts/degraded_product_review.py validate "$tmp/review/review.json"
python3 scripts/degraded_product_review.py record \
  --review "$tmp/review/review.json" \
  --product-verdict pass \
  --risk-state-visible yes \
  --reason-useful yes \
  --next-action-understandable yes \
  --next-action "Confirm the grid before bar-locked moves." \
  --reviewer fixture-listener
python3 scripts/degraded_product_review.py validate "$tmp/review/review.json"

mkdir -p "$tmp/live-review"
jq '.evidence_role = "live_product_review" | .human_review.reviewer = "Markus"' \
  "$tmp/review/review.json" >"$tmp/live-review/review.json"
cp "$tmp/review/prompt.md" "$tmp/live-review/prompt.md"
python3 scripts/degraded_product_review.py validate \
  --require-human-pass "$tmp/live-review/review.json"

jq -n '{
  schema:"riotbox.release_grade_demo_bank.v1",
  schema_version:1,
  readiness_rubric_schema:"riotbox.sound_product_readiness_rubric.v1",
  evidence_role:"live_review",
  hidden_taste_oracle_allowed:false,
  entries:[]
}' >"$tmp/live-bank.json"
python3 scripts/degraded_product_review.py promote \
  --review "$tmp/live-review/review.json" \
  --bank "$tmp/live-bank.json" \
  --entry-id "bad-timing-human-degraded" \
  --fix-category ui_cue \
  --demo-worthiness-note "Reviewed safe degraded handling, not demo-ready music."
python3 scripts/validate_release_grade_demo_bank.py "$tmp/live-bank.json"

jq '.entries += [{entry_id:"expired-legacy-evidence"}]' \
  "$tmp/live-bank.json" >"$tmp/invalid-existing-bank.json"
invalid_bank_sha="$(sha256sum "$tmp/invalid-existing-bank.json" | cut -d ' ' -f1)"
if python3 scripts/degraded_product_review.py promote \
  --review "$tmp/live-review/review.json" \
  --bank "$tmp/invalid-existing-bank.json" \
  --entry-id "bad-timing-human-degraded" \
  --fix-category ui_cue \
  --demo-worthiness-note "An invalid existing bank must fail before overwrite." \
  >"$tmp/invalid-existing-bank.out" 2>&1; then
  echo "expected invalid existing live bank promotion to fail" >&2
  exit 1
fi
grep -q "source_family" "$tmp/invalid-existing-bank.out"
test "$(sha256sum "$tmp/invalid-existing-bank.json" | cut -d ' ' -f1)" = "$invalid_bank_sha"

if python3 scripts/degraded_product_review.py validate \
  --require-human-pass "$tmp/review/review.json" >"$tmp/fixture-live.out" 2>&1; then
  echo "expected fixture-calibration review to fail live human-pass validation" >&2
  exit 1
fi
grep -q "accepted human product pass" "$tmp/fixture-live.out"

jq '.product_path_proof.fallback_music_present = true' \
  "$tmp/review/review.json" >"$tmp/fallback.json"
if python3 scripts/degraded_product_review.py validate "$tmp/fallback.json" \
  >"$tmp/fallback.out" 2>&1; then
  echo "expected fallback-music mutation to fail" >&2
  exit 1
fi
grep -q "fallback music" "$tmp/fallback.out"

jq '.product_path_proof.audio_callback_count += 1' \
  "$tmp/review/review.json" >"$tmp/forged-proof.json"
if python3 scripts/degraded_product_review.py validate "$tmp/forged-proof.json" \
  >"$tmp/forged-proof.out" 2>&1; then
  echo "expected forged product-path proof to fail" >&2
  exit 1
fi
grep -q "does not match the bound runtime artifacts" "$tmp/forged-proof.out"

cp "$observer" "$tmp/events-safe.ndjson"
jq -c \
  '., (. | .event = "key_outcome" | .key = "fixture" | .outcome = "unsafe" | .snapshot.transport.is_playing = true)' \
  "$tmp/events-safe.ndjson" >"$tmp/transport-started.ndjson"
mv "$tmp/transport-started.ndjson" "$observer"
if python3 scripts/degraded_product_review.py pack \
  --ticket RIOTBOX-FIXTURE \
  --output "$tmp/transport-started-review" \
  --source-family bad_timing \
  --outcome degraded \
  --source "$source_wav" \
  --source-graph "$graph" \
  --session "$session" \
  --observer "$observer" \
  --reason "A transiently unsafe observer path must not pass review." \
  --evidence-role fixture_calibration >"$tmp/transport-started.out" 2>&1; then
  echo "expected earlier transport activity to fail" >&2
  exit 1
fi
grep -q "transport started during assigned review" "$tmp/transport-started.out"

jq -c \
  '., (. | .event = "key_outcome" | .key = "fixture" | .outcome = "source-switch" | .snapshot.source_timing.source_id = "src-other")' \
  "$tmp/events-safe.ndjson" >"$tmp/source-switch.ndjson"
mv "$tmp/source-switch.ndjson" "$observer"
if python3 scripts/degraded_product_review.py pack \
  --ticket RIOTBOX-FIXTURE \
  --output "$tmp/source-switch-review" \
  --source-family bad_timing \
  --outcome degraded \
  --source "$source_wav" \
  --source-graph "$graph" \
  --session "$session" \
  --observer "$observer" \
  --reason "Every assigned snapshot must retain the exact reviewed source identity." \
  --evidence-role fixture_calibration >"$tmp/source-switch.out" 2>&1; then
  echo "expected intermediate source switch to fail" >&2
  exit 1
fi
grep -q "source id changed during assigned review" "$tmp/source-switch.out"

printf '\n' >>"$observer"
if python3 scripts/degraded_product_review.py validate "$tmp/review/review.json" \
  >"$tmp/stale.out" 2>&1; then
  echo "expected stale observer identity to fail" >&2
  exit 1
fi
grep -q "observer hash is stale" "$tmp/stale.out"

echo "valid degraded product review fixtures"

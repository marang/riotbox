#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-tonal-hook-live-journey}"
source='data/test_audio/examples/DH_RushArp_120_A.wav'
expected_sha='ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e'

actual_sha="$(sha256sum -- "$source" | cut -d ' ' -f 1)"
test "$actual_sha" = "$expected_sha"

cargo run -p riotbox-app --bin dense_break_live_path_render -- \
  --source "$source" \
  --output "$output" \
  --bpm 120 \
  --downbeat-seconds 0 \
  --tonal-live-review

python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$output/tonal-live-manifest.json"

for artifact in \
  00_source.wav \
  tonal/00_source_context.wav \
  tonal/01_held_hook.wav \
  tonal/02_pitch_dive.wav \
  tonal/03_ordinary_reentry.wav \
  tonal/04_restart_recall.wav \
  tonal/05_live_journey.wav \
  stems/01_w30_source_hook.wav \
  stems/02_tr909_support.wav \
  stems/03_mc202_stay_out.wav \
  stems/04_w30_pitch_dive.wav \
  session.json \
  source-graph.json
do
  test -s "$output/$artifact"
done

jq -e \
  --arg expected_sha "sha256:$expected_sha" \
  '
  .schema_version == 1
  and .pack_id == "tonal-hook-live-journey"
  and .result == "pass"
  and .source_backed == true
  and .source_timing_backed == true
  and .quality_proof == false
  and .human_verdict == "unverified"
  and .source.content_hash == $expected_sha
  and .timing_identity.manual_bpm == 120
  and .timing_identity.manual_downbeat_seconds == 0
  and .source_character_policy.character == "tonal_hook"
  and .source_character_policy.schema == "riotbox.live_performance_policy.v2"
  and .source_character_policy.lead == "w30_hook"
  and .source_character_policy.bass_owner == "unassigned"
  and .source_character_policy.mc202_intent == "stay_out"
  and .source_character_policy.tr909_intent == "stay_out"
  and .journey.contrast == "w30.pitch_dive"
  and .journey.ordinary_reentry_cleared_articulation == true
  and .journey.saved_before_restart == true
  and .journey.restart_preset_survived == true
  and .exact_mixer_proof.callback_partitions_sample_exact == true
  and .exact_mixer_proof.pitch_dive_first_eight_beats_sample_exact_to_held_w30 == true
  and .exact_mixer_proof.pitch_dive_active_tail_delta_rms >= .thresholds.min_isolated_pitch_dive_active_tail_delta_rms
  and .exact_mixer_proof.human_review_sequence_duration_seconds == 30
  and .metrics.w30.rms >= .thresholds.min_lane_rms
  and .metrics.tr909.rms <= .thresholds.max_tr909_stay_out_rms
  and .metrics.mc202.rms <= .thresholds.max_mc202_stay_out_rms
  and (.failures | length) == 0
  ' \
  "$output/tonal-live-manifest.json"

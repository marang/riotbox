#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-sparse-pressure-live-journey}"
source='data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav'
expected_sha='8a970e5d7bd9b29771aba85f75e697c7510940d4404714bfb1e55e210c15f46c'

actual_sha="$(sha256sum -- "$source" | cut -d ' ' -f 1)"
test "$actual_sha" = "$expected_sha"

cargo run -p riotbox-app --bin dense_break_live_path_render -- \
  --source "$source" \
  --output "$output" \
  --bpm 120 \
  --downbeat-seconds 0 \
  --sparse-live-review

python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$output/sparse-live-manifest.json"

for artifact in \
  00_source.wav \
  sparse/00_source_context.wav \
  sparse/01_held.wav \
  sparse/02_transient_bite.wav \
  sparse/03_ordinary_reentry.wav \
  sparse/04_restart_recall.wav \
  sparse/05_live_journey.wav \
  sparse/06_human_review_sequence.wav \
  stems/01_w30_held.wav \
  stems/02_tr909_held.wav \
  stems/03_mc202_held.wav \
  stems/04_w30_damage.wav \
  session.json \
  source-graph.json
do
  test -s "$output/$artifact"
done

jq -e \
  --arg expected_sha "sha256:$expected_sha" \
  '
  .schema_version == 1
  and .pack_id == "sparse-pressure-live-journey"
  and .result == "pass"
  and .source_backed == true
  and .source_timing_backed == true
  and .quality_proof == false
  and .human_verdict == "unverified"
  and .source.content_hash == $expected_sha
  and .timing_identity.manual_bpm == 120
  and .timing_identity.manual_downbeat_seconds == 0
  and .source_character_policy.schema == "riotbox.live_performance_policy.v2"
  and .source_character_policy.character == "sparse_pressure"
  and .source_character_policy.destructive_intent == "transient_bite"
  and .source_character_policy.lead == "tr909_pressure"
  and .source_character_policy.bass_owner == "unassigned"
  and .source_character_policy.tr909_intent == "lead"
  and .source_character_policy.mc202_intent == "punctuate"
  and .mc202_source_phrase_renderer_schema == "riotbox.mc202_source_phrase_renderer.v2"
  and .journey.held_beats == 16
  and .journey.damage_beats == 16
  and .journey.ordinary_reentry_beats == 8
  and .journey.damage_intensity == 0.82
  and .journey.bypass_intensity == 0
  and .journey.saved_before_restart == true
  and .journey.restart_preset_survived == true
  and .exact_mixer_proof.callback_partitions_sample_exact == true
  and (.exact_mixer_proof.callback_partition_stage_sample_exact | all)
  and .exact_mixer_proof.w30_callback_partitions_sample_exact == true
  and .exact_mixer_proof.tr909_callback_partitions_sample_exact == true
  and .exact_mixer_proof.mc202_callback_partitions_sample_exact == true
  and .exact_mixer_proof.damage_gate_step_fraction == 0.3608
  and .exact_mixer_proof.expected_damage_gate_step_fraction == 0.3608
  and .exact_mixer_proof.reentry_gate_step_fraction == 0
  and .exact_mixer_proof.restart_gate_step_fraction == 0
  and .exact_mixer_proof.source_monitor_max_rms <= .thresholds.max_source_monitor_rms
  and .metrics.damage_delta.rms >= .thresholds.min_damage_delta_rms
  and .metrics.w30_damage_delta.rms >= .thresholds.min_damage_delta_rms
  and .metrics.w30_reentry_delta.rms >= .thresholds.min_damage_delta_rms
  and .metrics.tr909_held.peak_abs > .metrics.w30_held.peak_abs
  and .metrics.tr909_held.crest_factor > .metrics.w30_held.crest_factor
  and (.failures | length) == 0
  ' \
  "$output/sparse-live-manifest.json"

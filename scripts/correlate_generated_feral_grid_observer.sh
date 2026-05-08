#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="$tmpdir/feral-grid-observer/events.ndjson"
locked_observer_fixture="$tmpdir/feral-grid-observer/events-locked.ndjson"
mismatched_observer_fixture="$tmpdir/feral-grid-observer/events-mismatched-source-timing.ndjson"
mismatch_output="$tmpdir/feral-grid-observer/mismatch-output.txt"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0
python3 scripts/write_synthetic_break_wav.py "$tmpdir/source-locked.wav" 16.0
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
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_anchor_count == 0)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_kick_anchor_count == 0)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_backbeat_anchor_count == 0)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_transient_anchor_count == 0)
    and all(.[]; .snapshot.source_timing.groove_evidence.primary_groove_residual_count == 0)
    and all(.[]; .snapshot.source_timing.groove_evidence.primary_groove_preview == [])
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
jq -e \
  '.grid_bpm_source == "static_default"
    and .grid_bpm_decision_reason == "source_timing_requires_manual_confirm"
    and .source_timing.readiness == "ready"
    and .source_timing.requires_manual_confirm == true
    and .source_timing.anchor_evidence.primary_anchor_count > 0
    and .metrics.tr909_groove_timing.applied == false
    and .metrics.tr909_groove_timing.reason == "not_source_timing_grid"
    and (.source_timing_bpm_delta | type == "number")' \
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
    and .output_path.grid_bpm_source == "static_default"
    and .output_path.grid_bpm_decision_reason == "source_timing_requires_manual_confirm"
    and (.output_path.source_timing_bpm_delta | type == "number")
    and .output_path.source_timing_alignment.status == "aligned"
    and .output_path.source_timing_anchor_alignment.status == "partial"
    and .output_path.source_timing_anchor_alignment.observer.primary_anchor_count == 0
    and .output_path.source_timing_anchor_alignment.manifest.primary_anchor_count > 0
    and .output_path.source_timing_groove_alignment.status == "partial"
    and .output_path.source_timing_groove_alignment.observer.primary_groove_residual_count == 0
    and .output_path.source_timing_groove_alignment.manifest.primary_groove_residual_count > 0
    and .output_path.source_timing_alignment.bpm_delta <= .output_path.source_timing_alignment.bpm_tolerance
    and (.output_path.source_timing_alignment.warning_overlap | index("phrase_uncertain")) != null
    and (.output_path.source_timing_alignment.issues | length == 0)
    and .output_path.present == true
    and (.output_path.issues | length == 0)' \
  "$tmpdir/observer-audio-summary.json"
python3 scripts/validate_observer_audio_summary_json.py \
  "$tmpdir/observer-audio-summary.json"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe feral-grid-jam-locked \
  --observer "$locked_observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$locked_observer_fixture"
jq -s -e \
  'length >= 6
    and .[0].event == "observer_started"
    and .[0].launch.probe == "feral-grid-jam-locked"
    and all(.[]; has("snapshot"))
    and all(.[]; .snapshot.source_timing.source_id == "src-feral-grid-probe")
    and all(.[]; .snapshot.source_timing.quality == "high")
    and all(.[]; .snapshot.source_timing.degraded_policy == "locked")
    and all(.[]; .snapshot.source_timing.beat_status == "grid")
    and all(.[]; .snapshot.source_timing.beat_count == 16)
    and all(.[]; .snapshot.source_timing.downbeat_status == "bar_locked")
    and all(.[]; .snapshot.source_timing.bar_count == 4)
    and all(.[]; .snapshot.source_timing.phrase_status == "phrase_locked")
    and all(.[]; .snapshot.source_timing.phrase_count == 1)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_anchor_count == 16)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_kick_anchor_count == 4)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_backbeat_anchor_count == 8)
    and all(.[]; .snapshot.source_timing.anchor_evidence.primary_transient_anchor_count == 4)
    and all(.[]; .snapshot.source_timing.groove_evidence.primary_groove_residual_count == 2)
    and all(.[]; .snapshot.source_timing.groove_evidence.primary_max_abs_offset_ms == 6.0)
    and all(.[]; .snapshot.source_timing.groove_evidence.primary_groove_preview[0].subdivision == "eighth")
    and all(.[]; .snapshot.source_timing.primary_warning_code == null)
    and all(.[]; .snapshot.source_timing.warning_codes == [])
    and any(.[]; .event == "key_outcome" and .key == "f" and .outcome == "queue_tr909_fill")
    and any(.[]; .event == "key_outcome" and .key == "g" and .outcome == "queue_mc202_generate_follower")' \
  "$locked_observer_fixture"

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$tmpdir/source-locked.wav" \
  --output-dir "$tmpdir/feral-grid-locked" \
  --bars 4 \
  --source-window-seconds 1.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/feral-grid-locked/manifest.json"
jq -e \
  '.grid_bpm_source == "source_timing"
    and .grid_bpm_decision_reason == "source_timing_ready"
    and .source_timing.readiness == "ready"
    and .source_timing.requires_manual_confirm == false
    and .source_timing.phrase_status == "stable"
    and .source_timing.anchor_evidence.primary_kick_anchor_count > 0
    and .source_timing.anchor_evidence.primary_backbeat_anchor_count > 0
    and .source_timing.warning_codes == []
    and .metrics.tr909_groove_timing.applied == true
    and .metrics.tr909_groove_timing.reason == "source_timing_groove_residual"
    and (.metrics.tr909_groove_timing.offset_ms | type == "number")
    and .source_timing_bpm_delta == 0.0' \
  "$tmpdir/feral-grid-locked/manifest.json"

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$locked_observer_fixture" \
  --manifest "$tmpdir/feral-grid-locked/manifest.json" \
  --output "$tmpdir/observer-audio-summary-locked.json" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .schema_version == 1
    and .control_path.present == true
    and .control_path.observer_source_timing.source_id == "src-feral-grid-probe"
    and .control_path.observer_source_timing.quality == "high"
    and .control_path.observer_source_timing.degraded_policy == "locked"
    and .control_path.observer_source_timing.beat_status == "grid"
    and .control_path.observer_source_timing.beat_count == 16
    and .control_path.observer_source_timing.downbeat_status == "bar_locked"
    and .control_path.observer_source_timing.bar_count == 4
    and .control_path.observer_source_timing.phrase_status == "phrase_locked"
    and .control_path.observer_source_timing.phrase_count == 1
    and .control_path.observer_source_timing.primary_warning_code == null
    and .output_path.grid_bpm_source == "source_timing"
    and .output_path.grid_bpm_decision_reason == "source_timing_ready"
    and .output_path.source_timing_bpm_delta == 0.0
    and .output_path.source_timing_alignment.status == "aligned"
    and .output_path.source_timing_anchor_alignment.status == "aligned"
    and .output_path.source_timing_anchor_alignment.observer.primary_anchor_count == 16
    and .output_path.source_timing_anchor_alignment.manifest.primary_anchor_count > 0
    and .output_path.source_timing_groove_alignment.status == "aligned"
    and .output_path.source_timing_groove_alignment.observer.primary_groove_residual_count == 2
    and .output_path.source_timing_groove_alignment.manifest.primary_groove_residual_count > 0
    and (.output_path.source_timing_alignment.warning_overlap | length == 0)
    and (.output_path.source_timing_alignment.issues | length == 0)
    and .output_path.present == true
    and (.output_path.issues | length == 0)' \
  "$tmpdir/observer-audio-summary-locked.json"
python3 scripts/validate_observer_audio_summary_json.py \
  "$tmpdir/observer-audio-summary-locked.json"

jq -c '.snapshot.source_timing.bpm_estimate = 118.0' \
  "$observer_fixture" > "$mismatched_observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py \
  "$mismatched_observer_fixture"
if cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$mismatched_observer_fixture" \
  --manifest "$tmpdir/feral-grid/manifest.json" \
  --require-evidence > "$mismatch_output" 2>&1; then
  cat "$mismatch_output" >&2
  echo "expected source timing alignment mismatch to fail strict evidence" >&2
  exit 1
fi
grep -q "source_timing_alignment.bpm_delta" "$mismatch_output"
grep -q "missing passing output-path manifest evidence" "$mismatch_output"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="$tmpdir/events.ndjson"
manifest="$tmpdir/manifest.json"
summary="$tmpdir/observer-audio-summary.json"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe p014-scene-movement \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 4
    and .[0].event == "observer_started"
    and .[0].launch.probe == "p014-scene-movement"
    and any(.[]; .event == "key_outcome"
      and .key == "y"
      and .outcome == "queue_scene_select"
      and .snapshot.scene.arrangement_contract.has_pending_scene_transition == true)
    and any(.[]; .event == "transport_commit"
      and .committed[0].boundary == "Bar"
      and .snapshot.scene.active_scene == "scene-02-drop"
      and .snapshot.scene.last_movement.kind == "launch"
      and .snapshot.scene.last_movement.direction == "rise"
      and .snapshot.scene.last_movement.tr909_intent == "drive"
      and .snapshot.scene.last_movement.mc202_intent == "lift"
      and .snapshot.scene.last_movement.from_scene == "scene-01-break"
      and .snapshot.scene.last_movement.to_scene == "scene-02-drop"
      and .snapshot.scene.arrangement_contract.can_use_source_locked_scene_movement == true
      and .snapshot.scene.source_monitor.source_anchor_seconds == 16.0
      and .snapshot.scene.source_monitor.source_anchor_position_beats == 36.0)' \
  "$observer_fixture"

cat > "$manifest" <<'JSON'
{
  "schema_version": 1,
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "grid_bpm_source": "source_timing",
  "grid_bpm_decision_reason": "source_timing_ready",
  "source_timing_bpm_delta": 0.0,
  "artifacts": [
    { "role": "tr909_scene_movement", "kind": "wav", "path": "stems/01_tr909_scene_movement.wav" },
    { "role": "mc202_scene_movement", "kind": "wav", "path": "stems/02_mc202_scene_movement.wav" },
    { "role": "full_grid_mix", "kind": "wav", "path": "03_riotbox_scene_movement_mix.wav" },
    { "role": "report", "kind": "markdown", "path": "report.md" },
    { "role": "manifest", "kind": "json", "path": "manifest.json" }
  ],
  "source_timing": {
    "source_id": "src-p014-scene-movement",
    "policy_profile": "p014_scene_movement_probe",
    "grid_use": "locked_grid",
    "readiness": "ready",
    "requires_manual_confirm": false,
    "cue": "grid locked",
    "actionability": "grid can steer moves",
    "primary_bpm": 120.0,
    "bpm_agrees_with_grid": true,
    "beat_status": "stable",
    "downbeat_status": "ambiguous",
    "primary_downbeat_offset_beats": 0,
    "primary_downbeat_score": null,
    "primary_downbeat_margin": null,
    "alternate_downbeat_phase_count": 0,
    "confidence_result": "candidate_cautious",
    "drift_status": "stable",
    "phrase_status": "stable",
    "primary_phrase_count": 1,
    "primary_phrase_bar_count": 8,
    "alternate_evidence_count": 0,
    "anchor_evidence": {
      "primary_anchor_count": 0,
      "primary_kick_anchor_count": 0,
      "primary_backbeat_anchor_count": 0,
      "primary_transient_anchor_count": 0
    },
    "groove_evidence": {
      "primary_groove_residual_count": 0,
      "primary_max_abs_offset_ms": 0.0,
      "primary_groove_preview": []
    },
    "warning_codes": []
  },
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.18 },
      "low_band": { "rms": 0.06 }
    },
    "source_grid_output_drift": {
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    },
    "tr909_source_grid_alignment": {
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    },
    "mc202_bass_pressure": {
      "pattern_origin": "primitive_renderer",
      "applied": true
    },
    "mc202_source_grid_alignment": {
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_grid_alignment": {
      "hit_ratio": 1.0,
      "max_peak_offset_ms": 1.0,
      "max_allowed_peak_offset_ms": 70.0
    },
    "w30_source_loop_closure": {
      "passed": true,
      "preview_rms": 0.12,
      "edge_delta_abs": 0.004,
      "max_allowed_edge_delta_abs": 0.06,
      "edge_abs_max": 0.006,
      "max_allowed_edge_abs": 0.04,
      "source_contains_selection": true
    }
  }
}
JSON

cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer "$observer_fixture" \
  --manifest "$manifest" \
  --output "$summary" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .control_path.present == true
    and .control_path.observer_scene_movement.kind == "launch"
    and .control_path.observer_scene_movement.direction == "rise"
    and .control_path.observer_scene_movement.to_scene == "scene-02-drop"
    and .control_path.observer_scene_movement.can_use_source_locked_scene_movement == true
    and .control_path.observer_scene_movement.source_anchor_seconds == 16.0
    and .output_path.present == true
    and .output_path.scene_movement_audio_evidence.present == true
    and (.output_path.scene_movement_audio_evidence.issues | length == 0)' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

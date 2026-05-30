use crossterm::event::KeyCode;
use riotbox_core::{
    action::Action, persistence::SessionRecoveryCandidateKind, queue::CommittedActionRef,
    view::jam::source_timing_confirmation_matches_graph,
};
use serde_json::{Value, json};

use crate::jam_app::{RecoveryCandidateTrust, SessionRecoverySurface};
use crate::ui::{JamShellState, ShellKeyOutcome};

pub fn observer_snapshot(shell: &JamShellState) -> Value {
    let transport = &shell.app.runtime.transport;
    let runtime = &shell.app.runtime_view;
    json!({
        "status_message": shell.status_message,
        "active_screen": shell.active_screen.label(),
        "jam_mode": shell.jam_mode.label(),
        "show_help": shell.show_help,
        "transport": {
            "is_playing": transport.is_playing,
            "position_beats": transport.position_beats,
            "beat_index": transport.beat_index,
            "bar_index": transport.bar_index,
            "phrase_index": transport.phrase_index,
            "current_scene": transport.current_scene.as_ref().map(ToString::to_string),
        },
        "queue": {
            "pending_count": shell.app.queue.pending_actions().len(),
            "queue_history_count": shell.app.queue.history().len(),
            "session_log_count": shell.app.session.action_log.actions.len(),
            "pending": shell.app.queue.pending_actions().into_iter().map(compact_action).collect::<Vec<_>>(),
            "recent_history": shell.app.queue.history().iter().rev().take(5).map(compact_action).collect::<Vec<_>>(),
        },
        "runtime": {
            "audio_status": runtime.audio_status,
            "audio_callback_count": runtime.audio_callback_count,
            "audio_last_error": runtime.audio_last_error,
            "capture_length_intent": shell.app.session.runtime_state.capture.length_intent.to_string(),
            "sidecar_status": runtime.sidecar_status,
            "source_monitor_mode": runtime.source_monitor_mode,
            "source_monitor_audio_route": runtime.source_monitor_audio_route,
            "tr909_mode": runtime.tr909_render_mode,
            "tr909_routing": runtime.tr909_render_routing,
            "tr909_profile": runtime.tr909_render_profile,
            "tr909_support_context": runtime.tr909_render_support_context,
            "tr909_support_accent": runtime.tr909_render_support_accent,
            "mc202_mode": runtime.mc202_render_mode,
            "mc202_routing": runtime.mc202_render_routing,
            "mc202_phrase_shape": runtime.mc202_render_phrase_shape,
            "mc202_mix": runtime.mc202_render_mix_summary,
            "w30_preview_mode": runtime.w30_preview_mode,
            "w30_preview_target": runtime.w30_preview_target_summary,
            "w30_resample_tap_mode": runtime.w30_resample_tap_mode,
            "warnings": runtime.runtime_warnings,
        },
        "source_timing": source_timing_observer_snapshot(shell),
        "source_map": source_map_observer_snapshot(shell),
        "scene": scene_observer_snapshot(shell),
        "capture": capture_observer_snapshot(shell),
        "recovery": recovery_observer_snapshot(shell),
    })
}

pub fn compact_commit(committed: &CommittedActionRef) -> Value {
    json!({
        "action_id": committed.action_id.0,
        "boundary": format!("{:?}", committed.boundary.kind),
        "beat_index": committed.boundary.beat_index,
        "bar_index": committed.boundary.bar_index,
        "phrase_index": committed.boundary.phrase_index,
        "scene_id": committed.boundary.scene_id.as_ref().map(ToString::to_string),
        "commit_sequence": committed.commit_sequence,
    })
}

pub fn key_code_label(code: KeyCode) -> String {
    match code {
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(character) => character.to_string(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "escape".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::BackTab => "backtab".into(),
        other => format!("{other:?}"),
    }
}

pub fn shell_key_outcome_label(outcome: ShellKeyOutcome) -> &'static str {
    match outcome {
        ShellKeyOutcome::Continue => "continue",
        ShellKeyOutcome::RequestRefresh => "request_refresh",
        ShellKeyOutcome::ToggleTransport => "toggle_transport",
        ShellKeyOutcome::QueueSceneMutation => "queue_scene_mutation",
        ShellKeyOutcome::QueueSceneSelect => "queue_scene_select",
        ShellKeyOutcome::QueueSceneRestore => "queue_scene_restore",
        ShellKeyOutcome::QueueMc202RoleToggle => "queue_mc202_role_toggle",
        ShellKeyOutcome::QueueMc202GenerateFollower => "queue_mc202_generate_follower",
        ShellKeyOutcome::QueueMc202GenerateAnswer => "queue_mc202_generate_answer",
        ShellKeyOutcome::QueueMc202GeneratePressure => "queue_mc202_generate_pressure",
        ShellKeyOutcome::QueueMc202GenerateInstigator => "queue_mc202_generate_instigator",
        ShellKeyOutcome::QueueMc202MutatePhrase => "queue_mc202_mutate_phrase",
        ShellKeyOutcome::QueueTr909Fill => "queue_tr909_fill",
        ShellKeyOutcome::QueueTr909Reinforce => "queue_tr909_reinforce",
        ShellKeyOutcome::QueueTr909Slam => "queue_tr909_slam",
        ShellKeyOutcome::QueueTr909Takeover => "queue_tr909_takeover",
        ShellKeyOutcome::QueueTr909SceneLock => "queue_tr909_scene_lock",
        ShellKeyOutcome::QueueTr909Release => "queue_tr909_release",
        ShellKeyOutcome::QueueCaptureBar => "queue_capture_bar",
        ShellKeyOutcome::PromoteLastCapture => "promote_last_capture",
        ShellKeyOutcome::QueueW30TriggerPad => "queue_w30_trigger_pad",
        ShellKeyOutcome::QueueW30StepFocus => "queue_w30_step_focus",
        ShellKeyOutcome::QueueW30SwapBank => "queue_w30_swap_bank",
        ShellKeyOutcome::QueueW30BrowseSlicePool => "queue_w30_browse_slice_pool",
        ShellKeyOutcome::QueueW30ApplyDamageProfile => "queue_w30_apply_damage_profile",
        ShellKeyOutcome::QueueW30LoopFreeze => "queue_w30_loop_freeze",
        ShellKeyOutcome::QueueW30LiveRecall => "queue_w30_live_recall",
        ShellKeyOutcome::QueueW30Audition => "queue_w30_audition",
        ShellKeyOutcome::QueueW30Resample => "queue_w30_resample",
        ShellKeyOutcome::ConfirmSourceTimingGrid => "confirm_source_timing_grid",
        ShellKeyOutcome::RevertSourceTimingGrid => "revert_source_timing_grid",
        ShellKeyOutcome::NavigateSourceMapPreviousBar => "navigate_source_map_previous_bar",
        ShellKeyOutcome::NavigateSourceMapNextBar => "navigate_source_map_next_bar",
        ShellKeyOutcome::NavigateSourceMapPreviousPhrase => "navigate_source_map_previous_phrase",
        ShellKeyOutcome::NavigateSourceMapNextPhrase => "navigate_source_map_next_phrase",
        ShellKeyOutcome::PreviousCaptureLength => "previous_capture_length",
        ShellKeyOutcome::NextCaptureLength => "next_capture_length",
        ShellKeyOutcome::TogglePinLatestCapture => "toggle_pin_latest_capture",
        ShellKeyOutcome::LowerDrumBusLevel => "lower_drum_bus_level",
        ShellKeyOutcome::RaiseDrumBusLevel => "raise_drum_bus_level",
        ShellKeyOutcome::LowerMc202Touch => "lower_mc202_touch",
        ShellKeyOutcome::RaiseMc202Touch => "raise_mc202_touch",
        ShellKeyOutcome::AcceptCurrentGhostSuggestion => "accept_current_ghost_suggestion",
        ShellKeyOutcome::RejectCurrentGhostSuggestion => "reject_current_ghost_suggestion",
        ShellKeyOutcome::UndoLast => "undo_last",
        ShellKeyOutcome::Quit => "quit",
    }
}

fn recovery_observer_snapshot(shell: &JamShellState) -> Value {
    let Some(surface) = shell.recovery_surface.as_ref() else {
        return json!({
            "present": false,
            "has_manual_candidates": false,
            "selected_candidate": null,
            "candidate_count": 0,
            "candidates": [],
            "manual_choice_dry_run": null,
        });
    };

    json!({
        "present": true,
        "headline": surface.headline,
        "safety_note": surface.safety_note,
        "target_path": surface.target_path,
        "has_manual_candidates": surface.has_manual_candidates(),
        "selected_candidate": surface.selected_candidate,
        "candidate_count": surface.candidates.len(),
        "manual_choice_dry_run": recovery_manual_choice_dry_run_snapshot(surface),
        "candidates": surface.candidates.iter().map(|candidate| {
            json!({
                "path": candidate.path,
                "kind": candidate.kind_label,
                "status": candidate.status_label,
                "artifact_availability": candidate.artifact_availability_label,
                "replay_readiness": candidate.replay_readiness_label,
                "payload_readiness": candidate.payload_readiness_label,
                "replay_suffix": candidate.replay_suffix_label,
                "replay_family": candidate.replay_family_label,
                "replay_unsupported": candidate.replay_unsupported_label,
                "decision": candidate.decision_label,
                "guidance": candidate.guidance.as_ref().map(|guidance| guidance.help_label()),
                "trust": format!("{:?}", candidate.trust),
                "action_hint": candidate.action_hint,
            })
        }).collect::<Vec<_>>(),
    })
}

fn source_timing_observer_snapshot(shell: &JamShellState) -> Value {
    let Some(graph) = shell.app.source_graph.as_ref() else {
        return Value::Null;
    };
    let timing = &shell.app.jam_view.source.timing;
    let confirmed_grid = shell
        .app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref();
    let confirmed_grid_matches_current_source =
        source_timing_confirmation_matches_graph(graph, &shell.app.session);

    json!({
        "present": true,
        "source_id": graph.source.source_id.to_string(),
        "bpm_estimate": graph.timing.bpm_estimate,
        "bpm_confidence": graph.timing.bpm_confidence,
        "quality": timing.quality.as_str(),
        "degraded_policy": timing.degraded_policy.as_str(),
        "cue": timing.cue.as_str(),
        "actionability": timing.actionability.as_str(),
        "grid_use": timing.grid_use.as_str(),
        "beat_status": timing.beat_status.as_str(),
        "beat_count": timing.beat_count,
        "downbeat_status": timing.downbeat_status.as_str(),
        "primary_downbeat_offset_beats": timing.primary_downbeat_offset_beats,
        "primary_downbeat_score": timing.primary_downbeat_score,
        "primary_downbeat_score_gap": timing.primary_downbeat_score_gap,
        "alternate_downbeat_phase_count": timing.alternate_downbeat_phase_count,
        "bar_count": timing.bar_count,
        "phrase_status": timing.phrase_status.as_str(),
        "phrase_count": timing.phrase_count,
        "primary_hypothesis_id": graph.timing.primary_hypothesis_id.as_deref(),
        "grid_confirmed": confirmed_grid_matches_current_source,
        "confirmed_grid_source_id": confirmed_grid.map(|confirmed| confirmed.source_id.to_string()),
        "confirmed_grid_hypothesis_id": confirmed_grid.and_then(|confirmed| confirmed.hypothesis_id.as_deref()),
        "confirmed_grid_action_id": confirmed_grid.map(|confirmed| confirmed.confirmed_by_action.0),
        "confirmed_grid_at": confirmed_grid.map(|confirmed| confirmed.confirmed_at),
        "hypothesis_count": graph.timing.hypotheses.len(),
        "anchor_evidence": source_timing_anchor_evidence_observer_snapshot(timing),
        "primary_anchor_cue": timing.primary_anchor_cue.as_str(),
        "groove_evidence": source_timing_groove_evidence_observer_snapshot(timing),
        "primary_warning_code": timing.primary_warning.as_deref(),
        "warning_codes": graph
            .timing
            .warnings
            .iter()
            .map(|warning| observer_timing_warning_code_label(&warning.code))
            .collect::<Vec<_>>(),
    })
}

fn source_map_observer_snapshot(shell: &JamShellState) -> Value {
    let source_map = &shell.app.jam_view.source.source_map;
    json!({
        "present": source_map.mode != riotbox_core::view::jam::SourceMapModeView::Missing,
        "mode": source_map.mode.label(),
        "trust_label": source_map.trust_label.as_str(),
        "width": source_map.width,
        "energy_row": source_map.energy_row.as_str(),
        "peak_row": source_map.peak_row.as_str(),
        "grid_row": source_map.grid_row.as_str(),
        "playhead_row": source_map.playhead_row.as_str(),
        "playhead_column": source_map.playhead_column,
        "capture_range_row": source_map.capture_range_row.as_str(),
        "capture_range_available": source_map.capture_range_row.contains('[')
            || source_map.capture_range_row.contains('*'),
        "current_region_label": source_map.current_region_label.as_str(),
        "navigation_hint": source_map.navigation_hint.as_str(),
        "capture_hint": source_map.capture_hint.as_str(),
    })
}

fn scene_observer_snapshot(shell: &JamShellState) -> Value {
    let scene = &shell.app.jam_view.scene;
    let contract = &scene.arrangement_contract;
    let source_monitor = shell.app.source_monitor_control_state();
    json!({
        "present": true,
        "active_scene": scene.active_scene.as_deref(),
        "restore_scene": scene.restore_scene.as_deref(),
        "next_scene": scene.next_scene.as_deref(),
        "scene_count": scene.scene_count,
        "scene_jump_availability": scene_jump_availability_label(scene.scene_jump_availability),
        "active_scene_energy": scene.active_scene_energy.as_deref(),
        "restore_scene_energy": scene.restore_scene_energy.as_deref(),
        "next_scene_energy": scene.next_scene_energy.as_deref(),
        "last_movement": scene.last_movement.as_ref().map(|movement| json!({
            "kind": movement.kind.as_str(),
            "direction": movement.direction.as_str(),
            "tr909_intent": movement.tr909_intent.as_str(),
            "mc202_intent": movement.mc202_intent.as_str(),
            "intensity": movement.intensity,
            "from_scene": movement.from_scene.as_deref(),
            "to_scene": movement.to_scene.as_str(),
            "committed_bar_index": movement.committed_bar_index,
            "committed_phrase_index": movement.committed_phrase_index,
        })),
        "arrangement_contract": {
            "readiness": contract.readiness.label(),
            "timing_readiness": contract.timing_readiness.label(),
            "scene_count": contract.scene_count,
            "has_active_scene": contract.has_active_scene,
            "has_next_scene": contract.has_next_scene,
            "has_restore_scene": contract.has_restore_scene,
            "has_pending_scene_transition": contract.has_pending_scene_transition,
            "has_landed_movement": contract.has_landed_movement,
            "can_use_source_locked_scene_movement": contract.can_use_source_locked_scene_movement,
            "requires_p012_source_grid_gate": contract.requires_p012_source_grid_gate,
            "requires_p013_musical_quality_gate": contract.requires_p013_musical_quality_gate,
            "requires_replay_state_proof": contract.requires_replay_state_proof,
            "requires_output_path_proof_for_audible_changes": contract.requires_output_path_proof_for_audible_changes,
        },
        "source_monitor": {
            "source_anchor_seconds": source_monitor.source_anchor_seconds,
            "source_anchor_position_beats": source_monitor.source_anchor_position_beats,
            "audio_route": shell.app.runtime_view.source_monitor_audio_route.as_str(),
        },
    })
}

fn scene_jump_availability_label(
    availability: riotbox_core::view::jam::SceneJumpAvailabilityView,
) -> &'static str {
    match availability {
        riotbox_core::view::jam::SceneJumpAvailabilityView::Ready => "ready",
        riotbox_core::view::jam::SceneJumpAvailabilityView::WaitingForMoreScenes => {
            "waiting_for_more_scenes"
        }
        riotbox_core::view::jam::SceneJumpAvailabilityView::Unknown => "unknown",
    }
}

fn capture_observer_snapshot(shell: &JamShellState) -> Value {
    let Some(capture) = shell.app.session.captures.last() else {
        return json!({
            "present": false,
            "capture_count": 0,
            "latest_capture_id": null,
            "source_window_available": false,
            "source_window": null,
        });
    };
    let source_window = capture.source_window.as_ref().map(|window| {
        json!({
            "source_id": window.source_id.to_string(),
            "start_seconds": window.start_seconds,
            "end_seconds": window.end_seconds,
            "duration_seconds": window.end_seconds - window.start_seconds,
            "start_frame": window.start_frame,
            "end_frame": window.end_frame,
        })
    });

    json!({
        "present": true,
        "capture_count": shell.app.session.captures.len(),
        "latest_capture_id": capture.capture_id.to_string(),
        "latest_capture_type": format!("{:?}", capture.capture_type),
        "created_from_action": capture.created_from_action.map(|action_id| action_id.0),
        "source_origin_count": capture.source_origin_refs.len(),
        "source_window_available": capture.source_window.is_some(),
        "source_window": source_window,
    })
}

fn source_timing_groove_evidence_observer_snapshot(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> Value {
    json!({
        "primary_groove_residual_count": timing.primary_groove_residual_count,
        "primary_max_abs_offset_ms": timing.primary_max_abs_groove_offset_ms,
        "primary_groove_preview": timing
            .primary_groove_preview
            .iter()
            .map(|residual| {
                json!({
                    "subdivision": residual.subdivision.as_str(),
                    "offset_ms": residual.offset_ms,
                    "confidence": residual.confidence,
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn source_timing_anchor_evidence_observer_snapshot(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> Value {
    json!({
        "primary_anchor_count": timing.primary_anchor_count,
        "primary_kick_anchor_count": timing.primary_kick_anchor_count,
        "primary_backbeat_anchor_count": timing.primary_backbeat_anchor_count,
        "primary_transient_anchor_count": timing.primary_transient_anchor_count,
    })
}

fn observer_timing_warning_code_label(
    code: &riotbox_core::source_graph::TimingWarningCode,
) -> &'static str {
    match code {
        riotbox_core::source_graph::TimingWarningCode::SparseOnsets => "sparse_onsets",
        riotbox_core::source_graph::TimingWarningCode::WeakKickAnchor => "weak_kick_anchor",
        riotbox_core::source_graph::TimingWarningCode::WeakBackbeatAnchor => "weak_backbeat_anchor",
        riotbox_core::source_graph::TimingWarningCode::AmbiguousDownbeat => "ambiguous_downbeat",
        riotbox_core::source_graph::TimingWarningCode::HalfTimePossible => "half_time_possible",
        riotbox_core::source_graph::TimingWarningCode::DoubleTimePossible => "double_time_possible",
        riotbox_core::source_graph::TimingWarningCode::DriftHigh => "drift_high",
        riotbox_core::source_graph::TimingWarningCode::PhraseUncertain => "phrase_uncertain",
        riotbox_core::source_graph::TimingWarningCode::LowTimingConfidence => {
            "low_timing_confidence"
        }
    }
}

fn recovery_manual_choice_dry_run_snapshot(surface: &SessionRecoverySurface) -> Option<Value> {
    let candidate = surface.candidates.iter().find(|candidate| {
        !matches!(
            candidate.kind,
            SessionRecoveryCandidateKind::CanonicalTarget
        ) && matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue)
    })?;
    let dry_run = surface.dry_run_manual_choice(&candidate.path)?;

    Some(json!({
        "candidate_path": dry_run.candidate_path,
        "decision": dry_run.decision_label,
        "artifact_availability": dry_run.artifact_availability_label,
        "replay_readiness": dry_run.replay_readiness_label,
        "payload_readiness": dry_run.payload_readiness_label,
        "replay_suffix": dry_run.replay_suffix_label,
        "replay_family": dry_run.replay_family_label,
        "replay_unsupported": dry_run.replay_unsupported_label,
        "guidance": dry_run.guidance_label,
        "trust": format!("{:?}", dry_run.trust),
        "action_hint": dry_run.action_hint,
        "selected_for_restore": dry_run.selected_for_restore,
        "safety_note": dry_run.safety_note,
    }))
}

fn compact_action(action: &Action) -> Value {
    json!({
        "id": action.id.0,
        "command": action.command.as_str(),
        "actor": action.actor.to_string(),
        "quantization": action.quantization.to_string(),
        "status": format!("{:?}", action.status),
        "requested_at": action.requested_at,
        "committed_at": action.committed_at,
        "result": action.result.as_ref().map(|result| result.summary.clone()),
    })
}

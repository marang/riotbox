use crossterm::event::KeyCode;
use riotbox_core::{
    action::Action, persistence::SessionRecoveryCandidateKind, queue::CommittedActionRef,
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
            "sidecar_status": runtime.sidecar_status,
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
                "replay_unsupported": candidate.replay_unsupported_label,
                "decision": candidate.decision_label,
                "guidance": candidate.guidance.as_ref().map(|guidance| guidance.help_label()),
                "trust": format!("{:?}", candidate.trust),
                "action_hint": candidate.action_hint,
            })
        }).collect::<Vec<_>>(),
    })
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

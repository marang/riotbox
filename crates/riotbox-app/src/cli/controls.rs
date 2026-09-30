use crate::cli::model::ProductMixExportHandoff;
use crate::cli::observer::UserSessionObserver;
use crate::jam_app::JamAppState;
use crate::observer::shell_key_outcome_label;
use crate::ui::JamShellState;
use crate::ui::ShellKeyOutcome;
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::SourceMonitorMode;
use riotbox_core::queue::CommittedActionRef;
use riotbox_core::style::PerformancePresetId;
use riotbox_core::view::jam::SceneJumpAvailabilityView;
use std::io;

pub(in crate::cli) fn accept_current_ghost_suggestion(
    shell: &mut JamShellState,
    requested_at: u64,
) {
    match shell.app.accept_current_ghost_suggestion(requested_at) {
        crate::jam_app::GhostSuggestionQueueResult::Enqueued(action_id) => {
            shell.set_error_status(format!(
                "accepted ghost suggestion | queued action {}",
                action_id.0
            ));
        }
        crate::jam_app::GhostSuggestionQueueResult::Rejected { reason } => {
            if reason == crate::jam_app::NO_CURRENT_GHOST_SUGGESTION_REASON
                && shell.app.refresh_current_ghost_suggestion_from_jam_state()
                && let Some(suggestion) = shell.app.runtime.current_ghost_suggestion.as_ref()
            {
                shell.set_error_status(format!("ghost suggestion ready: {}", suggestion.summary));
            } else {
                shell.set_error_status(format!("ghost accept ignored: {reason}"));
            }
        }
    }
}

pub(in crate::cli) fn reject_current_ghost_suggestion(shell: &mut JamShellState) {
    if shell.app.reject_current_ghost_suggestion() {
        shell.set_error_status("rejected current ghost suggestion");
    } else {
        shell.set_error_status("ghost reject ignored: no current ghost suggestion");
    }
}

pub(in crate::cli) fn scene_select_unavailable_status(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "scene jump waits for 2 scenes",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
            "no next scene candidate available"
        }
    }
}

pub(in crate::cli) fn queue_and_commit_source_monitor_mode(
    shell: &mut JamShellState,
    mode: SourceMonitorMode,
    requested_at: u64,
) -> Vec<CommittedActionRef> {
    match shell.app.queue_source_monitor_mode(mode, requested_at) {
        crate::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            shell.set_error_status(
                source_monitor_commit_status(shell, &committed)
                    .unwrap_or_else(|| format!("monitor {mode} queued; immediate commit pending")),
            );
            committed
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("source monitor change already queued");
            Vec::new()
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status(format!("monitor already {mode}"));
            Vec::new()
        }
    }
}

pub(in crate::cli) fn queue_and_commit_performance_preset(
    shell: &mut JamShellState,
    preset_id: PerformancePresetId,
    requested_at: u64,
) -> Vec<CommittedActionRef> {
    match shell.app.queue_performance_preset(preset_id, requested_at) {
        crate::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            let landed = committed.iter().any(|committed| {
                shell
                    .app
                    .queue
                    .history_action(committed.action_id)
                    .is_some_and(|action| action.command == ActionCommand::PresetActivate)
            });
            shell.set_error_status(if landed {
                format!(
                    "{} active | monitor {} | source role policy {}",
                    preset_id.label(),
                    shell.app.runtime_view.source_monitor_mode,
                    preset_id.definition().mc202_role.label()
                )
            } else {
                format!("{} queued; immediate commit pending", preset_id.label())
            });
            committed
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("performance preset activation already queued");
            Vec::new()
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status(format!("{} already active", preset_id.label()));
            Vec::new()
        }
    }
}

pub(in crate::cli) fn source_monitor_commit_status(
    shell: &JamShellState,
    committed: &[riotbox_core::queue::CommittedActionRef],
) -> Option<String> {
    let monitor_landed = committed.iter().any(|committed| {
        shell
            .app
            .queue
            .history_action(committed.action_id)
            .is_some_and(|action| action.command == ActionCommand::SourceMonitorSetMode)
    });

    monitor_landed.then(|| {
        format!(
            "monitor {} landed | route {}",
            shell.app.runtime_view.source_monitor_mode,
            shell.app.runtime_view.source_monitor_audio_route
        )
    })
}

pub(in crate::cli) fn commit_transport_toggle(
    shell: &mut JamShellState,
    requested_at: u64,
) -> Vec<CommittedActionRef> {
    let toggle = shell.app.commit_transport_toggle(requested_at);
    shell.set_error_status(match toggle.command {
        ActionCommand::TransportPlay => "transport started",
        ActionCommand::TransportPause => "transport paused",
        _ => unreachable!("transport toggle only emits play or pause"),
    });
    toggle.committed
}

pub(in crate::cli) fn record_key_outcome_then_immediate_commit(
    observer: &mut UserSessionObserver,
    timestamp_ms: u64,
    key_label: &str,
    outcome: ShellKeyOutcome,
    shell: &JamShellState,
    immediate_committed: &[CommittedActionRef],
) -> io::Result<()> {
    observer.record_key_event(
        timestamp_ms,
        key_label,
        shell_key_outcome_label(outcome),
        shell,
    )?;

    if !immediate_committed.is_empty() {
        observer.record_transport_commit(timestamp_ms, immediate_committed, shell)?;
    }

    Ok(())
}

pub(in crate::cli) fn persist_and_record_quit(
    shell: &JamShellState,
    observer: Option<&mut UserSessionObserver>,
    timestamp_ms: u64,
    key_label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    shell.app.save()?;

    if let Some(observer) = observer {
        observer.record_key_event(
            timestamp_ms,
            key_label,
            shell_key_outcome_label(ShellKeyOutcome::Quit),
            shell,
        )?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cli) enum AudioRuntimeRefreshAction {
    RetryUnavailable,
    Restart,
}

pub(in crate::cli) fn replace_app_state_after_refresh(
    shell: &mut JamShellState,
    mut refreshed: JamAppState,
    has_audio_runtime: bool,
) -> AudioRuntimeRefreshAction {
    if !has_audio_runtime
        && let Some(faulted_health) = shell
            .app
            .runtime
            .audio
            .as_ref()
            .filter(|health| health.lifecycle == AudioRuntimeLifecycle::Faulted)
            .cloned()
    {
        refreshed.set_audio_health(faulted_health);
    }
    shell.replace_app_state(refreshed);
    if has_audio_runtime {
        AudioRuntimeRefreshAction::Restart
    } else {
        AudioRuntimeRefreshAction::RetryUnavailable
    }
}

pub(in crate::cli) fn commit_capture_length_change(
    shell: &mut JamShellState,
    requested_at: u64,
    next: bool,
) {
    let result = if next {
        shell.app.queue_next_capture_length_intent(requested_at)
    } else {
        shell.app.queue_previous_capture_length_intent(requested_at)
    };
    match result {
        crate::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status("capture length change queued");
            } else {
                shell.set_error_status(format!(
                    "capture length {}",
                    shell.app.session.runtime_state.capture.length_intent
                ));
            }
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("capture length change already queued");
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status(format!(
                "capture length already {}",
                shell.app.session.runtime_state.capture.length_intent
            ));
        }
    }
}

pub(in crate::cli) fn execute_product_mix_export(
    shell: &mut JamShellState,
    handoff: Option<&ProductMixExportHandoff>,
    requested_at: u64,
) {
    let Some(handoff) = handoff else {
        let reason = "product mix export unavailable: launch with --product-export-proof and --product-export-destination";
        shell
            .app
            .reject_product_mix_export_request(requested_at, reason);
        shell.set_error_status(reason);
        return;
    };

    match shell
        .app
        .commit_product_mix_export_from_active_source_proof(
            &handoff.proof_path,
            &handoff.destination_path,
            requested_at,
        ) {
        Ok(receipt) => shell.set_error_status(format!(
            "exported full_grid_mix | receipt {}",
            receipt.receipt_id
        )),
        Err(error) => {
            shell.set_error_status(format!("product mix export failed: {error}"));
        }
    }
}

pub(in crate::cli) fn navigate_source_map(
    shell: &mut JamShellState,
    intent: crate::jam_app::SourceMapNavigationIntent,
    requested_at: u64,
) {
    match shell.app.queue_source_map_navigation(intent, requested_at) {
        crate::jam_app::SourceMapNavigationResult::Enqueued {
            target_label,
            target_position_beats,
        } => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status(format!("source map navigation queued to {target_label}"));
            } else {
                shell.set_error_status(format!(
                    "source map moved to {target_label} @ beat {target_position_beats}"
                ));
            }
        }
        crate::jam_app::SourceMapNavigationResult::AlreadyPending => {
            shell.set_error_status("source map navigation already queued");
        }
        crate::jam_app::SourceMapNavigationResult::AlreadyAtBoundary { target_label } => {
            shell.set_error_status(format!("source map already at {target_label}"));
        }
        crate::jam_app::SourceMapNavigationResult::Unavailable { reason } => {
            shell.set_error_status(reason);
        }
    }
}

pub(in crate::cli) fn confirm_source_timing_grid(shell: &mut JamShellState, requested_at: u64) {
    match shell
        .app
        .queue_source_timing_grid_confirmation(requested_at)
    {
        crate::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status("source timing grid confirmation queued");
            } else {
                shell.set_error_status("confirmed source timing grid");
            }
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("source timing grid trust change already queued");
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            if shell.app.source_graph.is_some() {
                shell.set_error_status("source timing grid already confirmed");
            } else {
                shell.set_error_status("no source timing grid available to confirm");
            }
        }
    }
}

pub(in crate::cli) fn revert_source_timing_grid(shell: &mut JamShellState, requested_at: u64) {
    match shell.app.queue_source_timing_grid_revert(requested_at) {
        crate::jam_app::QueueControlResult::Enqueued => {
            let transport = shell.app.runtime.transport.clone();
            let committed = shell.app.commit_ready_actions(
                riotbox_core::transport::CommitBoundaryState {
                    kind: riotbox_core::action::CommitBoundary::Immediate,
                    beat_index: transport.beat_index,
                    bar_index: transport.bar_index,
                    phrase_index: transport.phrase_index,
                    scene_id: transport.current_scene,
                },
                requested_at,
            );
            if committed.is_empty() {
                shell.set_error_status("source timing grid revert queued");
            } else {
                shell.set_error_status("reverted source timing grid confirmation");
            }
        }
        crate::jam_app::QueueControlResult::AlreadyPending => {
            shell.set_error_status("source timing grid trust change already queued");
        }
        crate::jam_app::QueueControlResult::AlreadyInState => {
            shell.set_error_status("no source timing grid confirmation to revert");
        }
    }
}

fn accept_current_ghost_suggestion(shell: &mut JamShellState, requested_at: u64) {
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

fn reject_current_ghost_suggestion(shell: &mut JamShellState) {
    if shell.app.reject_current_ghost_suggestion() {
        shell.set_error_status("rejected current ghost suggestion");
    } else {
        shell.set_error_status("ghost reject ignored: no current ghost suggestion");
    }
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn scene_select_unavailable_status(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "scene jump waits for 2 scenes",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
            "no next scene candidate available"
        }
    }
}

fn queue_and_commit_source_monitor_mode(
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

fn queue_and_commit_performance_preset(
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

fn source_monitor_commit_status(
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

fn commit_transport_toggle(
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

fn record_key_outcome_then_immediate_commit(
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

fn persist_and_record_quit(
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
enum AudioRuntimeRefreshAction {
    RetryUnavailable,
    Restart,
}

fn replace_app_state_after_refresh(
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

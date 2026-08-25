#[test]
fn source_timing_grid_confirmation_queues_commits_and_persists_session_truth() {
    let mut graph = sample_graph();
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100),
        QueueControlResult::Enqueued
    );
    assert!(matches!(
        state.queue.pending_actions()[0].undo_policy,
        UndoPolicy::NotUndoable { .. }
    ));
    assert_eq!(
        state.queue_source_timing_grid_confirmation(101),
        QueueControlResult::AlreadyPending
    );

    let committed = state.commit_ready_actions(immediate_boundary(), 120);

    assert_eq!(committed.len(), 1);
    let confirmed = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .expect("source timing grid confirmation");
    assert_eq!(confirmed.source_id, SourceId::from("src-1"));
    assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
    assert_eq!(confirmed.confirmed_by_action, committed[0].action_id);
    assert_eq!(confirmed.confirmed_at, 120);
    assert_eq!(
        state.session.runtime_state.source_timing.confirmed_bpm,
        Some(126.0)
    );
    assert_eq!(
        state.queue_source_timing_grid_confirmation(121),
        QueueControlResult::AlreadyInState
    );
}

#[test]
fn source_timing_grid_revert_queues_commits_and_clears_session_truth() {
    let mut graph = sample_graph();
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_source_timing_grid_revert(99),
        QueueControlResult::AlreadyInState
    );
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100),
        QueueControlResult::Enqueued
    );
    let committed_confirm = state.commit_ready_actions(immediate_boundary(), 120);
    assert_eq!(committed_confirm.len(), 1);
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_some());

    assert_eq!(
        state.queue_source_timing_grid_revert(121),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_source_timing_grid_revert(122),
        QueueControlResult::AlreadyPending
    );
    let committed_revert = state.commit_ready_actions(immediate_boundary(), 140);

    assert_eq!(committed_revert.len(), 1);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .expect("committed source timing revert")
            .command,
        ActionCommand::SourceTimingRevertGrid
    );
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert!(state.session.runtime_state.source_timing.confirmed_bpm.is_none());
    assert_eq!(
        state.queue_source_timing_grid_revert(141),
        QueueControlResult::AlreadyInState
    );
}

#[test]
fn unconfirmed_source_timing_keeps_riotbox_only_exact_runtime_mix_silent() {
    let mut graph = sample_graph();
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("ambiguous-primary".into());
    let session = SessionFile::new("session-edge", "0.1.0", "2026-07-21T00:00:00Z");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 0.0,
        beat_index: 0,
        bar_index: 1,
        phrase_index: 1,
        current_scene: None,
    });

    let readiness = riotbox_core::view::jam::source_timing_consumer_readiness(
        state.source_graph.as_ref(),
        &state.session,
    );
    assert_eq!(
        readiness,
        riotbox_core::view::jam::SourceTimingConsumerReadiness::NeedsUserConfirmation
    );
    assert!(!readiness.can_use_source_window_grid());
    assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Idle);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);
    assert_eq!(
        state.runtime.w30_preview.routing,
        W30PreviewRenderRouting::Silent
    );

    let plan = riotbox_audio::runtime::RuntimeMixRenderPlan {
        transport: riotbox_audio::runtime::AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        tr909_render: state.runtime.tr909_render.clone(),
        mc202_render: state.runtime.mc202_render,
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: state.runtime.w30_resample_tap.clone(),
        source_monitor_render: riotbox_audio::runtime::SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ),
    };
    let output = riotbox_audio::runtime::render_runtime_mix_realtime_simulation_offline(
        &plan, 48_000, 2, 4_800, 128,
    );
    let metrics = signal_metrics(&output);

    assert_eq!(metrics.active_samples, 0);
    assert_eq!(metrics.peak_abs, 0.0);
    assert_eq!(metrics.rms, 0.0);
    assert_eq!(metrics.clip_count, 0);
}

fn immediate_boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 0,
        bar_index: 0,
        phrase_index: 0,
        scene_id: None,
    }
}

fn cut_hit_return_state() -> JamAppState {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    riotbox_core::style::PerformancePresetId::FeralBreakAlphaV2.apply_to_session(&mut session);
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph.source.source_id.clone(),
            hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
            confirmed_by_action: ActionId(1),
            confirmed_at: 100,
        });
    let mut source_plan = section_one_mc202_source_plan(graph.source.source_id.clone());
    source_plan.source_section_id = None;
    source_plan.phrase_slot.phrase_index = 0;
    session.runtime_state.lane_state.mc202.source_phrase_plan = Some(source_plan);
    session.action_log.actions.clear();
    session.snapshots.clear();
    JamAppState::from_parts(session, Some(graph), ActionQueue::new())
}

#[test]
fn cut_hit_return_queues_two_canonical_actions_at_one_next_bar_boundary() {
    let mut state = cut_hit_return_state();

    assert_eq!(
        state.queue_tr909_cut_hit_return(110),
        Tr909CutHitReturnQueueResult::Enqueued
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].command, ActionCommand::Tr909FillNext);
    assert_eq!(pending[1].command, ActionCommand::Tr909SetSlam);
    assert!(
        pending
            .iter()
            .all(|action| action.quantization == Quantization::NextBar)
    );
    assert_eq!(
        pending[1].params,
        ActionParams::Mutation {
            intensity: 0.85,
            target_id: Some("enabled".into()),
        }
    );
}

#[test]
fn cut_hit_return_commits_fill_and_slam_then_returns_with_slam_held_and_replays() {
    let mut state = cut_hit_return_state();
    let replay_base = state.session.clone();
    assert_eq!(
        state.queue_tr909_cut_hit_return(110),
        Tr909CutHitReturnQueueResult::Enqueued
    );

    let committed = state.advance_transport_by(1.0, 120);
    assert_eq!(committed.len(), 2);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .iter()
            .map(|action| action.command)
            .collect::<Vec<_>>(),
        vec![ActionCommand::Tr909FillNext, ActionCommand::Tr909SetSlam]
    );
    assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Fill);
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDriveHardCut)
    );
    assert!(state.runtime.tr909_render.slam_enabled);
    let fill_bar = state
        .session
        .runtime_state
        .lane_state
        .tr909
        .last_fill_bar
        .expect("cut-hit return records its fill bar");

    assert!(state.advance_transport_by(4.0, 130).is_empty());
    assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::BreakReinforce);
    assert!(state.runtime.tr909_render.slam_enabled);

    let plan = riotbox_core::replay::build_committed_replay_plan(&state.session.action_log)
        .expect("cut-hit return action pair builds a replay plan");
    assert_eq!(plan.len(), 2);
    let mut replayed = replay_base;
    riotbox_core::replay::apply_replay_plan_to_session(&mut replayed, &plan)
        .expect("cut-hit return action pair replays");
    assert_eq!(
        replayed.runtime_state.lane_state.tr909.last_fill_bar,
        Some(fill_bar)
    );
    assert!(replayed.runtime_state.lane_state.tr909.slam_enabled);
    assert_eq!(replayed.runtime_state.macro_state.tr909_slam, 0.85);
}

#[test]
fn cut_hit_return_refuses_inapplicable_source_and_runtime_states_without_queueing() {
    fn assert_refusal(
        mut state: JamAppState,
        expected_reason: Tr909CutHitReturnUnavailableReason,
    ) {
        assert_eq!(
            state.queue_tr909_cut_hit_return(110),
            Tr909CutHitReturnQueueResult::Unavailable(expected_reason)
        );
        assert!(state.queue.pending_actions().is_empty());
    }

    {
        let mut state = cut_hit_return_state();
        state.session.runtime_state.transport.is_playing = false;
        assert_refusal(
            state,
            Tr909CutHitReturnUnavailableReason::TransportStopped,
        );
    }
    {
        let mut state = cut_hit_return_state();
        state.session.runtime_state.style.active_preset = None;
        assert_refusal(state, Tr909CutHitReturnUnavailableReason::PresetInactive);
    }
    {
        let mut state = cut_hit_return_state();
        state.session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
        assert_refusal(
            state,
            Tr909CutHitReturnUnavailableReason::SourceMonitorNotRiotbox,
        );
    }
    {
        let mut state = cut_hit_return_state();
        state.session.runtime_state.source_timing.confirmed_grid = None;
        assert_refusal(
            state,
            Tr909CutHitReturnUnavailableReason::MissingTrustedLivePolicy,
        );
    }
    {
        let mut state = cut_hit_return_state();
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = None;
        assert_refusal(
            state,
            Tr909CutHitReturnUnavailableReason::Tr909NotReinforcingBreak,
        );
    }
    {
        let mut state = cut_hit_return_state();
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .slam_enabled = true;
        assert_refusal(
            state,
            Tr909CutHitReturnUnavailableReason::SlamAlreadyEnabled,
        );
    }
}

#[test]
fn cut_hit_return_refuses_tonal_and_sparse_source_characters() {
    for (low_mid_ratio, offbeat_density, brightness, restraint) in
        [(0.30, 0.40, 0.80, 0.70), (0.80, 0.10, 0.30, 0.70)]
    {
        let mut state = cut_hit_return_state();
        add_phrase_audio_features(
            state.source_graph.as_mut().expect("source graph"),
            0,
            0.10,
            low_mid_ratio,
            0.10,
            1.0,
            offbeat_density,
            0.05,
            brightness,
            restraint,
        );

        assert_eq!(
            state.queue_tr909_cut_hit_return(110),
            Tr909CutHitReturnQueueResult::Unavailable(
                Tr909CutHitReturnUnavailableReason::SourceCharacterNotDenseBreak
            )
        );
        assert!(state.queue.pending_actions().is_empty());
    }
}

#[test]
fn cut_hit_return_does_not_duplicate_pending_fill_or_slam() {
    let mut fill_pending = cut_hit_return_state();
    fill_pending.queue_tr909_fill(100);
    assert_eq!(
        fill_pending.queue_tr909_cut_hit_return(110),
        Tr909CutHitReturnQueueResult::AlreadyPending
    );
    assert_eq!(fill_pending.queue.pending_actions().len(), 1);

    let mut slam_pending = cut_hit_return_state();
    assert!(slam_pending.queue_tr909_slam_toggle(100));
    assert_eq!(
        slam_pending.queue_tr909_cut_hit_return(110),
        Tr909CutHitReturnQueueResult::AlreadyPending
    );
    assert_eq!(slam_pending.queue.pending_actions().len(), 1);
}

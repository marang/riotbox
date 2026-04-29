#[test]
fn queueing_mc202_follower_generation_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_generate_follower(301),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_mutate_phrase(302),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_generate_pressure(303),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_generate_instigator(304),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202GenerateFollower);
    assert!(state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(!state.jam_view.lanes.mc202_pending_pressure_generation);
    assert!(!state.jam_view.lanes.mc202_pending_instigator_generation);
}

#[test]
fn queueing_mc202_answer_generation_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_answer(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_generate_answer(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202GenerateAnswer);
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(!state.jam_view.lanes.mc202_pending_pressure_generation);
    assert!(!state.jam_view.lanes.mc202_pending_instigator_generation);
}

#[test]
fn queueing_mc202_pressure_generation_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_pressure(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_generate_pressure(301),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_generate_answer(302),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_generate_instigator(303),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202GeneratePressure);
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(state.jam_view.lanes.mc202_pending_pressure_generation);
    assert!(!state.jam_view.lanes.mc202_pending_instigator_generation);
}

#[test]
fn queueing_mc202_instigator_generation_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_instigator(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_generate_instigator(301),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_mc202_generate_answer(302),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202GenerateInstigator);
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(!state.jam_view.lanes.mc202_pending_pressure_generation);
    assert!(state.jam_view.lanes.mc202_pending_instigator_generation);
}

#[test]
fn queueing_mc202_phrase_mutation_requires_committed_voice_and_blocks_duplicates() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = None;
    session.runtime_state.lane_state.mc202.phrase_ref = None;
    let mut empty_state =
        JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        empty_state.queue_mc202_mutate_phrase(299),
        QueueControlResult::AlreadyInState
    );

    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_mutate_phrase(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_mutate_phrase(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202MutatePhrase);
    assert!(state.jam_view.lanes.mc202_pending_phrase_mutation);
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(!state.jam_view.lanes.mc202_pending_pressure_generation);
    assert!(!state.jam_view.lanes.mc202_pending_instigator_generation);
}

#[test]
fn queueing_mc202_role_and_generation_blocks_conflicting_phrase_controls() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_role_toggle(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_generate_follower(301),
        QueueControlResult::AlreadyPending
    );

    let mut other_state = JamAppState::from_parts(
        sample_session(&graph),
        Some(graph.clone()),
        ActionQueue::new(),
    );
    assert_eq!(
        other_state.queue_mc202_generate_follower(302),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        other_state.queue_mc202_role_toggle(303),
        QueueControlResult::AlreadyPending
    );

    let mut answer_state = JamAppState::from_parts(
        sample_session(&graph),
        Some(graph.clone()),
        ActionQueue::new(),
    );
    assert_eq!(
        answer_state.queue_mc202_generate_answer(304),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        answer_state.queue_mc202_role_toggle(305),
        QueueControlResult::AlreadyPending
    );

    let mut pressure_state = JamAppState::from_parts(
        sample_session(&graph),
        Some(graph.clone()),
        ActionQueue::new(),
    );
    assert_eq!(
        pressure_state.queue_mc202_generate_pressure(306),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        pressure_state.queue_mc202_generate_follower(307),
        QueueControlResult::AlreadyPending
    );

    let mut instigator_state = JamAppState::from_parts(
        sample_session(&graph),
        Some(graph.clone()),
        ActionQueue::new(),
    );
    assert_eq!(
        instigator_state.queue_mc202_generate_instigator(308),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        instigator_state.queue_mc202_mutate_phrase(309),
        QueueControlResult::AlreadyPending
    );

    let mut mutation_session = sample_session(&graph);
    mutation_session.runtime_state.lane_state.mc202.role = Some("follower".into());
    let mut mutation_state =
        JamAppState::from_parts(mutation_session, Some(graph), ActionQueue::new());
    assert_eq!(
        mutation_state.queue_mc202_mutate_phrase(310),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        mutation_state.queue_mc202_generate_answer(311),
        QueueControlResult::AlreadyPending
    );
}

#[test]
fn queueing_tr909_slam_blocks_duplicate_pending_slam_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.queue_tr909_slam_toggle(300));
    assert!(!state.queue_tr909_slam_toggle(301));

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Tr909SetSlam);
}

#[test]
fn queueing_tr909_takeover_requires_clear_pending_and_inactive_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_tr909_takeover(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_tr909_takeover(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Tr909Takeover);
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_pending_target,
        Some(true)
    );
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_pending_profile,
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
    );
    assert!(!state.jam_view.lanes.tr909_takeover_enabled);
}

#[test]
fn queueing_tr909_scene_lock_requires_clear_pending_and_non_scene_lock_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        state.queue_tr909_scene_lock(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_tr909_scene_lock(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Tr909SceneLock);
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_pending_target,
        Some(true)
    );
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_pending_profile,
        Some(Tr909TakeoverProfileState::SceneLockTakeover)
    );
    assert!(!state.jam_view.lanes.tr909_takeover_enabled);

    let mut already_locked =
        JamAppState::from_parts(sample_session(&graph), Some(graph), ActionQueue::new());
    already_locked
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_enabled = true;
    already_locked
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_profile = Some(Tr909TakeoverProfileState::SceneLockTakeover);
    already_locked.refresh_view();

    assert_eq!(
        already_locked.queue_tr909_scene_lock(302),
        QueueControlResult::AlreadyInState
    );
}

#[test]
fn queueing_tr909_release_requires_takeover_to_be_active() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.queue_tr909_release(300), QueueControlResult::Enqueued);
    assert_eq!(
        state.queue_tr909_release(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Tr909Release);
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_pending_target,
        Some(false)
    );
    assert_eq!(state.jam_view.lanes.tr909_takeover_pending_profile, None);
    assert!(state.jam_view.lanes.tr909_takeover_enabled);
}

#[test]
fn audio_timing_snapshot_commits_crossed_bar_boundary() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.update_transport_clock(TransportClockState {
        is_playing: false,
        position_beats: 28.0,
        beat_index: 28,
        bar_index: 7,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-1")),
    });
    state.set_transport_playing(true);
    state.queue_tr909_fill(300);

    let committed = state.apply_audio_timing_snapshot(
        AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 124.0,
            position_beats: 32.0,
        },
        3_100,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0].boundary.kind, CommitBoundary::Bar);
    assert_eq!(state.queue.pending_actions().len(), 0);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .map(|action| action.command),
        Some(ActionCommand::Tr909FillNext)
    );
}

#[test]
fn audio_timing_snapshot_advances_transport_from_callback_position() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.update_transport_clock(TransportClockState {
        is_playing: false,
        position_beats: 32.0,
        beat_index: 32,
        bar_index: 8,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-1")),
    });
    state.set_transport_playing(true);

    let committed = state.apply_audio_timing_snapshot(
        AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 124.0,
            position_beats: 33.0,
        },
        2_500,
    );

    assert!(committed.is_empty());
    assert!(state.runtime.transport.position_beats > 32.9);
    assert!(state.runtime.transport.position_beats < 33.1);
    assert_eq!(
        state.runtime.transport_driver.last_audio_position_beats,
        Some(33)
    );
}


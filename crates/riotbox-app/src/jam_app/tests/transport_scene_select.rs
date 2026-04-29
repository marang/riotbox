#[test]
fn updates_transport_clock_and_refreshes_jam_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let clock = TransportClockState {
        is_playing: false,
        position_beats: 48.5,
        beat_index: 48,
        bar_index: 13,
        phrase_index: 4,
        current_scene: Some(SceneId::from("scene-2")),
    };

    state.update_transport_clock(clock.clone());

    assert_eq!(state.runtime.transport, clock);
    assert!(!state.session.runtime_state.transport.is_playing);
    assert_eq!(state.session.runtime_state.transport.position_beats, 48.5);
    assert_eq!(
        state
            .session
            .runtime_state
            .transport
            .current_scene
            .as_ref()
            .map(ToString::to_string),
        Some("scene-2".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .active_scene
            .as_ref()
            .map(ToString::to_string),
        Some("scene-2".into())
    );
    assert!(!state.jam_view.transport.is_playing);
    assert_eq!(state.jam_view.transport.position_beats, 48.5);
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-2")
    );
}

#[test]
fn setting_transport_playing_records_audio_transport_anchor() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.set_transport_playing(true);

    assert!(state.runtime.transport.is_playing);
    assert_eq!(
        state.runtime.transport_driver.last_audio_position_beats,
        Some(state.runtime.transport.beat_index)
    );

    state.set_transport_playing(false);

    assert!(!state.runtime.transport.is_playing);
    assert_eq!(
        state.runtime.transport_driver.last_audio_position_beats,
        None
    );
}

#[test]
fn reconstructs_bar_and_phrase_indices_from_loaded_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.transport.beat_index, 32);
    assert_eq!(state.runtime.transport.bar_index, 8);
    assert_eq!(state.runtime.transport.phrase_index, 1);
}

#[test]
fn default_tr909_render_state_stays_idle_until_lane_state_requests_support() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Idle);
    assert_eq!(
        state.runtime.tr909_render.routing,
        Tr909RenderRouting::SourceOnly
    );
    assert_eq!(state.runtime.tr909_render.pattern_ref, None);
    assert_eq!(state.runtime.tr909_render.drum_bus_level, 0.72);
    assert!(state.runtime.tr909_render.is_transport_running);
    assert_eq!(state.runtime.tr909_render.tempo_bpm, 126.0);
    assert_eq!(state.runtime.tr909_render.position_beats, 32.0);
    assert_eq!(
        state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-1")
    );
}

#[test]
fn commits_ready_actions_into_session_log_in_stable_order() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let first = state.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::CaptureNow,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        ),
        300,
    );
    let second = state.queue.enqueue(
        ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::Scene),
                ..Default::default()
            },
        ),
        301,
    );

    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Bar,
        beat_index: 64,
        bar_index: 17,
        phrase_index: 4,
        scene_id: Some(SceneId::from("scene-1")),
    };

    let committed = state.commit_ready_actions(boundary.clone(), 400);

    assert_eq!(committed.len(), 2);
    assert_eq!(committed[0].action_id, first);
    assert_eq!(committed[0].commit_sequence, 1);
    assert_eq!(committed[1].action_id, second);
    assert_eq!(committed[1].commit_sequence, 2);
    assert_eq!(state.runtime.last_commit_boundary, Some(boundary));
    assert_eq!(state.queue.pending_actions().len(), 0);
    assert_eq!(state.session.action_log.actions.len(), 3);
    assert_eq!(state.session.action_log.actions[1].id, first);
    assert_eq!(state.session.action_log.actions[2].id, second);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .iter()
            .map(|action| action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(1), first, second]
    );
    assert_eq!(state.jam_view.pending_actions.len(), 0);
    assert_eq!(state.jam_view.recent_actions[0].id, second.to_string());
    assert_eq!(state.jam_view.recent_actions[1].id, first.to_string());
}

#[test]
fn queues_first_live_safe_jam_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_scene_mutation(300);
    assert_eq!(
        state.queue_mc202_role_toggle(301),
        QueueControlResult::Enqueued
    );
    state.queue_tr909_fill(302);
    state.queue_tr909_reinforce(303);
    assert!(state.queue_tr909_slam_toggle(304));
    state.queue_capture_bar(305);
    assert!(state.queue_promote_last_capture(306));

    let pending = state.queue.pending_actions();

    assert_eq!(pending.len(), 7);
    assert_eq!(pending[0].command, ActionCommand::MutateScene);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(pending[1].command, ActionCommand::Mc202SetRole);
    assert_eq!(pending[1].quantization, Quantization::NextPhrase);
    assert_eq!(pending[2].command, ActionCommand::Tr909FillNext);
    assert_eq!(pending[2].quantization, Quantization::NextBar);
    assert_eq!(pending[3].command, ActionCommand::Tr909ReinforceBreak);
    assert_eq!(pending[3].quantization, Quantization::NextPhrase);
    assert_eq!(pending[4].command, ActionCommand::Tr909SetSlam);
    assert_eq!(pending[4].quantization, Quantization::NextBeat);
    assert_eq!(pending[5].command, ActionCommand::CaptureBarGroup);
    assert_eq!(pending[5].quantization, Quantization::NextPhrase);
    assert_eq!(pending[6].command, ActionCommand::PromoteCaptureToPad);
    assert_eq!(pending[6].quantization, Quantization::NextBar);
    assert_eq!(
        state.jam_view.lanes.mc202_pending_role.as_deref(),
        Some("leader")
    );
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert!(
        !state
            .session
            .runtime_state
            .lane_state
            .tr909
            .fill_armed_next_bar
    );
    assert!(state.jam_view.lanes.tr909_fill_armed_next_bar);
    assert_eq!(state.jam_view.pending_actions.len(), 7);
}

#[test]
fn queue_scene_select_enqueues_scene_launch_for_next_bar() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["contrast".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(
        state.jam_view.scene.next_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.jam_view.scene.next_scene_energy.as_deref(),
        Some("medium")
    );
    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
    assert_eq!(
        state.queue_scene_select(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::SceneLaunch);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(
        pending[0].target.scene_id,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        pending[0].params,
        ActionParams::Scene {
            scene_id: Some(SceneId::from("scene-02-break"))
        }
    );
}

#[test]
fn queue_scene_select_rejects_single_current_scene_without_pending_action() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.jam_view.scene.scene_jump_availability,
        SceneJumpAvailabilityView::WaitingForMoreScenes
    );
    assert_eq!(
        state.queue_scene_select(300),
        QueueControlResult::AlreadyInState
    );
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn queue_scene_select_prefers_energy_contrast_candidate() {
    let graph = scene_regression_graph(&[
        "drop".to_string(),
        "chorus".to_string(),
        "intro".to_string(),
    ]);
    let mut session = sample_session(&graph);
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-chorus"),
        SceneId::from("scene-03-intro"),
    ];
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.jam_view.scene.next_scene.as_deref(),
        Some("scene-03-intro")
    );
    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
    assert_eq!(
        state.queue.pending_actions()[0].target.scene_id,
        Some(SceneId::from("scene-03-intro"))
    );
    assert_eq!(
        state.queue.pending_actions()[0].explanation.as_deref(),
        Some("launch contrast scene scene-03-intro on next bar")
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        350,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("launched contrast scene scene-03-intro at bar 9 / phrase 2")
    );
}

#[test]
fn committed_scene_select_updates_transport_and_scene_state() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["contrast".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        350,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.session.runtime_state.transport.current_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.runtime.transport.current_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.jam_view.scene.restore_scene.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state.jam_view.scene.active_scene_energy.as_deref(),
        Some("medium")
    );
    assert_eq!(
        state.jam_view.scene.restore_scene_energy.as_deref(),
        Some("high")
    );
    assert_eq!(
        state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("launched scene scene-02-break at bar 9 / phrase 2")
    );
}


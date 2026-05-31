#[test]
fn scene_launch_repositions_source_monitor_to_target_section_and_replays() {
    let mut graph = scene_regression_graph(&["break".into(), "drop".into()]);
    graph.timing.bpm_estimate = Some(120.0);
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-break"));
    session.runtime_state.scene_state.restore_scene = None;
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-break"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;

    let base_session = session.clone();
    let source_audio_cache = source_monitor_reposition_source_cache();
    let mut committed_state =
        JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    committed_state.source_audio_cache = Some(source_audio_cache.clone());
    committed_state.runtime.transport.is_playing = true;
    committed_state.runtime.transport.position_beats = 36.0;
    committed_state.refresh_view();
    assert_eq!(
        committed_state
            .source_monitor_render_state()
            .source_anchor_seconds,
        None
    );

    assert_eq!(
        committed_state.queue_scene_select(300),
        QueueControlResult::Enqueued
    );
    let committed = committed_state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-break")),
        },
        360,
    );
    assert_eq!(committed.len(), 1);
    committed_state.runtime.transport.is_playing = true;
    committed_state.runtime.transport.position_beats = 36.0;
    committed_state.refresh_view();

    let anchored_render = committed_state.source_monitor_render_state();
    assert_eq!(anchored_render.source_anchor_seconds, Some(16.0));
    assert_eq!(anchored_render.source_anchor_position_beats, 36.0);
    assert_eq!(
        committed_state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-02-drop"))
    );
    assert_eq!(
        committed_state.runtime_view.source_monitor_audio_route,
        "source_only"
    );

    let mut transport_only_render = anchored_render.clone();
    transport_only_render.source_anchor_seconds = None;
    transport_only_render.source_anchor_position_beats = 0.0;
    let generated = vec![0.0; usize::from(source_monitor_reposition_channel_count()) * 400];
    let transport_only_output = riotbox_audio::runtime::render_source_monitor_mix_offline(
        &generated,
        source_monitor_reposition_sample_rate(),
        source_monitor_reposition_channel_count(),
        &transport_only_render,
    );
    let anchored_output = riotbox_audio::runtime::render_source_monitor_mix_offline(
        &generated,
        source_monitor_reposition_sample_rate(),
        source_monitor_reposition_channel_count(),
        &anchored_render,
    );
    assert_recipe_buffers_differ(
        "scene launch source monitor anchor -> transport-only source playback",
        &transport_only_output,
        &anchored_output,
        0.05,
    );

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("source monitor scene reposition action log builds replay plan");
    let mut replayed_session = base_session;
    replayed_session.action_log = committed_state.session.action_log.clone();
    riotbox_core::replay::apply_graph_aware_replay_plan_to_session(
        &mut replayed_session,
        &plan,
        &graph,
    )
    .expect("graph-aware replay restores scene movement for source reposition");
    let mut replayed_state =
        JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    replayed_state.runtime.transport.is_playing = true;
    replayed_state.runtime.transport.position_beats = 36.0;
    replayed_state.refresh_view();
    let replayed_render = replayed_state.source_monitor_render_state();
    let replayed_output = riotbox_audio::runtime::render_source_monitor_mix_offline(
        &generated,
        source_monitor_reposition_sample_rate(),
        source_monitor_reposition_channel_count(),
        &replayed_render,
    );

    assert_eq!(
        replayed_state.session.runtime_state.scene_state,
        committed_state.session.runtime_state.scene_state
    );
    assert_eq!(replayed_render.source_anchor_seconds, Some(16.0));
    assert_eq!(replayed_render.source_anchor_position_beats, 36.0);
    assert_recipe_buffers_match(
        "replayed scene source monitor reposition -> committed source monitor reposition",
        &replayed_output,
        &anchored_output,
        0.00001,
    );
}

#[test]
fn source_monitor_scene_reposition_waits_for_trusted_source_timing() {
    struct SceneRepositionTrustCase {
        name: &'static str,
        policy: TimingDegradedPolicy,
        bpm: Option<f32>,
        confirmed_grid: bool,
        expected_anchor_seconds: Option<f64>,
    }

    let cases = [
        SceneRepositionTrustCase {
            name: "locked analyzer timing",
            policy: TimingDegradedPolicy::Locked,
            bpm: Some(120.0),
            confirmed_grid: false,
            expected_anchor_seconds: Some(16.0),
        },
        SceneRepositionTrustCase {
            name: "manual confirm without user trust",
            policy: TimingDegradedPolicy::ManualConfirm,
            bpm: Some(120.0),
            confirmed_grid: false,
            expected_anchor_seconds: None,
        },
        SceneRepositionTrustCase {
            name: "manual confirm with user trust",
            policy: TimingDegradedPolicy::ManualConfirm,
            bpm: Some(120.0),
            confirmed_grid: true,
            expected_anchor_seconds: Some(16.0),
        },
        SceneRepositionTrustCase {
            name: "fallback grid",
            policy: TimingDegradedPolicy::FallbackGrid,
            bpm: Some(120.0),
            confirmed_grid: false,
            expected_anchor_seconds: None,
        },
        SceneRepositionTrustCase {
            name: "disabled timing",
            policy: TimingDegradedPolicy::Disabled,
            bpm: Some(120.0),
            confirmed_grid: false,
            expected_anchor_seconds: None,
        },
        SceneRepositionTrustCase {
            name: "missing bpm",
            policy: TimingDegradedPolicy::Locked,
            bpm: None,
            confirmed_grid: false,
            expected_anchor_seconds: None,
        },
    ];

    for case in cases {
        let mut graph = scene_regression_graph(&["break".into(), "drop".into()]);
        graph.timing.bpm_estimate = case.bpm;
        graph.timing.primary_hypothesis_id = Some("scene-grid".into());
        graph.timing.degraded_policy = case.policy;
        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-break"));
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-break"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
        if case.confirmed_grid {
            session.runtime_state.source_timing.confirmed_grid =
                Some(SourceTimingGridConfirmationState {
                    source_id: graph.source.source_id.clone(),
                    hypothesis_id: Some("scene-grid".into()),
                    confirmed_by_action: ActionId(7),
                    confirmed_at: 1_777_777,
                });
        }

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_monitor_reposition_source_cache());
        assert_eq!(
            state.queue_scene_select(300),
            QueueControlResult::Enqueued,
            "{} queue",
            case.name
        );
        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-01-break")),
            },
            360,
        );
        assert_eq!(committed.len(), 1, "{} committed action count", case.name);
        state.runtime.transport.is_playing = true;
        state.runtime.transport.position_beats = 36.0;
        state.refresh_view();

        let render = state.source_monitor_render_state();
        assert_eq!(
            render.source_anchor_seconds, case.expected_anchor_seconds,
            "{} source anchor",
            case.name
        );
        if case.expected_anchor_seconds.is_some() {
            assert_eq!(render.source_anchor_position_beats, 36.0, "{} anchor beat", case.name);
        }
    }
}

fn source_monitor_reposition_source_cache() -> SourceAudioCache {
    let sample_rate = source_monitor_reposition_sample_rate();
    let channel_count = source_monitor_reposition_channel_count();
    let frame_count = sample_rate as usize * 48;
    let mut samples = Vec::with_capacity(frame_count * usize::from(channel_count));
    for frame in 0..frame_count {
        let value = ((frame as f32 * 0.017).sin() * 0.72).clamp(-0.9, 0.9);
        samples.push(value);
        samples.push(-value);
    }
    SourceAudioCache::from_interleaved_samples(
        "scene-source.wav",
        sample_rate,
        channel_count,
        samples,
    )
    .expect("source monitor reposition source cache")
}

const fn source_monitor_reposition_sample_rate() -> u32 {
    100
}

const fn source_monitor_reposition_channel_count() -> u16 {
    2
}

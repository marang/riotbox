#[test]
fn committed_mc202_instigator_generation_updates_phrase_ref_touch_and_render_shape() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_instigator(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("instigator")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("instigator-scene-1")
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.90);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Instigator);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::InstigatorSpike
    );
    assert_eq!(
        state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Push
    );
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert_eq!(
        state.jam_view.lanes.mc202_role.as_deref(),
        Some("instigator")
    );
    assert!(!state.jam_view.lanes.mc202_pending_instigator_generation);
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("instigator-scene-1")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("generated MC-202 instigator phrase instigator-scene-1 at 0.90")
    );
}

#[test]
fn committed_mc202_phrase_mutation_updates_variant_and_render_shape() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    session.runtime_state.macro_state.mc202_touch = 0.78;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_mutate_phrase(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("follower")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("follower-mutated_drive-bar-10")
    );
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.phrase_variant,
        Some(Mc202PhraseVariantState::MutatedDrive)
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.88);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::MutatedDrive
    );
    assert_eq!(
        state.runtime_view.mc202_render_phrase_shape,
        "mutated_drive"
    );
    assert!(!state.jam_view.lanes.mc202_pending_phrase_mutation);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("mutated MC-202 phrase follower-mutated_drive-bar-10 as mutated_drive")
    );
}

#[test]
fn mc202_render_projection_consumes_typed_role_and_phrase_intent_contract() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Pressure.label().into());
    session.runtime_state.lane_state.mc202.phrase_variant =
        Mc202PhraseIntentState::MutatedDrive.phrase_variant();
    session.runtime_state.macro_state.mc202_touch = Mc202RoleState::Pressure.default_touch();

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Pressure);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::MutatedDrive
    );
    assert_eq!(
        state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Wide
    );
    let rendered = render_mc202_recipe_buffer(&state.runtime.mc202_render);
    let metrics = signal_metrics(&rendered);
    assert!(metrics.rms > 0.001, "typed MC-202 projection rendered silent");
}

#[test]
fn mc202_render_projection_rejects_unknown_role_label_without_rendering() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some("future_role".into());
    session.runtime_state.lane_state.mc202.phrase_variant =
        Mc202PhraseIntentState::MutatedDrive.phrase_variant();
    session.runtime_state.macro_state.mc202_touch = 0.90;

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    let mut rendered = vec![0.0; 44_100 * 2];
    render_mc202_buffer(&mut rendered, 44_100, 2, &state.runtime.mc202_render);
    let metrics = signal_metrics(&rendered);
    assert_eq!(metrics.rms, 0.0, "unknown MC-202 role should not render");
}

#[test]
fn mc202_recipe_replay_proves_control_and_audio_path() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 1, 400);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
    let follower = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    assert_eq!(
        state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 2, 600);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
    let answer = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    assert_eq!(
        state.queue_mc202_generate_pressure(700),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 3, 800);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Pressure);
    let pressure = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    assert_eq!(
        state.queue_mc202_generate_instigator(900),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 4, 1_000);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Instigator);
    let instigator = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    assert_eq!(
        state.queue_mc202_mutate_phrase(1_100),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 5, 1_200);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::MutatedDrive
    );
    let mutated = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    let touch_before = state.runtime.mc202_render.touch;
    state.adjust_mc202_touch(-0.24);
    assert!(state.runtime.mc202_render.touch < touch_before);
    let lower_touch = render_mc202_recipe_buffer(&state.runtime.mc202_render);

    assert!(state.session.action_log.actions.len() >= 5);
    assert_recipe_buffers_differ("follower -> answer", &follower, &answer, 0.005);
    assert_recipe_buffers_differ("answer -> pressure", &answer, &pressure, 0.004);
    assert_recipe_buffers_differ("pressure -> instigator", &pressure, &instigator, 0.004);
    assert_recipe_buffers_differ("instigator -> mutation", &instigator, &mutated, 0.004);
    assert_recipe_buffers_differ("mutation -> lower touch", &mutated, &lower_touch, 0.001);
}

#[test]
fn mc202_replay_executor_matches_committed_app_state_and_audio_path() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        committed_state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 1, 400);
    let follower = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 2, 600);
    let committed_answer = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_pressure(700),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 3, 800);
    let pressure = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_instigator(900),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 4, 1_000);
    let instigator = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_mutate_phrase(1_100),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 5, 1_200);
    let committed_mutation = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("committed MC-202 action log builds a replay plan");
    let mut replayed_session = base_session;
    let report = riotbox_core::replay::apply_replay_plan_to_session(&mut replayed_session, &plan)
        .expect("MC-202 replay executor applies committed phrase family");
    let replayed_state = JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    let replayed_mutation = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

    assert_eq!(report.applied_action_ids.len(), 5);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.mc202,
        committed_state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.mc202_touch,
        committed_state.session.runtime_state.macro_state.mc202_touch
    );
    assert_eq!(replayed_state.runtime.mc202_render, committed_state.runtime.mc202_render);
    assert_recipe_buffers_match(
        "replayed MC-202 mutation -> committed MC-202 mutation",
        &replayed_mutation,
        &committed_mutation,
        0.00001,
    );
    assert_recipe_buffers_differ("follower -> committed answer", &follower, &committed_answer, 0.005);
    assert_recipe_buffers_differ("answer -> pressure", &committed_answer, &pressure, 0.004);
    assert_recipe_buffers_differ("pressure -> instigator", &pressure, &instigator, 0.004);
    assert_recipe_buffers_differ(
        "instigator -> replayed mutation",
        &instigator,
        &replayed_mutation,
        0.004,
    );
}

#[test]
fn undo_mc202_phrase_move_restores_lane_state_and_audio_path() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 1, 400);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
    let follower_render = state.runtime.mc202_render;
    let follower = render_mc202_recipe_buffer(&follower_render);

    assert_eq!(
        state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 2, 600);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
    let answer = render_mc202_recipe_buffer(&state.runtime.mc202_render);
    assert_recipe_buffers_differ("follower -> answer", &follower, &answer, 0.005);

    let undo = state
        .undo_last_action(700)
        .expect("undo latest committed MC-202 phrase action");
    assert_eq!(undo.command, ActionCommand::UndoLast);
    let undone_answer = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::Mc202GenerateAnswer)
        .expect("answer action remains in log");
    assert_eq!(undone_answer.status, ActionStatus::Undone);
    assert_eq!(state.runtime.mc202_render, follower_render);
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("follower-scene-1")
    );

    let restored = render_mc202_recipe_buffer(&state.runtime.mc202_render);
    assert_recipe_buffers_match("undo -> previous follower", &follower, &restored, 0.00001);
    assert_recipe_buffers_differ("answer -> undo", &answer, &restored, 0.005);
}

#[test]
fn undo_mc202_phrase_move_without_snapshot_does_not_claim_success() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut state, 1, 400);
    state
        .session
        .runtime_state
        .undo_state
        .mc202_snapshots
        .clear();

    assert_eq!(state.undo_last_action(500), None);
    let follower_action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::Mc202GenerateFollower)
        .expect("follower action remains in log");
    assert_eq!(follower_action.status, ActionStatus::Committed);
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
}

fn commit_mc202_recipe_step(state: &mut JamAppState, phrase_index: u64, committed_at: u64) {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: phrase_index * 16,
            bar_index: phrase_index * 4,
            phrase_index,
            scene_id: Some(SceneId::from("scene-1")),
        },
        committed_at,
    );
    assert_eq!(committed.len(), 1);
}

fn render_mc202_recipe_buffer(render_state: &Mc202RenderState) -> Vec<f32> {
    let mut buffer = vec![0.0; 44_100 * 2];
    render_mc202_buffer(&mut buffer, 44_100, 2, render_state);
    assert!(
        buffer.iter().any(|sample| sample.abs() > 0.0001),
        "MC-202 recipe step rendered silence"
    );
    buffer
}

fn render_scene_recipe_mix_buffer(state: &JamAppState) -> Vec<f32> {
    let frame_count = 44_100;
    let mut tr909 = render_tr909_offline(&state.runtime.tr909_render, 44_100, 2, frame_count);
    let mc202 = render_mc202_offline(&state.runtime.mc202_render, 44_100, 2, frame_count);

    for (left, right) in tr909.iter_mut().zip(mc202.iter()) {
        *left += *right;
    }

    let metrics = signal_metrics(&tr909);
    assert!(
        metrics.rms > 0.001,
        "Scene Brain mixed render RMS too low: {}",
        metrics.rms
    );
    tr909
}

fn assert_recipe_buffers_match(label: &str, left: &[f32], right: &[f32], max_delta: f32) {
    let delta = signal_delta_metrics(left, right);

    assert!(
        delta.rms <= max_delta,
        "{label} signal delta RMS {} above {max_delta}; peak {}, active {}, zero crossings {}",
        delta.rms,
        delta.peak_abs,
        delta.active_samples,
        delta.zero_crossings
    );
}

fn assert_recipe_buffers_differ(label: &str, left: &[f32], right: &[f32], min_delta: f32) {
    let delta = signal_delta_metrics(left, right);

    assert!(
        delta.rms >= min_delta,
        "{label} signal delta RMS {} below {min_delta}; peak {}, active {}, zero crossings {}",
        delta.rms,
        delta.peak_abs,
        delta.active_samples,
        delta.zero_crossings
    );
    assert!(
        delta.peak_abs > 0.001,
        "{label} signal delta peak too low: {}",
        delta.peak_abs
    );
}

fn recipe_signal_delta_rms(left: &[f32], right: &[f32]) -> f32 {
    signal_delta_metrics(left, right).rms
}

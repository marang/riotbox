use std::{error::Error, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    mc202::Mc202RenderRouting,
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, SourceMonitorAudioRoute,
        SourceMonitorRenderState,
    },
    tr909::Tr909RenderMode,
};
use riotbox_core::{
    action::{ActionCommand, CaptureLengthIntent, CommitBoundary, SourceMonitorMode, TargetScope},
    ids::SceneId,
    live_performance_policy::derive_live_performance_policy,
    queue::CommittedActionRef,
    source_graph::{primary_grid_anchor_seconds_for_projected_scene, section_for_projected_scene},
    transport::{
        CommitBoundaryState, DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState,
        TransportGridPosition,
    },
};

use crate::model::{
    ConfirmedSourceTiming, GestureTransition, MonitorProof, PreparedLivePath, RenderStage,
    SceneTransitionProof,
};

pub fn prepare(
    source_path: &Path,
    output_dir: &Path,
    cli_bpm_hint: f32,
) -> Result<PreparedLivePath, Box<dyn Error>> {
    let mut state = JamAppState::analyze_source_file_to_json_with_source_bpm_confirmation(
        source_path,
        output_dir.join("session.json"),
        Some(output_dir.join("source-graph.json")),
        "python/sidecar/json_stdio_sidecar.py",
        23,
        Some(cli_bpm_hint),
    )?;
    let source_timing = confirmed_source_timing(&state, cli_bpm_hint)?;
    let bpm = source_timing.bpm;
    let capture_cursor = source_timing
        .bar_start_beat_cursor(1)
        .ok_or("dense-break source timing cannot resolve bar 1")?;
    let promote_cursor = source_timing
        .bar_start_beat_cursor(2)
        .ok_or("dense-break source timing cannot resolve bar 2")?;
    let phrase_cursor = source_timing
        .bar_start_beat_cursor(5)
        .ok_or("dense-break source timing cannot resolve phrase-2 bar 5")?;
    let w_cursor = phrase_cursor.saturating_add(1);
    let fill_cursor = source_timing
        .bar_start_beat_cursor(6)
        .ok_or("dense-break source timing cannot resolve Fill bar 6")?;
    let slam_cursor = source_timing
        .bar_start_beat_cursor(7)
        .ok_or("dense-break source timing cannot resolve Slam bar 7")?;
    let scene_cursor = source_timing
        .bar_start_beat_cursor(8)
        .ok_or("dense-break source timing cannot resolve scene bar 8")?;
    let restore_cursor = source_timing
        .bar_start_beat_cursor(9)
        .ok_or("dense-break source timing cannot resolve restore bar 9")?;
    state.set_transport_playing(true);
    let initial_scene = current_scene(&state);

    if state.session.runtime_state.source_monitor.mode != SourceMonitorMode::Source {
        return Err("dense-break live path must begin in Source monitor mode".into());
    }
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 90);
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        initial_scene.clone(),
        95,
        1,
    )?;
    state.queue_capture_bar(100);
    commit(
        &mut state,
        CommitBoundary::Bar,
        capture_cursor,
        initial_scene.clone(),
        200,
        1,
    )?;
    if !state.queue_promote_last_capture(210) {
        return Err("capture promotion was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Bar,
        promote_cursor,
        initial_scene.clone(),
        300,
        1,
    )?;
    state.queue_tr909_reinforce(310);
    if state.queue_mc202_generate_pressure(311) != QueueControlResult::Enqueued {
        return Err("MC-202 pressure generation was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Phrase,
        phrase_cursor,
        initial_scene,
        400,
        2,
    )?;

    let live_policy = state
        .source_graph
        .as_ref()
        .and_then(|graph| derive_live_performance_policy(&state.session, graph))
        .ok_or("dense-break live performance policy was unavailable")?;
    println!(
        "live performance policy: lead={} bass_owner={} mc202_intent={}",
        live_policy.lead.label(),
        live_policy.bass_owner.label(),
        live_policy.mc202_intent.label()
    );

    let source_plan = render_plan(&state, bpm, phrase_cursor as f64);
    let to_blend = set_monitor_mode(&mut state, SourceMonitorMode::Blend, phrase_cursor, 410)?;
    let blend_plan = render_plan(&state, bpm, phrase_cursor as f64);
    let to_riotbox = set_monitor_mode(&mut state, SourceMonitorMode::Riotbox, phrase_cursor, 420)?;
    let riotbox_plan = render_plan(&state, bpm, phrase_cursor as f64);
    let back_to_source =
        set_monitor_mode(&mut state, SourceMonitorMode::Source, phrase_cursor, 430)?;
    let back_to_blend = set_monitor_mode(&mut state, SourceMonitorMode::Blend, phrase_cursor, 440)?;

    let monitor_proofs = vec![
        MonitorProof {
            case_id: "monitor-source",
            artifact_path: "monitor/00_source.wav",
            expected_route: SourceMonitorAudioRoute::SourceOnly,
            action_id: None,
            plan: source_plan,
        },
        MonitorProof {
            case_id: "monitor-blend",
            artifact_path: "monitor/01_blend.wav",
            expected_route: SourceMonitorAudioRoute::Blend,
            action_id: Some(to_blend.action_id.0),
            plan: blend_plan,
        },
        MonitorProof {
            case_id: "monitor-riotbox",
            artifact_path: "monitor/02_riotbox.wav",
            expected_route: SourceMonitorAudioRoute::RiotboxOnly,
            action_id: Some(to_riotbox.action_id.0),
            plan: riotbox_plan,
        },
    ];

    let ready_plan = render_plan(&state, bpm, phrase_cursor as f64);
    let mut stages = vec![stage(
        "ready-blend",
        "gestures/00_ready_blend.wav",
        1,
        None,
        None,
        None,
        None,
        &state,
        ready_plan.clone(),
    )];
    let mut prefix = vec![(ready_plan, 1)];
    let mut transitions = Vec::new();

    let before_w = render_plan(&state, bpm, w_cursor as f64);
    if state.queue_w30_trigger_pad(500) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 hit was unavailable".into());
    }
    let scene = current_scene(&state);
    let w_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        w_cursor,
        scene,
        510,
        1,
    )?)?;
    require_committed_command(&state, &w_commit, ActionCommand::W30TriggerPad)?;
    let after_w = render_plan(&state, bpm, w_cursor as f64);
    let (normal_plan, damaged_plan) = prepare_legacy_pressure_regression(&state, bpm, fill_cursor)?;
    transitions.push(gesture_transition(
        "w-hit",
        "w",
        ActionCommand::W30TriggerPad,
        CommitBoundary::Beat,
        &w_commit,
        &prefix,
        before_w,
        after_w.clone(),
    ));
    stages.push(stage(
        "after-w-hit",
        "gestures/01_after_w_hit.wav",
        3,
        Some("w"),
        Some(ActionCommand::W30TriggerPad),
        Some(CommitBoundary::Beat),
        Some(w_commit.action_id.0),
        &state,
        after_w.clone(),
    ));
    prefix.push((after_w, 3));

    update_transport_position(&mut state, fill_cursor);
    let before_f = render_plan(&state, bpm, fill_cursor as f64);
    state.queue_tr909_fill(600);
    let scene = current_scene(&state);
    let f_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        fill_cursor,
        scene,
        610,
        1,
    )?)?;
    require_committed_command(&state, &f_commit, ActionCommand::Tr909FillNext)?;
    let after_f = render_plan(&state, bpm, fill_cursor as f64);
    if !matches!(after_f.tr909_render.mode, Tr909RenderMode::Fill) {
        return Err(format!(
            "f did not own its committed bar: got {:?}",
            after_f.tr909_render.mode
        )
        .into());
    }
    transitions.push(gesture_transition(
        "f-fill",
        "f",
        ActionCommand::Tr909FillNext,
        CommitBoundary::Bar,
        &f_commit,
        &prefix,
        before_f,
        after_f.clone(),
    ));
    stages.push(stage(
        "after-f-fill",
        "gestures/02_after_f_fill.wav",
        4,
        Some("f"),
        Some(ActionCommand::Tr909FillNext),
        Some(CommitBoundary::Bar),
        Some(f_commit.action_id.0),
        &state,
        after_f.clone(),
    ));
    prefix.push((after_f, 4));

    update_transport_position(&mut state, slam_cursor);
    let before_s = render_plan(&state, bpm, slam_cursor as f64);
    if !matches!(before_s.tr909_render.mode, Tr909RenderMode::BreakReinforce) {
        return Err(format!(
            "f did not return to break reinforcement before s: got {:?}",
            before_s.tr909_render.mode
        )
        .into());
    }
    if !state.queue_tr909_slam_toggle(700) {
        return Err("TR-909 slam was already pending".into());
    }
    let scene = current_scene(&state);
    let s_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        slam_cursor,
        scene,
        710,
        1,
    )?)?;
    require_committed_command(&state, &s_commit, ActionCommand::Tr909SetSlam)?;
    let after_s = render_plan(&state, bpm, slam_cursor as f64);
    if !matches!(after_s.tr909_render.mode, Tr909RenderMode::BreakReinforce)
        || !after_s.tr909_render.slam_enabled
    {
        return Err(format!(
            "s did not retain break reinforcement with explicit slam: mode={:?} enabled={}",
            after_s.tr909_render.mode, after_s.tr909_render.slam_enabled
        )
        .into());
    }
    transitions.push(gesture_transition(
        "s-slam",
        "s",
        ActionCommand::Tr909SetSlam,
        CommitBoundary::Beat,
        &s_commit,
        &prefix,
        before_s,
        after_s.clone(),
    ));
    stages.push(stage(
        "after-s-slam",
        "gestures/03_after_s_slam.wav",
        4,
        Some("s"),
        Some(ActionCommand::Tr909SetSlam),
        Some(CommitBoundary::Beat),
        Some(s_commit.action_id.0),
        &state,
        after_s.clone(),
    ));
    prefix.push((after_s, 4));

    update_transport_position(&mut state, scene_cursor);
    let before_y = render_plan(&state, bpm, scene_cursor as f64);
    let pre_y_scene = current_scene(&state).ok_or("scene jump had no source scene")?;
    let pre_y_control_state = state.clone();
    let pre_y_audio_projection = state
        .session
        .runtime_state
        .scene_state
        .active_projection_movement
        .clone();
    let pre_y_render_anchor = before_y.source_monitor_render.source_anchor_seconds;
    let expected_restore_anchor = scene_source_anchor_seconds(&state, &pre_y_scene);
    if state.queue_scene_select(800) != QueueControlResult::Enqueued {
        return Err("scene jump was unavailable".into());
    }
    let scene = current_scene(&state);
    let y_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        scene_cursor,
        scene,
        810,
        1,
    )?)?;
    require_committed_command(&state, &y_commit, ActionCommand::SceneLaunch)?;
    let after_y = render_plan(&state, bpm, scene_cursor as f64);
    let after_y_anchor = after_y.source_monitor_render.source_anchor_seconds;
    let after_y_scene = current_scene(&state).ok_or("scene jump cleared the active scene")?;
    if after_y_scene == pre_y_scene {
        return Err(format!("scene jump remained on {pre_y_scene}").into());
    }
    require_committed_scene_target(&state, &y_commit, &after_y_scene)?;
    let expected_y_anchor = scene_source_anchor_seconds(&state, &after_y_scene);
    require_same_anchor("scene jump", after_y_anchor, expected_y_anchor)?;
    let mc202_plan_source_section = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .and_then(|plan| plan.source_section_id.clone());
    let launched_source_section = state
        .source_graph
        .as_ref()
        .and_then(|graph| section_for_projected_scene(graph, &after_y_scene))
        .map(|section| section.section_id.clone());
    let launch_has_explicit_mc202_section_mismatch = matches!(
        (&mc202_plan_source_section, &launched_source_section),
        (Some(plan_section), Some(target_section)) if plan_section != target_section
    );
    if !launch_has_explicit_mc202_section_mismatch
        || after_y.mc202_render.routing != Mc202RenderRouting::Silent
    {
        return Err(format!(
            "scene jump must keep mismatched MC-202 source plan out: plan_section={mc202_plan_source_section:?} target_section={launched_source_section:?} routing={:?}",
            after_y.mc202_render.routing
        )
        .into());
    }
    transitions.push(gesture_transition(
        "y-scene-jump",
        "y",
        ActionCommand::SceneLaunch,
        CommitBoundary::Bar,
        &y_commit,
        &prefix,
        before_y,
        after_y.clone(),
    ));
    stages.push(stage(
        "after-y-scene-jump",
        "gestures/04_after_y_scene_jump.wav",
        4,
        Some("y"),
        Some(ActionCommand::SceneLaunch),
        Some(CommitBoundary::Bar),
        Some(y_commit.action_id.0),
        &state,
        after_y.clone(),
    ));
    prefix.push((after_y, 4));

    update_transport_position(&mut state, restore_cursor);
    let before_restore = render_plan(&state, bpm, restore_cursor as f64);
    let before_restore_scene = current_scene(&state).ok_or("scene restore had no source scene")?;
    if before_restore_scene != after_y_scene {
        return Err(format!(
            "scene changed between jump and restore: {after_y_scene} -> {before_restore_scene}"
        )
        .into());
    }
    require_same_anchor(
        "pre-restore",
        before_restore.source_monitor_render.source_anchor_seconds,
        after_y_anchor,
    )?;
    if state.queue_scene_restore(900) != QueueControlResult::Enqueued {
        return Err("scene restore was unavailable after the landed jump".into());
    }
    let scene = current_scene(&state);
    let restore_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        restore_cursor,
        scene,
        910,
        1,
    )?)?;
    require_committed_command(&state, &restore_commit, ActionCommand::SceneRestore)?;
    let after_restore = render_plan(&state, bpm, restore_cursor as f64);
    let restored_scene = current_scene(&state).ok_or("scene restore cleared the active scene")?;
    require_committed_scene_target(&state, &restore_commit, &pre_y_scene)?;
    if restored_scene != pre_y_scene {
        return Err(
            format!("scene restore landed on {restored_scene}, expected {pre_y_scene}").into(),
        );
    }
    require_same_anchor(
        "scene restore",
        after_restore.source_monitor_render.source_anchor_seconds,
        expected_restore_anchor,
    )?;
    let restore_audio_projection_matches_pre_jump = state
        .session
        .runtime_state
        .scene_state
        .active_projection_movement
        == pre_y_audio_projection;
    // Restore keeps the pre-jump Scene truth, but global transport does not rewind.
    // Advance a no-jump clone to the restore cursor so phrase-dependent policies
    // (notably TR-909 variation) are compared at the same musical instant.
    let expected_restored_scene_plan =
        counterfactual_plan_at_position(&pre_y_control_state, bpm, restore_cursor);
    let restore_lane_projection_matches_pre_jump =
        lane_audio_projection_matches(&expected_restored_scene_plan, &after_restore);
    if !restore_audio_projection_matches_pre_jump || !restore_lane_projection_matches_pre_jump {
        return Err(format!(
            "scene restore did not recover pre-jump projection at the current transport cursor: session_projection={} lane_projection={}",
            restore_audio_projection_matches_pre_jump, restore_lane_projection_matches_pre_jump,
        )
        .into());
    }
    let scene_transition_proof = SceneTransitionProof {
        launch_action_id: y_commit.action_id.0,
        restore_action_id: restore_commit.action_id.0,
        pre_jump_scene: pre_y_scene.to_string(),
        launched_scene: after_y_scene.to_string(),
        restored_scene: restored_scene.to_string(),
        pre_jump_render_anchor_seconds: pre_y_render_anchor,
        expected_launch_anchor_seconds: expected_y_anchor,
        expected_restore_anchor_seconds: expected_restore_anchor,
        launched_anchor_seconds: after_y_anchor,
        restored_anchor_seconds: after_restore.source_monitor_render.source_anchor_seconds,
        mc202_plan_source_section: mc202_plan_source_section.map(|section| section.to_string()),
        launched_source_section: launched_source_section.map(|section| section.to_string()),
        launch_mc202_stayed_out_for_section_mismatch: true,
        restore_audio_projection_matches_pre_jump,
        restore_lane_projection_matches_pre_jump,
    };
    transitions.push(gesture_transition(
        "Y-scene-restore",
        "Y",
        ActionCommand::SceneRestore,
        CommitBoundary::Bar,
        &restore_commit,
        &prefix,
        before_restore,
        after_restore.clone(),
    ));
    stages.push(stage(
        "after-Y-scene-restore",
        "gestures/05_after_Y_scene_restore.wav",
        4,
        Some("Y"),
        Some(ActionCommand::SceneRestore),
        Some(CommitBoundary::Bar),
        Some(restore_commit.action_id.0),
        &state,
        after_restore,
    ));

    let to_legacy_riotbox = set_monitor_mode(&mut state, SourceMonitorMode::Riotbox, 32, 920)?;

    Ok(PreparedLivePath {
        state,
        source_timing,
        live_policy,
        monitor_proofs,
        stages,
        transitions,
        scene_transition_proof,
        monitor_action_ids: [
            to_blend.action_id.0,
            to_riotbox.action_id.0,
            back_to_source.action_id.0,
            back_to_blend.action_id.0,
        ],
        legacy_riotbox_action_id: to_legacy_riotbox.action_id.0,
        normal_plan,
        damaged_plan,
    })
}

fn confirmed_source_timing(
    state: &JamAppState,
    cli_bpm_hint: f32,
) -> Result<ConfirmedSourceTiming, Box<dyn Error>> {
    let graph = state
        .source_graph
        .as_ref()
        .ok_or("dense-break source graph missing after ingest")?;
    let hypothesis_id = graph
        .timing
        .primary_hypothesis_id
        .as_deref()
        .ok_or("dense-break source timing has no primary hypothesis")?;
    let hypothesis = graph
        .timing
        .primary_hypothesis()
        .ok_or("dense-break primary source timing hypothesis is unresolved")?;
    let confirmation = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .ok_or("dense-break source timing grid was not confirmed during ingest")?;

    if confirmation.source_id != graph.source.source_id
        || confirmation.hypothesis_id.as_deref() != Some(hypothesis_id)
    {
        return Err(format!(
            "dense-break confirmed timing identity {:?}/{:?} does not match primary {}/{}",
            confirmation.source_id,
            confirmation.hypothesis_id,
            graph.source.source_id,
            hypothesis_id,
        )
        .into());
    }
    if !hypothesis.bpm.is_finite() || hypothesis.bpm <= 0.0 {
        return Err(format!(
            "dense-break confirmed primary hypothesis has invalid BPM {}",
            hypothesis.bpm
        )
        .into());
    }
    let bar_anchor = hypothesis
        .transport_bar_grid_anchor()
        .ok_or("dense-break primary timing has no consistent bar/beat phase anchor")?;
    let first_bar = hypothesis
        .bar_grid
        .iter()
        .min_by_key(|bar| bar.bar_index)
        .ok_or("dense-break primary timing has no bar grid")?;
    let first_downbeat = hypothesis
        .bar_start_beat_point(first_bar.bar_index)
        .ok_or("dense-break primary bar start is not backed by a beat point")?;

    Ok(ConfirmedSourceTiming {
        source_id: graph.source.source_id.clone(),
        hypothesis_id: hypothesis_id.to_owned(),
        cli_bpm_hint,
        bpm: hypothesis.bpm,
        beats_per_bar: u64::from(hypothesis.meter.beats_per_bar).max(1),
        primary_bar_anchor_beat_index: first_downbeat.beat_index,
        primary_bar_anchor_beat_cursor: bar_anchor.beat_cursor,
        primary_bar_anchor_bar_index: bar_anchor.bar_index,
    })
}

fn prepare_legacy_pressure_regression(
    state: &JamAppState,
    bpm: f32,
    damage_commit_cursor: u64,
) -> Result<(RuntimeMixRenderPlan, RuntimeMixRenderPlan), Box<dyn Error>> {
    if !matches!(
        state.runtime.tr909_render.mode,
        Tr909RenderMode::BreakReinforce
    ) {
        return Err(format!(
            "legacy pressure regression requires TR-909 break reinforcement before f/s, got {:?}",
            state.runtime.tr909_render.mode
        )
        .into());
    }

    let mut legacy_state = state.clone();
    let normal = legacy_riotbox_plan(&legacy_state, bpm);
    if legacy_state.queue_w30_apply_damage_profile(1_000).is_none() {
        return Err("legacy W-30 damage gesture was unavailable".into());
    }
    let scene = current_scene(&legacy_state);
    let damage_commit = one_commit(commit(
        &mut legacy_state,
        CommitBoundary::Bar,
        damage_commit_cursor,
        scene,
        1_010,
        1,
    )?)?;
    require_committed_command(
        &legacy_state,
        &damage_commit,
        ActionCommand::W30ApplyDamageProfile,
    )?;
    let damaged = legacy_riotbox_plan(&legacy_state, bpm);
    Ok((normal, damaged))
}

fn legacy_riotbox_plan(state: &JamAppState, bpm: f32) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
        ..render_plan(state, bpm, 0.0)
    }
}

fn set_monitor_mode(
    state: &mut JamAppState,
    mode: SourceMonitorMode,
    position_beats: u64,
    timestamp: u64,
) -> Result<CommittedActionRef, Box<dyn Error>> {
    if state.queue_source_monitor_mode(mode, timestamp) != QueueControlResult::Enqueued {
        return Err(format!("monitor {mode} was not queueable").into());
    }
    let committed = one_commit(commit(
        state,
        CommitBoundary::Immediate,
        position_beats,
        current_scene(state),
        timestamp + 1,
        1,
    )?)?;
    require_committed_command(state, &committed, ActionCommand::SourceMonitorSetMode)?;
    Ok(committed)
}

#[allow(clippy::too_many_arguments)]
fn stage(
    case_id: &'static str,
    artifact_path: &'static str,
    duration_beats: u32,
    key: Option<&'static str>,
    command: Option<ActionCommand>,
    boundary: Option<CommitBoundary>,
    action_id: Option<u64>,
    state: &JamAppState,
    plan: RuntimeMixRenderPlan,
) -> RenderStage {
    RenderStage {
        case_id,
        artifact_path,
        duration_beats,
        key,
        command,
        boundary,
        action_id,
        scene_id: current_scene(state)
            .map(|scene| scene.to_string())
            .unwrap_or_else(|| "none".into()),
        source_anchor_seconds: plan.source_monitor_render.source_anchor_seconds,
        plan,
    }
}

#[allow(clippy::too_many_arguments)]
fn gesture_transition(
    case_id: &'static str,
    key: &'static str,
    command: ActionCommand,
    boundary: CommitBoundary,
    committed: &CommittedActionRef,
    prefix: &[(RuntimeMixRenderPlan, u32)],
    before: RuntimeMixRenderPlan,
    after: RuntimeMixRenderPlan,
) -> GestureTransition {
    GestureTransition {
        case_id,
        key,
        command,
        boundary,
        action_id: committed.action_id.0,
        commit_boundary: committed.boundary.clone(),
        prefix: prefix.to_vec(),
        before,
        after,
    }
}

fn render_plan(state: &JamAppState, bpm: f32, position_beats: f64) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats,
        },
        tr909_render: state.runtime.tr909_render.clone(),
        mc202_render: state.runtime.mc202_render,
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: state.runtime.w30_resample_tap.clone(),
        source_monitor_render: state.source_monitor_render_state(),
    }
}

fn current_scene(state: &JamAppState) -> Option<SceneId> {
    state
        .session
        .runtime_state
        .scene_state
        .active_scene
        .clone()
        .or_else(|| state.runtime.transport.current_scene.clone())
}

fn update_transport_position(state: &mut JamAppState, position_beats: u64) {
    let current_scene = current_scene(state);
    let grid_position = grid_position(state, position_beats);
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: position_beats as f64,
        beat_index: grid_position.beat_cursor,
        bar_index: grid_position.bar_index,
        phrase_index: grid_position.phrase_index,
        current_scene,
    });
}

fn counterfactual_plan_at_position(
    pre_jump_state: &JamAppState,
    bpm: f32,
    position_beats: u64,
) -> RuntimeMixRenderPlan {
    let mut control = pre_jump_state.clone();
    update_transport_position(&mut control, position_beats);
    render_plan(&control, bpm, position_beats as f64)
}

fn lane_audio_projection_matches(
    expected: &RuntimeMixRenderPlan,
    actual: &RuntimeMixRenderPlan,
) -> bool {
    expected.tr909_render == actual.tr909_render
        && expected.mc202_render == actual.mc202_render
        && expected.w30_preview_render == actual.w30_preview_render
        && expected.w30_resample_tap == actual.w30_resample_tap
}

fn one_commit(
    mut committed: Vec<CommittedActionRef>,
) -> Result<CommittedActionRef, Box<dyn Error>> {
    committed
        .pop()
        .ok_or_else(|| "expected one committed action".into())
}

fn require_committed_command(
    state: &JamAppState,
    committed: &CommittedActionRef,
    expected: ActionCommand,
) -> Result<(), Box<dyn Error>> {
    let actual = state
        .queue
        .history_action(committed.action_id)
        .map(|action| action.command)
        .ok_or_else(|| {
            format!(
                "committed action {} missing from queue history",
                committed.action_id.0
            )
        })?;
    if actual != expected {
        return Err(format!(
            "committed action {} was {}, expected {}",
            committed.action_id.0,
            actual.as_str(),
            expected.as_str()
        )
        .into());
    }
    Ok(())
}

fn require_committed_scene_target(
    state: &JamAppState,
    committed: &CommittedActionRef,
    expected_scene: &SceneId,
) -> Result<(), Box<dyn Error>> {
    let action = state
        .queue
        .history_action(committed.action_id)
        .ok_or_else(|| {
            format!(
                "scene action {} missing from history",
                committed.action_id.0
            )
        })?;
    if action.target.scope.as_ref() != Some(&TargetScope::Scene)
        || action.target.scene_id.as_ref() != Some(expected_scene)
    {
        return Err(format!(
            "scene action {} target {:?}/{:?}, expected Scene/{expected_scene}",
            committed.action_id.0, action.target.scope, action.target.scene_id
        )
        .into());
    }
    Ok(())
}

fn scene_source_anchor_seconds(state: &JamAppState, scene: &SceneId) -> Option<f64> {
    state
        .source_graph
        .as_ref()
        .and_then(|graph| primary_grid_anchor_seconds_for_projected_scene(graph, scene))
}

fn require_same_anchor(
    label: &str,
    actual: Option<f64>,
    expected: Option<f64>,
) -> Result<(), Box<dyn Error>> {
    let matches = match (actual, expected) {
        (Some(actual), Some(expected)) => (actual - expected).abs() <= 1.0e-6,
        _ => false,
    };
    if !matches {
        return Err(format!("{label} source anchor {actual:?}, expected {expected:?}").into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    position_beats: u64,
    scene_id: Option<SceneId>,
    timestamp: u64,
    expected_count: usize,
) -> Result<Vec<CommittedActionRef>, Box<dyn Error>> {
    let grid_position = grid_position(state, position_beats);
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: position_beats as f64,
        beat_index: grid_position.beat_cursor,
        bar_index: grid_position.bar_index,
        phrase_index: grid_position.phrase_index,
        current_scene: scene_id.clone(),
    });
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            // Session V1 persists the integral zero-based cursor here.
            beat_index: position_beats,
            bar_index: grid_position.bar_index,
            phrase_index: grid_position.phrase_index,
            scene_id,
        },
        timestamp,
    );
    if committed.len() != expected_count {
        return Err(format!(
            "expected {expected_count} {kind:?} commits, got {}",
            committed.len()
        )
        .into());
    }
    Ok(committed)
}

fn grid_position(state: &JamAppState, position_beats: u64) -> TransportGridPosition {
    state
        .source_graph
        .as_ref()
        .and_then(|graph| graph.timing.primary_hypothesis())
        .and_then(|hypothesis| hypothesis.transport_grid_position(position_beats as f64))
        .unwrap_or_else(|| {
            TransportGridPosition::from_zero_based_position_beats(
                position_beats as f64,
                DEFAULT_BEATS_PER_BAR,
                DEFAULT_BARS_PER_PHRASE,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_audio::tr909::Tr909PhraseVariation;
    use riotbox_core::{
        ids::{ActionId, SceneId, SourceId},
        queue::ActionQueue,
        session::{SessionFile, SourceTimingGridConfirmationState, Tr909ReinforcementModeState},
        source_graph::{
            BarSpan, BeatPoint, DecodeProfile, GraphProvenance, MeterHint, SourceDescriptor,
            SourceGraph, TimingHypothesis, TimingHypothesisKind, TimingQuality,
        },
    };

    #[test]
    fn confirmed_timing_uses_primary_hypothesis_bpm_instead_of_cli_hint() {
        let state = timing_state("primary-grid");

        let timing = confirmed_source_timing(&state, 132.0).expect("confirmed timing");

        assert_eq!(timing.cli_bpm_hint, 132.0);
        assert_eq!(timing.bpm, 132.512_1);
        assert_eq!(timing.hypothesis_id, "primary-grid");
        assert_eq!(timing.primary_bar_anchor_beat_index, 4);
        assert_eq!(timing.primary_bar_anchor_beat_cursor, 3);
    }

    #[test]
    fn confirmed_timing_rejects_confirmation_for_non_primary_hypothesis() {
        let state = timing_state("stale-grid");

        let error = confirmed_source_timing(&state, 132.0)
            .expect_err("stale confirmation must not drive rendering");

        assert!(error.to_string().contains("does not match primary"));
    }

    #[test]
    fn restore_counterfactual_advances_phrase_policy_to_the_restore_cursor() {
        let mut state = timing_state("primary-grid");
        let scene = SceneId::from("scene-01-intro");
        state.session.runtime_state.scene_state.active_scene = Some(scene.clone());
        state.session.runtime_state.transport.current_scene = Some(scene);
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = Some(Tr909ReinforcementModeState::BreakReinforce);
        state.session.runtime_state.lane_state.tr909.pattern_ref =
            Some("reinforce-scene-01-intro".into());
        update_transport_position(&mut state, 31);

        let stale_pre_jump_plan = render_plan(&state, 132.512_1, 31.0);
        let restore_cursor_plan = counterfactual_plan_at_position(&state, 132.512_1, 35);

        assert_eq!(
            stale_pre_jump_plan.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseDrive)
        );
        assert_eq!(
            restore_cursor_plan.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseLift)
        );
        assert!(
            !lane_audio_projection_matches(&stale_pre_jump_plan, &restore_cursor_plan),
            "the old pre-jump render must not masquerade as the current-time restore oracle"
        );
        assert!(lane_audio_projection_matches(
            &restore_cursor_plan,
            &counterfactual_plan_at_position(&state, 132.512_1, 35)
        ));
    }

    fn timing_state(confirmed_hypothesis_id: &str) -> JamAppState {
        let source_id = SourceId::from("src-dense-timing-test");
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: source_id.clone(),
                path: "dense.wav".into(),
                content_hash: "dense-hash".into(),
                duration_seconds: 12.0,
                sample_rate: 44_100,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec!["rust-timing-test".into()],
                generated_at: "2026-07-16T00:00:00Z".into(),
                source_hash: "dense-hash".into(),
                analysis_seed: 23,
                run_notes: None,
            },
        );
        graph.timing.primary_hypothesis_id = Some("primary-grid".into());
        let seconds_per_beat = 60.0 / 132.512_1;
        graph.timing.hypotheses = vec![TimingHypothesis {
            hypothesis_id: "primary-grid".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 132.512_1,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 0.9,
            score: 0.9,
            beat_grid: (4..=43)
                .map(|beat_index| BeatPoint {
                    beat_index,
                    time_seconds: (beat_index - 4) as f32 * seconds_per_beat,
                    confidence: 0.9,
                })
                .collect(),
            bar_grid: (1..=10)
                .map(|bar_index| BarSpan {
                    bar_index,
                    start_seconds: (bar_index - 1) as f32 * 4.0 * seconds_per_beat,
                    end_seconds: bar_index as f32 * 4.0 * seconds_per_beat,
                    downbeat_confidence: 0.9,
                    phrase_index: Some((bar_index - 1) / 4 + 1),
                })
                .collect(),
            phrase_grid: Vec::new(),
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: vec!["test".into()],
        }];
        let mut session = SessionFile::new("dense-timing-test", "0.1.0", "2026-07-16T00:00:00Z");
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id,
                hypothesis_id: Some(confirmed_hypothesis_id.into()),
                confirmed_by_action: ActionId(1),
                confirmed_at: 1,
            });

        JamAppState::from_parts(session, Some(graph), ActionQueue::new())
    }
}

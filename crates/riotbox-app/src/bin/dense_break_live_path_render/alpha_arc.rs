use std::{error::Error, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::RuntimeMixRenderPlan, tr909::Tr909FillRecipeId, w30::W30PreviewRenderRouting,
};
use riotbox_core::{
    action::{ActionCommand, CommitBoundary},
    style::PerformancePresetId,
};

use crate::{
    live_flow::{
        commit, committed_for_command, current_scene, one_commit, render_plan,
        require_committed_command, stage,
    },
    model::{AlphaArcProof, ConfirmedSourceTiming, RenderStage, RestartRecallProof},
};

pub(super) fn prepare_restart_recall(
    output_dir: &Path,
    source_timing: &ConfirmedSourceTiming,
    bpm: f32,
    preset_id: PerformancePresetId,
    recall_bar_index: u64,
) -> Result<(Box<RuntimeMixRenderPlan>, RestartRecallProof), Box<dyn Error>> {
    let mut state = JamAppState::from_json_files(
        output_dir.join("session.json"),
        Some(output_dir.join("source-graph.json")),
    )?;
    let preset_survived_restart =
        state.session.runtime_state.style.active_preset == Some(preset_id);
    if !preset_survived_restart {
        return Err("live path preset identity did not survive restart".into());
    }
    let capture_id = state
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .clone()
        .ok_or("live path restart lost the promoted W-30 capture")?;
    let recall_cursor = source_timing
        .bar_start_beat_cursor(recall_bar_index)
        .ok_or_else(|| format!("live path cannot resolve restart recall bar {recall_bar_index}"))?;
    state.set_transport_playing(true);
    if state.queue_w30_live_recall(3_000) != Some(QueueControlResult::Enqueued) {
        return Err("live path W-30 recall was unavailable after restart".into());
    }
    let scene = current_scene(&state);
    let recall_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        recall_cursor,
        scene,
        3_010,
        1,
    )?)?;
    require_committed_command(&state, &recall_commit, ActionCommand::W30LiveRecall)?;

    if state.queue_w30_trigger_pad(3_100) != Some(QueueControlResult::Enqueued) {
        return Err("live path W-30 trigger was unavailable after restart recall".into());
    }
    let trigger_cursor = recall_cursor.saturating_add(1);
    let scene = current_scene(&state);
    let trigger_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        trigger_cursor,
        scene,
        3_110,
        1,
    )?)?;
    require_committed_command(&state, &trigger_commit, ActionCommand::W30TriggerPad)?;
    let plan = render_plan(&state, bpm, trigger_cursor as f64);
    if plan.w30_preview_render.routing != W30PreviewRenderRouting::MusicBusPreview {
        return Err(format!(
            "live path restart trigger routing was {:?}",
            plan.w30_preview_render.routing
        )
        .into());
    }

    Ok((
        Box::new(plan),
        RestartRecallProof {
            preset_survived_restart,
            capture_id: capture_id.to_string(),
            recall_action_id: recall_commit.action_id.0,
            trigger_action_id: trigger_commit.action_id.0,
        },
    ))
}

pub(super) fn prepare_alpha_arc(
    ready_state: &JamAppState,
    source_timing: &ConfirmedSourceTiming,
    bpm: f32,
) -> Result<(Vec<RenderStage>, AlphaArcProof), Box<dyn Error>> {
    let hook_cursor = source_timing
        .bar_start_beat_cursor(5)
        .ok_or("Feral Break Alpha cannot resolve hook bar 5")?;
    let pressure_cursor = source_timing
        .bar_start_beat_cursor(7)
        .ok_or("Feral Break Alpha cannot resolve pressure bar 7")?;
    let fill_cursor = source_timing
        .bar_start_beat_cursor(9)
        .ok_or("Feral Break Alpha cannot resolve destructive bar 9")?;
    let role_swap_cursor = source_timing
        .bar_start_beat_cursor(10)
        .ok_or("Feral Break Alpha cannot resolve role-swap bar 10")?;
    let return_cursor = source_timing
        .bar_start_beat_cursor(11)
        .ok_or("Feral Break Alpha cannot resolve return bar 11")?;

    let mut state = ready_state.clone();
    let original_scene = current_scene(&state).ok_or("Feral Break Alpha has no original scene")?;
    if state.queue_w30_trigger_pad(2_000) != Some(QueueControlResult::Enqueued) {
        return Err("Feral Break Alpha W-30 hook trigger was unavailable".into());
    }
    let hook_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        hook_cursor,
        Some(original_scene.clone()),
        2_010,
        1,
    )?)?;
    require_committed_command(&state, &hook_commit, ActionCommand::W30TriggerPad)?;
    let mut stages = Vec::with_capacity(5);
    push_alpha_stage(
        &mut stages,
        "alpha-hook-establish",
        "alpha/00_hook_establish.wav",
        8,
        Some("w"),
        Some(ActionCommand::W30TriggerPad),
        Some(CommitBoundary::Beat),
        Some(hook_commit.action_id.0),
        &state,
        bpm,
        hook_cursor,
    );

    if !state.queue_tr909_slam_toggle(2_100) {
        return Err("Feral Break Alpha pressure lift was unavailable".into());
    }
    let pressure_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        pressure_cursor,
        Some(original_scene.clone()),
        2_110,
        1,
    )?)?;
    require_committed_command(&state, &pressure_commit, ActionCommand::Tr909SetSlam)?;
    push_alpha_stage(
        &mut stages,
        "alpha-pressure-lift",
        "alpha/01_pressure_lift.wav",
        8,
        Some("s"),
        Some(ActionCommand::Tr909SetSlam),
        Some(CommitBoundary::Beat),
        Some(pressure_commit.action_id.0),
        &state,
        bpm,
        pressure_cursor,
    );

    state.queue_tr909_fill(2_200);
    let fill_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        fill_cursor,
        Some(original_scene.clone()),
        2_210,
        1,
    )?)?;
    require_committed_command(&state, &fill_commit, ActionCommand::Tr909FillNext)?;
    push_alpha_stage(
        &mut stages,
        "alpha-destructive-fill",
        "alpha/02_destructive_fill.wav",
        4,
        Some("f"),
        Some(ActionCommand::Tr909FillNext),
        Some(CommitBoundary::Bar),
        Some(fill_commit.action_id.0),
        &state,
        bpm,
        fill_cursor,
    );
    let alpha_fill_recipe = stages
        .last()
        .and_then(|stage| stage.plan.tr909_render.fill_recipe_id())
        .ok_or("Feral Break Alpha destructive stage did not retain its committed Fill recipe")?;
    if alpha_fill_recipe != Tr909FillRecipeId::PhraseDriveBreakCutStompV2 {
        return Err(format!(
            "Feral Break Alpha destructive stage selected {}, expected {}",
            alpha_fill_recipe.label(),
            Tr909FillRecipeId::PhraseDriveBreakCutStompV2.label()
        )
        .into());
    }

    if state.queue_scene_select(2_300) != QueueControlResult::Enqueued {
        return Err("Feral Break Alpha destructive role swap was unavailable".into());
    }
    let role_swap_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        role_swap_cursor,
        Some(original_scene.clone()),
        2_310,
        1,
    )?)?;
    require_committed_command(&state, &role_swap_commit, ActionCommand::SceneLaunch)?;
    let contrast_scene =
        current_scene(&state).ok_or("Feral Break Alpha role swap cleared the scene")?;
    if contrast_scene == original_scene {
        return Err("Feral Break Alpha role swap did not change scene".into());
    }
    push_alpha_stage(
        &mut stages,
        "alpha-destructive-role-swap",
        "alpha/03_destructive_role_swap.wav",
        4,
        Some("y"),
        Some(ActionCommand::SceneLaunch),
        Some(CommitBoundary::Bar),
        Some(role_swap_commit.action_id.0),
        &state,
        bpm,
        role_swap_cursor,
    );

    if state.queue_scene_restore(2_400) != QueueControlResult::Enqueued {
        return Err("Feral Break Alpha return was unavailable".into());
    }
    if state.queue_w30_apply_damage_profile(2_401) != Some(QueueControlResult::Enqueued) {
        return Err("Feral Break Alpha changed-return damage was unavailable".into());
    }
    let return_commits = commit(
        &mut state,
        CommitBoundary::Bar,
        return_cursor,
        Some(contrast_scene.clone()),
        2_410,
        2,
    )?;
    let return_commit =
        committed_for_command(&state, &return_commits, ActionCommand::SceneRestore)?;
    let damage_commit = committed_for_command(
        &state,
        &return_commits,
        ActionCommand::W30ApplyDamageProfile,
    )?;
    let returned_scene =
        current_scene(&state).ok_or("Feral Break Alpha return cleared the scene")?;
    if returned_scene != original_scene {
        return Err(format!(
            "Feral Break Alpha returned to {returned_scene}, expected {original_scene}"
        )
        .into());
    }
    push_alpha_stage(
        &mut stages,
        "alpha-changed-return",
        "alpha/04_changed_return.wav",
        8,
        Some("Y+D"),
        Some(ActionCommand::SceneRestore),
        Some(CommitBoundary::Bar),
        Some(return_commit.action_id.0),
        &state,
        bpm,
        return_cursor,
    );

    Ok((
        stages,
        AlphaArcProof {
            hook_action_id: hook_commit.action_id.0,
            pressure_action_id: pressure_commit.action_id.0,
            destructive_fill_action_id: fill_commit.action_id.0,
            role_swap_action_id: role_swap_commit.action_id.0,
            return_action_id: return_commit.action_id.0,
            return_damage_action_id: damage_commit.action_id.0,
            original_scene: original_scene.to_string(),
            contrast_scene: contrast_scene.to_string(),
            returned_scene: returned_scene.to_string(),
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn push_alpha_stage(
    stages: &mut Vec<RenderStage>,
    case_id: &'static str,
    artifact_path: &'static str,
    duration_beats: u32,
    key: Option<&'static str>,
    command: Option<ActionCommand>,
    boundary: Option<CommitBoundary>,
    action_id: Option<u64>,
    state: &JamAppState,
    bpm: f32,
    position_beats: u64,
) {
    stages.push(stage(
        case_id,
        artifact_path,
        duration_beats,
        key,
        command,
        boundary,
        action_id,
        state,
        render_plan(state, bpm, position_beats as f64),
    ));
}

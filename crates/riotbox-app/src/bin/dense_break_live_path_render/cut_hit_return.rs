use std::error::Error;

use riotbox_app::jam_app::{JamAppState, Tr909CutHitReturnQueueResult};
use riotbox_audio::tr909::{Tr909FillRecipeId, Tr909RenderMode};
use riotbox_core::action::{ActionCommand, CommitBoundary};

use crate::{
    live_flow::{commit, committed_for_command, current_scene, render_plan},
    model::{ConfirmedSourceTiming, CutHitReturnProof},
};

pub(super) fn prepare_cut_hit_return(
    ready_state: &JamAppState,
    source_timing: &ConfirmedSourceTiming,
    bpm: f32,
) -> Result<CutHitReturnProof, Box<dyn Error>> {
    let impact_cursor = source_timing
        .bar_start_beat_cursor(9)
        .ok_or("cut-hit return cannot resolve bar 9")?;
    let return_cursor = impact_cursor.saturating_add(source_timing.beats_per_bar);
    let scene = current_scene(ready_state);

    let mut candidate_state = ready_state.clone();
    if candidate_state.queue_tr909_cut_hit_return(3_000) != Tr909CutHitReturnQueueResult::Enqueued {
        return Err("source-backed cut-hit return was unavailable on the exact live state".into());
    }
    let committed = commit(
        &mut candidate_state,
        CommitBoundary::Bar,
        impact_cursor,
        scene.clone(),
        3_010,
        2,
    )?;
    let fill_commit =
        committed_for_command(&candidate_state, &committed, ActionCommand::Tr909FillNext)?;
    let slam_commit =
        committed_for_command(&candidate_state, &committed, ActionCommand::Tr909SetSlam)?;
    if fill_commit.boundary != slam_commit.boundary {
        return Err("cut-hit return actions did not commit at one boundary".into());
    }
    let candidate_plan = render_plan(&candidate_state, bpm, impact_cursor as f64);
    if candidate_plan.tr909_render.mode != Tr909RenderMode::Fill
        || !candidate_plan.tr909_render.slam_enabled
        || candidate_plan.tr909_render.fill_recipe_id()
            != Some(Tr909FillRecipeId::PhraseDriveBreakCutStompV2)
    {
        return Err("cut-hit return did not project the frozen slammed V2 Fill recipe".into());
    }

    let mut slam_control_state = ready_state.clone();
    if !slam_control_state.queue_tr909_slam_toggle(3_100) {
        return Err("cut-hit Slam-only control was unavailable".into());
    }
    commit(
        &mut slam_control_state,
        CommitBoundary::Bar,
        impact_cursor,
        scene.clone(),
        3_110,
        1,
    )?;
    let slam_only_control_plan = render_plan(&slam_control_state, bpm, impact_cursor as f64);

    let mut fill_control_state = ready_state.clone();
    fill_control_state.queue_tr909_fill(3_200);
    commit(
        &mut fill_control_state,
        CommitBoundary::Bar,
        impact_cursor,
        scene.clone(),
        3_210,
        1,
    )?;
    let fill_only_control_plan = render_plan(&fill_control_state, bpm, impact_cursor as f64);

    commit(
        &mut candidate_state,
        CommitBoundary::Bar,
        return_cursor,
        scene,
        3_020,
        0,
    )?;
    let changed_return_plan = render_plan(&candidate_state, bpm, return_cursor as f64);
    if changed_return_plan.tr909_render.mode != Tr909RenderMode::BreakReinforce
        || !changed_return_plan.tr909_render.slam_enabled
    {
        return Err("cut-hit return did not restore BreakReinforce with Slam held".into());
    }

    Ok(CutHitReturnProof {
        fill_action_id: fill_commit.action_id.0,
        slam_action_id: slam_commit.action_id.0,
        commit_boundary: fill_commit.boundary,
        slam_only_control_plan: Box::new(slam_only_control_plan),
        fill_only_control_plan: Box::new(fill_only_control_plan),
        candidate_plan: Box::new(candidate_plan),
        changed_return_plan: Box::new(changed_return_plan),
    })
}

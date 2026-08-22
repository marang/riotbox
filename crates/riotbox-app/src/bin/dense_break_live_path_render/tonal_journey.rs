use std::{error::Error, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_core::{
    action::{ActionCommand, CommitBoundary},
    style::PerformancePresetId,
};

use crate::{
    alpha_arc::prepare_restart_recall,
    live_flow::{commit, current_scene, one_commit, render_plan, require_committed_command},
    model::{
        ConfirmedSourceTiming, TONAL_PITCH_DIVE_ACTIVE_BEATS, TonalJourney, TonalJourneyProof,
    },
};

pub(super) fn prepare(
    ready_state: &JamAppState,
    output_dir: &Path,
    source_timing: &ConfirmedSourceTiming,
    bpm: f32,
    preset_id: PerformancePresetId,
) -> Result<TonalJourney, Box<dyn Error>> {
    let held_cursor = source_timing
        .bar_start_beat_cursor(5)
        .ok_or("tonal journey cannot resolve held-loop bar 5")?;
    let contrast_cursor = source_timing
        .bar_start_beat_cursor(9)
        .ok_or("tonal journey cannot resolve Pitch Dive bar 9")?;
    let reentry_cursor = contrast_cursor.saturating_add(u64::from(TONAL_PITCH_DIVE_ACTIVE_BEATS));

    let held_plan = Box::new(render_plan(ready_state, bpm, held_cursor as f64));
    let mut state = Box::new(ready_state.clone());
    if state.queue_w30_pitch_dive(2_000) != Some(QueueControlResult::Enqueued) {
        return Err("tonal Pitch Dive was unavailable".into());
    }
    let scene = current_scene(&state);
    let contrast_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        contrast_cursor,
        scene,
        2_010,
        1,
    )?)?;
    require_committed_command(&state, &contrast_commit, ActionCommand::W30PitchDive)?;
    let contrast_plan = Box::new(render_plan(&state, bpm, contrast_cursor as f64));
    if state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .is_none()
    {
        return Err("tonal Pitch Dive did not reach committed W-30 articulation state".into());
    }

    if state.queue_w30_trigger_pad(2_100) != Some(QueueControlResult::Enqueued) {
        return Err("ordinary tonal W-30 re-entry was unavailable".into());
    }
    let scene = current_scene(&state);
    let reentry_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Beat,
        reentry_cursor,
        scene,
        2_110,
        1,
    )?)?;
    require_committed_command(&state, &reentry_commit, ActionCommand::W30TriggerPad)?;
    let ordinary_reentry_cleared_articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .is_none()
        && state
            .runtime
            .w30_preview
            .pad_playback
            .as_ref()
            .and_then(|pad| pad.hook_articulation)
            .is_none();
    if !ordinary_reentry_cleared_articulation {
        return Err("ordinary tonal W-30 re-entry retained Pitch Dive articulation".into());
    }
    let reentry_plan = Box::new(render_plan(&state, bpm, reentry_cursor as f64));

    state.save()?;
    let (restart_recall_plan, restart_recall_proof) =
        prepare_restart_recall(output_dir, source_timing, bpm, preset_id, 14)?;

    Ok(TonalJourney {
        held_plan,
        contrast_plan,
        reentry_plan,
        restart_recall_plan,
        proof: TonalJourneyProof {
            contrast_action_id: contrast_commit.action_id.0,
            reentry_action_id: reentry_commit.action_id.0,
            ordinary_reentry_cleared_articulation,
        },
        restart_recall_proof,
    })
}

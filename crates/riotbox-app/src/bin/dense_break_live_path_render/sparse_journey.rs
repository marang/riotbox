use std::{error::Error, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_core::{
    action::{ActionCommand, ActionParams, CommitBoundary},
    ids::ActionId,
    style::PerformancePresetId,
    w30_damage_policy::W30_DAMAGE_PROFILE_ACTIVE_INTENSITY,
};

use crate::{
    alpha_arc::prepare_restart_recall,
    live_flow::{commit, current_scene, one_commit, render_plan, require_committed_command},
    model::{ConfirmedSourceTiming, SparseJourney, SparseJourneyProof},
};

const DAMAGE_BEATS: u64 = 16;

pub(super) fn prepare(
    ready_state: &JamAppState,
    output_dir: &Path,
    source_timing: &ConfirmedSourceTiming,
    bpm: f32,
    preset_id: PerformancePresetId,
) -> Result<SparseJourney, Box<dyn Error>> {
    let held_cursor = source_timing
        .bar_start_beat_cursor(5)
        .ok_or("sparse journey cannot resolve held-loop bar 5")?;
    let damage_cursor = source_timing
        .bar_start_beat_cursor(9)
        .ok_or("sparse journey cannot resolve damage bar 9")?;
    let reentry_cursor = damage_cursor.saturating_add(DAMAGE_BEATS);

    let held_plan = Box::new(render_plan(ready_state, bpm, held_cursor as f64));
    require_ordinary_sparse_transform(&held_plan, "held")?;

    let mut state = Box::new(ready_state.clone());
    if state.queue_w30_apply_damage_profile(2_000) != Some(QueueControlResult::Enqueued) {
        return Err("sparse damage apply was unavailable".into());
    }
    let scene = current_scene(&state);
    let damage_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        damage_cursor,
        scene,
        2_010,
        1,
    )?)?;
    require_committed_command(&state, &damage_commit, ActionCommand::W30ApplyDamageProfile)?;
    let damage_intensity = committed_damage_intensity(&state, damage_commit.action_id)?;
    if (damage_intensity - W30_DAMAGE_PROFILE_ACTIVE_INTENSITY).abs() > f32::EPSILON {
        return Err(format!("sparse damage apply intensity changed: {damage_intensity}").into());
    }
    let damage_plan = Box::new(render_plan(&state, bpm, damage_cursor as f64));
    let damage_gate_step_fraction = damage_plan
        .w30_preview_render
        .pad_playback
        .as_ref()
        .ok_or("sparse damage apply lost W-30 pad playback")?
        .gate_step_fraction;
    if damage_gate_step_fraction <= 0.0 {
        return Err("sparse damage apply did not reach transient-bite gate state".into());
    }

    if state.queue_w30_apply_damage_profile(2_100) != Some(QueueControlResult::Enqueued) {
        return Err("sparse damage bypass was unavailable".into());
    }
    let scene = current_scene(&state);
    let bypass_commit = one_commit(commit(
        &mut state,
        CommitBoundary::Bar,
        reentry_cursor,
        scene,
        2_110,
        1,
    )?)?;
    require_committed_command(&state, &bypass_commit, ActionCommand::W30ApplyDamageProfile)?;
    let bypass_intensity = committed_damage_intensity(&state, bypass_commit.action_id)?;
    if bypass_intensity != 0.0 {
        return Err(format!(
            "sparse damage bypass committed non-zero intensity {bypass_intensity}"
        )
        .into());
    }
    let reentry_plan = Box::new(render_plan(&state, bpm, reentry_cursor as f64));
    require_ordinary_sparse_transform(&reentry_plan, "re-entry")?;
    let reentry_gate_step_fraction = reentry_plan
        .w30_preview_render
        .pad_playback
        .as_ref()
        .ok_or("sparse re-entry lost W-30 pad playback")?
        .gate_step_fraction;

    state.save()?;
    let (restart_recall_plan, restart_recall_proof) =
        prepare_restart_recall(output_dir, source_timing, bpm, preset_id, 14)?;
    require_ordinary_sparse_transform(&restart_recall_plan, "restart recall")?;

    Ok(SparseJourney {
        held_plan,
        damage_plan,
        reentry_plan,
        restart_recall_plan,
        proof: SparseJourneyProof {
            damage_action_id: damage_commit.action_id.0,
            bypass_action_id: bypass_commit.action_id.0,
            damage_intensity,
            bypass_intensity,
            damage_gate_step_fraction,
            reentry_gate_step_fraction,
        },
        restart_recall_proof,
    })
}

fn committed_damage_intensity(
    state: &JamAppState,
    action_id: ActionId,
) -> Result<f32, Box<dyn Error>> {
    state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .and_then(|action| match action.params {
            ActionParams::Mutation { intensity, .. } => Some(intensity),
            _ => None,
        })
        .ok_or_else(|| format!("damage action {action_id} omitted mutation intensity").into())
}

fn require_ordinary_sparse_transform(
    plan: &riotbox_audio::runtime::RuntimeMixRenderPlan,
    stage: &str,
) -> Result<(), Box<dyn Error>> {
    let playback = plan
        .w30_preview_render
        .pad_playback
        .as_ref()
        .ok_or_else(|| format!("sparse {stage} lost W-30 pad playback"))?;
    if playback.playback_rate != 1.0 || playback.gate_step_fraction != 0.0 || playback.reverse {
        return Err(format!(
            "sparse {stage} retained a damage transform: rate={} gate={} reverse={}",
            playback.playback_rate, playback.gate_step_fraction, playback.reverse
        )
        .into());
    }
    Ok(())
}

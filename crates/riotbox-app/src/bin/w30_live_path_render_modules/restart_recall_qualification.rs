use std::path::Path;

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::{
        RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    },
    w30::W30PreviewRenderState,
};
use riotbox_core::{
    action::{ActionCommand, CommitBoundary},
    ids::SceneId,
    style::PerformancePresetId,
};

use super::super::{CHANNEL_COUNT, SAMPLE_RATE, commit, isolated_w30_plan};

pub(crate) fn qualify_gesture_vocabulary_restart_recall(
    state: &JamAppState,
    ordinary_reentry: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
    scene_id: Option<SceneId>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    const REFERENCE_BEAT: u64 = 41;
    const RECALL_BEAT: u64 = 48;
    const TRIGGER_BEAT: u64 = 49;

    state.save()?;
    let mut restarted = JamAppState::from_json_files(
        output_dir.join("session.json"),
        Some(output_dir.join("source-graph.json")),
    )?;
    let preset_survived = restarted.session.runtime_state.style.active_preset
        == Some(PerformancePresetId::FeralBreakAlphaV2);
    if !preset_survived {
        return Err("gesture-vocabulary restart lost the active preset".into());
    }
    let expected_capture = state
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
        .ok_or("gesture-vocabulary state has no promoted capture")?;
    let restarted_capture = restarted
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
        .ok_or("gesture-vocabulary restart lost the promoted capture")?;
    if restarted_capture != expected_capture {
        return Err("gesture-vocabulary restart changed the promoted capture".into());
    }

    restarted.set_transport_playing(true);
    if restarted.queue_w30_live_recall(5_000) != Some(QueueControlResult::Enqueued) {
        return Err("gesture-vocabulary W-30 recall was unavailable after restart".into());
    }
    commit(
        &mut restarted,
        CommitBoundary::Bar,
        RECALL_BEAT,
        RECALL_BEAT / 4 + 1,
        RECALL_BEAT / 16 + 1,
        scene_id.clone(),
        5_010,
    )?;
    if restarted.queue_w30_trigger_pad(5_100) != Some(QueueControlResult::Enqueued) {
        return Err("gesture-vocabulary W-30 trigger was unavailable after restart".into());
    }
    commit(
        &mut restarted,
        CommitBoundary::Beat,
        TRIGGER_BEAT,
        TRIGGER_BEAT / 4 + 1,
        TRIGGER_BEAT / 16 + 1,
        scene_id,
        5_110,
    )?;
    if restarted
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .is_some()
        || restarted
            .runtime
            .w30_preview
            .pad_playback
            .as_ref()
            .and_then(|pad| pad.hook_articulation)
            .is_some()
    {
        return Err("gesture-vocabulary restart recall retained timed articulation".into());
    }

    let frame_count = (4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let reference_plan = isolated_w30_plan(ordinary_reentry.clone(), bpm, REFERENCE_BEAT as f64);
    let restarted_plan = isolated_w30_plan(
        restarted.runtime.w30_preview.clone(),
        bpm,
        TRIGGER_BEAT as f64,
    );
    let reference = render_once(&reference_plan, frame_count, 128, "restart reference")?;
    let restarted_128 = render_once(&restarted_plan, frame_count, 128, "restart")?;
    let restarted_257 = render_once(&restarted_plan, frame_count, 257, "restart partition")?;
    if restarted_128.samples != restarted_257.samples {
        return Err("gesture-vocabulary restart changed across callback partitions".into());
    }
    if restarted_128.samples != reference.samples {
        return Err("gesture-vocabulary restart was not sample-exact to ordinary W-30".into());
    }
    if restarted_128.limiter.post.active_samples == 0
        || restarted_128.limiter.post.rms <= 0.001
        || restarted_128.limiter.pre.clip_count != 0
        || restarted_128.limiter.limited_sample_count != 0
        || restarted_128.limiter.post.clip_count != 0
    {
        return Err("gesture-vocabulary restart was silent, clipped, or limited".into());
    }

    Ok(serde_json::json!({
        "preset_survived": preset_survived,
        "capture_identity_preserved": true,
        "recall_action": ActionCommand::W30LiveRecall.as_str(),
        "trigger_action": ActionCommand::W30TriggerPad.as_str(),
        "articulation_cleared": true,
        "sample_exact_to_ordinary_reentry": true,
        "callback_partition_128_vs_257_sample_exact": true,
        "active_samples": restarted_128.limiter.post.active_samples,
        "peak_abs": restarted_128.limiter.post.peak_abs,
        "rms": restarted_128.limiter.post.rms,
        "pre_limiter_clip_count": restarted_128.limiter.pre.clip_count,
        "limited_sample_count": restarted_128.limiter.limited_sample_count,
        "post_limiter_clip_count": restarted_128.limiter.post.clip_count
    }))
}

fn render_once(
    plan: &riotbox_audio::runtime::RuntimeMixRenderPlan,
    frame_count: usize,
    callback_frames: usize,
    role: &str,
) -> Result<riotbox_audio::runtime::RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        callback_frames,
    )
    .pop()
    .ok_or_else(|| format!("gesture-vocabulary {role} produced no output").into())
}

use std::{fs, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::{
        RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        signal_delta_metrics, signal_metrics,
    },
    source_audio::write_interleaved_pcm16_wav,
    w30::W30PreviewRenderState,
};
use riotbox_core::{
    action::CommitBoundary, ids::SceneId, session::W30HookArticulationProfileState,
};

use super::super::{
    CHANNEL_COUNT, SAMPLE_RATE, commit, isolated_w30_plan,
    qualification_sample_offset_at_beat_boundary,
};

pub(crate) fn qualify_pitch_dive_v1(
    state: &mut JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
    scene_id: Option<SceneId>,
    prepare_review: bool,
    start_beat: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    const CONTROL_BEATS: f32 = 13.0;
    let frame_count = (CONTROL_BEATS * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let beat_eight = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 8.0,
        bpm,
    )?;
    let beat_twelve = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 12.0,
        bpm,
    )?;
    let control_plan = isolated_w30_plan(control_render.clone(), bpm, start_beat as f64);
    let control = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &control_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("pitch-dive control render produced no output")?;

    let captures_before = state.session.captures.clone();
    let grit_before = state.session.runtime_state.macro_state.w30_grit;
    let source_monitor_before = state.session.runtime_state.source_monitor.clone();
    let requested_at = 400 + start_beat * 10;
    if state.queue_w30_pitch_dive(requested_at) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 pitch dive was unavailable".into());
    }
    commit(
        state,
        CommitBoundary::Bar,
        start_beat,
        start_beat / 4 + 1,
        start_beat / 16 + 1,
        scene_id,
        requested_at + 1,
    )?;
    let articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .as_ref()
        .ok_or("pitch-dive commit produced no Session articulation")?;
    if articulation.profile != W30HookArticulationProfileState::PitchDiveV1
        || articulation.started_at_beat != start_beat
        || articulation.capture_id.to_string()
            != state
                .runtime
                .w30_preview
                .capture_id
                .as_deref()
                .unwrap_or_default()
    {
        return Err("pitch-dive Session profile, target, or start beat diverged".into());
    }
    if state.session.captures != captures_before
        || state.session.runtime_state.macro_state.w30_grit != grit_before
        || state.session.runtime_state.source_monitor != source_monitor_before
    {
        return Err("pitch dive changed frozen capture, grit, or Source Monitor state".into());
    }

    let candidate_plan =
        isolated_w30_plan(state.runtime.w30_preview.clone(), bpm, start_beat as f64);
    let candidate = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &candidate_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("pitch-dive candidate render produced no output")?;
    let partition_control =
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
            &[RuntimeMixRenderSequenceStep::new(
                &candidate_plan,
                frame_count,
            )],
            SAMPLE_RATE,
            CHANNEL_COUNT,
            257,
        )
        .pop()
        .ok_or("pitch-dive partition-control render produced no output")?;

    if candidate.samples != partition_control.samples {
        return Err("pitch dive changed across callback partitions".into());
    }
    if candidate.samples[..beat_eight] != control.samples[..beat_eight] {
        return Err("pitch dive changed the frozen first eight beats".into());
    }
    let dive_delta = signal_delta_metrics(
        &control.samples[beat_eight..beat_twelve],
        &candidate.samples[beat_eight..beat_twelve],
    );
    if dive_delta.rms <= 0.001 {
        return Err("pitch-dive window collapsed into control".into());
    }
    if candidate.samples[beat_twelve..]
        .iter()
        .any(|sample| *sample != 0.0)
    {
        return Err("pitch dive did not enter explicit silence at beat twelve".into());
    }
    if signal_metrics(&control.samples[beat_twelve..]).active_samples == 0 {
        return Err("pitch-dive terminal silence has no active control counterfactual".into());
    }
    for (label, output) in [("control", &control), ("candidate", &candidate)] {
        if output.limiter.pre.clip_count != 0
            || output.limiter.limited_sample_count != 0
            || output.limiter.post.clip_count != 0
        {
            return Err(format!("{label} pitch-dive render clipped or invoked the limiter").into());
        }
    }

    let mut missing_source_render = state.runtime.w30_preview.clone();
    missing_source_render.pad_playback = None;
    missing_source_render.source_window_preview = None;
    missing_source_render.routing = riotbox_audio::w30::W30PreviewRenderRouting::Silent;
    let missing_source_plan = isolated_w30_plan(missing_source_render, bpm, start_beat as f64);
    let missing_source = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &missing_source_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("pitch-dive missing-source render produced no output")?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("pitch-dive missing source emitted fallback audio".into());
    }

    write_interleaved_pcm16_wav(
        output_dir.join("05_w30_pitch_dive_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control.samples,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("06_w30_pitch_dive_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate.samples,
    )?;
    let result = serde_json::json!({
        "schema": "riotbox.w30_pitch_dive_qualification_case.v1",
        "mechanism": "w30_pitch_dive_v1",
        "exact_product_tempo_bpm": bpm,
        "committed_start_beat": start_beat,
        "isolated_contributors": ["w30_preview"],
        "control": limiter_metrics(&control),
        "candidate": limiter_metrics(&candidate),
        "final_four_beat_delta_rms": dive_delta.rms,
        "first_eight_beats_sample_exact": true,
        "silence_from_beat_twelve": true,
        "callback_partition_128_vs_257_sample_exact": true,
        "capture_lineage_unchanged": true,
        "grit_unchanged": true,
        "source_monitor_unchanged": true,
        "missing_source_active_samples": missing_source.limiter.post.active_samples,
        "session_round_trip_and_replay_equivalence": "source_blind_automated_tests",
        "formal_review_candidate": prepare_review
    });
    fs::write(
        output_dir.join("pitch-dive-qualification.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("pitch-dive qualification: {result}");
    Ok(())
}

fn limiter_metrics(output: &riotbox_audio::runtime::RuntimeMixRenderOutput) -> serde_json::Value {
    serde_json::json!({
        "peak_abs": output.limiter.post.peak_abs,
        "rms": output.limiter.post.rms,
        "pre_limiter_clip_count": output.limiter.pre.clip_count,
        "limited_sample_count": output.limiter.limited_sample_count,
        "post_limiter_clip_count": output.limiter.post.clip_count
    })
}

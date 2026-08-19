use std::{fs, path::Path};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::{
        RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        signal_delta_metrics,
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

pub(crate) fn qualify_filter_slam_v1(
    state: &mut JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
    scene_id: Option<SceneId>,
    prepare_review: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    const START_BEAT: u64 = 8;
    const CONTROL_BEATS: f32 = 9.0;
    const RETURN_SECONDS: f32 = 0.02;
    let frame_count = (CONTROL_BEATS * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let beat_seven = qualification_sample_offset_at_beat_boundary(
        START_BEAT as f64,
        START_BEAT as f64 + 7.0,
        bpm,
    )?;
    let return_sample = beat_seven.saturating_add(
        (RETURN_SECONDS * SAMPLE_RATE as f32).round() as usize * usize::from(CHANNEL_COUNT),
    );
    let control_plan = isolated_w30_plan(control_render.clone(), bpm, START_BEAT as f64);
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
    .ok_or("filter-slam control render produced no output")?;

    let captures_before = state.session.captures.clone();
    let grit_before = state.session.runtime_state.macro_state.w30_grit;
    let music_level_before = state.session.runtime_state.mixer_state.music_level;
    let source_monitor_before = state.session.runtime_state.source_monitor.clone();
    let mc202_before = state.session.runtime_state.lane_state.mc202.clone();
    let tr909_before = state.session.runtime_state.lane_state.tr909.clone();
    if state.queue_w30_filter_slam(410) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 filter slam was unavailable".into());
    }
    commit(state, CommitBoundary::Bar, START_BEAT, 3, 1, scene_id, 500)?;
    let articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .as_ref()
        .ok_or("filter-slam commit produced no Session articulation")?;
    if articulation.profile != W30HookArticulationProfileState::FilterSlamV1
        || articulation.started_at_beat != START_BEAT
        || articulation.capture_id.to_string()
            != state
                .runtime
                .w30_preview
                .capture_id
                .as_deref()
                .unwrap_or_default()
    {
        return Err("filter-slam Session profile, target, or start beat diverged".into());
    }
    if state.session.captures != captures_before
        || state.session.runtime_state.macro_state.w30_grit != grit_before
        || state.session.runtime_state.mixer_state.music_level != music_level_before
        || state.session.runtime_state.source_monitor != source_monitor_before
        || state.session.runtime_state.lane_state.mc202 != mc202_before
        || state.session.runtime_state.lane_state.tr909 != tr909_before
    {
        return Err("filter slam changed frozen capture, mix, or another lane".into());
    }

    let candidate_plan =
        isolated_w30_plan(state.runtime.w30_preview.clone(), bpm, START_BEAT as f64);
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
    .ok_or("filter-slam candidate render produced no output")?;
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
        .ok_or("filter-slam partition-control render produced no output")?;

    if candidate.samples != partition_control.samples {
        return Err("filter slam changed across callback partitions".into());
    }
    if return_sample >= candidate.samples.len() || return_sample >= control.samples.len() {
        return Err("filter-slam return boundary exceeded render".into());
    }
    if candidate.samples[return_sample..] != control.samples[return_sample..] {
        return Err("filter slam did not return sample-exactly after twenty milliseconds".into());
    }
    let effect_delta = signal_delta_metrics(
        &control.samples[..return_sample],
        &candidate.samples[..return_sample],
    );
    if effect_delta.rms <= 0.001 {
        return Err("filter-slam window collapsed into control".into());
    }
    for (label, output) in [("control", &control), ("candidate", &candidate)] {
        if output.limiter.pre.clip_count != 0
            || output.limiter.limited_sample_count != 0
            || output.limiter.post.clip_count != 0
        {
            return Err(
                format!("{label} filter-slam render clipped or invoked the limiter").into(),
            );
        }
    }

    let mut missing_source_render = state.runtime.w30_preview.clone();
    missing_source_render.pad_playback = None;
    missing_source_render.source_window_preview = None;
    missing_source_render.routing = riotbox_audio::w30::W30PreviewRenderRouting::Silent;
    let missing_source_plan = isolated_w30_plan(missing_source_render, bpm, START_BEAT as f64);
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
    .ok_or("filter-slam missing-source render produced no output")?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("filter-slam missing source emitted fallback audio".into());
    }

    write_interleaved_pcm16_wav(
        output_dir.join("05_w30_filter_slam_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control.samples,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("06_w30_filter_slam_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate.samples,
    )?;
    let result = serde_json::json!({
        "schema": "riotbox.w30_filter_slam_qualification_case.v1",
        "mechanism": "w30_filter_slam_v1",
        "exact_product_tempo_bpm": bpm,
        "committed_start_beat": START_BEAT,
        "isolated_contributors": ["w30_preview"],
        "control": limiter_metrics(&control),
        "candidate": limiter_metrics(&candidate),
        "effect_through_return_delta_rms": effect_delta.rms,
        "return_after_seconds": RETURN_SECONDS,
        "sample_exact_after_return": true,
        "callback_partition_128_vs_257_sample_exact": true,
        "capture_lineage_unchanged": true,
        "grit_unchanged": true,
        "music_bus_level_unchanged": true,
        "source_monitor_unchanged": true,
        "other_lanes_unchanged": true,
        "missing_source_active_samples": missing_source.limiter.post.active_samples,
        "session_round_trip_and_replay_equivalence": "source_blind_automated_tests",
        "formal_review_candidate": prepare_review
    });
    fs::write(
        output_dir.join("filter-slam-qualification.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("filter-slam qualification: {result}");
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

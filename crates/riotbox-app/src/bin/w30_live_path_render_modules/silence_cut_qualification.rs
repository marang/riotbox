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

pub(crate) fn qualify_silence_cut_v1(
    state: &mut JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
    scene_id: Option<SceneId>,
    prepare_review: bool,
    start_beat: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    const CONTROL_BEATS: f32 = 6.0;
    const FADE_SECONDS: f32 = 0.005;
    const MINIMUM_EFFECT_DELTA_RMS: f32 = 0.001;
    const MAXIMUM_ABSOLUTE_BOUNDARY_DELTA: f32 = 0.02;

    let frame_count = (CONTROL_BEATS * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let cut_start = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 4.0,
        bpm,
    )?;
    let cut_end = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 5.0,
        bpm,
    )?;
    let fade_samples =
        (FADE_SECONDS * SAMPLE_RATE as f32).round() as usize * usize::from(CHANNEL_COUNT);
    let fade_out_start = cut_start
        .checked_sub(fade_samples)
        .ok_or("silence-cut fade-out precedes render start")?;
    let fade_in_end = cut_end.saturating_add(fade_samples);

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
    .ok_or("silence-cut control render produced no output")?;

    let captures_before = state.session.captures.clone();
    let grit_before = state.session.runtime_state.macro_state.w30_grit;
    let music_level_before = state.session.runtime_state.mixer_state.music_level;
    let source_monitor_before = state.session.runtime_state.source_monitor.clone();
    let mc202_before = state.session.runtime_state.lane_state.mc202.clone();
    let tr909_before = state.session.runtime_state.lane_state.tr909.clone();
    let requested_at = 400 + start_beat * 10;
    if state.queue_w30_silence_cut(requested_at) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 silence cut was unavailable".into());
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
        .ok_or("silence-cut commit produced no Session articulation")?;
    if articulation.profile != W30HookArticulationProfileState::SilenceCutV1
        || articulation.started_at_beat != start_beat
        || articulation.capture_id.to_string()
            != state
                .runtime
                .w30_preview
                .capture_id
                .as_deref()
                .unwrap_or_default()
    {
        return Err("silence-cut Session profile, target, or start beat diverged".into());
    }
    if state.session.captures != captures_before
        || state.session.runtime_state.macro_state.w30_grit != grit_before
        || state.session.runtime_state.mixer_state.music_level != music_level_before
        || state.session.runtime_state.source_monitor != source_monitor_before
        || state.session.runtime_state.lane_state.mc202 != mc202_before
        || state.session.runtime_state.lane_state.tr909 != tr909_before
    {
        return Err("silence cut changed frozen capture, mix, or another lane".into());
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
    .ok_or("silence-cut candidate render produced no output")?;
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
        .ok_or("silence-cut partition-control render produced no output")?;

    if candidate.samples != partition_control.samples {
        return Err("silence cut changed across callback partitions".into());
    }
    if fade_in_end >= candidate.samples.len() || fade_in_end >= control.samples.len() {
        return Err("silence-cut return boundary exceeded render".into());
    }
    if candidate.samples[..fade_out_start] != control.samples[..fade_out_start] {
        return Err("silence cut changed playback before its frozen fade-out".into());
    }
    if candidate.samples[cut_start..cut_end]
        .iter()
        .any(|sample| *sample != 0.0)
    {
        return Err("silence cut was not exact PCM zero for one beat".into());
    }
    let control_cut_metrics = signal_metrics(&control.samples[cut_start..cut_end]);
    if control_cut_metrics.active_samples == 0 {
        return Err("silence cut has no active control counterfactual".into());
    }
    if candidate.samples[fade_in_end..] != control.samples[fade_in_end..] {
        return Err("silence cut did not return sample-exactly after five milliseconds".into());
    }
    let effect_delta = signal_delta_metrics(
        &control.samples[fade_out_start..fade_in_end],
        &candidate.samples[fade_out_start..fade_in_end],
    );
    if effect_delta.rms <= MINIMUM_EFFECT_DELTA_RMS {
        return Err("silence-cut window collapsed into control".into());
    }
    for boundary in [fade_out_start, cut_start, cut_end, fade_in_end] {
        let candidate_delta = maximum_boundary_delta(&candidate.samples, boundary);
        let control_delta = maximum_boundary_delta(&control.samples, boundary);
        if candidate_delta > control_delta.max(MAXIMUM_ABSOLUTE_BOUNDARY_DELTA) {
            return Err(format!("silence-cut boundary at sample {boundary} clicked").into());
        }
    }
    for (label, output) in [("control", &control), ("candidate", &candidate)] {
        if output.limiter.pre.clip_count != 0
            || output.limiter.limited_sample_count != 0
            || output.limiter.post.clip_count != 0
        {
            return Err(
                format!("{label} silence-cut render clipped or invoked the limiter").into(),
            );
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
    .ok_or("silence-cut missing-source render produced no output")?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("silence-cut missing source emitted fallback audio".into());
    }

    write_interleaved_pcm16_wav(
        output_dir.join("05_w30_silence_cut_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control.samples,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("06_w30_silence_cut_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate.samples,
    )?;
    let result = serde_json::json!({
        "schema": "riotbox.w30_silence_cut_qualification_case.v1",
        "mechanism": "w30_choke_silence_cut_v1",
        "exact_product_tempo_bpm": bpm,
        "committed_start_beat": start_beat,
        "isolated_contributors": ["w30_preview"],
        "control": limiter_metrics(&control),
        "candidate": limiter_metrics(&candidate),
        "effect_window_delta_rms": effect_delta.rms,
        "control_cut_window_active_samples": control_cut_metrics.active_samples,
        "fade_milliseconds": FADE_SECONDS * 1000.0,
        "silence_cut_beats": 1.0,
        "prefix_before_fade_sample_exact": true,
        "one_beat_cut_pcm_zero": true,
        "sample_exact_after_five_ms_return": true,
        "boundary_delta_rule_passed": true,
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
        output_dir.join("silence-cut-qualification.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("silence-cut qualification: {result}");
    Ok(())
}

fn maximum_boundary_delta(samples: &[f32], boundary: usize) -> f32 {
    let channel_count = usize::from(CHANNEL_COUNT);
    if boundary < channel_count || boundary.saturating_add(channel_count) > samples.len() {
        return f32::INFINITY;
    }
    (0..channel_count)
        .map(|channel| {
            (samples[boundary + channel] - samples[boundary + channel - channel_count]).abs()
        })
        .fold(0.0, f32::max)
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

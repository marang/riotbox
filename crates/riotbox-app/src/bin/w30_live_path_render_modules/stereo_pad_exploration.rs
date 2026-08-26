use std::{fs, path::Path};

use riotbox_app::jam_app::JamAppState;
use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState, project_w30_stereo_pad_development_window,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        render_runtime_mix_w30_stereo_development_offline_with_report, signal_delta_metrics,
    },
    source_audio::write_interleaved_pcm16_wav,
    w30::W30PreviewRenderState,
};
use riotbox_core::action::SourceMonitorMode;
use serde_json::json;

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;
const BEATS_PER_BAR: f32 = 4.0;
const PRESENTATION_BARS: f32 = 4.0;

pub(crate) fn explore_stereo_pad_v1(
    state: &JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !bpm.is_finite() || bpm <= 0.0 {
        return Err("stereo-pad exploration requires positive finite BPM".into());
    }
    let pad = control_render
        .pad_playback
        .as_ref()
        .ok_or("stereo-pad exploration requires promoted W-30 pad audio")?;
    if !pad.loop_enabled
        || pad.reverse
        || (pad.playback_rate - 1.0).abs() > f32::EPSILON
        || pad.hook_articulation.is_some()
    {
        return Err("stereo-pad control no longer matches the frozen clean W-30 path".into());
    }
    let capture_id = control_render
        .capture_id
        .as_deref()
        .ok_or("stereo-pad control has no capture identity")?;
    let capture_cache = state
        .capture_audio_cache
        .iter()
        .find(|(id, _)| id.to_string() == capture_id)
        .map(|(_, cache)| cache)
        .ok_or("stereo-pad capture audio is unavailable")?;
    if capture_cache.channel_count != CHANNEL_COUNT {
        return Err("stereo-pad candidate requires an exact stereo capture".into());
    }
    let stereo = project_w30_stereo_pad_development_window(
        capture_cache.interleaved_samples(),
        usize::from(capture_cache.channel_count),
    )
    .ok_or("stereo-pad side projection refused the capture")?;
    if stereo.sample_count != pad.sample_count {
        return Err("stereo-pad side and mono projection lengths diverged".into());
    }

    let frame_count = (SAMPLE_RATE as f32 * 60.0 / bpm * BEATS_PER_BAR * PRESENTATION_BARS)
        .round()
        .max(1.0) as usize;
    let plan = isolated_w30_plan(control_render.clone(), bpm);
    let control_128 = render_control(&plan, frame_count, 128)?;
    let control_257 = render_control(&plan, frame_count, 257)?;
    let candidate_128 = render_runtime_mix_w30_stereo_development_offline_with_report(
        &plan,
        &stereo,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
    .ok_or("stereo-pad candidate render refused the frozen plan")?;
    let candidate_257 = render_runtime_mix_w30_stereo_development_offline_with_report(
        &plan,
        &stereo,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        257,
    )
    .ok_or("stereo-pad 257-frame render refused the frozen plan")?;
    let candidate_restart_128 = render_runtime_mix_w30_stereo_development_offline_with_report(
        &plan,
        &stereo,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
    .ok_or("stereo-pad restart render refused the frozen plan")?;
    if control_128.samples != control_257.samples || candidate_128.samples != candidate_257.samples
    {
        return Err("stereo-pad callback partitions diverged".into());
    }
    if candidate_128.samples != candidate_restart_128.samples {
        return Err("stereo-pad restart render diverged".into());
    }
    let mut missing_source_plan = plan.clone();
    missing_source_plan.w30_preview_render.pad_playback = None;
    missing_source_plan.w30_preview_render.source_window_preview = None;
    let missing_source = render_control(&missing_source_plan, frame_count.min(4_096), 128)?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("stereo-pad missing source emitted fallback audio".into());
    }
    for (label, output) in [("control", &control_128), ("candidate", &candidate_128)] {
        if output.limiter.pre.clip_count != 0
            || output.limiter.limited_sample_count != 0
            || output.limiter.post.clip_count != 0
        {
            return Err(format!("stereo-pad {label} clipped or invoked the limiter").into());
        }
    }
    let delta = signal_delta_metrics(&control_128.samples, &candidate_128.samples);
    if control_128.limiter.post.rms <= 0.001
        || candidate_128.limiter.post.rms <= 0.001
        || delta.rms < 0.001
    {
        return Err("stereo-pad render was silent or collapsed into the control".into());
    }

    write_interleaved_pcm16_wav(
        output_dir.join("01_w30_mono_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control_128.samples,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("02_w30_stereo_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate_128.samples,
    )?;
    fs::write(
        output_dir.join("stereo-pad-runtime-report.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "riotbox.w30_stereo_pad_runtime.v1",
            "ticket": "RIOTBOX-1469",
            "result": "pass",
            "sample_rate_hz": SAMPLE_RATE,
            "channels": CHANNEL_COUNT,
            "bpm": bpm,
            "presentation_bars": PRESENTATION_BARS,
            "frame_count": frame_count,
            "control": "mono_folded_v1",
            "candidate": "stereo_preserved_v1",
            "stereo_storage": "exact_mono_plus_symmetric_side",
            "callback_frame_counts": [128, 257],
            "callback_outputs_sample_exact": true,
            "restart_outputs_sample_exact": true,
            "missing_source_silence": true,
            "side_sample_count": stereo.sample_count,
            "control_metrics": {
                "rms": control_128.limiter.post.rms,
                "peak_abs": control_128.limiter.post.peak_abs,
                "crest_factor": control_128.limiter.post.crest_factor,
                "active_samples": control_128.limiter.post.active_samples,
                "clip_count": control_128.limiter.post.clip_count
            },
            "candidate_metrics": {
                "rms": candidate_128.limiter.post.rms,
                "peak_abs": candidate_128.limiter.post.peak_abs,
                "crest_factor": candidate_128.limiter.post.crest_factor,
                "active_samples": candidate_128.limiter.post.active_samples,
                "clip_count": candidate_128.limiter.post.clip_count
            },
            "control_candidate_delta": {
                "rms": delta.rms,
                "peak_abs": delta.peak_abs,
                "crest_factor": delta.crest_factor
            },
            "limiter": {
                "control_limited_sample_count": control_128.limiter.limited_sample_count,
                "candidate_limited_sample_count": candidate_128.limiter.limited_sample_count
            },
            "audible_lanes": ["w30"],
            "additional_effect_count": 0
        }))?,
    )?;
    Ok(())
}

fn render_control(
    plan: &RuntimeMixRenderPlan,
    frame_count: usize,
    callback_frame_count: usize,
) -> Result<riotbox_audio::runtime::RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        callback_frame_count,
    )
    .pop()
    .ok_or_else(|| "stereo-pad control render produced no output".into())
}

fn isolated_w30_plan(render: W30PreviewRenderState, bpm: f32) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: render,
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
    }
}

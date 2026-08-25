use std::{fs, path::Path};

use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, SourceMonitorRenderState,
        render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
    },
    source_audio::write_interleaved_pcm16_wav,
    w30::{W30PadPlaybackGrammar, W30PreviewRenderState},
};
use riotbox_core::action::SourceMonitorMode;
use serde_json::json;

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;
const BEATS_PER_BAR: f32 = 4.0;
const PRESENTATION_BARS: f32 = 4.0;
const CONTRACT_SHA256: &str = "ac71e8daa9f862a8341910d63e0457cd657e6506808eda4032d132b4fb443517";

pub(crate) fn explore_source_native_full_bar_v1(
    control_render: &W30PreviewRenderState,
    declared_bpm: f32,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !declared_bpm.is_finite() || declared_bpm <= 0.0 {
        return Err("source-native full-bar exploration requires positive finite BPM".into());
    }
    let control_pad = control_render
        .pad_playback
        .as_ref()
        .ok_or("source-native full-bar exploration requires promoted W-30 pad audio")?;
    if control_pad.playback_grammar != W30PadPlaybackGrammar::HalfBeatChopV1
        || !control_pad.loop_enabled
        || (control_pad.playback_rate - 1.0).abs() > f32::EPSILON
        || control_pad.reverse
        || control_pad.gate_step_fraction.abs() > f32::EPSILON
        || control_pad.hook_articulation.is_some()
    {
        return Err(
            "source-native full-bar control no longer matches the frozen clean W-30 path".into(),
        );
    }
    if control_pad.source_sample_rate == 0 || control_pad.playback_frame_count == 0 {
        return Err("source-native full-bar capture has no timing identity".into());
    }
    let bpm = control_pad.source_sample_rate as f64 * 60.0 * f64::from(BEATS_PER_BAR)
        / control_pad.playback_frame_count as f64;
    if !bpm.is_finite() || bpm <= 0.0 {
        return Err("source-native full-bar confirmed capture BPM is invalid".into());
    }
    let bpm = bpm as f32;
    let expected_bar_frames =
        (control_pad.source_sample_rate as f32 * 60.0 / bpm * BEATS_PER_BAR).round() as u64;
    let duration_mismatch = control_pad
        .playback_frame_count
        .abs_diff(expected_bar_frames);
    if duration_mismatch > 1 {
        return Err(format!(
            "captured source bar differs from runtime bar by {duration_mismatch} frames"
        )
        .into());
    }

    let mut candidate_render = control_render.clone();
    candidate_render
        .pad_playback
        .as_mut()
        .ok_or("candidate lost promoted W-30 pad audio")?
        .playback_grammar = W30PadPlaybackGrammar::SourceNativeFullBarV1;

    let frame_count = (SAMPLE_RATE as f32 * 60.0 / bpm * BEATS_PER_BAR * PRESENTATION_BARS)
        .round()
        .max(1.0) as usize;
    let control_128 = render_w30_only(control_render, bpm, frame_count, 128);
    let control_257 = render_w30_only(control_render, bpm, frame_count, 257);
    let candidate_128 = render_w30_only(&candidate_render, bpm, frame_count, 128);
    let candidate_257 = render_w30_only(&candidate_render, bpm, frame_count, 257);
    if control_128 != control_257 || candidate_128 != candidate_257 {
        return Err("source-native full-bar callback partitions diverged".into());
    }

    let control_metrics = signal_metrics(&control_128);
    let candidate_metrics = signal_metrics(&candidate_128);
    let delta = signal_delta_metrics(&control_128, &candidate_128);
    if control_metrics.rms <= 0.001
        || candidate_metrics.rms <= 0.001
        || control_metrics.peak_abs >= 0.99
        || candidate_metrics.peak_abs >= 0.99
        || delta.rms <= 0.001
    {
        return Err(
            "source-native full-bar render was silent, clipped, or indistinguishable".into(),
        );
    }

    write_interleaved_pcm16_wav(
        output_dir.join("01_w30_half_beat_chop_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control_128,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("02_w30_source_native_full_bar_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate_128,
    )?;
    fs::write(
        output_dir.join("source-native-full-bar-runtime-report.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "riotbox.dense_break_source_native_bar_runtime.v1",
            "ticket": "RIOTBOX-1468",
            "contract_sha256": CONTRACT_SHA256,
            "result": "pass",
            "bpm": bpm,
            "declared_bpm_metadata_only": declared_bpm,
            "runtime_tempo_authority": "confirmed_capture_bar_bpm_v1",
            "sample_rate_hz": SAMPLE_RATE,
            "channels": CHANNEL_COUNT,
            "presentation_bars": PRESENTATION_BARS,
            "frame_count": frame_count,
            "source_bar_frame_count": control_pad.playback_frame_count,
            "expected_runtime_bar_frame_count": expected_bar_frames,
            "bar_duration_mismatch_frames": duration_mismatch,
            "control_grammar": "half_beat_chop_v1",
            "candidate_grammar": "source_native_full_bar_v1",
            "callback_frame_counts": [128, 257],
            "callback_outputs_sample_exact": true,
            "control_metrics": {
                "rms": control_metrics.rms,
                "peak_abs": control_metrics.peak_abs,
                "crest_factor": control_metrics.crest_factor
            },
            "candidate_metrics": {
                "rms": candidate_metrics.rms,
                "peak_abs": candidate_metrics.peak_abs,
                "crest_factor": candidate_metrics.crest_factor
            },
            "control_candidate_delta": {
                "rms": delta.rms,
                "peak_abs": delta.peak_abs,
                "crest_factor": delta.crest_factor
            },
            "lane_count": 1,
            "lane": "w30",
            "additional_effect_count": 0
        }))?,
    )?;
    Ok(())
}

fn render_w30_only(
    render: &W30PreviewRenderState,
    bpm: f32,
    frame_count: usize,
    callback_frames: usize,
) -> Vec<f32> {
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: render.clone(),
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
    };
    render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        callback_frames,
    )
}

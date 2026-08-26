use std::{fs, path::Path};

use riotbox_app::jam_app::JamAppState;
use riotbox_audio::{
    runtime::{
        RuntimeMixRenderOutput, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report, signal_metrics,
    },
    source_audio::write_interleaved_pcm16_wav,
    w30::{W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState},
};
use riotbox_core::action::{ActionCommand, ActionStatus};

use super::super::{CHANNEL_COUNT, SAMPLE_RATE, isolated_w30_plan};

const START_BEAT: f64 = 8.0;
const RENDER_BEATS: f32 = 13.0;

pub(crate) fn qualify_dense_w30_foundation_v1(
    state: &JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_product_state(state, control_render)?;
    let frame_count = (RENDER_BEATS * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let control = render_foundation(control_render, bpm, frame_count, 128)?;
    let partition = render_foundation(control_render, bpm, frame_count, 257)?;
    let restart = render_foundation(control_render, bpm, frame_count, 128)?;
    if control.samples != partition.samples {
        return Err("Dense W-30 foundation changed across callback partitions".into());
    }
    if control.samples != restart.samples {
        return Err("Dense W-30 foundation changed after exact restart".into());
    }
    require_clean_output("control", &control)?;
    if signal_metrics(&control.samples).active_samples == 0 {
        return Err("Dense W-30 foundation control was silent".into());
    }

    let mut missing_render = control_render.clone();
    missing_render.pad_playback = None;
    missing_render.source_window_preview = None;
    missing_render.routing = W30PreviewRenderRouting::Silent;
    let missing_source = render_foundation(&missing_render, bpm, frame_count, 128)?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("Dense W-30 foundation missing source emitted fallback audio".into());
    }

    let audio_path = output_dir.join("05_w30_dense_foundation_control.wav");
    write_interleaved_pcm16_wav(&audio_path, SAMPLE_RATE, CHANNEL_COUNT, &control.samples)?;
    let pad = control_render
        .pad_playback
        .as_ref()
        .ok_or("Dense W-30 foundation has no promoted pad window")?;
    let metrics = signal_metrics(&control.samples);
    let report = serde_json::json!({
        "schema": "riotbox.dense_w30_foundation_runtime.v1",
        "ticket": "RIOTBOX-1470",
        "product_path": "ordinary_promoted_w30_control_v1",
        "render": {
            "start_beat": START_BEAT,
            "beat_count": RENDER_BEATS,
            "sample_rate_hz": SAMPLE_RATE,
            "channel_count": CHANNEL_COUNT,
            "frame_count": frame_count,
            "callback_partitions_sample_exact": true,
            "restart_sample_exact": true,
            "isolated_contributors": ["w30_preview"],
            "peak_abs": metrics.peak_abs,
            "rms": metrics.rms,
            "active_samples": metrics.active_samples,
            "pre_limiter_clip_count": control.limiter.pre.clip_count,
            "limited_sample_count": control.limiter.limited_sample_count,
            "post_limiter_clip_count": control.limiter.post.clip_count,
            "missing_source_active_samples": missing_source.limiter.post.active_samples
        },
        "w30": {
            "mode": control_render.mode.label(),
            "routing": control_render.routing.label(),
            "source_profile": control_render.source_profile.map(|value| value.label()),
            "active_bank_id": control_render.active_bank_id.as_deref(),
            "focused_pad_id": control_render.focused_pad_id.as_deref(),
            "capture_id": control_render.capture_id.as_deref(),
            "trigger_revision": control_render.trigger_revision,
            "trigger_velocity": control_render.trigger_velocity,
            "music_bus_level": control_render.music_bus_level,
            "grit_level": control_render.grit_level,
            "tempo_bpm": control_render.tempo_bpm,
            "source_start_frame": pad.source_start_frame,
            "source_end_frame": pad.source_end_frame,
            "source_sample_rate": pad.source_sample_rate,
            "playback_frame_count": pad.playback_frame_count,
            "sample_count": pad.sample_count,
            "loop_enabled": pad.loop_enabled,
            "playback_rate": pad.playback_rate,
            "reverse": pad.reverse,
            "gate_step_fraction": pad.gate_step_fraction,
            "loop_crossfade_sample_count": pad.loop_crossfade_sample_count,
            "chop_slice_count": pad.chop_slice_count,
            "chop_slice_starts": pad.chop_slice_starts[..pad.chop_slice_count.min(pad.chop_slice_starts.len())]
        },
        "lane_roles": {
            "w30": "source_transform_foundation",
            "tr909": "stay_out",
            "mc202": "stay_out",
            "source_monitor": "stay_out"
        },
        "session_proof": {
            "expected_action_prefix_committed": true,
            "capture_lineage_present": true,
            "w30_articulation_absent": true,
            "tr909_pattern_absent": true,
            "mc202_phrase_absent": true
        }
    });
    fs::write(
        output_dir.join("dense-w30-foundation-runtime.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("Dense W-30 foundation qualification: {report}");
    Ok(())
}

fn validate_product_state(
    state: &JamAppState,
    control_render: &W30PreviewRenderState,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = [
        ActionCommand::SourceTimingConfirmGrid,
        ActionCommand::PresetActivate,
        ActionCommand::CaptureSetLength,
        ActionCommand::CaptureBarGroup,
        ActionCommand::PromoteCaptureToPad,
        ActionCommand::W30TriggerPad,
    ];
    let actions = &state.session.action_log.actions;
    if actions.len() != expected.len()
        || actions.iter().zip(expected).any(|(action, command)| {
            action.command != command || action.status != ActionStatus::Committed
        })
    {
        return Err("Dense W-30 foundation action/commit path diverged".into());
    }
    if state.session.captures.len() != 1
        || state.session.captures[0].source_window.is_none()
        || state.session.captures[0].assigned_target.is_none()
    {
        return Err("Dense W-30 foundation capture lineage is incomplete".into());
    }
    let lanes = &state.session.runtime_state.lane_state;
    if lanes.w30.hook_articulation.is_some()
        || lanes.tr909.pattern_ref.is_some()
        || lanes.tr909.takeover_enabled
        || lanes.tr909.slam_enabled
        || lanes.tr909.fill_armed_next_bar
        || lanes.mc202.role.is_some()
        || lanes.mc202.phrase_ref.is_some()
        || lanes.mc202.phrase_variant.is_some()
        || lanes.mc202.source_phrase_plan.is_some()
    {
        return Err("Dense W-30 foundation support lanes or articulation did not stay out".into());
    }
    if control_render.mode != W30PreviewRenderMode::LiveRecall
        || control_render.routing != W30PreviewRenderRouting::MusicBusPreview
        || control_render.pad_playback.is_none()
        || control_render
            .pad_playback
            .as_ref()
            .is_some_and(|pad| pad.hook_articulation.is_some())
    {
        return Err("Dense W-30 foundation control render is not ordinary promoted recall".into());
    }
    Ok(())
}

fn render_foundation(
    render: &W30PreviewRenderState,
    bpm: f32,
    frame_count: usize,
    callback_frames: usize,
) -> Result<RuntimeMixRenderOutput, Box<dyn std::error::Error>> {
    let plan = isolated_w30_plan(render.clone(), bpm, START_BEAT);
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        callback_frames,
    )
    .pop()
    .ok_or_else(|| "Dense W-30 foundation render produced no output".into())
}

fn require_clean_output(
    label: &str,
    output: &RuntimeMixRenderOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    if output.limiter.pre.clip_count != 0
        || output.limiter.limited_sample_count != 0
        || output.limiter.post.clip_count != 0
    {
        return Err(format!("{label} Dense W-30 foundation clipped or invoked the limiter").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use riotbox_audio::w30::{
        W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30PadPlaybackSampleWindow,
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile,
    };

    use super::{SAMPLE_RATE, render_foundation};

    fn synthetic_render() -> W30PreviewRenderState {
        let mut pad = W30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: 2_048,
            source_sample_rate: SAMPLE_RATE,
            playback_frame_count: 2_048,
            sample_count: 2_048,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 0,
            chop_slice_count: 1,
            chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
            hook_articulation: None,
            samples: [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        };
        for (index, sample) in pad.samples.iter_mut().take(pad.sample_count).enumerate() {
            let phase = index as f32 * std::f32::consts::TAU / 73.0;
            *sample = phase.sin() * 0.24 + (phase * 0.37).sin() * 0.08;
        }
        W30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-01".into()),
            trigger_revision: 1,
            trigger_velocity: 0.84,
            pad_playback: Some(pad),
            music_bus_level: 0.58,
            grit_level: 0.6888,
            is_transport_running: true,
            tempo_bpm: 130.0,
            ..W30PreviewRenderState::default()
        }
    }

    #[test]
    fn dense_foundation_is_partition_and_restart_exact() {
        let render = synthetic_render();
        let first = render_foundation(&render, 130.0, 16_000, 128).expect("first render");
        let partition = render_foundation(&render, 130.0, 16_000, 257).expect("partition render");
        let restart = render_foundation(&render, 130.0, 16_000, 128).expect("restart render");
        assert_eq!(first.samples, partition.samples);
        assert_eq!(first.samples, restart.samples);
        assert!(first.limiter.post.active_samples > 0);
    }

    #[test]
    fn dense_foundation_missing_source_is_silent() {
        let mut render = synthetic_render();
        render.pad_playback = None;
        render.routing = W30PreviewRenderRouting::Silent;
        let output = render_foundation(&render, 130.0, 4_000, 128).expect("missing render");
        assert_eq!(output.limiter.post.active_samples, 0);
    }
}

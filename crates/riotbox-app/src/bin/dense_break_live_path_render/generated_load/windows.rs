use std::{error::Error, fs, path::Path};

use riotbox_app::jam_app::{JamAppState, LiveMasterRecordingQueueResult};
use riotbox_audio::{
    runtime::{
        AudioOutputInfo, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report, signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::action::LiveRecordingDuration;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{contract::Contract, file_hash};

#[derive(Serialize)]
pub(super) struct WindowReport {
    sample_rate_hz: u32,
    channel_count: u16,
    duration_bars: u8,
    frame_count: usize,
    requested_start_position_beats: f64,
    generated_output_geometry: bool,
    recording_plan_admitted: bool,
    finite_samples: bool,
    active_samples: usize,
    peak_abs: Option<f32>,
    rms: Option<f32>,
    clip_count: usize,
    limiter_applied_sample_count: usize,
    expected_sample_count: bool,
    artifact_path: String,
    artifact_sha256: Option<String>,
    artifact_error: Option<String>,
    pattern_origin: &'static str,
    pub(super) passed: bool,
}

struct RenderWindow {
    rate: u32,
    channels: u16,
    duration: LiveRecordingDuration,
    frames: usize,
    start_beats: f64,
}

pub(super) fn measure(
    root: &Path,
    contract: &Contract,
    rate: u32,
    duration: LiveRecordingDuration,
    expected_session: &[u8],
    expected_file_hash: &str,
) -> Result<WindowReport, Box<dyn Error>> {
    if file_hash(&root.join("session.json"))? != expected_file_hash {
        return Err("saved generated Session changed before window preparation".into());
    }
    let mut state = JamAppState::from_json_files(root.join("session.json"), Option::<&Path>::None)?;
    let capture = state
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
        .ok_or("ready capture missing during window preparation")?;
    if serde_json::to_vec(&state.session)? != expected_session
        || state.source_audio_cache.is_none()
        || !state.capture_audio_cache.contains_key(capture)
    {
        return Err("generated identity/hydration changed during window preparation".into());
    }
    let output = AudioOutputInfo {
        host_name: "generated-output-geometry-not-a-host".into(),
        device_name: "fixture-no-device".into(),
        sample_format: "F32".into(),
        sample_rate: rate,
        channel_count: contract.output_channels,
        buffer_size: format!(
            "generated {} frames, not negotiated",
            contract.offline_callback_frames
        ),
        supported_output_config_count: None,
    };
    let target = root.join(format!(
        "prospective-live-{rate}-{}bars.wav",
        duration.bars()
    ));
    let plan =
        match state.queue_live_master_recording_with_duration(5_000, &output, &target, duration)? {
            LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
            LiveMasterRecordingQueueResult::Rejected { reason } => return Err(reason.into()),
            LiveMasterRecordingQueueResult::AlreadyPending => {
                return Err("unexpected pending recording in the saved fixture".into());
            }
        };
    let render_plan = crate::ready_prefix::render_plan(
        &state,
        plan.confirmed_bpm,
        plan.requested_start_position_beats,
    );
    let frames = plan.request.target_frame_count;
    let rendered = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&render_plan, frames)],
        rate,
        contract.output_channels,
        contract.offline_callback_frames,
    )
    .pop()
    .ok_or("generated recording window produced no render")?;
    let artifact = root
        .join("offline")
        .join(format!("master-{rate}-{}bars.wav", duration.bars()));
    Ok(report_render(
        &artifact,
        RenderWindow {
            rate,
            channels: contract.output_channels,
            duration,
            frames,
            start_beats: plan.requested_start_position_beats,
        },
        &rendered.samples,
        rendered.limiter.limited_sample_count,
    ))
}

// Computed evidence survives artifact errors; the caller persists it before
// checking aggregate acceptance. This is never an actual callback receipt.
fn report_render(
    artifact: &Path,
    window: RenderWindow,
    samples: &[f32],
    limiter_applied_sample_count: usize,
) -> WindowReport {
    let RenderWindow {
        rate,
        channels,
        duration,
        frames,
        start_beats,
    } = window;
    let finite = samples.iter().all(|sample| sample.is_finite());
    let metrics = signal_metrics(samples);
    let expected_count = Some(samples.len()) == frames.checked_mul(usize::from(channels));
    let publication = if finite && expected_count {
        publish_offline_wave(artifact, rate, channels, samples)
    } else {
        Err("invalid computed payload was not written".into())
    };
    let (artifact_sha256, artifact_error) = match publication {
        Ok(hash) => (Some(hash), None),
        Err(error) => (None, Some(error)),
    };
    let artifact_written = artifact_sha256.is_some();
    WindowReport {
        sample_rate_hz: rate,
        channel_count: channels,
        duration_bars: duration.bars(),
        frame_count: frames,
        requested_start_position_beats: start_beats,
        generated_output_geometry: true,
        recording_plan_admitted: true,
        finite_samples: finite,
        active_samples: metrics.active_samples,
        peak_abs: metrics.peak_abs.is_finite().then_some(metrics.peak_abs),
        rms: metrics.rms.is_finite().then_some(metrics.rms),
        clip_count: metrics.clip_count,
        limiter_applied_sample_count,
        expected_sample_count: expected_count,
        artifact_path: artifact.to_string_lossy().into_owned(),
        artifact_sha256,
        artifact_error,
        pattern_origin: "fixture",
        passed: finite
            && expected_count
            && artifact_written
            && metrics.active_samples > 0
            && metrics.peak_abs > 0.0
            && metrics.peak_abs < 1.0
            && metrics.clip_count == 0,
    }
}

fn publish_offline_wave(
    path: &Path,
    rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<String, String> {
    let parent = path.parent().ok_or("offline artifact has no parent")?;
    // The existing writer may truncate, so give it only our exclusive temporary
    // file. Publication must never overwrite an occupied file or follow a link.
    let temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    write_interleaved_pcm16_wav(temporary.path(), rate, channels, samples)
        .map_err(|error| error.to_string())?;
    let bytes = fs::read(temporary.path()).map_err(|error| error.to_string())?;
    let decoded =
        SourceAudioCache::from_pcm_wav_bytes(path, &bytes).map_err(|error| error.to_string())?;
    if decoded.sample_rate != rate
        || decoded.channel_count != channels
        || decoded.interleaved_samples().len() != samples.len()
    {
        return Err("offline artifact readback disagrees with computed geometry".into());
    }
    let hash = format!("{:x}", Sha256::digest(&bytes));
    temporary
        .persist_noclobber(path)
        .map_err(|error| error.error.to_string())?;
    Ok(hash)
}

#[cfg(test)]
mod tests;

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Instant,
};

use crate::mc202::{
    Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
    Mc202RenderRouting, Mc202RenderState, render_mc202_buffer,
};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
    Tr909TakeoverRenderProfile,
};
use crate::w30::{
    W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PadPlaybackSampleWindow,
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSampleWindow,
    W30PreviewSourceProfile, W30ResampleTapMode, W30ResampleTapRouting,
    W30ResampleTapSourceProfile, W30ResampleTapState,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioRuntimeLifecycle {
    Idle,
    Running,
    Stopped,
    Faulted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutputInfo {
    pub host_name: String,
    pub device_name: String,
    pub sample_format: String,
    pub sample_rate: u32,
    pub channel_count: u16,
    pub buffer_size: String,
    pub supported_output_config_count: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioRuntimeHealth {
    pub lifecycle: AudioRuntimeLifecycle,
    pub output: Option<AudioOutputInfo>,
    pub callback_count: u64,
    pub max_callback_gap_micros: Option<u64>,
    pub stream_error_count: u64,
    pub last_stream_error: Option<String>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct OfflineAudioMetrics {
    pub active_samples: usize,
    pub peak_abs: f32,
    pub rms: f32,
    pub sum: f32,
    pub mean_abs: f32,
    pub zero_crossings: usize,
    pub crest_factor: f32,
    pub active_sample_ratio: f32,
    pub silence_ratio: f32,
    pub dc_offset: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AudioRuntimeTimingSnapshot {
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

#[derive(Debug)]
pub enum AudioRuntimeError {
    NoDefaultOutputDevice {
        host_name: String,
    },
    DefaultOutputConfig {
        host_name: String,
        device_name: String,
        reason: String,
    },
    UnsupportedSampleFormat {
        host_name: String,
        device_name: String,
        sample_format: String,
    },
    BuildStream {
        host_name: String,
        device_name: String,
        reason: String,
    },
    PlayStream {
        host_name: String,
        device_name: String,
        reason: String,
    },
}

impl AudioRuntimeError {
    #[must_use]
    pub fn host_name(&self) -> &str {
        match self {
            Self::NoDefaultOutputDevice { host_name }
            | Self::DefaultOutputConfig { host_name, .. }
            | Self::UnsupportedSampleFormat { host_name, .. }
            | Self::BuildStream { host_name, .. }
            | Self::PlayStream { host_name, .. } => host_name,
        }
    }

    #[must_use]
    pub fn device_name(&self) -> Option<&str> {
        match self {
            Self::NoDefaultOutputDevice { .. } => None,
            Self::DefaultOutputConfig { device_name, .. }
            | Self::UnsupportedSampleFormat { device_name, .. }
            | Self::BuildStream { device_name, .. }
            | Self::PlayStream { device_name, .. } => Some(device_name),
        }
    }
}

impl Display for AudioRuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDefaultOutputDevice { host_name } => {
                write!(f, "no default output device available on host {host_name}")
            }
            Self::DefaultOutputConfig {
                host_name,
                device_name,
                reason,
            } => write!(
                f,
                "failed to read default output config for device {device_name} on host {host_name}: {reason}"
            ),
            Self::UnsupportedSampleFormat {
                host_name,
                device_name,
                sample_format,
            } => write!(
                f,
                "unsupported sample format {sample_format} for device {device_name} on host {host_name}"
            ),
            Self::BuildStream {
                host_name,
                device_name,
                reason,
            } => write!(
                f,
                "failed to build output stream for device {device_name} on host {host_name}: {reason}"
            ),
            Self::PlayStream {
                host_name,
                device_name,
                reason,
            } => write!(
                f,
                "failed to start output stream for device {device_name} on host {host_name}: {reason}"
            ),
        }
    }
}

impl Error for AudioRuntimeError {}

#[must_use]
pub fn render_w30_preview_offline(
    render_state: &W30PreviewRenderState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let shared_state = SharedW30PreviewRenderState::new(render_state);
    let mut callback_state = W30PreviewCallbackState::default();
    let mut buffer = vec![0.0; frame_count.saturating_mul(usize::from(channel_count))];

    render_w30_preview_buffer(
        &mut buffer,
        sample_rate,
        usize::from(channel_count),
        &shared_state.snapshot(),
        &mut callback_state,
    );

    buffer
}

#[must_use]
pub fn render_w30_resample_tap_offline(
    render_state: &W30ResampleTapState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let shared_state = SharedW30ResampleTapState::new(render_state);
    let mut callback_state = W30ResampleTapCallbackState::default();
    let mut buffer = vec![0.0; frame_count.saturating_mul(usize::from(channel_count))];

    render_w30_resample_tap_buffer(
        &mut buffer,
        sample_rate,
        usize::from(channel_count),
        &shared_state.snapshot(),
        &mut callback_state,
    );

    buffer
}

#[must_use]
pub fn render_tr909_offline(
    render_state: &Tr909RenderState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let shared_state = SharedTr909RenderState::new(render_state);
    let mut callback_state = Tr909CallbackState::default();
    let mut buffer = vec![0.0; frame_count.saturating_mul(usize::from(channel_count))];

    render_tr909_buffer(
        &mut buffer,
        sample_rate,
        usize::from(channel_count),
        &shared_state.snapshot(),
        &mut callback_state,
    );

    buffer
}

#[must_use]
pub fn render_mc202_offline(
    render_state: &Mc202RenderState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let mut buffer = vec![0.0; frame_count.saturating_mul(usize::from(channel_count))];

    render_mc202_buffer(
        &mut buffer,
        sample_rate,
        usize::from(channel_count),
        render_state,
    );

    buffer
}

#[must_use]
pub fn signal_metrics(samples: &[f32]) -> OfflineAudioMetrics {
    const ACTIVE_THRESHOLD: f32 = 0.0001;

    let active_samples = samples
        .iter()
        .filter(|sample| sample.abs() > ACTIVE_THRESHOLD)
        .count();
    let peak_abs = samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let sum = samples.iter().sum::<f32>();
    let mean_abs = if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|sample| sample.abs()).sum::<f32>() / samples.len() as f32
    };
    let dc_offset = if samples.is_empty() {
        0.0
    } else {
        sum / samples.len() as f32
    };
    let active_sample_ratio = if samples.is_empty() {
        0.0
    } else {
        active_samples as f32 / samples.len() as f32
    };
    let silence_ratio = if samples.is_empty() {
        0.0
    } else {
        1.0 - active_sample_ratio
    };
    let rms = if samples.is_empty() {
        0.0
    } else {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    };
    let zero_crossings = samples
        .windows(2)
        .filter(|window| {
            let left = window[0];
            let right = window[1];
            (left > 0.0 && right < 0.0) || (left < 0.0 && right > 0.0)
        })
        .count();
    let crest_factor = if rms > 0.0 { peak_abs / rms } else { 0.0 };

    OfflineAudioMetrics {
        active_samples,
        peak_abs,
        rms,
        sum,
        mean_abs,
        zero_crossings,
        crest_factor,
        active_sample_ratio,
        silence_ratio,
        dc_offset,
    }
}

#[must_use]
pub fn signal_delta_metrics(left: &[f32], right: &[f32]) -> OfflineAudioMetrics {
    let sample_count = left.len().max(right.len());
    let delta = (0..sample_count)
        .map(|index| left.get(index).copied().unwrap_or(0.0) - right.get(index).copied().unwrap_or(0.0))
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

pub struct AudioRuntimeShell {
    lifecycle: AudioRuntimeLifecycle,
    output: Option<AudioOutputInfo>,
    telemetry: Arc<RuntimeTelemetry>,
    transport: Arc<SharedTransportTimingState>,
    tr909_render: Arc<SharedTr909RenderState>,
    mc202_render: Arc<SharedMc202RenderState>,
    w30_preview: Arc<SharedW30PreviewRenderState>,
    w30_resample_tap: Arc<SharedW30ResampleTapState>,
    stream: Option<cpal::Stream>,
}

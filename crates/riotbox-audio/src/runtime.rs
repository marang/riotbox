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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OfflineAudioMetrics {
    pub active_samples: usize,
    pub peak_abs: f32,
    pub rms: f32,
    pub sum: f32,
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
    let active_samples = samples
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();
    let peak_abs = samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let sum = samples.iter().sum::<f32>();
    let rms = if samples.is_empty() {
        0.0
    } else {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    };

    OfflineAudioMetrics {
        active_samples,
        peak_abs,
        rms,
        sum,
    }
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

impl AudioRuntimeShell {
    pub fn start_default_output() -> Result<Self, AudioRuntimeError> {
        Self::start_default_output_with_render_states(
            Tr909RenderState::default(),
            Mc202RenderState::default(),
            W30PreviewRenderState::default(),
            W30ResampleTapState::default(),
        )
    }

    pub fn start_default_output_with_tr909(
        render_state: Tr909RenderState,
    ) -> Result<Self, AudioRuntimeError> {
        Self::start_default_output_with_render_states(
            render_state,
            Mc202RenderState::default(),
            W30PreviewRenderState::default(),
            W30ResampleTapState::default(),
        )
    }

    pub fn start_default_output_with_render_states(
        tr909_render_state: Tr909RenderState,
        mc202_render_state: Mc202RenderState,
        w30_preview_render_state: W30PreviewRenderState,
        w30_resample_tap_state: W30ResampleTapState,
    ) -> Result<Self, AudioRuntimeError> {
        let host = cpal::default_host();
        let host_name = format!("{:?}", host.id());

        let Some(device) = host.default_output_device() else {
            return Err(AudioRuntimeError::NoDefaultOutputDevice { host_name });
        };

        #[allow(deprecated)]
        let device_name = device
            .name()
            .unwrap_or_else(|_| "<unknown-device>".to_string());

        let supported_output_config_count = device
            .supported_output_configs()
            .ok()
            .map(|configs| configs.count());

        let default_config = device.default_output_config().map_err(|error| {
            AudioRuntimeError::DefaultOutputConfig {
                host_name: host_name.clone(),
                device_name: device_name.clone(),
                reason: error.to_string(),
            }
        })?;

        let output = AudioOutputInfo {
            host_name: host_name.clone(),
            device_name: device_name.clone(),
            sample_format: format!("{:?}", default_config.sample_format()),
            sample_rate: default_config.sample_rate(),
            channel_count: default_config.channels(),
            buffer_size: format!("{:?}", default_config.buffer_size()),
            supported_output_config_count,
        };

        let telemetry = Arc::new(RuntimeTelemetry::new());
        let transport = Arc::new(SharedTransportTimingState::new(
            tr909_render_state.is_transport_running,
            tr909_render_state.tempo_bpm,
            tr909_render_state.position_beats,
        ));
        let tr909_render = Arc::new(SharedTr909RenderState::new(&tr909_render_state));
        let mc202_render = Arc::new(SharedMc202RenderState::new(&mc202_render_state));
        let w30_preview = Arc::new(SharedW30PreviewRenderState::new(&w30_preview_render_state));
        let w30_resample_tap = Arc::new(SharedW30ResampleTapState::new(&w30_resample_tap_state));
        let stream_config = default_config.config();
        let start = Instant::now();

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => build_silent_output_stream::<f32>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            cpal::SampleFormat::I16 => build_silent_output_stream::<i16>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            cpal::SampleFormat::U16 => build_silent_output_stream::<u16>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            sample_format => {
                return Err(AudioRuntimeError::UnsupportedSampleFormat {
                    host_name,
                    device_name,
                    sample_format: format!("{sample_format:?}"),
                });
            }
        }
        .map_err(|error| AudioRuntimeError::BuildStream {
            host_name: host_name.clone(),
            device_name: device_name.clone(),
            reason: error.to_string(),
        })?;

        stream
            .play()
            .map_err(|error| AudioRuntimeError::PlayStream {
                host_name,
                device_name,
                reason: error.to_string(),
            })?;

        Ok(Self {
            lifecycle: AudioRuntimeLifecycle::Running,
            output: Some(output),
            telemetry,
            transport,
            tr909_render,
            mc202_render,
            w30_preview,
            w30_resample_tap,
            stream: Some(stream),
        })
    }

    #[must_use]
    pub fn lifecycle(&self) -> AudioRuntimeLifecycle {
        self.lifecycle
    }

    #[must_use]
    pub fn health_snapshot(&self) -> AudioRuntimeHealth {
        let telemetry = self.telemetry.snapshot();

        let lifecycle = if telemetry.stream_error_count > 0
            && matches!(self.lifecycle, AudioRuntimeLifecycle::Running)
        {
            AudioRuntimeLifecycle::Faulted
        } else {
            self.lifecycle
        };

        AudioRuntimeHealth {
            lifecycle,
            output: self.output.clone(),
            callback_count: telemetry.callback_count,
            max_callback_gap_micros: telemetry.max_callback_gap_micros,
            stream_error_count: telemetry.stream_error_count,
            last_stream_error: telemetry.last_stream_error,
        }
    }

    #[must_use]
    pub fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        self.telemetry.timing_snapshot()
    }

    pub fn update_transport_state(
        &self,
        is_transport_running: bool,
        tempo_bpm: f32,
        position_beats: f64,
    ) {
        self.transport
            .update(is_transport_running, tempo_bpm, position_beats);
    }

    pub fn update_tr909_render_state(&self, render_state: &Tr909RenderState) {
        self.tr909_render.update(render_state);
    }

    pub fn update_mc202_render_state(&self, render_state: &Mc202RenderState) {
        self.mc202_render.update(render_state);
    }

    pub fn update_w30_preview_render_state(&self, render_state: &W30PreviewRenderState) {
        self.w30_preview.update(render_state);
    }

    pub fn update_w30_resample_tap_state(&self, render_state: &W30ResampleTapState) {
        self.w30_resample_tap.update(render_state);
    }

    pub fn stop(&mut self) {
        self.stream.take();
        self.lifecycle = AudioRuntimeLifecycle::Stopped;
    }

    #[cfg(test)]
    fn from_test_parts(parts: AudioRuntimeShellTestParts) -> Self {
        Self {
            lifecycle: parts.lifecycle,
            output: parts.output,
            telemetry: parts.telemetry,
            transport: parts.transport,
            tr909_render: parts.tr909_render,
            mc202_render: parts.mc202_render,
            w30_preview: parts.w30_preview,
            w30_resample_tap: parts.w30_resample_tap,
            stream: None,
        }
    }
}

#[cfg(test)]
struct AudioRuntimeShellTestParts {
    lifecycle: AudioRuntimeLifecycle,
    output: Option<AudioOutputInfo>,
    telemetry: Arc<RuntimeTelemetry>,
    transport: Arc<SharedTransportTimingState>,
    tr909_render: Arc<SharedTr909RenderState>,
    mc202_render: Arc<SharedMc202RenderState>,
    w30_preview: Arc<SharedW30PreviewRenderState>,
    w30_resample_tap: Arc<SharedW30ResampleTapState>,
}

impl Drop for AudioRuntimeShell {
    fn drop(&mut self) {
        self.stop();
    }
}

fn build_silent_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared: AudioRuntimeSharedState,
    start: Instant,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let callback_telemetry = Arc::clone(&shared.telemetry);
    let error_telemetry = Arc::clone(&shared.telemetry);
    let callback_transport = Arc::clone(&shared.transport);
    let mut render_state = Tr909CallbackState::default();
    let mut transport_state = TransportTimingCallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::default();
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut mix_buffer = Vec::<f32>::new();
    let sample_rate = config.sample_rate;
    let channel_count = usize::from(config.channels.max(1));

    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            if mix_buffer.len() != data.len() {
                mix_buffer.resize(data.len(), 0.0);
            }

            let frame_count = data.len() / channel_count.max(1);
            let callback_timing = advance_transport_timing(
                &callback_transport.snapshot(),
                &mut transport_state,
                sample_rate,
                frame_count,
            );
            let mut tr909_render_state = shared.tr909_render.snapshot();
            tr909_render_state.is_transport_running = callback_timing.is_transport_running;
            tr909_render_state.tempo_bpm = callback_timing.tempo_bpm;
            tr909_render_state.position_beats = callback_timing.render_position_beats;
            let mut mc202_render_state = shared.mc202_render.snapshot();
            mc202_render_state.is_transport_running = callback_timing.is_transport_running;
            mc202_render_state.tempo_bpm = callback_timing.tempo_bpm;
            mc202_render_state.position_beats = callback_timing.render_position_beats;
            let mut w30_preview_render_state = shared.w30_preview.snapshot();
            w30_preview_render_state.is_transport_running = callback_timing.is_transport_running;
            w30_preview_render_state.tempo_bpm = callback_timing.tempo_bpm;
            w30_preview_render_state.position_beats = callback_timing.render_position_beats;
            let mut w30_resample_render_state = shared.w30_resample_tap.snapshot();
            w30_resample_render_state.is_transport_running = callback_timing.is_transport_running;

            render_mix_buffer(
                &mut mix_buffer,
                sample_rate,
                channel_count,
                &tr909_render_state,
                &mc202_render_state,
                &mut render_state,
                &mut W30MixRenderState {
                    preview_render: &w30_preview_render_state,
                    preview_state: &mut w30_preview_state,
                    resample_render: &w30_resample_render_state,
                    resample_state: &mut w30_resample_state,
                },
            );
            for (output, sample) in data.iter_mut().zip(mix_buffer.iter().copied()) {
                *output = T::from_sample(sample);
            }

            let now = start.elapsed().as_micros() as u64;
            callback_telemetry.record_callback_at(now, &callback_timing);
        },
        move |error| {
            error_telemetry.record_stream_error(error.to_string());
        },
        None,
    )
}

#[derive(Clone)]
struct AudioRuntimeSharedState {
    telemetry: Arc<RuntimeTelemetry>,
    transport: Arc<SharedTransportTimingState>,
    tr909_render: Arc<SharedTr909RenderState>,
    mc202_render: Arc<SharedMc202RenderState>,
    w30_preview: Arc<SharedW30PreviewRenderState>,
    w30_resample_tap: Arc<SharedW30ResampleTapState>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeTransportTimingState {
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

struct SharedTransportTimingState {
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedTransportTimingState {
    fn new(is_transport_running: bool, tempo_bpm: f32, position_beats: f64) -> Self {
        Self {
            is_transport_running: AtomicBool::new(is_transport_running),
            tempo_bpm_bits: AtomicU32::new(tempo_bpm.to_bits()),
            position_beats_bits: AtomicU64::new(position_beats.to_bits()),
        }
    }

    fn update(&self, is_transport_running: bool, tempo_bpm: f32, position_beats: f64) {
        self.is_transport_running
            .store(is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(position_beats.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeTransportTimingState {
        RealtimeTransportTimingState {
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeTr909RenderState {
    mode: Tr909RenderMode,
    routing: Tr909RenderRouting,
    source_support_profile: Option<Tr909SourceSupportProfile>,
    source_support_context: Option<Tr909SourceSupportContext>,
    pattern_adoption: Option<Tr909PatternAdoption>,
    phrase_variation: Option<Tr909PhraseVariation>,
    takeover_profile: Option<Tr909TakeoverRenderProfile>,
    drum_bus_level: f32,
    slam_intensity: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

struct SharedTr909RenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_support_profile: AtomicU32,
    source_support_context: AtomicU32,
    pattern_adoption: AtomicU32,
    phrase_variation: AtomicU32,
    takeover_profile: AtomicU32,
    drum_bus_level_bits: AtomicU32,
    slam_intensity_bits: AtomicU32,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedTr909RenderState {
    fn new(render_state: &Tr909RenderState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_support_profile: AtomicU32::new(0),
            source_support_context: AtomicU32::new(0),
            pattern_adoption: AtomicU32::new(0),
            phrase_variation: AtomicU32::new(0),
            takeover_profile: AtomicU32::new(0),
            drum_bus_level_bits: AtomicU32::new(0),
            slam_intensity_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &Tr909RenderState) {
        self.mode
            .store(mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing
            .store(routing_to_u32(render_state.routing), Ordering::Relaxed);
        self.source_support_profile.store(
            support_profile_to_u32(render_state.source_support_profile),
            Ordering::Relaxed,
        );
        self.source_support_context.store(
            support_context_to_u32(render_state.source_support_context),
            Ordering::Relaxed,
        );
        self.pattern_adoption.store(
            pattern_adoption_to_u32(render_state.pattern_adoption),
            Ordering::Relaxed,
        );
        self.phrase_variation.store(
            phrase_variation_to_u32(render_state.phrase_variation),
            Ordering::Relaxed,
        );
        self.takeover_profile.store(
            takeover_profile_to_u32(render_state.takeover_profile),
            Ordering::Relaxed,
        );
        self.drum_bus_level_bits
            .store(render_state.drum_bus_level.to_bits(), Ordering::Relaxed);
        self.slam_intensity_bits
            .store(render_state.slam_intensity.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeTr909RenderState {
        RealtimeTr909RenderState {
            mode: mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_support_profile: support_profile_from_u32(
                self.source_support_profile.load(Ordering::Relaxed),
            ),
            source_support_context: support_context_from_u32(
                self.source_support_context.load(Ordering::Relaxed),
            ),
            pattern_adoption: pattern_adoption_from_u32(
                self.pattern_adoption.load(Ordering::Relaxed),
            ),
            phrase_variation: phrase_variation_from_u32(
                self.phrase_variation.load(Ordering::Relaxed),
            ),
            takeover_profile: takeover_profile_from_u32(
                self.takeover_profile.load(Ordering::Relaxed),
            ),
            drum_bus_level: f32::from_bits(self.drum_bus_level_bits.load(Ordering::Relaxed)),
            slam_intensity: f32::from_bits(self.slam_intensity_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeMc202RenderState {
    mode: Mc202RenderMode,
    routing: Mc202RenderRouting,
    phrase_shape: Mc202PhraseShape,
    note_budget: Mc202NoteBudget,
    contour_hint: Mc202ContourHint,
    hook_response: Mc202HookResponse,
    touch: f32,
    music_bus_level: f32,
    tempo_bpm: f32,
    position_beats: f64,
    is_transport_running: bool,
}

impl From<RealtimeMc202RenderState> for Mc202RenderState {
    fn from(render: RealtimeMc202RenderState) -> Self {
        Self {
            mode: render.mode,
            routing: render.routing,
            phrase_shape: render.phrase_shape,
            note_budget: render.note_budget,
            contour_hint: render.contour_hint,
            hook_response: render.hook_response,
            touch: render.touch,
            music_bus_level: render.music_bus_level,
            tempo_bpm: render.tempo_bpm,
            position_beats: render.position_beats,
            is_transport_running: render.is_transport_running,
        }
    }
}

struct SharedMc202RenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    phrase_shape: AtomicU32,
    note_budget: AtomicU32,
    contour_hint: AtomicU32,
    hook_response: AtomicU32,
    touch_bits: AtomicU32,
    music_bus_level_bits: AtomicU32,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
    is_transport_running: AtomicBool,
}

impl SharedMc202RenderState {
    fn new(render_state: &Mc202RenderState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            phrase_shape: AtomicU32::new(0),
            note_budget: AtomicU32::new(mc202_note_budget_to_u32(Mc202NoteBudget::Balanced)),
            contour_hint: AtomicU32::new(mc202_contour_hint_to_u32(Mc202ContourHint::Neutral)),
            hook_response: AtomicU32::new(mc202_hook_response_to_u32(Mc202HookResponse::Direct)),
            touch_bits: AtomicU32::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
            is_transport_running: AtomicBool::new(false),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &Mc202RenderState) {
        self.mode
            .store(mc202_mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing.store(
            mc202_routing_to_u32(render_state.routing),
            Ordering::Relaxed,
        );
        self.phrase_shape.store(
            mc202_phrase_shape_to_u32(render_state.phrase_shape),
            Ordering::Relaxed,
        );
        self.note_budget.store(
            mc202_note_budget_to_u32(render_state.note_budget),
            Ordering::Relaxed,
        );
        self.contour_hint.store(
            mc202_contour_hint_to_u32(render_state.contour_hint),
            Ordering::Relaxed,
        );
        self.hook_response.store(
            mc202_hook_response_to_u32(render_state.hook_response),
            Ordering::Relaxed,
        );
        self.touch_bits
            .store(render_state.touch.to_bits(), Ordering::Relaxed);
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeMc202RenderState {
        RealtimeMc202RenderState {
            mode: mc202_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: mc202_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            phrase_shape: mc202_phrase_shape_from_u32(self.phrase_shape.load(Ordering::Relaxed)),
            note_budget: mc202_note_budget_from_u32(self.note_budget.load(Ordering::Relaxed)),
            contour_hint: mc202_contour_hint_from_u32(self.contour_hint.load(Ordering::Relaxed)),
            hook_response: mc202_hook_response_from_u32(self.hook_response.load(Ordering::Relaxed)),
            touch: f32::from_bits(self.touch_bits.load(Ordering::Relaxed)),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
        }
    }
}

fn mc202_mode_to_u32(mode: Mc202RenderMode) -> u32 {
    match mode {
        Mc202RenderMode::Idle => 0,
        Mc202RenderMode::Leader => 1,
        Mc202RenderMode::Follower => 2,
        Mc202RenderMode::Answer => 3,
        Mc202RenderMode::Pressure => 4,
        Mc202RenderMode::Instigator => 5,
    }
}

fn mc202_mode_from_u32(value: u32) -> Mc202RenderMode {
    match value {
        1 => Mc202RenderMode::Leader,
        2 => Mc202RenderMode::Follower,
        3 => Mc202RenderMode::Answer,
        4 => Mc202RenderMode::Pressure,
        5 => Mc202RenderMode::Instigator,
        _ => Mc202RenderMode::Idle,
    }
}

fn mc202_routing_to_u32(routing: Mc202RenderRouting) -> u32 {
    match routing {
        Mc202RenderRouting::Silent => 0,
        Mc202RenderRouting::MusicBusBass => 1,
    }
}

fn mc202_routing_from_u32(value: u32) -> Mc202RenderRouting {
    match value {
        1 => Mc202RenderRouting::MusicBusBass,
        _ => Mc202RenderRouting::Silent,
    }
}

fn mc202_phrase_shape_to_u32(shape: Mc202PhraseShape) -> u32 {
    match shape {
        Mc202PhraseShape::RootPulse => 0,
        Mc202PhraseShape::FollowerDrive => 1,
        Mc202PhraseShape::AnswerHook => 2,
        Mc202PhraseShape::MutatedDrive => 3,
        Mc202PhraseShape::PressureCell => 4,
        Mc202PhraseShape::InstigatorSpike => 5,
    }
}

fn mc202_phrase_shape_from_u32(value: u32) -> Mc202PhraseShape {
    match value {
        1 => Mc202PhraseShape::FollowerDrive,
        2 => Mc202PhraseShape::AnswerHook,
        3 => Mc202PhraseShape::MutatedDrive,
        4 => Mc202PhraseShape::PressureCell,
        5 => Mc202PhraseShape::InstigatorSpike,
        _ => Mc202PhraseShape::RootPulse,
    }
}

fn mc202_note_budget_to_u32(budget: Mc202NoteBudget) -> u32 {
    match budget {
        Mc202NoteBudget::Sparse => 0,
        Mc202NoteBudget::Balanced => 1,
        Mc202NoteBudget::Push => 2,
        Mc202NoteBudget::Wide => 3,
    }
}

fn mc202_note_budget_from_u32(value: u32) -> Mc202NoteBudget {
    match value {
        0 => Mc202NoteBudget::Sparse,
        2 => Mc202NoteBudget::Push,
        3 => Mc202NoteBudget::Wide,
        _ => Mc202NoteBudget::Balanced,
    }
}

fn mc202_contour_hint_to_u32(hint: Mc202ContourHint) -> u32 {
    match hint {
        Mc202ContourHint::Neutral => 0,
        Mc202ContourHint::Lift => 1,
        Mc202ContourHint::Drop => 2,
        Mc202ContourHint::Hold => 3,
    }
}

fn mc202_contour_hint_from_u32(value: u32) -> Mc202ContourHint {
    match value {
        1 => Mc202ContourHint::Lift,
        2 => Mc202ContourHint::Drop,
        3 => Mc202ContourHint::Hold,
        _ => Mc202ContourHint::Neutral,
    }
}

fn mc202_hook_response_to_u32(response: Mc202HookResponse) -> u32 {
    match response {
        Mc202HookResponse::Direct => 0,
        Mc202HookResponse::AnswerSpace => 1,
    }
}

fn mc202_hook_response_from_u32(value: u32) -> Mc202HookResponse {
    match value {
        1 => Mc202HookResponse::AnswerSpace,
        _ => Mc202HookResponse::Direct,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PreviewRenderState {
    mode: W30PreviewRenderMode,
    routing: W30PreviewRenderRouting,
    source_profile: Option<W30PreviewSourceProfile>,
    trigger_revision: u64,
    trigger_velocity: f32,
    source_window_preview: RealtimeW30PreviewSampleWindow,
    pad_playback: RealtimeW30PadPlaybackSampleWindow,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PreviewSampleWindow {
    source_start_frame: u64,
    source_end_frame: u64,
    sample_count: usize,
    samples: [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30PadPlaybackSampleWindow {
    source_start_frame: u64,
    source_end_frame: u64,
    sample_count: usize,
    loop_enabled: bool,
    samples: [f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
}

impl Default for RealtimeW30PreviewSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            sample_count: 0,
            samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
        }
    }
}

impl Default for RealtimeW30PadPlaybackSampleWindow {
    fn default() -> Self {
        Self {
            source_start_frame: 0,
            source_end_frame: 0,
            sample_count: 0,
            loop_enabled: false,
            samples: [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        }
    }
}

struct SharedW30PreviewRenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_profile: AtomicU32,
    trigger_revision: AtomicU64,
    trigger_velocity_bits: AtomicU32,
    source_start_frame: AtomicU64,
    source_end_frame: AtomicU64,
    source_sample_count: AtomicU32,
    source_samples: [AtomicU32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
    pad_start_frame: AtomicU64,
    pad_end_frame: AtomicU64,
    pad_sample_count: AtomicU32,
    pad_loop_enabled: AtomicBool,
    pad_samples: [AtomicU32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    music_bus_level_bits: AtomicU32,
    grit_level_bits: AtomicU32,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedW30PreviewRenderState {
    fn new(render_state: &W30PreviewRenderState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            trigger_revision: AtomicU64::new(0),
            trigger_velocity_bits: AtomicU32::new(0),
            source_start_frame: AtomicU64::new(0),
            source_end_frame: AtomicU64::new(0),
            source_sample_count: AtomicU32::new(0),
            source_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            pad_start_frame: AtomicU64::new(0),
            pad_end_frame: AtomicU64::new(0),
            pad_sample_count: AtomicU32::new(0),
            pad_loop_enabled: AtomicBool::new(false),
            pad_samples: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &W30PreviewRenderState) {
        self.mode
            .store(w30_mode_to_u32(render_state.mode), Ordering::Relaxed);
        self.routing
            .store(w30_routing_to_u32(render_state.routing), Ordering::Relaxed);
        self.source_profile.store(
            w30_source_profile_to_u32(render_state.source_profile),
            Ordering::Relaxed,
        );
        self.trigger_revision
            .store(render_state.trigger_revision, Ordering::Relaxed);
        self.trigger_velocity_bits
            .store(render_state.trigger_velocity.to_bits(), Ordering::Relaxed);
        self.update_source_window_preview(render_state.source_window_preview.as_ref());
        self.update_pad_playback(render_state.pad_playback.as_ref());
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(render_state.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(render_state.position_beats.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeW30PreviewRenderState {
        RealtimeW30PreviewRenderState {
            mode: w30_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            trigger_revision: self.trigger_revision.load(Ordering::Relaxed),
            trigger_velocity: f32::from_bits(self.trigger_velocity_bits.load(Ordering::Relaxed)),
            source_window_preview: self.source_window_preview_snapshot(),
            pad_playback: self.pad_playback_snapshot(),
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }

    fn update_source_window_preview(&self, source_window: Option<&W30PreviewSampleWindow>) {
        if let Some(source_window) = source_window {
            let sample_count = source_window
                .sample_count
                .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
            self.source_start_frame
                .store(source_window.source_start_frame, Ordering::Relaxed);
            self.source_end_frame
                .store(source_window.source_end_frame, Ordering::Relaxed);
            for (index, sample) in source_window.samples.iter().copied().enumerate() {
                self.source_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.source_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.source_sample_count.store(0, Ordering::Relaxed);
            self.source_start_frame.store(0, Ordering::Relaxed);
            self.source_end_frame.store(0, Ordering::Relaxed);
        }
    }

    fn source_window_preview_snapshot(&self) -> RealtimeW30PreviewSampleWindow {
        let sample_count = (self.source_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.source_samples[index].load(Ordering::Relaxed));
        }

        RealtimeW30PreviewSampleWindow {
            source_start_frame: self.source_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.source_end_frame.load(Ordering::Relaxed),
            sample_count,
            samples,
        }
    }

    fn update_pad_playback(&self, pad_playback: Option<&W30PadPlaybackSampleWindow>) {
        if let Some(pad_playback) = pad_playback {
            let sample_count = pad_playback
                .sample_count
                .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
            self.pad_start_frame
                .store(pad_playback.source_start_frame, Ordering::Relaxed);
            self.pad_end_frame
                .store(pad_playback.source_end_frame, Ordering::Relaxed);
            self.pad_loop_enabled
                .store(pad_playback.loop_enabled, Ordering::Relaxed);
            for (index, sample) in pad_playback.samples.iter().copied().enumerate() {
                self.pad_samples[index].store(sample.to_bits(), Ordering::Relaxed);
            }
            self.pad_sample_count
                .store(sample_count as u32, Ordering::Relaxed);
        } else {
            self.pad_sample_count.store(0, Ordering::Relaxed);
            self.pad_start_frame.store(0, Ordering::Relaxed);
            self.pad_end_frame.store(0, Ordering::Relaxed);
            self.pad_loop_enabled.store(false, Ordering::Relaxed);
        }
    }

    fn pad_playback_snapshot(&self) -> RealtimeW30PadPlaybackSampleWindow {
        let sample_count = (self.pad_sample_count.load(Ordering::Relaxed) as usize)
            .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
        let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = f32::from_bits(self.pad_samples[index].load(Ordering::Relaxed));
        }

        RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: self.pad_start_frame.load(Ordering::Relaxed),
            source_end_frame: self.pad_end_frame.load(Ordering::Relaxed),
            sample_count,
            loop_enabled: self.pad_loop_enabled.load(Ordering::Relaxed),
            samples,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeW30ResampleTapState {
    mode: W30ResampleTapMode,
    routing: W30ResampleTapRouting,
    source_profile: Option<W30ResampleTapSourceProfile>,
    lineage_capture_count: u8,
    generation_depth: u8,
    music_bus_level: f32,
    grit_level: f32,
    is_transport_running: bool,
}

struct SharedW30ResampleTapState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_profile: AtomicU32,
    lineage_capture_count: AtomicU32,
    generation_depth: AtomicU32,
    music_bus_level_bits: AtomicU32,
    grit_level_bits: AtomicU32,
    is_transport_running: AtomicBool,
}

impl SharedW30ResampleTapState {
    fn new(render_state: &W30ResampleTapState) -> Self {
        let shared = Self {
            mode: AtomicU32::new(0),
            routing: AtomicU32::new(0),
            source_profile: AtomicU32::new(0),
            lineage_capture_count: AtomicU32::new(0),
            generation_depth: AtomicU32::new(0),
            music_bus_level_bits: AtomicU32::new(0),
            grit_level_bits: AtomicU32::new(0),
            is_transport_running: AtomicBool::new(false),
        };
        shared.update(render_state);
        shared
    }

    fn update(&self, render_state: &W30ResampleTapState) {
        self.mode.store(
            w30_resample_mode_to_u32(render_state.mode),
            Ordering::Relaxed,
        );
        self.routing.store(
            w30_resample_routing_to_u32(render_state.routing),
            Ordering::Relaxed,
        );
        self.source_profile.store(
            w30_resample_source_profile_to_u32(render_state.source_profile),
            Ordering::Relaxed,
        );
        self.lineage_capture_count.store(
            u32::from(render_state.lineage_capture_count),
            Ordering::Relaxed,
        );
        self.generation_depth
            .store(u32::from(render_state.generation_depth), Ordering::Relaxed);
        self.music_bus_level_bits
            .store(render_state.music_bus_level.to_bits(), Ordering::Relaxed);
        self.grit_level_bits
            .store(render_state.grit_level.to_bits(), Ordering::Relaxed);
        self.is_transport_running
            .store(render_state.is_transport_running, Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeW30ResampleTapState {
        RealtimeW30ResampleTapState {
            mode: w30_resample_mode_from_u32(self.mode.load(Ordering::Relaxed)),
            routing: w30_resample_routing_from_u32(self.routing.load(Ordering::Relaxed)),
            source_profile: w30_resample_source_profile_from_u32(
                self.source_profile.load(Ordering::Relaxed),
            ),
            lineage_capture_count: self.lineage_capture_count.load(Ordering::Relaxed) as u8,
            generation_depth: self.generation_depth.load(Ordering::Relaxed) as u8,
            music_bus_level: f32::from_bits(self.music_bus_level_bits.load(Ordering::Relaxed)),
            grit_level: f32::from_bits(self.grit_level_bits.load(Ordering::Relaxed)),
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
        }
    }
}

fn w30_resample_mode_to_u32(mode: W30ResampleTapMode) -> u32 {
    match mode {
        W30ResampleTapMode::Idle => 0,
        W30ResampleTapMode::CaptureLineageReady => 1,
    }
}

fn w30_resample_mode_from_u32(value: u32) -> W30ResampleTapMode {
    match value {
        1 => W30ResampleTapMode::CaptureLineageReady,
        _ => W30ResampleTapMode::Idle,
    }
}

fn w30_resample_routing_to_u32(routing: W30ResampleTapRouting) -> u32 {
    match routing {
        W30ResampleTapRouting::Silent => 0,
        W30ResampleTapRouting::InternalCaptureTap => 1,
    }
}

fn w30_resample_routing_from_u32(value: u32) -> W30ResampleTapRouting {
    match value {
        1 => W30ResampleTapRouting::InternalCaptureTap,
        _ => W30ResampleTapRouting::Silent,
    }
}

fn w30_resample_source_profile_to_u32(profile: Option<W30ResampleTapSourceProfile>) -> u32 {
    match profile {
        None => 0,
        Some(W30ResampleTapSourceProfile::RawCapture) => 1,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 2,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 3,
    }
}

fn w30_resample_source_profile_from_u32(value: u32) -> Option<W30ResampleTapSourceProfile> {
    match value {
        1 => Some(W30ResampleTapSourceProfile::RawCapture),
        2 => Some(W30ResampleTapSourceProfile::PromotedCapture),
        3 => Some(W30ResampleTapSourceProfile::PinnedCapture),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct Tr909CallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    oscillator_hz: f32,
    envelope: f32,
    last_step: i64,
    was_running: bool,
}

#[derive(Debug, Default)]
struct TransportTimingCallbackState {
    beat_position: f64,
    was_running: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct CallbackTimingSnapshot {
    is_transport_running: bool,
    tempo_bpm: f32,
    render_position_beats: f64,
    completed_position_beats: f64,
}

#[derive(Debug, Default)]
struct W30PreviewCallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    lfo_phase: f32,
    source_sample_cursor: f32,
    pad_playback_cursor: f32,
    last_source_window_signature: u64,
    last_pad_playback_signature: u64,
    envelope: f32,
    last_step: i64,
    last_trigger_revision: u64,
    was_active: bool,
    last_mode: Option<W30PreviewRenderMode>,
    last_routing: Option<W30PreviewRenderRouting>,
    last_source_profile: Option<W30PreviewSourceProfile>,
    last_music_bus_level: f32,
    last_grit_level: f32,
    last_transport_running: bool,
    last_position_beats: f64,
}

#[derive(Debug, Default)]
struct W30ResampleTapCallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    shimmer_phase: f32,
    envelope: f32,
    last_step: i64,
    was_active: bool,
}

struct W30MixRenderState<'a> {
    preview_render: &'a RealtimeW30PreviewRenderState,
    preview_state: &'a mut W30PreviewCallbackState,
    resample_render: &'a RealtimeW30ResampleTapState,
    resample_state: &'a mut W30ResampleTapCallbackState,
}

fn sync_w30_preview_state(
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
) {
    state.last_mode = (!matches!(render.mode, W30PreviewRenderMode::Idle)).then_some(render.mode);
    state.last_routing =
        (!matches!(render.routing, W30PreviewRenderRouting::Silent)).then_some(render.routing);
    state.last_source_profile = render.source_profile;
    state.last_music_bus_level = render.music_bus_level;
    state.last_grit_level = render.grit_level;
    state.last_transport_running = render.is_transport_running;
    state.last_position_beats = render.position_beats;
}

fn render_mix_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    tr909_render: &RealtimeTr909RenderState,
    mc202_render: &RealtimeMc202RenderState,
    tr909_state: &mut Tr909CallbackState,
    w30: &mut W30MixRenderState<'_>,
) {
    data.fill(0.0);
    render_tr909_buffer(data, sample_rate, channel_count, tr909_render, tr909_state);
    render_mc202_buffer(data, sample_rate, channel_count, &(*mc202_render).into());
    sync_w30_preview_state(w30.preview_render, w30.preview_state);
    render_w30_preview_buffer(
        data,
        sample_rate,
        channel_count,
        w30.preview_render,
        w30.preview_state,
    );
    render_w30_resample_tap_buffer(
        data,
        sample_rate,
        channel_count,
        w30.resample_render,
        w30.resample_state,
    );
}

fn advance_transport_timing(
    control: &RealtimeTransportTimingState,
    state: &mut TransportTimingCallbackState,
    sample_rate: u32,
    frame_count: usize,
) -> CallbackTimingSnapshot {
    let transport_running = control.is_transport_running && control.tempo_bpm > 0.0;
    if !transport_running {
        state.was_running = false;
        state.beat_position = control.position_beats;
        return CallbackTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: control.tempo_bpm,
            render_position_beats: control.position_beats,
            completed_position_beats: control.position_beats,
        };
    }

    if !state.was_running || (state.beat_position - control.position_beats).abs() > 0.125 {
        state.beat_position = control.position_beats;
        state.was_running = true;
    }

    let render_position_beats = state.beat_position;
    let beats_per_sample = f64::from(control.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let completed_position_beats = render_position_beats + (beats_per_sample * frame_count as f64);
    state.beat_position = completed_position_beats;

    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: control.tempo_bpm,
        render_position_beats,
        completed_position_beats,
    }
}

fn render_tr909_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeTr909RenderState,
    state: &mut Tr909CallbackState,
) {
    if !render.is_transport_running
        || matches!(render.mode, Tr909RenderMode::Idle)
        || render.tempo_bpm <= 0.0
    {
        state.was_running = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats;
        return;
    }

    let subdivision = render_subdivision(render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    if !state.was_running || (state.beat_position - render.position_beats).abs() > 0.125 {
        state.beat_position = render.position_beats;
        state.last_step = current_step.saturating_sub(1);
        state.was_running = true;
    }

    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let frame_count = data.len() / channel_count.max(1);

    for frame_index in 0..frame_count {
        let step = (state.beat_position * f64::from(subdivision)).floor() as i64;
        if step != state.last_step {
            state.last_step = step;
            if should_trigger_step(render, step) {
                state.envelope = trigger_envelope(render);
                state.oscillator_hz = trigger_frequency(render, step);
            }
        }

        let sample = if state.envelope > 0.0005 {
            let gain = render_gain(render);
            let waveform = (std::f32::consts::TAU * state.oscillator_phase).sin();
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * gain;
            state.envelope *= envelope_decay(render);
            rendered
        } else {
            0.0
        };

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn render_w30_preview_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
) {
    let active = !matches!(render.mode, W30PreviewRenderMode::Idle)
        && matches!(render.routing, W30PreviewRenderRouting::MusicBusPreview)
        && render.music_bus_level > 0.0;

    if !active {
        state.was_active = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats;
        state.last_trigger_revision = render.trigger_revision;
        return;
    }

    if !state.was_active {
        state.beat_position = render.position_beats;
        state.envelope = 1.0;
        state.last_step = w30_current_step(render.position_beats, render);
        state.oscillator_phase = 0.0;
        state.lfo_phase = 0.0;
        state.source_sample_cursor = 0.0;
        state.pad_playback_cursor = 0.0;
        state.last_source_window_signature = w30_source_window_signature(render);
        state.last_pad_playback_signature = w30_pad_playback_signature(render);
        state.last_trigger_revision = render.trigger_revision;
        state.was_active = true;
    }

    let source_window_signature = w30_source_window_signature(render);
    if source_window_signature != state.last_source_window_signature {
        state.last_source_window_signature = source_window_signature;
        state.source_sample_cursor = 0.0;
    }
    let pad_playback_signature = w30_pad_playback_signature(render);
    if pad_playback_signature != state.last_pad_playback_signature {
        state.last_pad_playback_signature = pad_playback_signature;
        state.pad_playback_cursor = 0.0;
    }

    if render.trigger_revision > state.last_trigger_revision {
        state.last_trigger_revision = render.trigger_revision;
        state.envelope = state.envelope.max(
            w30_trigger_envelope(render) * (0.85 + render.trigger_velocity.clamp(0.0, 1.0) * 0.3),
        );
        state.oscillator_phase = 0.0;
        state.pad_playback_cursor = 0.0;
    }

    let frame_count = data.len() / channel_count.max(1);
    let transport_running = render.is_transport_running && render.tempo_bpm > 0.0;
    let beats_per_sample = if transport_running {
        f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1))
    } else {
        f64::from(w30_preview_idle_bpm(render)) / 60.0 / f64::from(sample_rate.max(1))
    };

    for frame_index in 0..frame_count {
        if transport_running {
            let step = w30_current_step(state.beat_position, render);
            if step != state.last_step {
                state.last_step = step;
                if should_trigger_w30_step(render, step) {
                    state.envelope = w30_trigger_envelope(render);
                    if w30_source_window_active(render) {
                        state.source_sample_cursor = 0.0;
                    }
                }
            }
        } else {
            state.envelope = (state.envelope * 0.9998).max(0.35);
        }

        let tremolo = if transport_running {
            1.0
        } else {
            state.lfo_phase = (state.lfo_phase + 1.8 / sample_rate.max(1) as f32).fract();
            0.45 + 0.55 * ((std::f32::consts::TAU * state.lfo_phase).sin() * 0.5 + 0.5)
        };
        let waveform = w30_preview_waveform_for_frame(render, state, sample_rate);
        let sample =
            waveform * state.envelope * tremolo * w30_render_gain(render, transport_running);
        if transport_running {
            state.envelope *= w30_envelope_decay(render);
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn w30_preview_waveform_for_frame(
    render: &RealtimeW30PreviewRenderState,
    state: &mut W30PreviewCallbackState,
    sample_rate: u32,
) -> f32 {
    if w30_pad_playback_active(render) {
        let sample = w30_pad_playback_sample(&render.pad_playback, state);
        let grit = render.grit_level.clamp(0.0, 1.0);
        return (sample * (1.0 + grit * 0.35)).clamp(-1.0, 1.0);
    }

    if w30_source_window_active(render) {
        let sample = w30_source_window_sample(&render.source_window_preview, state);
        let grit = render.grit_level.clamp(0.0, 1.0);
        return (sample * (1.0 + grit * 0.35)).clamp(-1.0, 1.0);
    }

    let frequency = w30_preview_frequency(render, state.last_step);
    let waveform = w30_preview_waveform(state.oscillator_phase, render.grit_level);
    state.oscillator_phase =
        (state.oscillator_phase + frequency / sample_rate.max(1) as f32).fract();
    waveform
}

fn w30_source_window_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle)
        && render.source_window_preview.sample_count > 0
}

fn w30_pad_playback_active(render: &RealtimeW30PreviewRenderState) -> bool {
    !matches!(render.mode, W30PreviewRenderMode::Idle) && render.pad_playback.sample_count > 0
}

fn w30_pad_playback_sample(
    window: &RealtimeW30PadPlaybackSampleWindow,
    state: &mut W30PreviewCallbackState,
) -> f32 {
    let sample_count = window.sample_count.min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let cursor = state.pad_playback_cursor as usize;
    let clamped_cursor = if window.loop_enabled {
        cursor % sample_count
    } else {
        cursor.min(sample_count - 1)
    };
    state.pad_playback_cursor = if window.loop_enabled {
        (state.pad_playback_cursor + 1.0) % sample_count as f32
    } else {
        (state.pad_playback_cursor + 1.0).min(sample_count.saturating_sub(1) as f32)
    };
    window.samples[clamped_cursor]
}

fn w30_source_window_sample(
    window: &RealtimeW30PreviewSampleWindow,
    state: &mut W30PreviewCallbackState,
) -> f32 {
    let sample_count = window.sample_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return 0.0;
    }

    let cursor = state.source_sample_cursor as usize % sample_count;
    state.source_sample_cursor = (state.source_sample_cursor + 0.5) % sample_count as f32;
    window.samples[cursor]
}

fn w30_source_window_signature(render: &RealtimeW30PreviewRenderState) -> u64 {
    render
        .source_window_preview
        .source_start_frame
        .wrapping_mul(31)
        .wrapping_add(render.source_window_preview.source_end_frame)
        .wrapping_add(render.source_window_preview.sample_count as u64)
}

fn w30_pad_playback_signature(render: &RealtimeW30PreviewRenderState) -> u64 {
    render
        .pad_playback
        .source_start_frame
        .wrapping_mul(31)
        .wrapping_add(render.pad_playback.source_end_frame)
        .wrapping_add(render.pad_playback.sample_count as u64)
        .wrapping_add(u64::from(render.pad_playback.loop_enabled))
}

fn render_w30_resample_tap_buffer(
    data: &mut [f32],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeW30ResampleTapState,
    state: &mut W30ResampleTapCallbackState,
) {
    let active = !matches!(render.mode, W30ResampleTapMode::Idle)
        && matches!(render.routing, W30ResampleTapRouting::InternalCaptureTap)
        && render.music_bus_level > 0.0;

    if !active {
        state.was_active = false;
        state.envelope = 0.0;
        state.beat_position = 0.0;
        return;
    }

    if !state.was_active {
        state.beat_position = 0.0;
        state.envelope = 1.0;
        state.last_step = 0;
        state.oscillator_phase = 0.0;
        state.shimmer_phase = 0.0;
        state.was_active = true;
    }

    let transport_running = render.is_transport_running;
    let beats_per_sample = if transport_running {
        124.0_f64 / 60.0 / f64::from(sample_rate.max(1))
    } else {
        92.0_f64 / 60.0 / f64::from(sample_rate.max(1))
    };
    let frame_count = data.len() / channel_count.max(1);

    for frame_index in 0..frame_count {
        if transport_running {
            let step =
                (state.beat_position * f64::from(w30_resample_subdivision(render))).floor() as i64;
            if step != state.last_step {
                state.last_step = step;
                if should_trigger_w30_resample_step(render, step) {
                    state.envelope = w30_resample_trigger_envelope(render);
                }
            }
        } else {
            state.envelope = state.envelope.max(0.42) * 0.99975;
        }

        let frequency = w30_resample_frequency(render, state.last_step);
        let shimmer_rate = 0.35 + f32::from(render.generation_depth) * 0.18;
        state.shimmer_phase =
            (state.shimmer_phase + shimmer_rate / sample_rate.max(1) as f32).fract();
        let shimmer =
            0.72 + 0.28 * ((std::f32::consts::TAU * state.shimmer_phase).sin() * 0.5 + 0.5);
        let waveform = w30_resample_waveform(state.oscillator_phase, render.grit_level);
        let sample = waveform
            * state.envelope
            * shimmer
            * w30_resample_render_gain(render, transport_running);
        state.oscillator_phase =
            (state.oscillator_phase + frequency / sample_rate.max(1) as f32).fract();
        if transport_running {
            state.envelope *= w30_resample_decay(render);
        }

        let base = frame_index * channel_count;
        for channel in 0..channel_count {
            data[base + channel] += sample;
        }

        state.beat_position += beats_per_sample;
    }
}

fn w30_resample_subdivision(render: &RealtimeW30ResampleTapState) -> u32 {
    let base = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) => 1,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 2,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 4,
        None => 1,
    };
    (base + u32::from(render.lineage_capture_count >= 2)).min(4)
}

fn should_trigger_w30_resample_step(render: &RealtimeW30ResampleTapState, step: i64) -> bool {
    match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => step.rem_euclid(2) == 0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => !matches!(step.rem_euclid(4), 1),
        Some(W30ResampleTapSourceProfile::PinnedCapture) => true,
    }
}

fn w30_resample_trigger_envelope(render: &RealtimeW30ResampleTapState) -> f32 {
    let profile_boost = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.0,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.05,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.1,
    };
    let lineage_boost = f32::from(render.lineage_capture_count.min(4)) * 0.03;
    let generation_boost = f32::from(render.generation_depth.min(4)) * 0.04;
    (0.24 + profile_boost + lineage_boost + generation_boost + render.grit_level * 0.12)
        .clamp(0.0, 0.9)
}

fn w30_resample_frequency(render: &RealtimeW30ResampleTapState, step: i64) -> f32 {
    let base = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 130.81,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 164.81,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 196.0,
    };
    let step_offset = match step.rem_euclid(4) {
        0 => 0.0,
        1 => 5.0,
        2 => 12.0,
        _ => 7.0,
    };
    let lineage_offset = f32::from(render.lineage_capture_count.min(5)) * 3.0;
    let generation_offset = f32::from(render.generation_depth.min(5)) * 5.0;
    let grit_offset = render.grit_level * 18.0;
    base + step_offset + lineage_offset + generation_offset + grit_offset
}

fn w30_resample_waveform(phase: f32, grit_level: f32) -> f32 {
    let sine = (std::f32::consts::TAU * phase).sin();
    let saw = ((phase * 2.0) - 1.0).clamp(-1.0, 1.0);
    let shimmer = (std::f32::consts::TAU * phase * 3.0).sin();
    let grit = grit_level.clamp(0.0, 1.0);
    (sine * (1.0 - grit * 0.35) + saw * 0.22 + shimmer * (0.12 + grit * 0.22)).clamp(-1.0, 1.0)
}

fn w30_resample_render_gain(render: &RealtimeW30ResampleTapState, transport_running: bool) -> f32 {
    let profile_gain = match render.source_profile {
        Some(W30ResampleTapSourceProfile::RawCapture) | None => 0.08,
        Some(W30ResampleTapSourceProfile::PromotedCapture) => 0.11,
        Some(W30ResampleTapSourceProfile::PinnedCapture) => 0.14,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.7 };
    (profile_gain
        * transport_gain
        * render.music_bus_level.clamp(0.0, 1.0)
        * (1.0 + render.grit_level.clamp(0.0, 1.0) * 0.18))
        .clamp(0.0, 0.22)
}

fn w30_resample_decay(render: &RealtimeW30ResampleTapState) -> f32 {
    let generation_offset = f32::from(render.generation_depth.min(4)) * 0.00003;
    let lineage_offset = f32::from(render.lineage_capture_count.min(4)) * 0.00002;
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 0.00005;
    (0.99978 - generation_offset - lineage_offset - grit_offset).clamp(0.0, 1.0)
}

fn w30_current_step(position_beats: f64, render: &RealtimeW30PreviewRenderState) -> i64 {
    (position_beats * f64::from(w30_preview_subdivision(render))).floor() as i64
}

fn w30_preview_subdivision(render: &RealtimeW30PreviewRenderState) -> u32 {
    match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 2,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 3,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 2,
        Some(W30PreviewSourceProfile::PromotedAudition) => 4,
    }
}

fn should_trigger_w30_step(render: &RealtimeW30PreviewRenderState, step: i64) -> bool {
    match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => true,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => step.rem_euclid(2) == 0,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => step.rem_euclid(3) != 1,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => step.rem_euclid(2) == 0,
        Some(W30PreviewSourceProfile::PromotedAudition) => {
            !matches!(step.rem_euclid(4), 1) || render.grit_level >= 0.65
        }
    }
}

fn w30_trigger_envelope(render: &RealtimeW30PreviewRenderState) -> f32 {
    let mode_boost = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.16,
        W30PreviewRenderMode::RawCaptureAudition => 0.2,
        W30PreviewRenderMode::PromotedAudition => 0.24,
    };
    let profile_boost = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 0.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 0.05,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 0.07,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 0.08,
        Some(W30PreviewSourceProfile::PromotedAudition) => 0.1,
    };
    (0.32 + mode_boost + profile_boost + render.grit_level.clamp(0.0, 1.0) * 0.18).clamp(0.0, 0.9)
}

fn w30_preview_frequency(render: &RealtimeW30PreviewRenderState, step: i64) -> f32 {
    let base = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 196.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 261.63,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 293.66,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 220.0,
        Some(W30PreviewSourceProfile::PromotedAudition) => 329.63,
    };
    let step_offset = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => {
            if step.rem_euclid(2) == 0 {
                -8.0
            } else {
                0.0
            }
        }
        Some(W30PreviewSourceProfile::PromotedRecall) | None => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 7.0,
            2 => 12.0,
            _ => 7.0,
        },
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => match step.rem_euclid(3) {
            0 => 0.0,
            1 => 5.0,
            _ => 10.0,
        },
        Some(W30PreviewSourceProfile::RawCaptureAudition) => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 3.0,
            2 => 10.0,
            _ => 5.0,
        },
        Some(W30PreviewSourceProfile::PromotedAudition) => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 12.0,
            2 => 19.0,
            _ => 7.0,
        },
    };
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 28.0;
    base + step_offset + grit_offset
}

fn w30_preview_waveform(phase: f32, grit_level: f32) -> f32 {
    let sine = (std::f32::consts::TAU * phase).sin();
    let overtone = (std::f32::consts::TAU * phase * 2.0).sin();
    let grit = grit_level.clamp(0.0, 1.0);
    let blended = sine * (1.0 - grit * 0.45) + overtone * (0.18 + grit * 0.3);
    let quant_steps = (24.0 - grit * 18.0).max(4.0);
    ((blended * quant_steps).round() / quant_steps).clamp(-1.0, 1.0)
}

fn w30_render_gain(render: &RealtimeW30PreviewRenderState, transport_running: bool) -> f32 {
    let base = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.12,
        W30PreviewRenderMode::RawCaptureAudition => 0.15,
        W30PreviewRenderMode::PromotedAudition => 0.18,
    };
    let profile_gain = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 1.08,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 1.12,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 1.16,
        Some(W30PreviewSourceProfile::PromotedAudition) => 1.2,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.72 };
    (base
        * profile_gain
        * transport_gain
        * render.music_bus_level.clamp(0.0, 1.0)
        * (1.0 + render.grit_level.clamp(0.0, 1.0) * 0.2))
        .clamp(0.0, 0.28)
}

fn w30_envelope_decay(render: &RealtimeW30PreviewRenderState) -> f32 {
    let base = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.99983,
        W30PreviewRenderMode::RawCaptureAudition => 0.99978,
        W30PreviewRenderMode::PromotedAudition => 0.99972,
    };
    let profile_offset = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 0.00002,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 0.0,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => -0.00001,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => -0.00002,
        Some(W30PreviewSourceProfile::PromotedAudition) => -0.00003,
    };
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 0.00008;
    (base + profile_offset - grit_offset).clamp(0.0, 1.0)
}

fn w30_preview_idle_bpm(render: &RealtimeW30PreviewRenderState) -> f32 {
    render.tempo_bpm.max(92.0)
}

const fn render_subdivision(render: &RealtimeTr909RenderState) -> u32 {
    let base = if let Some(adoption) = render.pattern_adoption {
        match adoption {
            Tr909PatternAdoption::SupportPulse => 1,
            Tr909PatternAdoption::MainlineDrive => 2,
            Tr909PatternAdoption::TakeoverGrid => 4,
        }
    } else {
        match render.mode {
            Tr909RenderMode::Idle => 1,
            Tr909RenderMode::SourceSupport => match render.source_support_profile {
                Some(
                    Tr909SourceSupportProfile::BreakLift | Tr909SourceSupportProfile::DropDrive,
                ) => 2,
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 1,
            },
            Tr909RenderMode::Fill | Tr909RenderMode::BreakReinforce | Tr909RenderMode::Takeover => {
                2
            }
        }
    };

    match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => base,
        Some(Tr909PhraseVariation::PhraseLift) => {
            if base < 2 {
                2
            } else {
                base
            }
        }
        Some(Tr909PhraseVariation::PhraseDrive) => {
            if base < 4 {
                4
            } else {
                base
            }
        }
        Some(Tr909PhraseVariation::PhraseRelease) => {
            if base > 2 {
                2
            } else {
                base
            }
        }
    }
}

fn should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
    let base = if let Some(adoption) = render.pattern_adoption {
        match adoption {
            Tr909PatternAdoption::SupportPulse => step % 2 == 0,
            Tr909PatternAdoption::MainlineDrive => true,
            Tr909PatternAdoption::TakeoverGrid => !matches!(step.rem_euclid(4), 1),
        }
    } else {
        match render.mode {
            Tr909RenderMode::Idle => false,
            Tr909RenderMode::SourceSupport => match render.source_support_profile {
                Some(Tr909SourceSupportProfile::BreakLift) => step % 2 == 0,
                Some(Tr909SourceSupportProfile::DropDrive) => true,
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => true,
            },
            Tr909RenderMode::Fill => true,
            Tr909RenderMode::BreakReinforce => true,
            Tr909RenderMode::Takeover => match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::SceneLock) => step % 4 != 3,
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => true,
            },
        }
    };

    match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => base,
        Some(Tr909PhraseVariation::PhraseLift) => base || step.rem_euclid(8) == 7,
        Some(Tr909PhraseVariation::PhraseDrive) => base || matches!(step.rem_euclid(4), 1 | 3),
        Some(Tr909PhraseVariation::PhraseRelease) => base && step.rem_euclid(4) == 0,
    }
}

fn trigger_envelope(render: &RealtimeTr909RenderState) -> f32 {
    let base = match render.routing {
        Tr909RenderRouting::SourceOnly => 0.0,
        Tr909RenderRouting::DrumBusSupport => 0.22,
        Tr909RenderRouting::DrumBusTakeover => 0.34,
    };
    let profile_boost = match render.mode {
        Tr909RenderMode::SourceSupport => match render.source_support_profile {
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => 0.0,
            Some(Tr909SourceSupportProfile::BreakLift) => 0.03,
            Some(Tr909SourceSupportProfile::DropDrive) => 0.08,
        },
        Tr909RenderMode::Takeover => match render.takeover_profile {
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 0.06,
            Some(Tr909TakeoverRenderProfile::SceneLock) => 0.1,
        },
        Tr909RenderMode::Fill => 0.04,
        Tr909RenderMode::BreakReinforce => 0.02,
        Tr909RenderMode::Idle => 0.0,
    };
    let pattern_boost = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 0.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 0.04,
        Some(Tr909PatternAdoption::TakeoverGrid) => 0.07,
    };
    let phrase_boost = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => 0.03,
        Some(Tr909PhraseVariation::PhraseDrive) => 0.06,
        Some(Tr909PhraseVariation::PhraseRelease) => -0.05,
    };
    let context_boost = match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => 0.035,
        _ => 0.0,
    };
    (base
        + profile_boost
        + pattern_boost
        + phrase_boost
        + context_boost
        + (render.slam_intensity * 0.2))
        .clamp(0.0, 0.8)
}

fn trigger_frequency(render: &RealtimeTr909RenderState, step: i64) -> f32 {
    let accent = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => {
            if step % 2 == 0 {
                0.0
            } else {
                14.0
            }
        }
        Some(Tr909PatternAdoption::MainlineDrive) => {
            if step.rem_euclid(4) == 3 {
                18.0
            } else {
                6.0
            }
        }
        Some(Tr909PatternAdoption::TakeoverGrid) => match step.rem_euclid(4) {
            0 => 22.0,
            2 => 10.0,
            _ => 4.0,
        },
    };
    let phrase_pitch = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => 6.0,
        Some(Tr909PhraseVariation::PhraseDrive) => 12.0,
        Some(Tr909PhraseVariation::PhraseRelease) => -8.0,
    };
    let slam = render.slam_intensity.clamp(0.0, 1.0) * 18.0;
    match render.mode {
        Tr909RenderMode::Idle => 0.0,
        Tr909RenderMode::SourceSupport => {
            let base = match render.source_support_profile {
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 52.0,
                Some(Tr909SourceSupportProfile::BreakLift) => 66.0,
                Some(Tr909SourceSupportProfile::DropDrive) => 78.0,
            };
            base + accent + phrase_pitch + slam
        }
        Tr909RenderMode::Fill => 78.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::BreakReinforce => 64.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::Takeover => {
            let base = match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 92.0,
                Some(Tr909TakeoverRenderProfile::SceneLock) => 108.0,
            };
            base + accent + phrase_pitch + slam
        }
    }
}

fn render_gain(render: &RealtimeTr909RenderState) -> f32 {
    let routing_gain = match render.routing {
        Tr909RenderRouting::SourceOnly => 0.0,
        Tr909RenderRouting::DrumBusSupport => 0.12,
        Tr909RenderRouting::DrumBusTakeover => 0.18,
    };
    let pattern_gain = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 1.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 1.08,
        Some(Tr909PatternAdoption::TakeoverGrid) => 1.16,
    };
    let phrase_gain = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 1.0,
        Some(Tr909PhraseVariation::PhraseLift) => 1.06,
        Some(Tr909PhraseVariation::PhraseDrive) => 1.14,
        Some(Tr909PhraseVariation::PhraseRelease) => 0.72,
    };
    let context_gain = match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => 1.08,
        _ => 1.0,
    };
    (routing_gain
        * pattern_gain
        * phrase_gain
        * context_gain
        * render.drum_bus_level.clamp(0.0, 1.0))
    .clamp(0.0, 0.25)
}

fn envelope_decay(render: &RealtimeTr909RenderState) -> f32 {
    let slam = render.slam_intensity.clamp(0.0, 1.0);
    let base = match render.mode {
        Tr909RenderMode::Idle => 0.0,
        Tr909RenderMode::SourceSupport => match render.source_support_profile {
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => 0.992 - (slam * 0.002),
            Some(Tr909SourceSupportProfile::BreakLift) => 0.989 - (slam * 0.003),
            Some(Tr909SourceSupportProfile::DropDrive) => 0.986 - (slam * 0.004),
        },
        Tr909RenderMode::Fill => 0.988 - (slam * 0.003),
        Tr909RenderMode::BreakReinforce => 0.989 - (slam * 0.003),
        Tr909RenderMode::Takeover => match render.takeover_profile {
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 0.986 - (slam * 0.004),
            Some(Tr909TakeoverRenderProfile::SceneLock) => 0.982 - (slam * 0.005),
        },
    };
    let pattern_decay = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 0.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 0.002,
        Some(Tr909PatternAdoption::TakeoverGrid) => 0.004,
    };
    let phrase_decay = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => -0.001,
        Some(Tr909PhraseVariation::PhraseDrive) => -0.003,
        Some(Tr909PhraseVariation::PhraseRelease) => 0.01,
    };
    (base - pattern_decay - phrase_decay).clamp(0.0, 1.0)
}

const fn mode_to_u32(mode: Tr909RenderMode) -> u32 {
    match mode {
        Tr909RenderMode::Idle => 0,
        Tr909RenderMode::SourceSupport => 1,
        Tr909RenderMode::Fill => 2,
        Tr909RenderMode::BreakReinforce => 3,
        Tr909RenderMode::Takeover => 4,
    }
}

const fn mode_from_u32(value: u32) -> Tr909RenderMode {
    match value {
        1 => Tr909RenderMode::SourceSupport,
        2 => Tr909RenderMode::Fill,
        3 => Tr909RenderMode::BreakReinforce,
        4 => Tr909RenderMode::Takeover,
        _ => Tr909RenderMode::Idle,
    }
}

const fn routing_to_u32(routing: Tr909RenderRouting) -> u32 {
    match routing {
        Tr909RenderRouting::SourceOnly => 0,
        Tr909RenderRouting::DrumBusSupport => 1,
        Tr909RenderRouting::DrumBusTakeover => 2,
    }
}

const fn routing_from_u32(value: u32) -> Tr909RenderRouting {
    match value {
        1 => Tr909RenderRouting::DrumBusSupport,
        2 => Tr909RenderRouting::DrumBusTakeover,
        _ => Tr909RenderRouting::SourceOnly,
    }
}

const fn support_profile_to_u32(profile: Option<Tr909SourceSupportProfile>) -> u32 {
    match profile {
        None => 0,
        Some(Tr909SourceSupportProfile::SteadyPulse) => 1,
        Some(Tr909SourceSupportProfile::BreakLift) => 2,
        Some(Tr909SourceSupportProfile::DropDrive) => 3,
    }
}

const fn support_profile_from_u32(value: u32) -> Option<Tr909SourceSupportProfile> {
    match value {
        1 => Some(Tr909SourceSupportProfile::SteadyPulse),
        2 => Some(Tr909SourceSupportProfile::BreakLift),
        3 => Some(Tr909SourceSupportProfile::DropDrive),
        _ => None,
    }
}

const fn support_context_to_u32(context: Option<Tr909SourceSupportContext>) -> u32 {
    match context {
        None => 0,
        Some(Tr909SourceSupportContext::SceneTarget) => 1,
        Some(Tr909SourceSupportContext::TransportBar) => 2,
    }
}

const fn support_context_from_u32(value: u32) -> Option<Tr909SourceSupportContext> {
    match value {
        1 => Some(Tr909SourceSupportContext::SceneTarget),
        2 => Some(Tr909SourceSupportContext::TransportBar),
        _ => None,
    }
}

const fn pattern_adoption_to_u32(pattern: Option<Tr909PatternAdoption>) -> u32 {
    match pattern {
        None => 0,
        Some(Tr909PatternAdoption::SupportPulse) => 1,
        Some(Tr909PatternAdoption::MainlineDrive) => 2,
        Some(Tr909PatternAdoption::TakeoverGrid) => 3,
    }
}

const fn pattern_adoption_from_u32(value: u32) -> Option<Tr909PatternAdoption> {
    match value {
        1 => Some(Tr909PatternAdoption::SupportPulse),
        2 => Some(Tr909PatternAdoption::MainlineDrive),
        3 => Some(Tr909PatternAdoption::TakeoverGrid),
        _ => None,
    }
}

const fn phrase_variation_to_u32(variation: Option<Tr909PhraseVariation>) -> u32 {
    match variation {
        None => 0,
        Some(Tr909PhraseVariation::PhraseAnchor) => 1,
        Some(Tr909PhraseVariation::PhraseLift) => 2,
        Some(Tr909PhraseVariation::PhraseDrive) => 3,
        Some(Tr909PhraseVariation::PhraseRelease) => 4,
    }
}

const fn phrase_variation_from_u32(value: u32) -> Option<Tr909PhraseVariation> {
    match value {
        1 => Some(Tr909PhraseVariation::PhraseAnchor),
        2 => Some(Tr909PhraseVariation::PhraseLift),
        3 => Some(Tr909PhraseVariation::PhraseDrive),
        4 => Some(Tr909PhraseVariation::PhraseRelease),
        _ => None,
    }
}

const fn takeover_profile_to_u32(profile: Option<Tr909TakeoverRenderProfile>) -> u32 {
    match profile {
        None => 0,
        Some(Tr909TakeoverRenderProfile::ControlledPhrase) => 1,
        Some(Tr909TakeoverRenderProfile::SceneLock) => 2,
    }
}

const fn takeover_profile_from_u32(value: u32) -> Option<Tr909TakeoverRenderProfile> {
    match value {
        1 => Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        2 => Some(Tr909TakeoverRenderProfile::SceneLock),
        _ => None,
    }
}

const fn w30_mode_to_u32(mode: W30PreviewRenderMode) -> u32 {
    match mode {
        W30PreviewRenderMode::Idle => 0,
        W30PreviewRenderMode::LiveRecall => 1,
        W30PreviewRenderMode::RawCaptureAudition => 2,
        W30PreviewRenderMode::PromotedAudition => 3,
    }
}

const fn w30_mode_from_u32(value: u32) -> W30PreviewRenderMode {
    match value {
        1 => W30PreviewRenderMode::LiveRecall,
        2 => W30PreviewRenderMode::RawCaptureAudition,
        3 => W30PreviewRenderMode::PromotedAudition,
        _ => W30PreviewRenderMode::Idle,
    }
}

const fn w30_routing_to_u32(routing: W30PreviewRenderRouting) -> u32 {
    match routing {
        W30PreviewRenderRouting::Silent => 0,
        W30PreviewRenderRouting::MusicBusPreview => 1,
    }
}

const fn w30_routing_from_u32(value: u32) -> W30PreviewRenderRouting {
    match value {
        1 => W30PreviewRenderRouting::MusicBusPreview,
        _ => W30PreviewRenderRouting::Silent,
    }
}

const fn w30_source_profile_to_u32(profile: Option<W30PreviewSourceProfile>) -> u32 {
    match profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1,
        Some(W30PreviewSourceProfile::PromotedRecall) => 2,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 3,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 4,
        Some(W30PreviewSourceProfile::PromotedAudition) => 5,
        None => 0,
    }
}

const fn w30_source_profile_from_u32(value: u32) -> Option<W30PreviewSourceProfile> {
    match value {
        1 => Some(W30PreviewSourceProfile::PinnedRecall),
        2 => Some(W30PreviewSourceProfile::PromotedRecall),
        3 => Some(W30PreviewSourceProfile::SlicePoolBrowse),
        4 => Some(W30PreviewSourceProfile::RawCaptureAudition),
        5 => Some(W30PreviewSourceProfile::PromotedAudition),
        _ => None,
    }
}

#[derive(Default)]
struct RuntimeTelemetrySnapshot {
    callback_count: u64,
    max_callback_gap_micros: Option<u64>,
    stream_error_count: u64,
    last_stream_error: Option<String>,
    timing: AudioRuntimeTimingSnapshot,
}

struct RuntimeTelemetry {
    callback_count: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    last_callback_micros: AtomicU64,
    stream_error_count: AtomicU64,
    last_stream_error: Mutex<Option<String>>,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl RuntimeTelemetry {
    fn new() -> Self {
        Self {
            callback_count: AtomicU64::new(0),
            max_callback_gap_micros: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            last_stream_error: Mutex::new(None),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0.0_f32.to_bits()),
            position_beats_bits: AtomicU64::new(0.0_f64.to_bits()),
        }
    }

    fn record_callback_at(&self, now_micros: u64, timing: &CallbackTimingSnapshot) {
        let previous = self
            .last_callback_micros
            .swap(now_micros, Ordering::Relaxed);
        if previous != 0 {
            let gap = now_micros.saturating_sub(previous);
            self.max_callback_gap_micros
                .fetch_max(gap, Ordering::Relaxed);
        }
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        self.is_transport_running
            .store(timing.is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(timing.tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(timing.completed_position_beats.to_bits(), Ordering::Relaxed);
    }

    fn record_stream_error(&self, message: String) {
        self.stream_error_count.fetch_add(1, Ordering::Relaxed);
        *self
            .last_stream_error
            .lock()
            .expect("lock stream error buffer") = Some(message);
    }

    fn snapshot(&self) -> RuntimeTelemetrySnapshot {
        let callback_count = self.callback_count.load(Ordering::Relaxed);
        let max_gap_micros = self.max_callback_gap_micros.load(Ordering::Relaxed);

        RuntimeTelemetrySnapshot {
            callback_count,
            max_callback_gap_micros: (callback_count > 1).then_some(max_gap_micros),
            stream_error_count: self.stream_error_count.load(Ordering::Relaxed),
            last_stream_error: self
                .last_stream_error
                .lock()
                .expect("lock stream error buffer")
                .clone(),
            timing: AudioRuntimeTimingSnapshot {
                is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
                tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
                position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
            },
        }
    }

    fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        self.snapshot().timing
    }
}

#[cfg(test)]
mod tests;

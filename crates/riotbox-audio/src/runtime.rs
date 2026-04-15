use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Instant,
};

use crate::tr909::{
    Tr909RenderMode, Tr909RenderRouting, Tr909RenderState, Tr909SourceSupportProfile,
    Tr909TakeoverRenderProfile,
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

pub struct AudioRuntimeShell {
    lifecycle: AudioRuntimeLifecycle,
    output: Option<AudioOutputInfo>,
    telemetry: Arc<RuntimeTelemetry>,
    tr909_render: Arc<SharedTr909RenderState>,
    stream: Option<cpal::Stream>,
}

impl AudioRuntimeShell {
    pub fn start_default_output() -> Result<Self, AudioRuntimeError> {
        Self::start_default_output_with_tr909(Tr909RenderState::default())
    }

    pub fn start_default_output_with_tr909(
        render_state: Tr909RenderState,
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
        let tr909_render = Arc::new(SharedTr909RenderState::new(&render_state));
        let stream_config = default_config.config();
        let start = Instant::now();

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => build_silent_output_stream::<f32>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
                Arc::clone(&tr909_render),
                start,
            ),
            cpal::SampleFormat::I16 => build_silent_output_stream::<i16>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
                Arc::clone(&tr909_render),
                start,
            ),
            cpal::SampleFormat::U16 => build_silent_output_stream::<u16>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
                Arc::clone(&tr909_render),
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
            tr909_render,
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

    pub fn update_tr909_render_state(&self, render_state: &Tr909RenderState) {
        self.tr909_render.update(render_state);
    }

    pub fn stop(&mut self) {
        self.stream.take();
        self.lifecycle = AudioRuntimeLifecycle::Stopped;
    }

    #[cfg(test)]
    fn from_test_parts(
        lifecycle: AudioRuntimeLifecycle,
        output: Option<AudioOutputInfo>,
        telemetry: Arc<RuntimeTelemetry>,
        render_state: Arc<SharedTr909RenderState>,
    ) -> Self {
        Self {
            lifecycle,
            output,
            telemetry,
            tr909_render: render_state,
            stream: None,
        }
    }
}

impl Drop for AudioRuntimeShell {
    fn drop(&mut self) {
        self.stop();
    }
}

fn build_silent_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    telemetry: Arc<RuntimeTelemetry>,
    tr909_render: Arc<SharedTr909RenderState>,
    start: Instant,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let callback_telemetry = Arc::clone(&telemetry);
    let error_telemetry = Arc::clone(&telemetry);
    let mut render_state = Tr909CallbackState::default();
    let sample_rate = config.sample_rate;
    let channel_count = usize::from(config.channels.max(1));

    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            render_tr909_buffer(
                data,
                sample_rate,
                channel_count,
                &tr909_render.snapshot(),
                &mut render_state,
            );

            let now = start.elapsed().as_micros() as u64;
            callback_telemetry.record_callback_at(now);
        },
        move |error| {
            error_telemetry.record_stream_error(error.to_string());
        },
        None,
    )
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeTr909RenderState {
    mode: Tr909RenderMode,
    routing: Tr909RenderRouting,
    source_support_profile: Option<Tr909SourceSupportProfile>,
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

#[derive(Debug, Default)]
struct Tr909CallbackState {
    beat_position: f64,
    oscillator_phase: f32,
    oscillator_hz: f32,
    envelope: f32,
    last_step: i64,
    was_running: bool,
}

fn render_tr909_buffer<T>(
    data: &mut [T],
    sample_rate: u32,
    channel_count: usize,
    render: &RealtimeTr909RenderState,
    state: &mut Tr909CallbackState,
) where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    if !render.is_transport_running
        || matches!(render.mode, Tr909RenderMode::Idle)
        || render.tempo_bpm <= 0.0
    {
        state.was_running = false;
        state.envelope = 0.0;
        state.beat_position = render.position_beats;
        for sample in data.iter_mut() {
            *sample = T::from_sample(0.0);
        }
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
            data[base + channel] = T::from_sample(sample);
        }

        state.beat_position += beats_per_sample;
    }
}

const fn render_subdivision(render: &RealtimeTr909RenderState) -> u32 {
    match render.mode {
        Tr909RenderMode::Idle => 1,
        Tr909RenderMode::SourceSupport => match render.source_support_profile {
            Some(Tr909SourceSupportProfile::BreakLift | Tr909SourceSupportProfile::DropDrive) => 2,
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => 1,
        },
        Tr909RenderMode::Fill | Tr909RenderMode::BreakReinforce | Tr909RenderMode::Takeover => 2,
    }
}

fn should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
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
    (base + profile_boost + (render.slam_intensity * 0.2)).clamp(0.0, 0.8)
}

fn trigger_frequency(render: &RealtimeTr909RenderState, step: i64) -> f32 {
    let accent = if step % 2 == 0 { 0.0 } else { 14.0 };
    let slam = render.slam_intensity.clamp(0.0, 1.0) * 18.0;
    match render.mode {
        Tr909RenderMode::Idle => 0.0,
        Tr909RenderMode::SourceSupport => {
            let base = match render.source_support_profile {
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 52.0,
                Some(Tr909SourceSupportProfile::BreakLift) => 66.0,
                Some(Tr909SourceSupportProfile::DropDrive) => 78.0,
            };
            base + accent + slam
        }
        Tr909RenderMode::Fill => 78.0 + accent + slam,
        Tr909RenderMode::BreakReinforce => 64.0 + accent + slam,
        Tr909RenderMode::Takeover => {
            let base = match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 92.0,
                Some(Tr909TakeoverRenderProfile::SceneLock) => 108.0,
            };
            base + accent + slam
        }
    }
}

fn render_gain(render: &RealtimeTr909RenderState) -> f32 {
    let routing_gain = match render.routing {
        Tr909RenderRouting::SourceOnly => 0.0,
        Tr909RenderRouting::DrumBusSupport => 0.12,
        Tr909RenderRouting::DrumBusTakeover => 0.18,
    };
    (routing_gain * render.drum_bus_level.clamp(0.0, 1.0)).clamp(0.0, 0.25)
}

fn envelope_decay(render: &RealtimeTr909RenderState) -> f32 {
    let slam = render.slam_intensity.clamp(0.0, 1.0);
    match render.mode {
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
    }
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

#[derive(Default)]
struct RuntimeTelemetrySnapshot {
    callback_count: u64,
    max_callback_gap_micros: Option<u64>,
    stream_error_count: u64,
    last_stream_error: Option<String>,
}

struct RuntimeTelemetry {
    callback_count: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    last_callback_micros: AtomicU64,
    stream_error_count: AtomicU64,
    last_stream_error: Mutex<Option<String>>,
}

impl RuntimeTelemetry {
    fn new() -> Self {
        Self {
            callback_count: AtomicU64::new(0),
            max_callback_gap_micros: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            last_stream_error: Mutex::new(None),
        }
    }

    fn record_callback_at(&self, now_micros: u64) {
        let previous = self
            .last_callback_micros
            .swap(now_micros, Ordering::Relaxed);
        if previous != 0 {
            let gap = now_micros.saturating_sub(previous);
            self.max_callback_gap_micros
                .fetch_max(gap, Ordering::Relaxed);
        }
        self.callback_count.fetch_add(1, Ordering::Relaxed);
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tr909::{
        Tr909RenderMode, Tr909RenderRouting, Tr909RenderState, Tr909SourceSupportProfile,
        Tr909TakeoverRenderProfile,
    };

    fn sample_output() -> AudioOutputInfo {
        AudioOutputInfo {
            host_name: "Alsa".into(),
            device_name: "default".into(),
            sample_format: "F32".into(),
            sample_rate: 44_100,
            channel_count: 2,
            buffer_size: "Default".into(),
            supported_output_config_count: Some(12),
        }
    }

    #[test]
    fn telemetry_tracks_callback_count_and_max_gap() {
        let telemetry = RuntimeTelemetry::new();
        telemetry.record_callback_at(100);
        telemetry.record_callback_at(350);
        telemetry.record_callback_at(775);

        let snapshot = telemetry.snapshot();

        assert_eq!(snapshot.callback_count, 3);
        assert_eq!(snapshot.max_callback_gap_micros, Some(425));
    }

    #[test]
    fn health_snapshot_reflects_faulted_runtime_state() {
        let telemetry = Arc::new(RuntimeTelemetry::new());
        let render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
        telemetry.record_callback_at(100);
        telemetry.record_callback_at(240);
        telemetry.record_stream_error("stream stalled".into());

        let shell = AudioRuntimeShell::from_test_parts(
            AudioRuntimeLifecycle::Running,
            Some(sample_output()),
            telemetry,
            render_state,
        );

        let snapshot = shell.health_snapshot();

        assert_eq!(snapshot.lifecycle, AudioRuntimeLifecycle::Faulted);
        assert_eq!(snapshot.callback_count, 2);
        assert_eq!(snapshot.max_callback_gap_micros, Some(140));
        assert_eq!(snapshot.stream_error_count, 1);
        assert_eq!(
            snapshot.last_stream_error.as_deref(),
            Some("stream stalled")
        );
    }

    #[test]
    fn stop_transitions_runtime_to_stopped() {
        let telemetry = Arc::new(RuntimeTelemetry::new());
        let render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
        let mut shell = AudioRuntimeShell::from_test_parts(
            AudioRuntimeLifecycle::Running,
            Some(sample_output()),
            telemetry,
            render_state,
        );

        shell.stop();

        assert_eq!(shell.lifecycle(), AudioRuntimeLifecycle::Stopped);
    }

    #[test]
    fn shared_render_state_tracks_updates() {
        let shared = SharedTr909RenderState::new(&Tr909RenderState::default());
        let mut state = Tr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.8,
            slam_intensity: 0.9,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 17.5,
            ..Tr909RenderState::default()
        };
        shared.update(&state);

        let snapshot = shared.snapshot();
        assert_eq!(snapshot.mode, Tr909RenderMode::Takeover);
        assert_eq!(snapshot.routing, Tr909RenderRouting::DrumBusTakeover);
        assert_eq!(
            snapshot.takeover_profile,
            Some(Tr909TakeoverRenderProfile::ControlledPhrase)
        );
        assert_eq!(snapshot.tempo_bpm, 128.0);
        assert_eq!(snapshot.position_beats, 17.5);

        state.mode = Tr909RenderMode::SourceSupport;
        state.routing = Tr909RenderRouting::DrumBusSupport;
        state.source_support_profile = Some(Tr909SourceSupportProfile::DropDrive);
        state.takeover_profile = None;
        shared.update(&state);

        let updated = shared.snapshot();
        assert_eq!(updated.mode, Tr909RenderMode::SourceSupport);
        assert_eq!(updated.routing, Tr909RenderRouting::DrumBusSupport);
        assert_eq!(
            updated.source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        );
    }

    #[test]
    fn render_buffer_stays_silent_when_idle() {
        let mut state = Tr909CallbackState::default();
        let mut buffer = [1.0_f32; 128];

        render_tr909_buffer(
            &mut buffer,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::Idle,
                routing: Tr909RenderRouting::SourceOnly,
                source_support_profile: None,
                takeover_profile: None,
                drum_bus_level: 0.8,
                slam_intensity: 0.2,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 0.0,
            },
            &mut state,
        );

        assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn render_buffer_produces_audible_samples_for_support_mode() {
        let mut state = Tr909CallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_tr909_buffer(
            &mut buffer,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::BreakReinforce,
                routing: Tr909RenderRouting::DrumBusSupport,
                source_support_profile: None,
                takeover_profile: None,
                drum_bus_level: 0.8,
                slam_intensity: 0.6,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 0.0,
            },
            &mut state,
        );

        assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
    }

    #[test]
    fn render_buffer_respects_zero_drum_bus_level() {
        let mut state = Tr909CallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_tr909_buffer(
            &mut buffer,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::BreakReinforce,
                routing: Tr909RenderRouting::DrumBusSupport,
                source_support_profile: None,
                takeover_profile: None,
                drum_bus_level: 0.0,
                slam_intensity: 0.6,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 0.0,
            },
            &mut state,
        );

        assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn source_support_profiles_produce_different_peak_levels() {
        let mut steady_state = Tr909CallbackState::default();
        let mut drive_state = Tr909CallbackState::default();
        let mut steady = [0.0_f32; 512];
        let mut drive = [0.0_f32; 512];

        render_tr909_buffer(
            &mut steady,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::SourceSupport,
                routing: Tr909RenderRouting::DrumBusSupport,
                source_support_profile: Some(Tr909SourceSupportProfile::SteadyPulse),
                takeover_profile: None,
                drum_bus_level: 0.8,
                slam_intensity: 0.35,
                is_transport_running: true,
                tempo_bpm: 126.0,
                position_beats: 0.0,
            },
            &mut steady_state,
        );

        render_tr909_buffer(
            &mut drive,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::SourceSupport,
                routing: Tr909RenderRouting::DrumBusSupport,
                source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
                takeover_profile: None,
                drum_bus_level: 0.8,
                slam_intensity: 0.35,
                is_transport_running: true,
                tempo_bpm: 126.0,
                position_beats: 0.0,
            },
            &mut drive_state,
        );

        let steady_peak = steady
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let drive_peak = drive
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

        assert!(drive_peak > steady_peak);
    }

    #[test]
    fn controlled_phrase_takeover_profile_is_more_active_than_scene_lock() {
        let mut controlled_state = Tr909CallbackState::default();
        let mut lock_state = Tr909CallbackState::default();
        let mut controlled = [0.0_f32; 512];
        let mut scene_lock = [0.0_f32; 512];

        render_tr909_buffer(
            &mut controlled,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::Takeover,
                routing: Tr909RenderRouting::DrumBusTakeover,
                source_support_profile: None,
                takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
                drum_bus_level: 0.8,
                slam_intensity: 0.45,
                is_transport_running: true,
                tempo_bpm: 126.0,
                position_beats: 0.0,
            },
            &mut controlled_state,
        );

        render_tr909_buffer(
            &mut scene_lock,
            44_100,
            2,
            &RealtimeTr909RenderState {
                mode: Tr909RenderMode::Takeover,
                routing: Tr909RenderRouting::DrumBusTakeover,
                source_support_profile: None,
                takeover_profile: Some(Tr909TakeoverRenderProfile::SceneLock),
                drum_bus_level: 0.8,
                slam_intensity: 0.45,
                is_transport_running: true,
                tempo_bpm: 126.0,
                position_beats: 0.0,
            },
            &mut lock_state,
        );

        let controlled_active = controlled
            .iter()
            .filter(|sample| sample.abs() > 0.0001)
            .count();
        let scene_lock_active = scene_lock
            .iter()
            .filter(|sample| sample.abs() > 0.0001)
            .count();

        assert!(controlled_active > scene_lock_active);
    }
}

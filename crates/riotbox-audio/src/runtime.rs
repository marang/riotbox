use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
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
    stream: Option<cpal::Stream>,
}

impl AudioRuntimeShell {
    pub fn start_default_output() -> Result<Self, AudioRuntimeError> {
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
        let stream_config = default_config.config();
        let start = Instant::now();

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => build_silent_output_stream::<f32>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
                start,
            ),
            cpal::SampleFormat::I16 => build_silent_output_stream::<i16>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
                start,
            ),
            cpal::SampleFormat::U16 => build_silent_output_stream::<u16>(
                &device,
                &stream_config,
                Arc::clone(&telemetry),
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

    pub fn stop(&mut self) {
        self.stream.take();
        self.lifecycle = AudioRuntimeLifecycle::Stopped;
    }

    #[cfg(test)]
    fn from_test_parts(
        lifecycle: AudioRuntimeLifecycle,
        output: Option<AudioOutputInfo>,
        telemetry: Arc<RuntimeTelemetry>,
    ) -> Self {
        Self {
            lifecycle,
            output,
            telemetry,
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
    start: Instant,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let callback_telemetry = Arc::clone(&telemetry);
    let error_telemetry = Arc::clone(&telemetry);

    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            for sample in data.iter_mut() {
                *sample = T::from_sample(0.0);
            }

            let now = start.elapsed().as_micros() as u64;
            callback_telemetry.record_callback_at(now);
        },
        move |error| {
            error_telemetry.record_stream_error(error.to_string());
        },
        None,
    )
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
        telemetry.record_callback_at(100);
        telemetry.record_callback_at(240);
        telemetry.record_stream_error("stream stalled".into());

        let shell = AudioRuntimeShell::from_test_parts(
            AudioRuntimeLifecycle::Running,
            Some(sample_output()),
            telemetry,
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
        let mut shell = AudioRuntimeShell::from_test_parts(
            AudioRuntimeLifecycle::Running,
            Some(sample_output()),
            telemetry,
        );

        shell.stop();

        assert_eq!(shell.lifecycle(), AudioRuntimeLifecycle::Stopped);
    }
}

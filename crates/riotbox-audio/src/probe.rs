use std::{
    fmt::{self, Display, Formatter},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioProbeSummary {
    pub host_name: String,
    pub default_output_device_name: Option<String>,
    pub default_output_config: Option<String>,
    pub supported_output_config_count: Option<usize>,
    pub callback_count: u64,
    pub max_callback_gap_micros: Option<u64>,
    pub stream_result: StreamProbeResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamProbeResult {
    NotRun { reason: String },
    Ok,
    Failed { reason: String },
}

impl Display for AudioProbeSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "host: {}", self.host_name)?;
        writeln!(
            f,
            "default_output_device: {}",
            self.default_output_device_name
                .as_deref()
                .unwrap_or("<none>")
        )?;
        writeln!(
            f,
            "default_output_config: {}",
            self.default_output_config.as_deref().unwrap_or("<none>")
        )?;
        writeln!(
            f,
            "supported_output_configs: {}",
            self.supported_output_config_count
                .map_or_else(|| "<unknown>".to_string(), |count| count.to_string())
        )?;
        writeln!(f, "callback_count: {}", self.callback_count)?;
        writeln!(
            f,
            "max_callback_gap_micros: {}",
            self.max_callback_gap_micros
                .map_or_else(|| "<none>".to_string(), |gap| gap.to_string())
        )?;
        write!(f, "stream_result: {:?}", self.stream_result)
    }
}

pub fn run_output_probe(run_for: Duration) -> AudioProbeSummary {
    let host = cpal::default_host();
    let host_name = format!("{:?}", host.id());

    let Some(device) = host.default_output_device() else {
        return AudioProbeSummary {
            host_name,
            default_output_device_name: None,
            default_output_config: None,
            supported_output_config_count: None,
            callback_count: 0,
            max_callback_gap_micros: None,
            stream_result: StreamProbeResult::NotRun {
                reason: "no default output device".into(),
            },
        };
    };

    #[allow(deprecated)]
    let device_name = device.name().ok();
    let supported_output_config_count = device
        .supported_output_configs()
        .ok()
        .map(|configs| configs.count());

    let default_config = match device.default_output_config() {
        Ok(config) => config,
        Err(error) => {
            return AudioProbeSummary {
                host_name,
                default_output_device_name: device_name,
                default_output_config: None,
                supported_output_config_count,
                callback_count: 0,
                max_callback_gap_micros: None,
                stream_result: StreamProbeResult::Failed {
                    reason: format!("default_output_config failed: {error}"),
                },
            };
        }
    };

    let default_output_config = Some(format!(
        "{:?}, channels={}, sample_rate={}, buffer_size={:?}",
        default_config.sample_format(),
        default_config.channels(),
        default_config.sample_rate(),
        default_config.buffer_size()
    ));

    let callback_count = Arc::new(AtomicU64::new(0));
    let max_gap_micros = Arc::new(AtomicU64::new(0));
    let last_callback_micros = Arc::new(AtomicU64::new(0));

    let callback_count_clone = Arc::clone(&callback_count);
    let max_gap_micros_clone = Arc::clone(&max_gap_micros);
    let last_callback_micros_clone = Arc::clone(&last_callback_micros);
    let start = Instant::now();

    let err_fn = |error| eprintln!("stream error: {error}");

    let stream_config = default_config.config();
    let stream_result = match default_config.sample_format() {
        cpal::SampleFormat::F32 => build_and_run_stream::<f32>(
            &device,
            &stream_config,
            run_for,
            callback_count_clone,
            max_gap_micros_clone,
            last_callback_micros_clone,
            start,
            err_fn,
        ),
        cpal::SampleFormat::I16 => build_and_run_stream::<i16>(
            &device,
            &stream_config,
            run_for,
            callback_count_clone,
            max_gap_micros_clone,
            last_callback_micros_clone,
            start,
            err_fn,
        ),
        cpal::SampleFormat::U16 => build_and_run_stream::<u16>(
            &device,
            &stream_config,
            run_for,
            callback_count_clone,
            max_gap_micros_clone,
            last_callback_micros_clone,
            start,
            err_fn,
        ),
        sample_format => StreamProbeResult::Failed {
            reason: format!("unsupported sample format in spike: {sample_format:?}"),
        },
    };

    let callback_count = callback_count.load(Ordering::Relaxed);
    let max_gap_micros = max_gap_micros.load(Ordering::Relaxed);

    AudioProbeSummary {
        host_name,
        default_output_device_name: device_name,
        default_output_config,
        supported_output_config_count,
        callback_count,
        max_callback_gap_micros: (callback_count > 1).then_some(max_gap_micros),
        stream_result,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_and_run_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    run_for: Duration,
    callback_count: Arc<AtomicU64>,
    max_gap_micros: Arc<AtomicU64>,
    last_callback_micros: Arc<AtomicU64>,
    start: Instant,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> StreamProbeResult
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let stream = match device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            for sample in data.iter_mut() {
                *sample = T::from_sample(0.0);
            }

            let now = start.elapsed().as_micros() as u64;
            let previous = last_callback_micros.swap(now, Ordering::Relaxed);
            if previous != 0 {
                let gap = now.saturating_sub(previous);
                max_gap_micros.fetch_max(gap, Ordering::Relaxed);
            }
            callback_count.fetch_add(1, Ordering::Relaxed);
        },
        err_fn,
        None,
    ) {
        Ok(stream) => stream,
        Err(error) => {
            return StreamProbeResult::Failed {
                reason: format!("build_output_stream failed: {error}"),
            };
        }
    };

    if let Err(error) = stream.play() {
        return StreamProbeResult::Failed {
            reason: format!("stream play failed: {error}"),
        };
    }

    thread::sleep(run_for);
    drop(stream);

    StreamProbeResult::Ok
}

use std::{
    fmt::{self, Display, Formatter},
    thread,
    time::Duration,
};

use crate::runtime::{
    AudioRuntimeError, AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeShell,
};

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
    match AudioRuntimeShell::start_default_output() {
        Ok(mut runtime) => {
            thread::sleep(run_for);
            let snapshot = runtime.health_snapshot();
            runtime.stop();

            AudioProbeSummary::from_health(snapshot)
        }
        Err(error) => AudioProbeSummary::from_error(error),
    }
}

impl AudioProbeSummary {
    fn from_health(health: AudioRuntimeHealth) -> Self {
        let default_output_config = health.output.as_ref().map(|output| {
            format!(
                "{}, channels={}, sample_rate={}, buffer_size={}",
                output.sample_format, output.channel_count, output.sample_rate, output.buffer_size
            )
        });

        let stream_result = if matches!(
            health.lifecycle,
            AudioRuntimeLifecycle::Running | AudioRuntimeLifecycle::Stopped
        ) {
            StreamProbeResult::Ok
        } else {
            StreamProbeResult::Failed {
                reason: health
                    .last_stream_error
                    .unwrap_or_else(|| "runtime entered a faulted state".into()),
            }
        };

        Self {
            host_name: health
                .output
                .as_ref()
                .map(|output| output.host_name.clone())
                .unwrap_or_else(|| "<unknown>".into()),
            default_output_device_name: health
                .output
                .as_ref()
                .map(|output| output.device_name.clone()),
            default_output_config,
            supported_output_config_count: health
                .output
                .as_ref()
                .and_then(|output| output.supported_output_config_count),
            callback_count: health.callback_count,
            max_callback_gap_micros: health.max_callback_gap_micros,
            stream_result,
        }
    }

    fn from_error(error: AudioRuntimeError) -> Self {
        Self {
            host_name: error.host_name().to_string(),
            default_output_device_name: error.device_name().map(ToOwned::to_owned),
            default_output_config: None,
            supported_output_config_count: None,
            callback_count: 0,
            max_callback_gap_micros: None,
            stream_result: StreamProbeResult::Failed {
                reason: error.to_string(),
            },
        }
    }
}

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
    pub callback_scratch_overflow_count: u64,
    pub stream_error_count: u64,
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
        writeln!(
            f,
            "callback_scratch_overflow_count: {}",
            self.callback_scratch_overflow_count
        )?;
        writeln!(f, "stream_error_count: {}", self.stream_error_count)?;
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

        let stream_result = if health.stream_error_count > 0 || health.last_stream_error.is_some() {
            StreamProbeResult::Failed {
                reason: format!(
                    "runtime stream error evidence (count {}): {}",
                    health.stream_error_count,
                    health
                        .last_stream_error
                        .as_deref()
                        .unwrap_or("no error detail available"),
                ),
            }
        } else if health.callback_scratch_overflow_count > 0 {
            StreamProbeResult::Failed {
                reason: format!(
                    "runtime callback scratch overflow count: {}",
                    health.callback_scratch_overflow_count
                ),
            }
        } else if !matches!(
            health.lifecycle,
            AudioRuntimeLifecycle::Running | AudioRuntimeLifecycle::Stopped
        ) {
            StreamProbeResult::Failed {
                reason: format!("runtime lifecycle is {:?}", health.lifecycle),
            }
        } else if health.callback_count == 0 {
            StreamProbeResult::Failed {
                reason: "no output callback observed in the bounded probe window; backend health is unproven".into(),
            }
        } else {
            StreamProbeResult::Ok
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
            callback_scratch_overflow_count: health.callback_scratch_overflow_count,
            stream_error_count: health.stream_error_count,
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
            callback_scratch_overflow_count: 0,
            stream_error_count: 0,
            stream_result: StreamProbeResult::Failed {
                reason: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioProbeSummary, StreamProbeResult};
    use crate::runtime::{
        AudioOutputInfo, AudioRuntimeError, AudioRuntimeHealth, AudioRuntimeLifecycle,
    };

    #[test]
    fn running_without_observed_callbacks_is_not_successful_probe_evidence() {
        for lifecycle in [
            AudioRuntimeLifecycle::Running,
            AudioRuntimeLifecycle::Stopped,
        ] {
            let mut observed = health(0);
            observed.lifecycle = lifecycle;
            let summary = AudioProbeSummary::from_health(observed);
            assert!(failure_reason(&summary).contains("no output callback observed"));
        }
    }

    #[test]
    fn observed_clean_callbacks_pass_without_an_invented_gap_threshold() {
        for lifecycle in [
            AudioRuntimeLifecycle::Running,
            AudioRuntimeLifecycle::Stopped,
        ] {
            let mut observed = health(1);
            observed.lifecycle = lifecycle;
            observed.max_callback_gap_micros = Some(u64::MAX);
            let summary = AudioProbeSummary::from_health(observed);
            assert_eq!(summary.stream_result, StreamProbeResult::Ok);
            assert_eq!(summary.callback_count, 1);
            assert_eq!(summary.max_callback_gap_micros, Some(u64::MAX));
            assert_eq!(summary.host_name, "generated-host");
            assert_eq!(
                summary.default_output_device_name.as_deref(),
                Some("generated-device")
            );
            assert_eq!(summary.stream_error_count, 0);
            assert_eq!(summary.callback_scratch_overflow_count, 0);
            let display = summary.to_string();
            assert!(display.contains("stream_error_count: 0"));
            assert!(display.contains("callback_scratch_overflow_count: 0"));
        }
    }

    #[test]
    fn idle_or_faulted_lifecycle_is_not_success_even_with_callbacks() {
        for lifecycle in [AudioRuntimeLifecycle::Idle, AudioRuntimeLifecycle::Faulted] {
            let mut observed = health(3);
            observed.lifecycle = lifecycle;
            let summary = AudioProbeSummary::from_health(observed);
            assert_eq!(
                failure_reason(&summary),
                format!("runtime lifecycle is {lifecycle:?}")
            );
        }
    }

    #[test]
    fn stream_error_counter_or_detail_precedes_other_failures() {
        for (count, detail) in [
            (2, None),
            (0, Some("retained detail")),
            (2, Some("retained detail")),
        ] {
            for lifecycle in [
                AudioRuntimeLifecycle::Running,
                AudioRuntimeLifecycle::Stopped,
                AudioRuntimeLifecycle::Faulted,
            ] {
                for callbacks in [0, 3] {
                    let mut observed = health(callbacks);
                    observed.lifecycle = lifecycle;
                    observed.stream_error_count = count;
                    observed.last_stream_error = detail.map(str::to_owned);
                    observed.callback_scratch_overflow_count = 7;
                    let summary = AudioProbeSummary::from_health(observed);
                    let reason = failure_reason(&summary);
                    assert!(
                        reason.contains(&format!("runtime stream error evidence (count {count})"))
                    );
                    assert!(reason.contains(detail.unwrap_or("no error detail available")));
                    assert_eq!(summary.stream_error_count, count);
                    assert_eq!(summary.callback_scratch_overflow_count, 7);
                }
            }
        }
    }

    #[test]
    fn scratch_overflow_precedes_lifecycle_or_zero_callback_failure() {
        for lifecycle in [
            AudioRuntimeLifecycle::Running,
            AudioRuntimeLifecycle::Stopped,
            AudioRuntimeLifecycle::Idle,
        ] {
            for callbacks in [0, 3] {
                let mut observed = health(callbacks);
                observed.lifecycle = lifecycle;
                observed.callback_scratch_overflow_count = 2;
                let summary = AudioProbeSummary::from_health(observed);
                assert_eq!(
                    failure_reason(&summary),
                    "runtime callback scratch overflow count: 2"
                );
                assert!(
                    summary
                        .to_string()
                        .contains("callback_scratch_overflow_count: 2")
                );
            }
        }
    }

    #[test]
    fn setup_errors_remain_failed_diagnostics_without_device_invocation() {
        let errors = [
            AudioRuntimeError::NoDefaultOutputDevice {
                host_name: "fixture".into(),
            },
            AudioRuntimeError::DefaultOutputConfig {
                host_name: "fixture".into(),
                device_name: "device".into(),
                reason: "config fault".into(),
            },
            AudioRuntimeError::UnsupportedSampleFormat {
                host_name: "fixture".into(),
                device_name: "device".into(),
                sample_format: "fixture-format".into(),
            },
            AudioRuntimeError::BuildStream {
                host_name: "fixture".into(),
                device_name: "device".into(),
                reason: "build fault".into(),
            },
            AudioRuntimeError::PlayStream {
                host_name: "fixture".into(),
                device_name: "device".into(),
                reason: "play fault".into(),
            },
        ];
        for error in errors {
            let expected = error.to_string();
            let summary = AudioProbeSummary::from_error(error);
            assert_eq!(failure_reason(&summary), expected);
            assert_eq!(summary.host_name, "fixture");
            assert_eq!(summary.callback_count, 0);
            assert_eq!(summary.stream_error_count, 0);
            assert_eq!(summary.callback_scratch_overflow_count, 0);
            assert!(summary.default_output_config.is_none());
        }
    }

    fn failure_reason(summary: &AudioProbeSummary) -> &str {
        match &summary.stream_result {
            StreamProbeResult::Failed { reason } => reason,
            other => panic!("expected failed probe evidence, got {other:?}"),
        }
    }

    fn health(callback_count: u64) -> AudioRuntimeHealth {
        AudioRuntimeHealth {
            lifecycle: AudioRuntimeLifecycle::Running,
            output: Some(AudioOutputInfo {
                host_name: "generated-host".into(),
                device_name: "generated-device".into(),
                sample_format: "F32".into(),
                sample_rate: 48_000,
                channel_count: 2,
                buffer_size: "generated".into(),
                supported_output_config_count: Some(1),
            }),
            callback_count,
            max_callback_gap_micros: None,
            callback_scratch_overflow_count: 0,
            stream_error_count: 0,
            last_stream_error: None,
        }
    }
}

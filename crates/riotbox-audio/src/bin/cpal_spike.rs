use std::{process::ExitCode, time::Duration};

use riotbox_audio::probe::{AudioProbeSummary, StreamProbeResult, run_output_probe};

fn main() -> ExitCode {
    let summary = run_output_probe(Duration::from_millis(250));
    report_summary(&summary)
}

fn report_summary(summary: &AudioProbeSummary) -> ExitCode {
    println!("{summary}");
    match summary.stream_result {
        StreamProbeResult::Ok => ExitCode::SUCCESS,
        StreamProbeResult::Failed { .. } | StreamProbeResult::NotRun { .. } => ExitCode::FAILURE,
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioProbeSummary, ExitCode, StreamProbeResult, report_summary};

    #[test]
    fn reporting_returns_success_only_for_typed_ok_without_starting_a_device() {
        for (stream_result, expected) in [
            (StreamProbeResult::Ok, ExitCode::SUCCESS),
            (
                StreamProbeResult::Failed {
                    reason: "generated startup failure".into(),
                },
                ExitCode::FAILURE,
            ),
            (
                StreamProbeResult::NotRun {
                    reason: "generated not-run outcome".into(),
                },
                ExitCode::FAILURE,
            ),
        ] {
            let summary = AudioProbeSummary {
                host_name: "fixture".into(),
                default_output_device_name: None,
                default_output_config: None,
                supported_output_config_count: None,
                callback_count: 0,
                max_callback_gap_micros: None,
                callback_scratch_overflow_count: 0,
                stream_error_count: 0,
                stream_result,
            };
            // Exercise only reporting/exit propagation, not runtime admission.
            assert_eq!(report_summary(&summary), expected);
        }
    }
}

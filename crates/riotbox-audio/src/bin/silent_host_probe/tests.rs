use super::{Event, Outcome, admit_args, frozen_schedule, validate_interval, write_observation};
use riotbox_audio::runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle};
use std::{
    cell::Cell,
    io::{self, Write},
    rc::Rc,
    time::Duration,
};

#[test]
fn complete_sequence_waits_each_interval_and_stops_before_terminal_reporting() {
    let (mut host, mut writer) = generated_observation();
    assert_eq!(
        super::observe(&mut host, &mut writer, &frozen_schedule().unwrap()),
        Ok(())
    );
    assert_eq!(host.waits, vec![Duration::from_secs(1); 60]);
    assert!(host.stopped.get());
    assert_eq!(
        writer.stopped_at_flush,
        [vec![false; 61], vec![true]].concat()
    );
    let records = writer.records();
    assert_eq!(records.len(), 62);
    assert_eq!(records[0]["event"], "started");
    assert_eq!(records[0]["result"], "observing");
    assert_eq!(records[0]["health"]["callback_count"], 0);
    for (index, record) in records.iter().enumerate().take(61).skip(1) {
        assert_eq!(record["event"], "sample");
        assert_eq!(record["sample_index"], index);
        assert_eq!(record["elapsed_ms"], index * 1017);
        assert_eq!(record["health"]["callback_count"], index);
        assert_eq!(record["health"]["lifecycle"], "running");
        assert_eq!(record["result"], "ok");
        assert!(record.get("reason").is_none());
    }
    assert_eq!(records[61]["event"], "stopped");
    assert_eq!(records[61]["sample_index"], 60);
    assert_eq!(records[61]["elapsed_ms"], 60 * 1017);
    assert_eq!(records[61]["health"]["lifecycle"], "stopped");
    assert_eq!(records[61]["result"], "ok");
}

#[test]
fn write_and_flush_failures_stop_at_started_sample_and_terminal_records() {
    for record_index in [0, 1, 61] {
        for failure in [
            OutputFailure::WriteAt(record_index),
            OutputFailure::FlushAt(record_index),
        ] {
            let (mut host, mut writer) = generated_observation();
            writer.failure = Some(failure);
            let error =
                super::observe(&mut host, &mut writer, &frozen_schedule().unwrap()).unwrap_err();
            assert!(error.contains("NDJSON output failed"));
            assert!(host.stopped.get());
            assert_eq!(host.waits.len(), record_index.min(60));
        }
    }
}

#[test]
fn a_failed_interval_stops_without_retry_and_terminal_evidence_stays_failed() {
    let (mut host, mut writer) = generated_observation();
    host.stall_at = Some(2);
    let error = super::observe(&mut host, &mut writer, &frozen_schedule().unwrap()).unwrap_err();
    assert!(error.contains("did not advance"));
    assert_eq!(host.waits.len(), 2);
    assert!(host.stopped.get());
    let records = writer.records();
    assert_eq!(records.len(), 4);
    for record in &records[2..] {
        assert_eq!(record["sample_index"], 2);
        assert_eq!(record["result"], "failed");
        assert_eq!(record["reason"], error);
    }
    assert_eq!(records[3]["event"], "stopped");
    assert_eq!(records[3]["health"]["lifecycle"], "stopped");
    assert_eq!(writer.stopped_at_flush, [false, false, false, true]);
}

#[test]
fn an_initial_error_stops_before_any_interval() {
    let (mut host, mut writer) = generated_observation();
    host.health.last_stream_error = Some("generated startup callback error".into());
    assert!(super::observe(&mut host, &mut writer, &frozen_schedule().unwrap()).is_err());
    assert!(host.waits.is_empty());
    assert!(host.stopped.get());
    let records = writer.records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["event"], "started");
    assert_eq!(records[0]["result"], "failed");
    assert_eq!(records[1]["event"], "stopped");
    assert_eq!(records[1]["sample_index"], 0);
    assert_eq!(records[1]["result"], "failed");
}

#[test]
fn intervals_reject_zero_stalled_and_decreased_callback_counts() {
    for (previous, current) in [(0, 0), (5, 5), (5, 4)] {
        assert!(validate_interval(previous, &health(current)).is_err());
    }
}

#[test]
fn initial_zero_is_observing_but_initial_faults_fail() {
    let mut observed = health(0);
    assert_eq!(super::validate_started(&observed), Ok(()));
    observed.callback_scratch_overflow_count = 1;
    assert!(super::validate_started(&observed).is_err());
    observed.callback_scratch_overflow_count = 0;
    observed.last_stream_error = Some("generated failure".into());
    assert!(super::validate_started(&observed).is_err());
    observed.last_stream_error = None;
    observed.lifecycle = AudioRuntimeLifecycle::Stopped;
    assert!(super::validate_started(&observed).is_err());
}

#[test]
fn stopped_requires_stopped_health_and_cannot_clear_prior_failure() {
    let mut observed = health(60);
    assert!(super::validate_stopped(Ok(()), &observed).is_err());
    observed.lifecycle = AudioRuntimeLifecycle::Stopped;
    assert_eq!(super::validate_stopped(Ok(()), &observed), Ok(()));
    let failed = Err("earlier sample failed".to_owned());
    assert_eq!(super::validate_stopped(failed.clone(), &observed), failed);
    observed.stream_error_count = 1;
    assert!(super::validate_stopped(Ok(()), &observed).is_err());
}

#[test]
fn interval_health_uses_existing_failure_precedence_without_a_gap_threshold() {
    let mut observed = health(6);
    observed.max_callback_gap_micros = Some(u64::MAX);
    assert_eq!(validate_interval(5, &observed), Ok(()));
    observed.lifecycle = AudioRuntimeLifecycle::Stopped;
    assert!(validate_interval(5, &observed).is_err());
    observed.callback_scratch_overflow_count = 1;
    assert!(
        validate_interval(5, &observed)
            .unwrap_err()
            .contains("scratch overflow")
    );
    observed.stream_error_count = 1;
    assert!(
        validate_interval(5, &observed)
            .unwrap_err()
            .contains("stream error")
    );
    observed.stream_error_count = 0;
    observed.last_stream_error = Some("retained error".into());
    assert!(
        validate_interval(5, &observed)
            .unwrap_err()
            .contains("retained error")
    );
}

#[test]
fn only_the_single_explicit_opt_in_admits_host_execution() {
    for args in [vec![], vec!["--help"], vec!["-h"]] {
        assert_eq!(admit_args(&args), Ok(false));
    }
    assert_eq!(admit_args(&["--isolated-silent-host"]), Ok(true));
    for args in [
        vec!["--seconds", "1"],
        vec!["source.wav"],
        vec!["--isolated-silent-host", "--help"],
        vec!["--isolated-silent-host", "--isolated-silent-host"],
    ] {
        assert!(admit_args(&args).is_err());
    }
}

#[test]
fn schedule_is_the_embedded_frozen_sixty_one_second_intervals() {
    let schedule = frozen_schedule().unwrap();
    assert_eq!(schedule.interval_count, 60);
    assert_eq!(schedule.interval, std::time::Duration::from_secs(1));
}

#[test]
fn reporting_preserves_raw_health_and_uses_one_ndjson_line() {
    let mut observed = health(9);
    observed.max_callback_gap_micros = Some(u64::MAX);
    observed.last_stream_error = Some("generated\nerror".into());
    let mut output = Vec::new();
    write_observation(
        &mut output,
        Event::Sample,
        3,
        3123,
        &observed,
        Outcome::Failed {
            reason: "stream error",
        },
    )
    .unwrap();
    assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
    let record: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(record["schema"], "riotbox.silent_host_sample.v1");
    assert_eq!(record["event"], "sample");
    assert_eq!(record["pid"], std::process::id());
    assert_eq!(record["sample_index"], 3);
    assert_eq!(record["elapsed_ms"], 3123);
    assert_eq!(record["result"], "failed");
    assert_eq!(record["reason"], "stream error");
    assert_eq!(record["health"]["lifecycle"], "running");
    assert_eq!(record["health"]["channel_count"], 2);
    assert_eq!(record["health"]["sample_rate"], 48_000);
    assert_eq!(record["health"]["callback_count"], 9);
    assert_eq!(record["health"]["max_callback_gap_micros"], u64::MAX);
    assert_eq!(record["health"]["last_stream_error"], "generated\nerror");
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

struct GeneratedHost {
    health: AudioRuntimeHealth,
    waits: Vec<Duration>,
    elapsed_ms: u128,
    stopped: Rc<Cell<bool>>,
    stall_at: Option<usize>,
}

impl super::ObservationHost for GeneratedHost {
    fn health(&self) -> AudioRuntimeHealth {
        self.health.clone()
    }
    fn elapsed_ms(&self) -> u128 {
        self.elapsed_ms
    }
    fn wait(&mut self, duration: Duration) {
        self.waits.push(duration);
        self.elapsed_ms += duration.as_millis() + 17;
        if self.stall_at != Some(self.waits.len()) {
            self.health.callback_count += 1;
        }
    }
    fn stop(&mut self) {
        self.stopped.set(true);
        self.health.lifecycle = AudioRuntimeLifecycle::Stopped;
    }
}

struct RecordingWriter {
    bytes: Vec<u8>,
    stopped: Rc<Cell<bool>>,
    stopped_at_flush: Vec<bool>,
    failure: Option<OutputFailure>,
}

#[derive(Clone, Copy)]
enum OutputFailure {
    WriteAt(usize),
    FlushAt(usize),
}

impl RecordingWriter {
    fn records(&self) -> Vec<serde_json::Value> {
        self.bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect()
    }
}

impl Write for RecordingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if matches!(self.failure, Some(OutputFailure::WriteAt(index)) if index == self.stopped_at_flush.len())
        {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "generated write failure",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if matches!(self.failure, Some(OutputFailure::FlushAt(index)) if index == self.stopped_at_flush.len())
        {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "generated flush failure",
            ));
        }
        self.stopped_at_flush.push(self.stopped.get());
        Ok(())
    }
}

fn generated_observation() -> (GeneratedHost, RecordingWriter) {
    let stopped = Rc::new(Cell::new(false));
    (
        GeneratedHost {
            health: health(0),
            waits: Vec::new(),
            elapsed_ms: 0,
            stopped: stopped.clone(),
            stall_at: None,
        },
        RecordingWriter {
            bytes: Vec::new(),
            stopped,
            stopped_at_flush: Vec::new(),
            failure: None,
        },
    )
}

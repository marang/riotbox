//! Dev-only silent observation driver. Effective routing and the independent
//! process watchdog belong to the frozen Linux operator, not this binary.

use riotbox_audio::{
    probe::{AudioProbeSummary, StreamProbeResult},
    runtime::{AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeShell},
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    io::{self, Write},
    process::ExitCode,
    time::{Duration, Instant},
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = match admit_args(&args) {
        Ok(true) => run_host(),
        Ok(false) => {
            let _ = writeln!(
                io::stderr().lock(),
                "Usage: silent_host_probe --isolated-silent-host\nRun only through the reviewed isolated Linux operator."
            );
            return ExitCode::SUCCESS;
        }
        Err(reason) => Err(reason.to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            let _ = writeln!(io::stderr().lock(), "silent_host_probe: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn run_host() -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("silent host observation is admitted only on Linux".into());
    }
    let schedule = frozen_schedule()?;
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    let mut host = LiveHost {
        runtime: AudioRuntimeShell::start_default_output()
            .map_err(|error| format!("startup failed: {error}"))?,
        started: Instant::now(),
    };
    observe(&mut host, &mut writer, &schedule)
}

struct LiveHost {
    runtime: AudioRuntimeShell,
    started: Instant,
}

impl ObservationHost for LiveHost {
    fn health(&self) -> AudioRuntimeHealth {
        self.runtime.health_snapshot()
    }
    fn elapsed_ms(&self) -> u128 {
        self.started.elapsed().as_millis()
    }
    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
    fn stop(&mut self) {
        self.runtime.stop();
    }
}

fn admit_args(args: &[impl AsRef<OsStr>]) -> Result<bool, &'static str> {
    match args {
        [] => Ok(false),
        [arg] if arg.as_ref() == "--help" || arg.as_ref() == "-h" => Ok(false),
        [arg] if arg.as_ref() == "--isolated-silent-host" => Ok(true),
        _ => Err("only --isolated-silent-host is admitted; no source or duration options"),
    }
}

struct Schedule {
    interval_count: u64,
    interval: Duration,
}

fn frozen_schedule() -> Result<Schedule, String> {
    #[derive(Deserialize)]
    struct Protocol {
        schema: String,
        run_seconds: u64,
        sample_interval_ms: u64,
    }
    let protocol: Protocol = serde_json::from_str(include_str!(
        "../../../../docs/benchmarks/silent_host_observation_v1.json"
    ))
    .map_err(|error| format!("invalid embedded observation protocol: {error}"))?;
    let milliseconds = protocol.run_seconds.checked_mul(1000);
    if protocol.schema != "riotbox.silent_host_observation.v1"
        || protocol.sample_interval_ms == 0
        || milliseconds.is_none_or(|value| value == 0 || value % protocol.sample_interval_ms != 0)
    {
        return Err("invalid embedded observation schedule".into());
    }
    Ok(Schedule {
        interval_count: milliseconds.expect("validated duration") / protocol.sample_interval_ms,
        interval: Duration::from_millis(protocol.sample_interval_ms),
    })
}

// Private boundary for generated-only tests; the real adapter owns the stream.
trait ObservationHost {
    fn health(&self) -> AudioRuntimeHealth;
    fn elapsed_ms(&self) -> u128;
    fn wait(&mut self, duration: Duration);
    fn stop(&mut self);
}

fn observe(
    host: &mut impl ObservationHost,
    writer: &mut impl Write,
    schedule: &Schedule,
) -> Result<(), String> {
    let mut sample_index = 0;
    let observation = (|| {
        let initial = host.health();
        report_result(
            writer,
            Event::Started,
            0,
            host.elapsed_ms(),
            &initial,
            validate_started(&initial),
        )?;
        let mut previous_count = initial.callback_count;
        for index in 1..=schedule.interval_count {
            // Each observation follows an actual complete interval; a slow write
            // must never produce instantaneous catch-up samples.
            host.wait(schedule.interval);
            sample_index = index;
            let health = host.health();
            report_result(
                writer,
                Event::Sample,
                index,
                host.elapsed_ms(),
                &health,
                validate_interval(previous_count, &health),
            )?;
            previous_count = health.callback_count;
        }
        Ok(())
    })();
    // No observation or stdout error can return before explicit teardown.
    // The real adapter's AudioRuntimeShell also owns its existing Drop guard.
    host.stop();
    let stopped = host.health();
    let result = validate_stopped(observation, &stopped);
    report_result(
        writer,
        Event::Stopped,
        sample_index,
        host.elapsed_ms(),
        &stopped,
        result,
    )
}

fn report_result(
    writer: &mut impl Write,
    event: Event,
    sample_index: u64,
    elapsed_ms: u128,
    health: &AudioRuntimeHealth,
    result: Result<(), String>,
) -> Result<(), String> {
    let outcome = match &result {
        Err(reason) => Outcome::Failed { reason },
        Ok(()) if matches!(event, Event::Started) => Outcome::Observing,
        Ok(()) => Outcome::Ok,
    };
    if let Err(error) = write_observation(writer, event, sample_index, elapsed_ms, health, outcome)
    {
        let detail = format!("NDJSON output failed: {error}");
        return Err(result
            .err()
            .map_or(detail.clone(), |reason| format!("{reason}; {detail}")));
    }
    result
}

fn project_health(health: &AudioRuntimeHealth) -> Result<(), String> {
    match AudioProbeSummary::from_health(health.clone()).stream_result {
        StreamProbeResult::Failed { reason } | StreamProbeResult::NotRun { reason } => Err(reason),
        StreamProbeResult::Ok => Ok(()),
    }
}

fn validate_started(health: &AudioRuntimeHealth) -> Result<(), String> {
    // No interval has elapsed yet: only a clean Running snapshot may temporarily
    // have zero callbacks. This exemption does not change or hide raw evidence.
    if !(health.callback_count == 0
        && health.lifecycle == AudioRuntimeLifecycle::Running
        && health.stream_error_count == 0
        && health.last_stream_error.is_none()
        && health.callback_scratch_overflow_count == 0)
    {
        project_health(health)?;
    }
    if health.lifecycle != AudioRuntimeLifecycle::Running {
        return Err("initial observation requires Running lifecycle".into());
    }
    Ok(())
}

fn validate_interval(previous_count: u64, health: &AudioRuntimeHealth) -> Result<(), String> {
    project_health(health)?;
    if health.lifecycle != AudioRuntimeLifecycle::Running {
        return Err("observation interval requires Running lifecycle".into());
    }
    if health.callback_count <= previous_count {
        return Err(format!(
            "callback_count did not advance (previous {previous_count}, current {})",
            health.callback_count
        ));
    }
    Ok(())
}

fn validate_stopped(
    previous_result: Result<(), String>,
    health: &AudioRuntimeHealth,
) -> Result<(), String> {
    previous_result?;
    project_health(health)?;
    if health.lifecycle != AudioRuntimeLifecycle::Stopped {
        return Err("terminal observation requires Stopped lifecycle".into());
    }
    Ok(())
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Event {
    Started,
    Sample,
    Stopped,
}

#[derive(Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
enum Outcome<'a> {
    Observing,
    Ok,
    Failed { reason: &'a str },
}

#[derive(Serialize)]
struct HealthRecord<'a> {
    lifecycle: &'static str,
    host_name: Option<&'a str>,
    device_name: Option<&'a str>,
    sample_format: Option<&'a str>,
    sample_rate: Option<u32>,
    channel_count: Option<u16>,
    buffer_size: Option<&'a str>,
    callback_count: u64,
    max_callback_gap_micros: Option<u64>,
    callback_scratch_overflow_count: u64,
    stream_error_count: u64,
    last_stream_error: Option<&'a str>,
}

#[derive(Serialize)]
struct ObservationRecord<'a> {
    schema: &'static str,
    event: Event,
    pid: u32,
    sample_index: u64,
    elapsed_ms: u128,
    health: HealthRecord<'a>,
    #[serde(flatten)]
    outcome: Outcome<'a>,
}

fn write_observation(
    writer: &mut impl Write,
    event: Event,
    sample_index: u64,
    elapsed_ms: u128,
    health: &AudioRuntimeHealth,
    outcome: Outcome<'_>,
) -> io::Result<()> {
    let output = health.output.as_ref();
    let record = ObservationRecord {
        schema: "riotbox.silent_host_sample.v1",
        event,
        pid: std::process::id(),
        sample_index,
        elapsed_ms,
        health: HealthRecord {
            lifecycle: match health.lifecycle {
                AudioRuntimeLifecycle::Idle => "idle",
                AudioRuntimeLifecycle::Running => "running",
                AudioRuntimeLifecycle::Stopped => "stopped",
                AudioRuntimeLifecycle::Faulted => "faulted",
            },
            host_name: output.map(|value| value.host_name.as_str()),
            device_name: output.map(|value| value.device_name.as_str()),
            sample_format: output.map(|value| value.sample_format.as_str()),
            sample_rate: output.map(|value| value.sample_rate),
            channel_count: output.map(|value| value.channel_count),
            buffer_size: output.map(|value| value.buffer_size.as_str()),
            callback_count: health.callback_count,
            max_callback_gap_micros: health.max_callback_gap_micros,
            callback_scratch_overflow_count: health.callback_scratch_overflow_count,
            stream_error_count: health.stream_error_count,
            last_stream_error: health.last_stream_error.as_deref(),
        },
        outcome,
    };
    serde_json::to_writer(&mut *writer, &record).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

#[cfg(test)]
#[path = "silent_host_probe/tests.rs"]
mod tests;

# CPAL Audio Latency Spike

Version: 0.1  
Status: Draft  
Ticket: `RIOTBOX-8`

---

## Purpose

Validate the current Rust audio direction with a minimal `cpal` prototype before deeper runtime work starts.

Questions:

- can we enumerate a default host and output device cleanly
- can we build and run a minimal output stream
- what callback-timing surfaces are easy to capture from the start
- what does this imply for `crates/riotbox-audio`

---

## Prototype

Repo prototype:

- crate: `crates/riotbox-audio`
- binary: `cargo run -p riotbox-audio --bin cpal_spike`

What it does:

- discovers the default host
- discovers the default output device
- reads the default output config
- tries to build a minimal silent output stream
- runs for a short fixed window
- counts callback invocations
- records maximum callback gap in microseconds

This is intentionally minimal. It is a confidence spike, not the final audio engine.

---

## What This Spike Validates

- `cpal` is a viable low-level entry point for Riotbox audio I/O
- callback-oriented stream execution fits the current audio-core direction
- basic health metrics can be captured close to the stream layer

---

## Local Observation

Two different execution contexts mattered here.

### Sandbox result

Inside the restricted execution environment, the earlier probe path could not reach the live user audio session cleanly and produced a failed default-device/config result.

Interpretation:

- this was an environment constraint
- it was not sufficient evidence against `cpal`

### Real session result

Observed on the same machine, outside the sandbox, after running:

```bash
cargo run -p riotbox-audio --bin cpal_spike
```

Result:

```text
host: Alsa
default_output_device: default
default_output_config: F32, channels=2, sample_rate=44100, buffer_size=Range { min: 1, max: 4194304 }
supported_output_configs: 160
callback_count: 18
max_callback_gap_micros: 21329
stream_result: Ok
```

Interpretation:

- `cpal` can open and run a stream on this machine
- the Linux host exposed by `cpal` here is `Alsa`
- the ALSA `default` path on this system is compatible with the live PipeWire-based desktop audio setup

This means the correct reading is:

- `cpal` remains a viable low-level direction
- the earlier failure was caused by the restricted execution environment
- meaningful latency work should still be validated on target machines, but the basic stream-open path is confirmed here

General rule from this spike:

- every future audio observation should be tagged as either `sandbox` or `real session`
- sandbox-only failures must not be treated as device or backend conclusions without real-session confirmation

---

## What This Spike Does Not Yet Validate

- target stage latency
- cross-platform parity
- input streams
- device hot-swap behavior
- capture-path design
- scheduler integration
- xrun monitoring quality across platforms

---

## Initial Recommendation

Use `cpal` as the low-level audio I/O layer for the next runtime slice.

Reasons:

- it aligns with the stack freeze already recorded in `docs/specs/technology_stack_spec.md`
- it exposes the device/config/stream concepts Riotbox needs
- it supports the callback-centered execution model required by the future audio core

Additional recommendation:

- wrap `cpal` inside a dedicated `riotbox-audio` crate boundary
- keep device probing and health metrics explicit
- keep transport, scheduler, and scene logic outside the stream layer

---

## Metrics To Preserve From Day One

- host ID
- device name
- sample format
- sample rate
- channel count
- buffer size or supported range
- callback count
- worst callback gap
- stream build / play errors

These should later feed the System screen and benchmark layer.

---

## Follow-Up Work

1. extend the probe to report buffer-size details more explicitly
2. add a transport-facing runtime shell above `cpal`
3. define health telemetry contract for the future audio crate
4. test on the actual target machine(s), not just the current dev environment

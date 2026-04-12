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

Observed in the current environment after running:

```bash
cargo run -p riotbox-audio --bin cpal_spike
```

Result:

```text
host: Alsa
default_output_device: default
default_output_config: <none>
supported_output_configs: <unknown>
callback_count: 0
max_callback_gap_micros: <none>
stream_result: Failed { reason: "default_output_config failed: The requested device is no longer available. For example, it has been unplugged." }
```

Interpretation:

- the host layer is reachable
- the current environment exposes an ALSA default output path
- the default output device is not actually usable here for a real stream

This is not a rejection of `cpal`.

It means this development environment is not sufficient for meaningful latency measurement, so real callback timing and buffer behavior must be validated on an actual target machine with a working audio device.

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

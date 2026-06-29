# Realtime Audio Callback Hot Path Audit

Date: 2026-06-29
Ticket: RIOTBOX-1326
Scope: `crates/riotbox-audio/src/runtime/*`

## Contract

The normal realtime callback is the closure passed to `cpal::Device::build_output_stream`
in `shared_transport_tr909.rs`.

It must not perform:

- file I/O
- JSON parsing or serialization
- product log string formatting
- sidecar or analysis calls
- UI work
- blocking locks
- unbounded allocation or repeated buffer growth

This audit covers the normal render callback. The CPAL stream error callback is
separate; it may format and store stream error text because it is not the
per-buffer render path.

## Current Hot Path

Per buffer, the callback now performs only:

1. transport timing snapshot and beat advancement
2. atomic render-state snapshots for TR-909, MC-202, W-30 preview, W-30
   resample tap, and source monitor
3. rendering into a preallocated `f32` scratch slice
4. source-monitor mixing from an already decoded in-memory source cache
5. sample conversion into the backend output buffer
6. atomic health telemetry update

No file I/O, JSON, sidecar calls, analysis calls, UI calls, blocking locks, or
product log string formatting were found in the normal callback.

## Hardened In This Slice

`mix_buffer.resize(data.len(), 0.0)` was removed from the callback. Scratch
storage is allocated before stream construction:

- fixed backend buffer: exact `frames * channels`
- default / variable backend buffer: bounded reserve of 4096 frames

If a backend delivers a larger buffer than the reserve, the callback fills that
buffer with silence and increments `callback_scratch_overflow_count` in
`AudioRuntimeHealth`. This keeps the audio thread away from allocation while
making the degraded state visible through `JamRuntimeView`, observer JSON, and
runtime warnings.

`SharedSourceMonitorRenderState::snapshot()` now borrows the cached source
instead of cloning the `Arc` on every callback. The source PCM remains decoded
and cached outside the callback.

## Suspicious But Not Changed Here

Render-state snapshots still read related fields from independent relaxed
atomics. That is callback-safe from a blocking/allocation perspective, but it is
not yet the coherent snapshot handoff required by `audio_core_spec.md` section
11.4. This should stay as a separate bounded slice because changing it touches
all lane state handoff semantics and replay-facing behavior.

Follow-up: RIOTBOX-1338.

## Proof

Regression coverage added:

- callback scratch sizing for fixed and default backend buffer strategies
- callback scratch overflow telemetry without stream-error formatting
- app runtime-view warning and observer visibility for scratch overflow
- source-monitor shared-state snapshot preserves the cached source without
  replacing it during anchor updates

Validation run:

- `cargo test -p riotbox-audio`
- `cargo test -p riotbox-app`

This slice does not change musical behavior. It changes callback safety and
runtime observability only.

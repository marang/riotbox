# P023 Runtime Audio Review - 2026-06-29

Scope: `crates/riotbox-audio/src/runtime*`, runtime tests, and the audio core /
audio QA specs after RIOTBOX-1325 through RIOTBOX-1329.

## Findings

### Minor: older runtime modules still hide dependencies behind glob imports

Locations:

- `crates/riotbox-audio/src/runtime/render_tr909_w30_preview.rs:1`
- `crates/riotbox-audio/src/runtime/shared_transport_tr909.rs:1`
- `crates/riotbox-audio/src/runtime/shared_mc202_w30_preview.rs:1`
- `crates/riotbox-audio/src/runtime/shared_w30_resample_callback.rs:1`
- `crates/riotbox-audio/src/runtime/source_monitor.rs:1`
- `crates/riotbox-audio/src/runtime/tr909_tail_telemetry.rs:1`
- `crates/riotbox-audio/src/runtime/w30_tr909_signal_helpers.rs:1`

These modules predate the newer explicit-import rule. The current behavior is
covered by tests, but broad imports make callback-state dependencies and
ownership boundaries harder to review as the runtime mix seam grows.

Suggested follow-up: replace broad production-module imports with explicit
imports in small semantic slices, without changing behavior.

### Minor: two runtime modules sit just over the 500-line soft review budget

Locations:

- `crates/riotbox-audio/src/runtime/shared_mc202_w30_preview.rs:526`
- `crates/riotbox-audio/src/runtime/shared_transport_tr909.rs:505`

The files remain cohesive today, and no immediate split is required. If future
P023 work adds behavior there, prefer semantic extraction over growing either
file further.

Suggested follow-up: track runtime import/file-budget hygiene separately from
musical feature branches.

## No Blockers Found

- Recent callback hot-path hardening keeps allocation and blocking I/O out of
  the audio callback.
- The runtime mix parity seam is CI-safe and does not depend on CPAL/device
  output.
- Clip/headroom reporting is explicit and does not add hidden limiter behavior.
- Source Monitor unavailable behavior is visible in typed route state; product
  paths should continue surfacing unavailable/degraded state to the musician
  rather than treating silence as success.

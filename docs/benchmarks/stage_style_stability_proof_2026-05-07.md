# Stage-Style Stability Proof

Date: 2026-05-07

## Boundary

The current normalized stability proof covers the generated stage-style
restore-diversity path:

- command: `just stage-style-stability-proof`
- validator: `scripts/validate_stage_style_stability_proof.py`
- existing smoke: `just stage-style-stability-smoke`
- stronger bounded gate: `just stage-style-stability-gate`

This is a repeated-run CI-safe proof. It is not a host-audio soak, a multi-hour
live-performance endurance test, or a device-driver stability benchmark.

## What The Proof Checks

Each run must produce:

- a valid user-session observer event stream
- required stage-style key outcomes for capture, W-30 trigger, TR-909 fill, and
  MC-202 follower generation
- an observer/audio summary with control-path and output-path evidence present
- Phrase, Bar, and Beat commit-boundary coverage
- at least 12 committed actions in the observer/audio summary
- a passing `feral-grid-demo` listening manifest
- non-collapsed full-mix RMS and low-band RMS
- the same generated-support full-mix WAV SHA-256 across repeated runs

The normalized proof records run count, observer event counts, summary and
manifest hashes, stable mix hash, commit counts, boundary coverage, and bounded
scope. It keeps temp paths out of the durable proof data.

## Product Scope

This proof strengthens P011 confidence that the current stage-style spine can be
repeated without obvious control-path, output-path, or deterministic-render
collapse. It does not prove real-time host audio, device I/O, or long unattended
performance behavior.

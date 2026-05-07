# Source Timing WAV Probe

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial deterministic WAV input-feature probe

## Purpose

`crates/riotbox-audio/src/source_timing_probe.rs` reads the existing
`SourceAudioCache` PCM WAV data and produces deterministic timing-input
features:

- mono window energy
- positive energy flux
- thresholded onset flags
- peak energy / peak flux
- onset count and onset density

This is the first real-WAV analysis seam for Source Timing Intelligence. It is
not a BPM estimator, downbeat detector, or lane timing authority yet.

## Gate

Run:

```bash
just source-timing-wav-probe
```

The gate uses synthetic PCM WAV fixtures to prove that impulse-like source
material produces onset evidence while silence stays quiet. It also includes a
long accented 120 BPM WAV fixture that must bridge through the candidate timing
model into stable preliminary phrase-grid confidence evidence.

The gate also checks the first conservative `TimingModel` diagnostic bridge:

- silence maps to `Unknown` timing quality and `Disabled` degraded policy
- sparse onset evidence stays low-confidence and disabled
- richer onset evidence can only request `ManualConfirm`
- no probe-only diagnostic may claim a BPM estimate or locked beat grid

## Boundary

This probe is input evidence for later timing hypotheses. Follow-up slices must
map it conservatively into `TimingModel` diagnostics and then into BPM/downbeat
candidates without falsely reporting a locked grid when evidence is weak. Phrase
readiness from this gate is still preliminary QA evidence, not production
structural segmentation.

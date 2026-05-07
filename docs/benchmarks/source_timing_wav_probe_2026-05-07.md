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
material produces onset evidence while silence stays quiet.

## Boundary

This probe is input evidence for later timing hypotheses. Follow-up slices must
map it conservatively into `TimingModel` diagnostics and then into BPM/downbeat
candidates without falsely reporting a locked grid when evidence is weak.

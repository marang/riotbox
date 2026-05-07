# Source Timing BPM Candidates

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial probe BPM-candidate and beat-period scoring spike

## Purpose

`crates/riotbox-core/src/source_graph/timing_probe_candidates.rs` turns
deterministic probe onset times into preliminary BPM candidate hypotheses.

The candidate path reports:

- primary BPM estimate from scored beat-period candidates
- primary beat/bar candidate grids
- half-time and double-time alternatives when they fit the policy range
- ambiguous downbeat and phrase-uncertain warnings
- fixture-like PCM WAV probe evidence that reaches the same candidate path

## Gate

Run:

```bash
just source-timing-bpm-candidates
```

The gate uses synthetic onset spacing plus generated PCM WAV probe fixture
paths to prove that a 120 BPM impulse train and a fixture-like pulse source can
produce a BPM candidate while sparse onsets degrade instead of claiming timing
lock.

## Boundary

This is not a production BPM/downbeat detector. It is an intentionally bounded
candidate-shape proof so later real-source detection can reuse the same
`TimingModel` contract and ambiguity behavior.

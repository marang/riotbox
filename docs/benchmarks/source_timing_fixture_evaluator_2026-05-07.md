# Source Timing Fixture Evaluator

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: active fixture-output tolerance gate

## Purpose

`crates/riotbox-core/src/source_graph/timing_evaluation.rs` compares a
`TimingModel` output against deterministic source timing fixture expectations.

The evaluator reports:

- BPM error
- beat, bar, and phrase count agreement
- primary hypothesis confidence
- primary mean-absolute drift and max-drift measurements
- quality and degraded-policy agreement
- required timing warnings
- required alternative hypotheses

The evaluator gates:

- BPM error against the fixture `bpm_tolerance`
- beat, bar, and phrase minimum counts
- primary confidence against the fixture `confidence_floor`
- primary drift-window `mean_abs_drift_ms` against
  `beat_hit_tolerance_ms`
- primary drift-window `max_drift_ms` against `downbeat_tolerance_ms`
- quality and degraded-policy agreement
- required warning and alternative-hypothesis presence

## Gate

Run:

```bash
just source-timing-fixture-evaluator
```

The gate includes positive fixture-output checks and negative regressions that
reject out-of-tolerance BPM, too few beat events, weak primary confidence,
missing primary drift evidence, and drift measurements that exceed the fixture
beat/downbeat tolerances.

## Boundary

This is a control/model QA seam. It does not decode audio and does not prove
audible lane alignment yet. Future real WAV analyzer and lane-proof slices
should reuse this evaluator before adding output-path metrics.

The exposed measurements are diagnostic contract fields, not detector behavior:
they let benchmark/reporting code explain why a fixture passed or failed without
recomputing primary hypothesis confidence or drift summaries.

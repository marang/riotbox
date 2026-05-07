# Source Timing Fixture Evaluator

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial fixture-output metric gate

## Purpose

`crates/riotbox-core/src/source_graph/timing_evaluation.rs` compares a
`TimingModel` output against deterministic source timing fixture expectations.

The evaluator reports:

- BPM error
- beat, bar, and phrase count agreement
- quality and degraded-policy agreement
- required timing warnings
- required alternative hypotheses

## Gate

Run:

```bash
just source-timing-fixture-evaluator
```

The gate includes positive fixture-output checks and a negative regression that
rejects out-of-tolerance BPM plus too few beat events.

## Boundary

This is a control/model QA seam. It does not decode audio and does not prove
audible lane alignment yet. Future real WAV analyzer and lane-proof slices
should reuse this evaluator before adding output-path metrics.

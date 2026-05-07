# Source Timing Analyzer Skeleton

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial Rust analyzer seam

## Purpose

`crates/riotbox-core/src/source_graph/timing_analysis.rs` provides the first
deterministic Rust seam for Source Timing Intelligence.

It converts fixture timing seeds into the richer `TimingModel` payload shape:

- compatibility BPM / confidence / meter fields
- selected primary hypothesis
- preserved alternative hypotheses
- beat, bar, and phrase grids
- timing anchors
- drift placeholders derived from fixture tolerances
- warnings and degraded policy

## Gate

Run:

```bash
just source-timing-analyzer-skeleton-fixtures
```

The gate proves that the source timing fixture catalog maps into the current
core timing contract and that weak or ambiguous timing stays visibly degraded.

## Boundary

This is not production BPM/downbeat detection. The skeleton is intentionally
fixture-first so the next analyzer implementation can prove its output shape
before lanes or the TUI consume timing-aware behavior.

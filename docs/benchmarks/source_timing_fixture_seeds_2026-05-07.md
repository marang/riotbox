# Source Timing Fixture Seeds

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial fixture contract

## Purpose

`crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json`
is the first machine-checkable seed catalog for Source Timing Intelligence.

The catalog defines expected timing outcomes before a real analyzer is promoted:

- clean high-confidence grid
- dense break with cautious confidence
- hook-forward phrase grid
- half-time ambiguity with preserved alternative hypothesis
- weak timing that must not report high confidence

These are expected timing contracts, not committed audio files.

## Gate

Run:

```bash
just source-timing-fixture-catalog
just source-timing-fixture-catalog-validator-fixtures
```

The gate validates schema, categories, expected metrics, confidence boundaries,
warnings, ambiguity alternatives, and degraded-policy consistency.

## Boundary

This is not a BPM/downbeat analyzer and not an audio-output proof. Later P012
slices should use this catalog as the first deterministic truth table when they
add analyzer output, source-grid drift checks, and lane timing proofs.

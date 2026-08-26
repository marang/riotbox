# RIOTBOX-1477 Product-Stem Handoff Review — 2026-08-26

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1477
- Boundary: existing deterministic Feral-grid generated-support mix
- Explicit non-goals: musician `export.stem_package`, Session/replay receipts,
  TUI/Ghost integration, DAW packaging, live recording, new audible tuning,
  source-general quality, Holdout access, and release readiness

## Problem Confirmed

The existing Feral-grid pack exposed raw TR-909, W-30, and MC-202 lane renders,
but those lanes enter a shared driven and limited product bus. Directly summing
the raw WAVs therefore cannot reconstruct the approved `full_grid_mix` and
would make a misleading product-stem promise. The hardcoded local-CI stem
fixture proves receipt/writer mechanics only and cannot be used as musician
material.

## Implemented

- The approved full-mix renderer and written bytes remain unchanged.
- A semantic Rust module allocates the post-bus mix across typed drum, music,
  and bass contributions using symmetric three-player Shapley attribution.
- The W-30 contribution owns the residual so the in-memory f32 sum is exact;
  written PCM16 stems are decoded and checked against the written full mix.
- The frozen `pcm_sum_v1` gate permits maximum absolute error `3 / 32768` and
  RMS error `1.5 / 32768`.
- `just product-stem-handoff <source> <destination>` renders the same explicit
  source twice, builds a versioned proof, stages a contained bundle, validates
  the published bytes, and promotes it atomically into a new destination.
- Proof validation binds source, normalized manifest, format/grid, artifact
  paths and hashes, reconstruction rule and tolerances, development status,
  and the MC-202 primitive-renderer limitation.
- Existing destinations, partial bundles, path/hash drift, relaxed tolerances,
  non-reconstruction, and readiness promotion fail closed.

## Evidence To Date

- Rust Feral-grid suite: 44/44 pass
- Python product-stem contract mutations: 8/8 pass
- Synthetic double-render handoff: pass
- Pre-change and post-change full-mix SHA-256:
  `612bbd6ad874c5308f639753ac28f42c61b7ca386759be64a0bc9c9b41e3a828`
- Measured maximum reconstruction error: `0.00003054738`
- Measured RMS reconstruction error: `0.000010656502`
- Existing destination: rejected before rerender
- Existing product-mix reproducibility smoke: pass with unchanged full-mix hash
- `cargo clippy --all-targets --all-features -- -D warnings`: pass
- `just ci`: pass, source-free
- `cargo fmt --all -- --check`: pass
- `git diff --check`: pass
- No registered Development source, Holdout, commercial reference, or human
  playback was used for this implementation proof

## Review Boundary

This slice proves a real source-matched, reconstructable developer handoff. It
does not yet prove musician stem-package interaction, receipt/replay ownership,
DAW import, live output, or musical quality. The MC-202 contribution remains a
primitive renderer and is named as such in the proof. Final source-free CI and
branch review found and fixed manifest-stage binding, late destination
no-clobber, proof-metric honesty, and one Rust lint issue. No correctness,
product-spine, realtime-boundary, replay, observer, documentation, or module
ownership blocker remains.

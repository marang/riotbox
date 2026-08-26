# RIOTBOX-1478 MC-202 Source-Expression Origin Review — 2026-08-27

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1478
- Boundary: existing deterministic Feral-grid render and Development-only
  product-stem handoff
- Explicit non-goals: DSP or phrase-planner changes, musician
  `export.stem_package`, Session/replay receipts, TUI/Ghost integration, DAW
  packaging, live recording, source-general quality, Holdout access, release
  readiness, and human listening

## Problem Confirmed

RIOTBOX-1341 already built and applied a non-empty
`Mc202SourcePhraseRenderPlan` from the selected source window, but RBX-092 kept
the Feral manifest's legacy `pattern_origin: primitive_renderer` until manifest
and export consumers could migrate together. RIOTBOX-1477 consequently had to
publish its reconstructable bass contribution as non-product primitive
material even though the rendered phrase plan and source contour were present.

## Implemented

- The MC-202 proof now uses a typed internal origin. It becomes
  `source_derived` only when bass pressure passes, a non-empty source-expression
  render plan was applied, the source contour measurably changed the signal,
  and no source-failure fallback exists; otherwise it is `unavailable`.
- The pack-level validation additionally requires the frozen MC-202 source-grid
  hit ratio before any manifest is written.
- Successful Feral manifests contain no `primitive_renderer` record and no
  stale `primitive_renderer_boundary`.
- `riotbox.product_stem_handoff.v2` records all three contribution stems as
  source-derived and embeds `riotbox.mc202_source_expression_origin.v1`
  evidence for plan, role, contour, grid, bass pressure, and no fallback.
- V2 remains `development_only`, `release_ready: false`, and
  `musician_export_action_ready: false`. V1 is not reinterpreted.
- Artifact containment, no-clobber atomic publication, hashes, double-render
  stability, grid identity, and frozen PCM reconstruction tolerances are
  unchanged.

## Evidence

- Rust Feral-grid suite: 44/44 pass
- Rust observer/audio correlation suite: 60/60 pass; source-derived MC-202
  evidence fails closed when its render plan is not applied
- Python product-stem contract mutations: 13/13 pass
- Source-free synthetic double-render and published-v2 validation: pass
- Synthetic source SHA-256:
  `64893eb3dad84f8bd7f741e60e62f3c804d3d48230647d7d1848775b91871f87`
- Pre-migration and post-migration full-mix SHA-256:
  `612bbd6ad874c5308f639753ac28f42c61b7ca386759be64a0bc9c9b41e3a828`
- Measured maximum reconstruction error: `0.00003054738`
- Measured RMS reconstruction error: `0.000010656502`
- Synthetic MC-202 evidence: plan applied, role `bass_pressure`, source contour
  delta RMS `0.021030188` against minimum `0.00025`, source-grid hit ratio
  `1.0` against minimum `0.5`, and no source-failure fallback
- No registered Development source, Holdout, commercial reference, or human
  playback was used

## Review Boundary

This migration proves that the existing Feral bass contribution is honestly
source-derived inside the Development handoff; it does not make the handoff
committed Session truth. A later musician stem-package action must still match
the proof source hash to active Source Graph lineage, write the reserved
receipt only after success, expose observer lifecycle from that truth, and
never substitute the hardcoded local-CI fixture as musician material.

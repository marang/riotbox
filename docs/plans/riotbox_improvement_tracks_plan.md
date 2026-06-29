# Riotbox Improvement Tracks Plan

Status: Accepted planning direction
Source: RIOTBOX-1320 incorporation of a temporary improvement README, removed
after capture

---

## Purpose

This plan turns the temporary improvement README into durable Riotbox work
tracks. It does not replace the execution roadmap. The product priority remains
clear: Riotbox must become a better audible instrument, while code quality,
runtime safety, and QA must stay strong enough to support that goal.

## Priority Principle

Audible instrument quality is the primary product goal. Engineering quality is
not optional; it protects the product from becoming unreviewable, unsafe in the
audio path, or misleading in its quality claims.

Use this ordering rule:

1. Do not postpone critical code-quality work that blocks safe musical progress.
2. Do not let broad refactoring displace source-backed musical improvement for
   weeks.
3. Keep structural work small, behavior-preserving, and PR-scoped.
4. Split musical improvement by lane or product seam, not as one mixed
   all-instrument PR.

## Track A: Semantic Rust Modules

Problem: Several Rust areas use textual `include!` splits that reduce visible
file size without creating real module ownership.

Direction:

- adopt `docs/engineering/module_policy.md`
- inventory all Rust `include!` sites
- add a manual guardrail for unexpected new textual includes
- migrate include shells to real modules in small behavior-preserving PRs

Initial candidates:

- `crates/riotbox-core/src/source_graph.rs`
- `crates/riotbox-core/src/session.rs`
- `crates/riotbox-audio/src/mc202.rs`
- `crates/riotbox-audio/src/source_audio.rs`
- `crates/riotbox-app/src/bin/riotbox-app.rs`

Non-goal: do not combine include migration with feature changes.

## Track B: Runtime Audio Quality

Problem: A musician-facing instrument needs realtime-safe audio boundaries,
coherent render state, consistent gain staging, and comparable offline/realtime
renders.

Direction:

- audit normal callback paths for allocations, locks, file I/O, JSON/string
  work, and analysis calls
- document the callback hot path
- design coherent render-state snapshots with revision or double-buffer
  semantics
- add master-bus gain staging, soft clipping or limiting, DC checks, and clip
  metrics where missing
- converge offline and realtime rendering around a shared render seam where
  practical

Acceptance shape:

- no normal product path writes, decodes, allocates heavily, or calls analysis
  from the realtime callback
- render-state changes are applied coherently
- offline and realtime-simulation renders can be compared with tolerances

## Track C: Source-Backed Musical Instrument Work

Problem: Riotbox must keep moving from diagnostic proof toward a playable,
source-backed instrument with strong hooks, pressure, destructive gestures, and
stage-meaningful controls.

Direction:

- split TR-909, MC-202, W-30, Source Timing, and Scene Brain work into separate
  tickets
- require source evidence, musical decision, product-spine representation,
  audible consequence, and quality proof boundaries for source-derived claims
- keep scripted/hardcoded renders labeled as scaffolds until accepted by
  stronger source-backed evidence and listening review

Lane slices:

- TR-909 becomes a differentiated drum lane with kick/snare/clap/hat identity,
  swing, microtiming, accents, and source-derived adoption
- MC-202 gains explicit accent, slide, tie, gate, filter, source-derived motif,
  and role semantics
- W-30 grows from preview seams into sampler workflow: capture slots, pad bank,
  pitch/envelope/trim/reverse/loop, promote/persist/recall, and resample
  lineage
- Source Timing improves multi-hypothesis timing, confidence calibration,
  reason codes, and user confirmation
- Scene Brain turns scene movement into audible lane targets and restore
  contrast

Non-negotiable: no musical fallback output on product paths. If trusted
source-backed material is unavailable, Riotbox must expose unavailable/degraded
state or silence instead of filling the gap with replacement music.

## Track D: Sidecar And Analysis Provenance

Problem: Analysis sidecars are useful, but stub/skeleton output must never look
like trusted source analysis.

Direction:

- version sidecar request/response contracts
- add golden fixtures for protocol I/O
- test timeout, error, and version mismatch paths
- keep product decisions in Core; sidecar provides measurements
- mark provider set, run notes, stub status, and source-derived status
  honestly

Acceptance shape:

- UI and reports distinguish stub, fallback, degraded, and source-derived data
- release/demo paths cannot silently consume stub output as quality proof

## Track E: QA, CI, And First-Playable UX

Problem: Riotbox needs quality signals that catch sound regressions before a
human finds them by chance.

Direction:

- keep the current local and GitHub CI baseline
- split broad validation into fast, audio, and full layers when practical
- extend audio metrics around peak, RMS, DC offset, clip count, source survival,
  diversity, and fallback collapse
- grow human listening packs with specific review questions
- harden first-run/first-playable probes around source load, timing confidence,
  queued action, boundary commit, audible change, and readable log explanation

Acceptance shape:

- audio-producing tickets prove control path and output path
- stronger gates remain honest about `quality_proof: false` and
  `human_verdict: unverified` when applicable
- UI explains degraded states in musician language

## Backlog Order

The first backlog wave should be:

1. Include inventory and module policy guardrail.
2. Source Graph include-shell migration.
3. Session include-shell migration.
4. Audio include-shell migration for MC-202 and source audio.
5. App binary thinning into library CLI modules.
6. Audio callback hot-path audit.
7. Render-state snapshot strategy.
8. Gain staging and offline metrics expansion.
9. Offline/realtime render parity seam.
10. Source Timing explainability and user-confirm flow.
11. TR-909 drum-lane musical differentiation.
12. MC-202 acid/bass role semantics.
13. W-30 sampler-flow expansion.
14. Sidecar protocol/provenance hardening.
15. First-playable UX/readability hardening.

This order can be adjusted per phase pressure, but the backlog must stay split:
do not collapse instrument-lane work into one broad "make audio better" ticket.

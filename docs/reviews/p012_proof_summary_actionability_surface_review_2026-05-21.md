# P012 Proof Summary Actionability Surface Review - 2026-05-21

Scope: cadence review after RIOTBOX-880 through RIOTBOX-884, focused on Source
Timing cue/actionability/readiness/grid-use consistency across the compact P012
proof summary, probe and manifest validators, observer/audio correlation, and
Jam/Source-facing timing language.

## Summary

The current P012 Source Timing surface is holding its architecture line. Source
Graph timing remains the contract source, Rust probe and Feral-grid producers now
share typed readiness/grid-use helpers, Python validators recompute the same
contracts as compatibility gates, and the compact P012 proof summary now exposes
cue, actionability, readiness, manual-confirm, grid source, decision, grid-use,
and lane hit ratios in one table.

No blocking defect was found. The remaining risks are reviewability and future
drift, not broken behavior: one app fallback still repeats readiness
actionability wording locally, and the compact proof summary still reduces the
generated Feral-grid observer/audio matrix to a pass/fail component line.

## Findings

- **Location**: `crates/riotbox-app/src/bin/observer_audio_correlate/summary_render.rs:261`,
  `crates/riotbox-app/src/source_timing_cues.rs:23`
- **Category**: scope
- **Severity**: minor
- **Title**: Observer/audio readiness actionability fallback still has a local
  string table
- **Description**: The observer/audio JSON renderer uses the shared app helper
  for readiness cue fallback, but the corresponding actionability fallback still
  branches locally on `readiness` and `requires_manual_confirm`. Most generated
  manifests now provide explicit `actionability`, so this is not a current output
  bug, but it leaves one remaining Rust-side readiness phrase table outside the
  helper path.
- **Suggestion**: Add `source_timing_readiness_actionability_label(...)` beside
  `source_timing_readiness_cue_label(...)` and use it from observer/audio
  fallback rendering. Keep Python validators as independent compatibility checks.

- **Location**: `scripts/write_p012_all_lane_proof_summary.py:68`,
  `scripts/write_p012_all_lane_proof_summary.py:74`,
  `scripts/correlate_generated_feral_grid_observer.sh:121`,
  `scripts/correlate_generated_feral_grid_observer.sh:336`,
  `scripts/correlate_generated_feral_grid_observer.sh:441`
- **Category**: scope
- **Severity**: minor
- **Title**: Compact P012 proof summary does not expose generated observer/audio
  alignment path details
- **Description**: The all-lane summary now gives a useful Recipe 15 table, but
  the generated Feral-grid observer/audio gate is still summarized as one
  component line. The underlying script asserts cautious/manual-confirm,
  user-override, fallback, and locked-grid paths with `grid_bpm_source`,
  `grid_bpm_decision_reason`, grid-use compatibility, downbeat-offset
  compatibility, anchor/groove alignment, and empty output issues. Those details
  are review-critical but remain visible only by reading the script or temporary
  JSON files.
- **Suggestion**: Persist a compact generated Feral-grid observer/audio summary
  artifact and have `write_p012_all_lane_proof_summary.py` include a second
  table for cautious/manual-confirm, user-override, fallback, and locked-grid
  path outcomes.

## Dimensions

- Architecture / boundaries: no second timing model, action system, replay truth,
  or JamAppState state was found. The reviewed surfaces consume Source Graph,
  readiness reports, manifest evidence, or observer/audio summaries.
- Design consistency: cue/actionability/readiness/grid-use language is now
  consistent across probe JSON, generated Feral-grid manifests, validators, local
  example expectations, and the P012 proof summary.
- Maintainability: the current highest-value cleanup is the last Rust fallback
  actionability table in observer/audio rendering.
- Coupling: Python validators intentionally duplicate policy as external
  compatibility checks; Rust producers should stay on shared helper paths.
- Safety / performance: no realtime audio callback work, blocking I/O, model
  calls, or detector threshold changes were introduced by the reviewed slices.

## Recommendation

Next implementation should stay small and proof-facing:

1. centralize the app-side readiness actionability fallback helper, or
2. make the compact P012 proof summary include generated Feral-grid
   observer/audio path details.

The second option has the stronger musician/reviewer payoff because it makes the
existing all-lane proof easier to understand without weakening any timing policy.

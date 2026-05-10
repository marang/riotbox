# P011 Import Boundary Follow-Up Review 2026-05-10

Context:

- ticket: `RIOTBOX-708`
- scope: `crates/riotbox-app/src/jam_app.rs` and `crates/riotbox-app/src/jam_app/`
- trigger: scheduled periodic review after the P011 `jam_app` import-boundary cleanup batch
- review mode: current-state codebase review, not diff-only review

## Summary

The RIOTBOX-690 boundary audit is materially improved. The main `jam_app.rs`
module root is still a small wiring file, and the recent cleanup removed the
highest-risk production wildcards from queue, commit, side-effect, artifact, and
restore seams.

The remaining production `super::*` imports are now concentrated in two areas:

- `state.rs`, which defines the public app state/error surface.
- `recovery.rs` plus its child guidance modules, which still share a broad parent
  namespace while performing session recovery classification and replay guidance.

No broad refactor is justified. The next work should be two small explicit-import
slices. The W-30 queue file is near the soft review budget, but still reads as one
cohesive queueing responsibility and should not be split mechanically.

## Findings

### 1. App state boundary still imports the full module root

- Location: `crates/riotbox-app/src/jam_app/state.rs:1`
- Category: scope
- Severity: minor
- Title: public Jam app state types still depend on a broad parent namespace
- Description: `state.rs` defines the exported app error, runtime state, file set,
  and core `JamAppState` struct. Because it starts with `use super::*`, the public
  state boundary implicitly depends on every root import in `jam_app.rs`, including
  filesystem, formatting, audio runtime, sidecar, core persistence, queue, session,
  Source Graph, transport, and Jam view types.
- Suggestion: Replace the wildcard with explicit `std`, `riotbox_audio`,
  `riotbox_core`, and `riotbox_sidecar` imports in one bounded slice. This should
  be a no-behavior-change import cleanup with a focused app test run.

### 2. Recovery guidance modules still hide replay and persistence dependencies

- Location: `crates/riotbox-app/src/jam_app/recovery.rs:1`
- Category: scope
- Severity: minor
- Title: recovery classification still relies on broad parent imports
- Description: `recovery.rs` owns session recovery presentation and calls into
  persistence, artifact hydration, replay readiness, and child guidance modules.
  The child modules also import `super::*` (`recovery/payload_guidance.rs:3`,
  `recovery/hydration_guidance.rs:1`), which makes it harder to see which replay
  and persistence contracts each recovery path needs.
- Suggestion: Replace the recovery wildcards with explicit imports in a bounded
  slice. Keep the existing module shape, but make child guidance dependencies
  visible at the file top so replay/recovery contracts remain reviewable.

## Non-Findings

- `crates/riotbox-app/src/jam_app/w30_queue.rs` is 498 lines, which is close to
  the soft review/context budget. It is still cohesive as W-30 queue gesture
  construction and should not be split only to satisfy line count.
- Test shards in `crates/riotbox-app/src/jam_app/tests/` remain under or near the
  soft budget. They are large, but the current split is behavior-oriented enough
  to avoid immediate churn.
- The remaining `use super::*` in `side_effects/ghost.rs` is test-local, not a
  production boundary issue.

## Recommended Follow-Ups

1. Make `jam_app/state.rs` imports explicit.
2. Make `jam_app/recovery.rs` and `jam_app/recovery/*_guidance.rs` imports
   explicit.

These should close the current P011 product-import cleanup loop. After those
land, the next review should return to product behavior: replay recovery,
source-timing confidence, and musician-facing output verification.

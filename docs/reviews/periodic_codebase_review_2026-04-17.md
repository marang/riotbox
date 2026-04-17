# Periodic Codebase Review 2026-04-17

Scope:

- repo root review, organized by current high-risk layers
- primary focus on `crates/riotbox-app`, `crates/riotbox-audio`, and `crates/riotbox-core`
- emphasis on the current W-30 MVP seam after the recent preview, trigger, and resample slice batch

## Findings

### 1. W-30 preview mode is reconstructed from action history instead of explicit lane/runtime state

- Location: [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:2315), [crates/riotbox-core/src/session.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/session.rs:153)
- Category: `scope`
- Severity: `major`

`build_w30_preview_render_state(...)` derives preview mode from `last_committed_w30_preview_action(session)` instead of from explicit persisted W-30 lane or runtime state. `W30LaneState` currently only stores `active_bank`, `focused_pad`, and `last_capture`, so the effective preview mode depends on action-log ordering rather than on a single committed state source. That is replay-hostile and becomes more fragile as more W-30 controls land, because later committed actions can change lane focus without changing the historical action that currently drives preview mode.

Suggestion:

- add explicit W-30 preview intent/state to the committed lane or runtime-facing state
- make render-state derivation depend on that explicit state rather than scanning the action log for the last relevant command

### 2. W-30 target selection is still capture-driven instead of focus-driven

- Location: [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:1343)
- Category: `scope`
- Severity: `major`

`recallable_w30_capture`, `auditionable_w30_capture`, and `triggerable_w30_capture` pick the latest pinned/promoted capture or `last_capture`, but they do not resolve from the committed lane focus (`active_bank` + `focused_pad`). That means the next planned bank/pad stepping slice can update visible focus while leaving the actual recall/audition/trigger capture resolution unchanged. In practice, that would make bank-step behavior partly cosmetic instead of truly controlling the committed preview seam.

Suggestion:

- introduce one focus-aware capture resolver for the W-30 lane
- have recall, audition, trigger, and later bank-step actions all resolve through that same committed focus path

### 3. The shell still bypasses the core Jam view model for pending W-30 resample cues

- Location: [crates/riotbox-core/src/view/jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:50), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1794)
- Category: `scope`
- Severity: `minor`

The core `JamViewModel` exposes pending W-30 trigger, audition, and recall targets, but not the pending resample cue. Because of that, `w30_pending_cue_label(...)` in the TUI directly scans `ActionQueue` to discover `PromoteResample`. This is a small but clear boundary leak: the shell is now carrying device-action knowledge that should live in the core presentation model.

Suggestion:

- extend `LaneSummaryView` / `JamViewModel` with a pending W-30 resample cue summary
- keep the shell renderer on `JamViewModel` data instead of querying queue internals directly

## Review Notes

- The current repo does not show broad architecture collapse; the findings are concentrated in the W-30 seam because that is where the last slice batch accumulated.
- The most immediate blocker is finding 2. The planned bank-step slice should not ship on top of the current capture-selection helpers without first making that selection respect committed lane focus.

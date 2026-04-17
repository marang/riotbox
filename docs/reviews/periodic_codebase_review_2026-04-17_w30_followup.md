# Periodic Codebase Review 2026-04-17 (W-30 Follow-up)

Scope:

- `crates/riotbox-app/src/jam_app.rs`
- `crates/riotbox-app/src/ui.rs`
- `crates/riotbox-core/src/view/jam.rs`
- current W-30 preview, trigger, resample, bank-manager, pad-forge, and loop-freezer seams

Review intent:

- catch cross-slice drift after the recent W-30 MVP batch
- verify that newer W-30 cues still stay on one honest queue, capture, preview, and shell seam
- identify follow-up work that should land before more W-30 breadth opens

## Findings

### 1. Conflicting W-30 phrase actions can still be queued together on the same lane

- Location:
  - `crates/riotbox-app/src/jam_app.rs:1364`
  - `crates/riotbox-app/src/jam_app.rs:1476`
  - `crates/riotbox-app/src/jam_app.rs:1774`
  - `crates/riotbox-app/src/jam_app.rs:1789`
- Category: `scope`
- Severity: `critical`
- Title: `w30.loop_freeze` and `promote.resample` use separate pending guards even though both mutate the same W-30 phrase seam
- Description:
  `queue_w30_loop_freeze(...)` blocks only `w30_pad_cue_pending()`, while `queue_w30_internal_resample(...)` blocks only `w30_resample_pending()`. Because `w30_pad_cue_pending()` excludes `PromoteResample` and `w30_resample_pending()` ignores the pad-cue family, both actions can be queued together for the same W-30 lane and the same `NextPhrase` boundary. Both actions then materialize capture-side effects on commit, so the resulting lineage, `last_capture`, and preview state become order-dependent instead of explicitly exclusive.
- Suggestion:
  introduce one shared W-30 phrase-cue conflict guard for capture-mutating actions, or explicitly include `PromoteResample` in the same exclusivity group as `W30LoopFreeze` when they target the W-30 lane.

### 2. The Capture screen still bypasses the app/core presentation contract for pending capture state

- Location:
  - `crates/riotbox-app/src/ui.rs:1715`
  - `crates/riotbox-app/src/ui.rs:1809`
- Category: `scope`
- Severity: `major`
- Title: `Capture` pending summaries still read `ActionQueue` directly instead of using projected app state
- Description:
  `capture_readiness_lines(...)` and `pending_capture_lines(...)` still read `shell.app.queue.pending_actions()` directly and re-derive capture filtering inside the UI. That repeats the same kind of boundary leak the repo already fixed for W-30 pending resample cues. The shell now owns a second copy of “which actions count as capture work,” so future capture-side commands can drift between the queue and what the capture surface shows.
- Suggestion:
  project pending capture count and pending capture items through `JamViewModel` or another app-layer summary struct, then have `Capture` render only that projected state.

### 3. W-30 operation diagnostics can stay stale after focus moves, and the lineage branch wastes one line on duplicate output

- Location:
  - `crates/riotbox-app/src/ui.rs:1411`
  - `crates/riotbox-app/src/ui.rs:1561`
  - `crates/riotbox-app/src/ui.rs:1910`
  - `crates/riotbox-app/src/ui.rs:1996`
- Category: `scope`
- Severity: `major`
- Title: W-30 operation summaries are derived from global last-committed actions instead of the current lane target
- Description:
  `w30_bank_manager_compact(...)`, `w30_damage_profile_compact(...)`, and `w30_loop_freeze_compact(...)` all look up the latest committed action of that command anywhere in history. After focus or bank moves, the Jam/Capture/Log surfaces can still show `swap`, `shred`, or `freeze` based on a previous pad rather than on the currently focused W-30 target. At the same time, the lineage-active `capture_routing_lines(...)` branch pushes `latest promoted ...` twice, which wastes a scarce shell line exactly where the W-30 surface is already crowded.
- Suggestion:
  scope committed W-30 operation summaries to the current bank/pad target or explicit lane state rather than to the global last committed action, and remove the duplicated `latest promoted` line in the lineage-active branch.

## Summary

The current W-30 MVP still has one coherent operator seam, but the next clean-up work should happen before more breadth lands:

1. unify W-30 phrase-cue conflict blocking for freezer vs resample
2. finish moving Capture pending summaries behind projected app/core state
3. tighten W-30 diagnostics so they track the current lane target instead of stale global history

The recent W-30 slices are otherwise holding their current architectural line: no second persistence model, no second preview runtime, and no shell-only shadow action path showed up in this pass.

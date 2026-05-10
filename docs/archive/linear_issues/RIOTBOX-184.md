# `RIOTBOX-184` Reduce Capture pending-cues panel to the most relevant queued action

- Ticket: `RIOTBOX-184`
- Title: `Reduce Capture pending-cues panel to the most relevant queued action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-184/reduce-capture-pending-cues-panel-to-the-most-relevant-queued-action`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-184-reduce-capture-pending-cues`
- Linear branch: `feature/riotbox-184-reduce-capture-pending-cues-panel-to-the-most-relevant`
- PR: `#174`
- Merge commit: `3378b88`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-185`

## Why This Ticket Existed

`Do Next` owned the primary Capture pending action, but the lower `Pending Capture Cues` panel still read like a full queue dump. That made multiple capture-path actions compete visually with the main next-step cue.

## What Shipped

- Reduced the `Pending Capture Cues` panel to the first relevant queued capture-path action.
- Added a compact `+n more in [2] Log` overflow cue for additional pending capture actions.
- Preserved full pending detail in `Log` and the action-history path.
- Added focused rendering coverage for the reduced panel and updated the TUI spec contract.

## Verification

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`

## Notes

- This changed Capture presentation only; queue semantics, layout, session state, sampler behavior, and audio output stayed unchanged.

# `RIOTBOX-181` Make Capture Do Next reflect pending capture and promotion state

- Ticket: `RIOTBOX-181`
- Title: `Make Capture Do Next reflect pending capture and promotion state`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-181/make-capture-do-next-reflect-pending-capture-and-promotion-state`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-181-capture-do-next-pending-state`
- Linear branch: `feature/riotbox-181-make-capture-do-next-reflect-pending-capture-and-promotion`
- PR: `#171`
- Merge commit: `802d88b`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-182`

## Why This Ticket Existed

`Do Next` made the Capture path easier to scan, but it still mostly reflected the latest committed capture. When capture or promotion was already queued, the screen could ask the user to perform a step that was already pending.

## What Shipped

- Made the Capture `Do Next` card prefer queued capture, promotion, and W-30 reshape intent over last-capture fallback guidance.
- Added compact pending-state wording for capture, promote-to-pad, promote-to-scene, loop-freeze, and resample paths.
- Updated Capture shell regressions so pending capture and pending promotion states stay visible in snapshots.
- Documented the pending-first Capture rule in the TUI screen spec.

## Verification

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`

## Notes

- This changed presentation only; queue semantics, commit timing, session state, and audio behavior stayed unchanged.

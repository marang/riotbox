# `RIOTBOX-183` Visually distinguish pending Capture Do Next cues

- Ticket: `RIOTBOX-183`
- Title: `Visually distinguish pending Capture Do Next cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-183/visually-distinguish-pending-capture-do-next-cues`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-183-capture-do-next-emphasis`
- Linear branch: `feature/riotbox-183-visually-distinguish-pending-capture-do-next-cues`
- PR: `#173`
- Merge commit: `56e71c8`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-184`

## Why This Ticket Existed

Capture `Do Next` could already explain queued intent, but pending and committed guidance still had the same terminal weight. That made the Capture surface harder to scan during first-use flow.

## What Shipped

- Added semantic pending emphasis to pending Capture `Do Next` intent lines.
- Kept pending detail lines visually distinct without changing text or layout.
- Added focused style-token coverage for the Capture pending hierarchy.
- Documented the pending emphasis rule in the TUI screen spec.

## Verification

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`

## Notes

- This changed visual emphasis only; queue semantics, session state, layout, and audio behavior stayed unchanged.

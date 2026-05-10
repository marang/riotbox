# `RIOTBOX-162` Add a bounded benchmark for restore wake-up vs ready contrast

- Ticket: `RIOTBOX-162`
- Title: `Add a bounded benchmark for restore wake-up vs ready contrast`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-162/add-a-bounded-benchmark-for-restore-wake-up-vs-ready-contrast`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-162-add-a-bounded-benchmark-for-restore-wake-up-vs-ready`
- PR: `#152`
- Merge commit: `991a18b`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-165`

## Why This Ticket Existed

The repo had a restore-ready benchmark, but the earlier wake-up-only state was still only covered indirectly through recipes and assertions.

## What Shipped

- Added the first explicit benchmark for the contrast between the restore wait-state and the restore-ready state.
- Linked that contrast baseline into the benchmark index so future copy changes could be checked against one stable reference.

## Notes

- This gave the restore state machine its first paired baseline instead of only a positive ready-state snapshot.

# `RIOTBOX-34` Ticket Archive

- Ticket: `RIOTBOX-34`
- Title: `Move transport clock and commit boundaries out of the TUI event loop`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-34/move-transport-clock-and-commit-boundaries-out-of-the-tui-event-loop`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-13`
- Finished: `2026-04-15`
- Branch: `riotbox-34-transport-runtime-seam`
- Assignee: `Markus`
- Labels: `TUI`, `Audio`, `Core`
- PR: `#31`
- Merge commit: `407d3be`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-39`, `RIOTBOX-40`

## Why This Ticket Existed

The periodic review found that transport advancement and action commit timing were still owned by terminal redraw cadence, which was the wrong long-term seam for a realtime-oriented product.

## What Shipped

- Added an app-runtime pulse source outside the TUI event loop.
- Moved elapsed-time transport advancement into `JamAppState`.
- Kept the TUI focused on rendering, input handling, and consuming runtime signals instead of owning timing.

## Notes

- This was an architecture-seam cleanup, not a user-facing feature slice.
- It preserved the existing queue model while making later scheduler or audio-runtime authority possible.

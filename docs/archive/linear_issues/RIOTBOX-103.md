# `RIOTBOX-103` Promote only the next 1-2 pending actions on the Jam shell

- Ticket: `RIOTBOX-103`
- Title: `Promote only the next 1-2 pending actions on the Jam shell`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-103/promote-only-the-next-1-2-pending-actions-on-the-jam-shell`
- Project: `Riotbox MVP Buildout`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-103-pending-priority`
- Linear branch: `feature/riotbox-103-promote-only-the-next-1-2-pending-actions-on-the-jam-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#96`
- Merge commit: `15584d987b4418869a96f2653bbe3c3dbe4b70a5`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#273`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-104`, `RIOTBOX-105`, `RIOTBOX-106`

## Why This Ticket Existed

The perform-first Jam shell was already showing pending work, but it still treated deeper queue depth too evenly on the main performance surface. Riotbox needed one bounded prioritization pass that made the next one or two musical changes obvious on `Jam` while preserving `Log` as the full queue-truth surface.

## What Shipped

- changed the `Pending / landed` Jam panel to show the first two pending actions directly
- collapsed any deeper queue depth into a compact `+N more` summary on the second line
- kept landed and status cues intact so the same panel still explains what just happened
- added a focused shell snapshot test that covers a three-action pending queue and guards the new summary format

## Notes

- this slice intentionally changed presentation only; queue semantics, action ordering, and `Log` output stayed untouched
- the new snapshot test had to avoid both the first-run onramp and the pre-seeded sample-shell queue so it could exercise a clean three-action user queue honestly
- the next honest surface cleanup is the gesture-rank split in help and footer wording, followed by lane-card compression and better post-commit next-step cues

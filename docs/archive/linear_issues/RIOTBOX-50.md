# `RIOTBOX-50` Ticket Archive

- Ticket: `RIOTBOX-50`
- Title: `Start W-30 MVP with the first bounded live recall cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-50/start-w-30-mvp-with-the-first-bounded-live-recall-cue`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-50-w30-live-recall`
- Linear branch: `feature/riotbox-50-start-w-30-mvp-with-the-first-bounded-live-recall-cue`
- Assignee: `Markus`
- Labels: `None`
- PR: `#46`
- Merge commit: `f33bed3`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`, `docs/README.md`, `docs/screenshots/capture_w30_live_recall_baseline.txt`
- Follow-ups: `RIOTBOX-54`, `RIOTBOX-56`

## Why This Ticket Existed

The roadmap had already entered `W-30 MVP`, but Riotbox still had no honest W-30 entry slice. Capture, promotion, and pinning existed, yet the backlog still lacked a real live-recall cue that stayed inside the current session, queue, and shell path.

## What Shipped

- Added the first bounded W-30 live-recall cue on the existing `w30.swap_bank` action seam.
- Made recall prefer the latest pinned promoted capture, then fall back to the latest promoted capture.
- Committed the cue on `NextBar` and updated `w30.active_bank`, `w30.focused_pad`, and `w30.last_capture` on commit.
- Extended `JamViewModel` with W-30 focused-pad and pending-recall summary state.
- Surfaced the recall cue in the Jam and Capture shell views and captured the artifact at `docs/screenshots/capture_w30_live_recall_baseline.txt`.

## Notes

- This slice intentionally stopped at a bounded recall cue and lane-focus update.
- Audible audition, recall variations, and deeper W-30 pad handling remain follow-up work on the same seam rather than separate control surfaces.

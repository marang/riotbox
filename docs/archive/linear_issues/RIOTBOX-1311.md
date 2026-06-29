# `RIOTBOX-1311` P023: Expose source/timing confidence risk as musician-visible cue

- Ticket: `RIOTBOX-1311`
- Title: `P023: Expose source/timing confidence risk as musician-visible cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1311/p023-expose-sourcetiming-confidence-risk-as-musician-visible-cue`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1311-p023-expose-sourcetiming-confidence-risk-as-musician-visible`
- Linear branch: `feature/riotbox-1311-p023-expose-sourcetiming-confidence-risk-as-musician-visible`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1285 (https://github.com/marang/riotbox/pull/1285)`
- Merge commit: `2f133cc06fd2b66731ab88bb9654d5b9d3b26d96`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt; cargo test -p riotbox-app --lib ui::tests -- --nocapture; cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 routing identified a ui_cue follow-up: the player needed a visible source/timing confidence cue before trusting grid-locked or live-trigger moves.

## What Shipped

- Added a compact perform risk cue on the Jam Trust panel and Source Confidence summary, derived from Source Graph confidence plus Jam source-timing degraded policy/grid use, with trusted/degraded/unavailable states and player actions.

## Notes

- The cue is UI-only and uses existing product-spine view evidence; it does not add app-local source truth or realtime audio work.

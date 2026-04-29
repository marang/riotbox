# `RIOTBOX-356` Split remaining app orchestration hotspots with semantic module extraction

- Ticket: `RIOTBOX-356`
- Title: `Split remaining app orchestration hotspots with semantic module extraction`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-356/split-remaining-app-orchestration-hotspots-with-semantic-module`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-356-split-app-orchestration`
- Linear branch: `feature/riotbox-356-split-remaining-app-orchestration-hotspots-with-semantic`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#346`
- Merge commit: `a782432f94b81777b0deec5d7bde7989a29033e2`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo test -p riotbox-app`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-360`

## Why This Ticket Existed

The remaining app orchestration files were too coupled and large for future audio/TUI work. Unlike the purely mechanical splits, these files needed semantic module extraction around responsibilities such as state, lifecycle, queueing, commit handling, side effects, and lane-specific projections.

## What Shipped

- split `jam_app.rs` into semantic modules for state, lifecycle, capture artifacts, transport, queue controls, commit/undo, controls, and helpers
- split `jam_app/side_effects.rs` into lane-specific side-effect modules
- kept all `.rs` files under the 500-line budget
- preserved existing app behavior and test coverage

## Notes

- this completed the first large Rust file-size cleanup pass
- RIOTBOX-360 later added the durable rule that split shards should use semantic names rather than numbered filenames

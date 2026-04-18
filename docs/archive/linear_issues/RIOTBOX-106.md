# `RIOTBOX-106` Add first-landed-result and next-step cue on the Jam shell

- Ticket: `RIOTBOX-106`
- Title: `Add first-landed-result and next-step cue on the Jam shell`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-106/add-first-landed-result-and-next-step-cue-on-the-jam-shell`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-106-next-step-cue`
- Linear branch: `feature/riotbox-106-add-first-landed-result-and-next-step-cue-on-the-jam-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#98`
- Merge commit: `39a0f52c4fc605c76d353196cd0f7229ef1f8759`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#279`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-105`

## Why This Ticket Existed

The first-run guidance had become good enough to land an initial move, but Riotbox still left too much interpretation work between that first landed result and the next useful gesture. The shell needed one bounded follow-up cue that explicitly said what changed and what to do next without introducing a full tutorial system.

## What Shipped

- rewrote the first-result onramp block into explicit `What changed` and `What next` language
- added a lightweight post-commit cue to the normal `Suggested gestures` panel when a landed action exists and no pending action is armed
- kept the suggested follow-up narrowed to a small choice set instead of reopening a long action list
- added snapshot coverage for both the first-result onramp and the normal post-commit Jam cue

## Notes

- this slice stayed intentionally lightweight and temporary; it did not introduce analytics, persistent onboarding state, or a separate tutorial architecture
- the guidance is biased toward `capture`, `undo`, `jump`, and `follow` because those actions make the first meaningful second step legible without reopening every lane verb at once
- the next honest follow-up was the lane-card compression pass that kept the same perform-first reading pressure intact

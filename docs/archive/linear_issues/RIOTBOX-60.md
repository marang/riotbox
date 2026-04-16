# `RIOTBOX-60` Ticket Archive

- Ticket: `RIOTBOX-60`
- Title: `Make W-30 preview audibly real from the new preview seam`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-60/make-w-30-preview-audibly-real-from-the-new-preview-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-60-w30-audible-preview`
- Linear branch: `feature/riotbox-60-make-w-30-preview-audibly-real-from-the-new-preview-seam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#54`
- Merge commit: `ff9ea9c`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-61`, `RIOTBOX-62`, `RIOTBOX-63`, `RIOTBOX-64`

## Why This Ticket Existed

`RIOTBOX-59` created the first typed audio-facing W-30 preview seam, but that seam was still callback-reachable and visually diagnosable while remaining silent in normal use. Riotbox needed the smallest honest slice that made W-30 preview audible without bypassing the committed session/capture/action seam or pretending full W-30 sample playback was already solved.

## What Shipped

- Made the typed `W30PreviewRenderState` audibly real inside the existing audio callback alongside the TR-909 render seam.
- Kept W-30 preview on the same committed session, capture, and action-log seam instead of opening a second playback architecture.
- Added bounded W-30 runtime audio tests for live recall, promoted audition, zero-music silence, and stopped-preview audibility.
- Opened the music bus to a modest default level for fresh ingest sessions so the new audible preview seam is reachable in normal app launches.
- Recorded the audible-preview boundary in the research decision log.

## Notes

- This slice deliberately uses a bounded lo-fi preview synth, not full W-30 sample playback.
- Later W-30 work should keep extending the same typed preview seam for richer audible diagnostics, real pad playback, and resample taps instead of bypassing it with callback-only or UI-only behavior.

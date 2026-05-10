# `RIOTBOX-279` Add source-backed W-30 hit regression

- Ticket: `RIOTBOX-279`
- Title: `Add source-backed W-30 hit regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-279/add-source-backed-w-30-hit-regression`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-279-add-source-backed-w-30-hit-regression`
- Linear branch: `feature/riotbox-279-add-source-backed-w-30-hit-regression`
- PR: `#269`
- Merge commit: `47306b6`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-280`, `RIOTBOX-281`

## Why This Ticket Existed

The W-30 `[w] hit` path needed explicit regression coverage that a promoted capture with a decoded source window keeps source-backed preview samples when triggered from the focused pad.

## What Shipped

- Added a source-backed W-30 trigger regression using a temporary PCM16 WAV and `SourceAudioCache`.
- Verified committed `W30TriggerPad` state preserves `LiveRecall`, promoted recall profile, capture identity, trigger revision, and non-empty source-window preview samples.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app committed_w30_trigger_preserves_source_window_preview_samples`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Focused app regression only; no DSP, playback behavior, fixture corpus, or user-facing UI behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

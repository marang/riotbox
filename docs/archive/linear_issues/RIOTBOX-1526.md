# `RIOTBOX-1526` Reject unrepresentable W-30 QA render and PCM16 capacities before hydration

- Ticket: `RIOTBOX-1526`
- Title: `Reject unrepresentable W-30 QA render and PCM16 capacities before hydration`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1526/reject-unrepresentable-w-30-qa-render-and-pcm16-capacities-before`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1526-w30-capacity-preflight`
- Linear branch: `feature/riotbox-1526-reject-unrepresentable-w-30-qa-render-and-pcm16-capacities`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `review-followup`
- PR: `#1596 (https://github.com/marang/riotbox/pull/1596)`
- Merge commit: `09cbb61b05aa3c76e8603f9d36e10244acf26c16`
- Deleted from Linear: `2026-10-01`
- Verification: `Debug and Release: 15 unit and 4 integration tests; 20 finite rejection paths, help control; 42 unchanged CLI outcomes and 24 unchanged artifact hashes including 9 synthetic WAVs; final source-free just ci green; PR 1596 Rust CI and Windows Sidecar transport both green, zero comments/reviews.`
- Docs touched: `docs/specs/audio_qa/automated_qa.md; docs/reviews/riotbox_1526_w30_capacity_preflight_2026-10-01.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prevent impossible W-30 CLI Vec/PCM16 RIFF sizes before source reads, rendering, and output mutation.

## What Shipped

- Checked frame, sample and byte capacities and existing PCM16 RIFF representation limits, preserving original rounding, valid render behavior and errors.

## Notes

- No source corpus, holdout, commercial reference, device, playback, 32-bit execution or musical/product quality claim. Routine regression slice does not advance the structural-refactor checkpoint cadence.

# `RIOTBOX-1536` Preserve W-30 preview input across WAV and metrics output collisions

- Ticket: `RIOTBOX-1536`
- Title: `Preserve W-30 preview input across WAV and metrics output collisions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1536/preserve-w-30-preview-input-across-wav-and-metrics-output-collisions`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1536-w30-source-output-collision`
- Linear branch: `feature/riotbox-1536-preserve-w-30-preview-input-across-wav-and-metrics-output`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1616 (https://github.com/marang/riotbox/pull/1616)`
- Merge commit: `5f3baab877afcccb791f302a8842a32de2309324`
- Deleted from Linear: `2026-10-01`
- Verification: `Actual W-30 CLI red/green; all 85 focused Debug/Release names/statuses pass; unchanged complete guard-body token stream; W-30 32 normal CLI and 9 hashes, Feral 31 CLI and 39 hashes exact; full source-free just ci actual zero and warning-free; exact-head native Rust/Windows transport SUCCESS; solo review.`
- Docs touched: `docs/reviews/riotbox_1536_w30_source_output_preservation_2026-10-01.md`, `docs/specs/audio_qa/manifests_and_artifacts.md`
- Follow-ups: `RIOTBOX-1537: separately reproduced Before/After pack source loss; reuse unchanged common guard with its own typed 14-output plan after this merge.`

## Why This Ticket Existed

W-30 preview could replace its original WAV or metrics input while reporting success; duplicating the platform identity loop would add safety drift.

## What Shipped

- Preserve explicit W-30 input against both planned output paths and aliases; one ordinary binary-only physical-identity owner reused by Feral and W-30; retain layouts and ordinary output bytes.

## Notes

- RBX-406. Stable namespace and readable output requirement remain explicit; no atomic pack/power-loss/concurrent namespace or Windows Audio claims. Generated controls only, no real audio/playback/DAW/subagents.

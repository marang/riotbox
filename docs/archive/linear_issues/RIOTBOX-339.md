# `RIOTBOX-339` Add Feral before-after audio QA render pack

- Ticket: `RIOTBOX-339`
- Title: `Add Feral before-after audio QA render pack`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-339/add-feral-before-after-audio-qa-render-pack`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-339-add-feral-before-after-audio-qa-render-pack`
- PR: `#329`
- Merge commit: `e73e02b`
- Labels: `benchmark`, `Audio`
- Verification: `cargo fmt --check`, `cargo test -p riotbox-audio`, `cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`, `just feral-before-after`
- Follow-ups: `None`

## Why This Ticket Existed

The useful manual Feral before/after demo needed to become a reproducible local QA helper rather than a one-off `ffmpeg` mix.

## What Shipped

- Added `feral_before_after_pack`.
- Added `just feral-before-after`.
- Wrote source, Riotbox-after, before/after, stems, metrics, comparison, and README artifacts for source WAV review.

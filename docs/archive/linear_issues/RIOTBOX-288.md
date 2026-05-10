# `RIOTBOX-288` Add W-30 smoke comparison report output

- Ticket: `RIOTBOX-288`
- Title: `Add W-30 smoke comparison report output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-288/add-w-30-smoke-comparison-report-output`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-288-add-w-30-smoke-comparison-report-output`
- Linear branch: `feature/riotbox-288-add-w-30-smoke-comparison-report-output`
- PR: `#278`
- Merge commit: `e18b7b6`
- Labels: `workflow`, `benchmark`
- Follow-ups: `RIOTBOX-289`

## Why This Ticket Existed

`RIOTBOX-287` made W-30 smoke metrics comparison visible in the terminal. The result also needed a durable local artifact path so a listening pass can keep metrics drift and listening notes together without committing generated audio QA outputs.

## What Shipped

- Made `w30_preview_compare` write its comparison summary to `comparison.md` by default.
- Added `--report PATH` for ad hoc report destinations while preserving stdout output.
- Updated the W-30 smoke/audio QA docs so the local artifact convention includes `comparison.md`.

## Verification

- `cargo test -p riotbox-audio --bin w30_preview_compare`
- `cargo run -p riotbox-audio --bin w30_preview_compare -- --date 2026-04-26`
- Verified local `comparison.md` output under ignored `artifacts/audio_qa/`.
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Local report persistence only; no waveform/perceptual audio diff, baseline promotion workflow, multi-pack runner, or CI audio artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

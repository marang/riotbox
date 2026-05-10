# `RIOTBOX-287` Add W-30 smoke metrics comparison helper

- Ticket: `RIOTBOX-287`
- Title: `Add W-30 smoke metrics comparison helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-287/add-w-30-smoke-metrics-comparison-helper`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-287-add-w-30-smoke-metrics-comparison-helper`
- Linear branch: `feature/riotbox-287-add-w-30-smoke-metrics-comparison-helper`
- PR: `#277`
- Merge commit: `ad02a96`
- Labels: `workflow`, `benchmark`
- Follow-ups: `RIOTBOX-288`

## Why This Ticket Existed

`RIOTBOX-285` defined the baseline-vs-candidate artifact shape and `RIOTBOX-286` made the W-30 smoke renderer write those paths directly. The next bounded QA slice was a local helper that compares the two sibling metrics files and reports drift without claiming waveform or perceptual audio comparison.

## What Shipped

- Added `w30_preview_compare` for local W-30 smoke baseline-vs-candidate Markdown metrics comparison.
- Reported active-sample, peak, RMS, and sum deltas with strict default limits and override flags.
- Updated W-30 smoke pack, audio QA convention/spec, and command shortlist docs.

## Verification

- `cargo test -p riotbox-audio --bin w30_preview_compare`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role baseline --duration-seconds 0.1`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role candidate --duration-seconds 0.1`
- `cargo run -p riotbox-audio --bin w30_preview_compare -- --date 2026-04-26`
- `cargo run -p riotbox-audio --bin w30_preview_compare -- --help`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Local metrics comparison only; no waveform/perceptual audio diff, baseline promotion workflow, multi-pack runner, or CI audio artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

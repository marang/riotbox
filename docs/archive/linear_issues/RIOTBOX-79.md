# `RIOTBOX-79` Add replay-safe W-30 internal resample audio regression fixtures

- Ticket: `RIOTBOX-79`
- Title: `Add replay-safe W-30 internal resample audio regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-79/add-replay-safe-w-30-internal-resample-audio-regression-fixtures`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-79-w30-resample-audio-regression`
- Linear branch: `feature/riotbox-79-add-replay-safe-w-30-internal-resample-audio-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#73`
- Merge commit: `e2eda43f95cafda0472b26f4bc98ec150cc86a1a`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#200`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-80`

## Why This Ticket Existed

`RIOTBOX-77` made the W-30 internal resample tap audibly real, and `RIOTBOX-78` made that seam visible in the shell, but the callback path still lacked the repo's normal fixture-backed audio hardening. The W-30 MVP needed the same replay-safe regression shape already protecting TR-909 and W-30 preview so later resample work would not depend only on one-off smoke tests.

## What Shipped

- added a dedicated `w30_resample_audio_regression.json` callback fixture corpus for the shipped audible resample seam
- mapped fixture rows into `RealtimeW30ResampleTapState` in `riotbox-audio` test code instead of inventing a new audio-harness style
- verified idle silence, transport-running lineage-ready taps, stopped-tap audibility, and zero-music-bus silence with the same active-sample and peak bounds used by the existing audio regression families
- recorded the verification choice in `docs/research_decision_log.md`

## Notes

- this slice is verification-only and does not change the shipped W-30 runtime behavior
- later W-30 audio callback work should keep widening the shared fixture-backed regression net instead of creating seam-specific harnesses

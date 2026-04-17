# `RIOTBOX-64` Ticket Archive

- Ticket: `RIOTBOX-64`
- Title: `Add replay-safe W-30 preview audio fixtures and regression checks`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-64/add-replay-safe-w-30-preview-audio-fixtures-and-regression-checks`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-64-w30-preview-audio-regression`
- Linear branch: `feature/riotbox-64-add-replay-safe-w-30-preview-audio-fixtures-and-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#58`
- Merge commit: `58292df`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`, `branch-level code-review`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-65`, `RIOTBOX-66`, `RIOTBOX-67`

## Why This Ticket Existed

The W-30 MVP already had a typed preview seam, audible preview output, a first playable trigger, and a typed resample tap seam, but the audible preview path still lacked the same fixture-backed callback regression net already protecting TR-909 and the earlier W-30 committed cue work. Riotbox needed one bounded verification slice that hardened the shipped preview path without adding new W-30 behavior.

## What Shipped

- Extended the shared `w30_regression.json` fixture corpus with expected committed preview projection fields at the app layer.
- Added a dedicated callback-level fixture corpus at `crates/riotbox-audio/tests/fixtures/w30_preview_audio_regression.json`.
- Added fixture-backed app assertions for committed W-30 preview projection in `jam_app` tests.
- Added fixture-backed audio assertions for the audible W-30 preview callback path in `riotbox-audio` tests.

## Notes

- This slice is verification-only and intentionally leaves W-30 behavior unchanged.
- Later W-30 work should keep extending the same committed preview and capture-lineage seams rather than bypassing them with a second playback or resample runtime.

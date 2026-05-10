# `RIOTBOX-186` Extend W-30 regression fixtures for raw capture audition

- Ticket: `RIOTBOX-186`
- Title: `Extend W-30 regression fixtures for raw capture audition`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-186/extend-w-30-regression-fixtures-for-raw-capture-audition`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-186-raw-audition-regressions`
- Linear branch: `feature/riotbox-186-extend-w-30-regression-fixtures-for-raw-capture-audition`
- PR: `#176`
- Merge commit: `60636f3`
- Labels: `benchmark`, `TUI`
- Follow-ups: `RIOTBOX-187`

## Why This Ticket Existed

`RIOTBOX-185` made raw-capture audition real with direct tests, but the shared W-30 fixture corpus still primarily described recall and promoted-audition paths. The raw path needed to join the same fixture-backed app, shell, and audio regression net before deeper W-30 work could build on it safely.

## What Shipped

- Added a raw-capture audition case to `crates/riotbox-app/tests/fixtures/w30_regression.json`.
- Extended the W-30 app and shell fixture harnesses so the primary capture can intentionally remain unassigned for raw-audition coverage.
- Added an explicit committed-command assertion to the app fixture regression test.
- Added a raw-capture audition case to the W-30 preview audio regression fixture corpus.

## Verification

- `cargo test -p riotbox-app w30_fixture_backed_committed_state_regressions_hold`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `cargo test -p riotbox-audio fixture_backed_w30_preview_audio_regressions_hold`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This was verification-only. Runtime behavior, queue semantics, session state, TUI layout, and audio synthesis behavior stayed unchanged.

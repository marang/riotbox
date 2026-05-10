# `RIOTBOX-199` Add fixture-backed W-30 source-window Log regression

- Ticket: `RIOTBOX-199`
- Title: `Add fixture-backed W-30 source-window Log regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-199/add-fixture-backed-w-30-source-window-log-regression`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-199-w30-source-window-fixture`
- Linear branch: `feature/riotbox-199-add-fixture-backed-w-30-source-window-log-regression`
- PR: `#189`
- Merge commit: `eebb701`
- Labels: `Audio`, `benchmark`, `ux`
- Follow-ups: `RIOTBOX-200`

## Why This Ticket Existed

`RIOTBOX-197` added focused unit coverage for the Log source-window cue, but the broader W-30 fixture regression corpus still only exercised fallback-style captures.

## What Shipped

- Extended W-30 regression fixture loading with optional source-window metadata for the primary capture.
- Added source-window metadata to the live-recall fixture.
- Added a fixture-backed Log expectation for the compact `win ...` cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Test/projection slice only; no audio behavior changed.
- This locks source-window Log visibility into the broader W-30 fixture regression path.

# `RIOTBOX-201` Add fixture coverage for recent Capture source-window shorthand

- Ticket: `RIOTBOX-201`
- Title: `Add fixture coverage for recent Capture source-window shorthand`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-201/add-fixture-coverage-for-recent-capture-source-window-shorthand`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-201-capture-recent-source-window-fixture`
- Linear branch: `feature/riotbox-201-add-fixture-coverage-for-recent-capture-source-window`
- PR: `#191`
- Merge commit: `979ed57`
- Labels: `Audio`, `benchmark`, `ux`
- Follow-ups: `RIOTBOX-202`

## Why This Ticket Existed

`RIOTBOX-200` added focused UI coverage for the Recent Captures source-window shorthand, but the broader W-30 fixture corpus also needed to lock the cue into the fixture-backed shell path.

## What Shipped

- Added the recent Capture source-window shorthand to the W-30 fixture-backed Capture expectations.
- Kept the change expectation-only against the existing W-30 source-window fixture metadata.

## Verification

- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture expectation only; no runtime, model, or audio behavior changed.
- This pairs the focused `RIOTBOX-200` test with broad W-30 shell fixture coverage.

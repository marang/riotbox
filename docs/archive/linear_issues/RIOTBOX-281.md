# `RIOTBOX-281` Add source-backed W-30 hit fixture coverage

- Ticket: `RIOTBOX-281`
- Title: `Add source-backed W-30 hit fixture coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-281/add-source-backed-w-30-hit-fixture-coverage`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-281-add-source-backed-w-30-hit-fixture-coverage`
- Linear branch: `feature/riotbox-281-add-source-backed-w-30-hit-fixture-coverage`
- PR: `#271`
- Merge commit: `d6d4c8a`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-282`, `RIOTBOX-283`

## Why This Ticket Existed

`RIOTBOX-279` pinned source-backed W-30 hit behavior in a focused app/runtime regression. The shared W-30 fixture corpus also needed to represent the trigger/source-window seam so fixture-backed state and shell regressions stay aligned with that product path.

## What Shipped

- Extended the committed-state W-30 fixture harness with `source_window` support.
- Added a `trigger_pad_source_window_committed` fixture row for promoted W-30 hit behavior.
- Wired `trigger_pad` through the W-30 shell fixture harness so Jam, Capture, and Log snapshots cover the case.

## Verification

- `cargo test -p riotbox-app w30_fixture_backed_committed_state_regressions_hold`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture coverage only; no audio DSP, new fixture harness architecture, or formal listening-pack gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

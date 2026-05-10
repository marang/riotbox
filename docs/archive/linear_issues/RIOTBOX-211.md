# `RIOTBOX-211` Reduce Scene regression fixture taxonomy drift

- Ticket: `RIOTBOX-211`
- Title: `Reduce Scene regression fixture taxonomy drift`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-211/reduce-scene-regression-fixture-taxonomy-drift`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-211-scene-fixture-taxonomy`
- Linear branch: `feature/riotbox-211-reduce-scene-regression-fixture-taxonomy-drift`
- PR: `#201`
- Merge commit: `6d64d8d`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-212`

## Why This Ticket Existed

The periodic Scene Brain TUI seam review found that the Scene regression fixture label/energy taxonomy was duplicated across app UI tests, app state tests, and core view tests. Fixture edits therefore had a growing drift risk.

## What Shipped

- Added an app-local `#[cfg(test)]` support module for Scene regression fixture label/energy mapping.
- Reused that mapping from both app-shell and app-state fixture tests.
- Documented the remaining cross-crate fixture taxonomy contract beside the regression fixture files.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app scene_fixture_backed_committed_state_regressions_hold`
- `cargo test -p riotbox-app scene_fixture_backed_shell_regressions_hold`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Test-support/documentation follow-up only; no production model, runtime Scene Brain policy, or broad fixture redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

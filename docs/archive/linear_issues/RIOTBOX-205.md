# `RIOTBOX-205` Add low-energy Scene Brain restore fixture coverage

- Ticket: `RIOTBOX-205`
- Title: `Add low-energy Scene Brain restore fixture coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-205/add-low-energy-scene-brain-restore-fixture-coverage`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-205-low-energy-scene-fixture`
- Linear branch: `feature/riotbox-205-add-low-energy-scene-brain-restore-fixture-coverage`
- PR: `#195`
- Merge commit: `0c8bac8`
- Labels: `benchmark`, `ux`
- Follow-ups: `RIOTBOX-206`

## Why This Ticket Existed

`RIOTBOX-204` made Scene Brain fixture energy vary by section label, but the shared fixture corpus still mostly proved `intro/medium <> drop/high`. The low-energy `break` path needed broad fixture coverage so the readability stack is not biased toward only medium/high examples.

## What Shipped

- Added a shared Scene Brain regression fixture for restoring from `drop/high` into `break/low`.
- Locked the expected Jam contrast cue as `live break/low <> restore drop/high`.
- Verified the app-shell, jam-app, and core fixture-backed tests consume the new expectation.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app scene_fixture_backed_committed_state_regressions_hold`
- `cargo test -p riotbox-app scene_fixture_backed_shell_regressions_hold`
- `cargo test -p riotbox-core fixture_backed_scene_energy_projection_holds`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture/test only; no runtime Scene Brain policy, UI redesign, persistence, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

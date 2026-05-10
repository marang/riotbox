# `RIOTBOX-204` Add varied Scene Brain energy fixture coverage

- Ticket: `RIOTBOX-204`
- Title: `Add varied Scene Brain energy fixture coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-204/add-varied-scene-brain-energy-fixture-coverage`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-204-varied-scene-energy-fixtures`
- Linear branch: `feature/riotbox-204-add-varied-scene-brain-energy-fixture-coverage`
- PR: `#194`
- Merge commit: `8085bab`
- Labels: `benchmark`, `ux`
- Follow-ups: `TBD`

## Why This Ticket Existed

The Scene Brain UI already shows compact `scene/energy` cues, but the shared Scene regression fixture flattened intro and drop sections to the same high-energy state. That under-covered the `intro/medium <> drop/high` contrast musicians need to read during jump/restore flows.

## What Shipped

- Added fixture energy helpers that assign varied Scene Brain energy by section label in the app-shell, jam-app, and core fixture paths.
- Updated shared scene regression expectations to lock intro/medium and drop/high contrast across restore/live surfaces.
- Kept the change fixture/test-focused with no new runtime Scene Brain policy.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app scene_fixture_backed_committed_state_regressions_hold`
- `cargo test -p riotbox-app scene_fixture_backed_shell_regressions_hold`
- `cargo test -p riotbox-core fixture_backed_scene_energy_projection_holds`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture/test realism only; no audio behavior, runtime scene policy, or persistence model changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

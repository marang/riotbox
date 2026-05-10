# `RIOTBOX-244` Add fixture-backed Scene restore TR-909 accent regression

- Ticket: `RIOTBOX-244`
- Title: `Add fixture-backed Scene restore TR-909 accent regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-244/add-fixture-backed-scene-restore-tr-909-accent-regression`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-244-scene-restore-fixture-accent`
- Linear branch: `feature/riotbox-244-add-fixture-backed-scene-restore-tr-909-accent-regression`
- PR: `#234`
- Merge commit: `ad0d885`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-245`

## Why This Ticket Existed

`RIOTBOX-243` added a focused unit regression for restore-to-TR-909 target coupling. The broader Scene fixture matrix also needed a representative restore case so fixture-backed Scene coverage protects session state, runtime projection, and visible Log wording together.

## What Shipped

- Extended the Scene regression fixture with optional TR-909 setup.
- Added optional fixture-backed state assertions for TR-909 profile, support context, and support accent.
- Updated the restore fixture row to exercise `source_support`, `scene_target`, and visible `accent scene` Log wording.
- Kept the fixture extension backward-compatible for existing Scene rows.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-app committed_scene_restore_projects_target_scene_into_tr909_source_support -- --nocapture`
- `cargo test -p riotbox-app scene_fixture_backed_committed_state_regressions_hold -- --nocapture`
- `cargo test -p riotbox-app scene_fixture_backed_shell_regressions_hold -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture/regression-only slice; no runtime policy, audio mix tuning, broad fixture redesign, or TUI layout changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

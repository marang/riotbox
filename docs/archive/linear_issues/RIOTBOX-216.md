# `RIOTBOX-216` Project next Scene launch target through Jam view model

- Ticket: `RIOTBOX-216`
- Title: `Project next Scene launch target through Jam view model`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-216/project-next-scene-launch-target-through-jam-view-model`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-216-next-scene-view-projection`
- Linear branch: `feature/riotbox-216-project-next-scene-launch-target-through-jam-view-model`
- PR: `#206`
- Merge commit: `588338d`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-217`

## Why This Ticket Existed

The Jam UI explained suggested Scene launch direction by duplicating next-Scene candidate logic inside the TUI layer. That risked drift between what `[y]` would queue and what the screen suggested.

## What Shipped

- Added `next_scene` and `next_scene_energy` to the core Jam scene summary.
- Removed duplicated next-Scene candidate selection from the TUI layer.
- Added focused regressions for projected next Scene target and energy.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core derives_scene_energy_from_projected_scene_id`
- `cargo test -p riotbox-app queue_scene_select_enqueues_scene_launch_for_next_bar`
- `cargo test -p riotbox-app renders_jam_shell_with_post_commit_next_step_cue`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- View-model/TUI alignment slice only; no Scene selection policy, audio behavior, persistence model, or broad Jam redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

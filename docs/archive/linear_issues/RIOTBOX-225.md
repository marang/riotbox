# `RIOTBOX-225` Align Scene queue candidate policy with Jam view

- Ticket: `RIOTBOX-225`
- Title: `Align Scene queue candidate policy with Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-225/align-scene-queue-candidate-policy-with-jam-view`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-225-scene-queue-candidate-policy`
- Linear branch: `feature/riotbox-225-align-scene-queue-candidate-policy-with-jam-view`
- PR: `#215`
- Merge commit: `58b9f2c`
- Labels: `ux`
- Follow-ups: `RIOTBOX-226`

## Why This Ticket Existed

The Jam view modeled Scene jump availability explicitly, but the app queue still computed a local next-scene candidate and then separately rejected the single-scene same-target case. That kept a small policy duplicate in the queue path.

## What Shipped

- Aligned the app Scene queue candidate helper with the Jam view rule that a single current Scene is not queueable as the next Scene.
- Removed the redundant same-scene rejection branch from `queue_scene_select`.
- Added focused queue regression coverage proving the single-current-scene case rejects without creating a pending action.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app queue_scene_select_`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Queue-policy cleanup only; no new Scene selection algorithm, UI copy, source analysis, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

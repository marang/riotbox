# `RIOTBOX-224` Align Scene jump unavailable queue feedback

- Ticket: `RIOTBOX-224`
- Title: `Align Scene jump unavailable queue feedback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-224/align-scene-jump-unavailable-queue-feedback`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-224-scene-jump-unavailable-feedback`
- Linear branch: `feature/riotbox-224-align-scene-jump-unavailable-queue-feedback`
- PR: `#214`
- Merge commit: `d87c48e`
- Labels: `ux`
- Follow-ups: `RIOTBOX-225`

## Why This Ticket Existed

The Jam surfaces could explain that Scene jump waits for more scene material, but pressing `y` in that state still fell back to the generic `no next scene candidate available` status. The action feedback needed to match the visible affordance.

## What Shipped

- Used the shared Scene jump availability state when a rejected `y` press reports status.
- Rendered `scene jump waits for 2 scenes` for the known too-few-scenes case.
- Preserved the generic missing-candidate fallback for other unknown cases.
- Added a bin-level regression for the unavailable Scene jump status helper.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app scene_select_unavailable_status_explains_waiting_for_scene_material`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Feedback-only slice; no Scene selection policy, source analysis, audio behavior, or broad status-message rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

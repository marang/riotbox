# `RIOTBOX-213` Surface next Scene launch direction in suggested gesture

- Ticket: `RIOTBOX-213`
- Title: `Surface next Scene launch direction in suggested gesture`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-213/surface-next-scene-launch-direction-in-suggested-gesture`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-213-scene-launch-suggested-direction`
- Linear branch: `feature/riotbox-213-surface-next-scene-launch-direction-in-suggested-gesture`
- PR: `#203`
- Merge commit: `b4f9a90`
- Labels: `ux`
- Follow-ups: `RIOTBOX-214`

## Why This Ticket Existed

Restore gestures already surfaced energy direction across the Jam cue stack. Scene launch still used a generic `[y] jump` suggestion even when the app could name the deterministic next target and whether it would rise, drop, or hold energy.

## What Shipped

- Added compact next Scene target and `rise/drop/hold` direction wording to the suggested jump gesture when deterministic.
- Preserved the generic `[y] jump` fallback when no next target is available.
- Updated focused Jam regression coverage for the post-commit next-step cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_post_commit_next_step_cue`
- `cargo test -p riotbox-app renders_jam_shell_with_scene_brain_summary`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI wording slice only; no Scene Brain policy, model, persistence, or audio behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

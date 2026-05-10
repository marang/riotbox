# `RIOTBOX-221` Align Help Scene jump availability wording

- Ticket: `RIOTBOX-221`
- Title: `Align Help Scene jump availability wording`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-221/align-help-scene-jump-availability-wording`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-221-help-scene-jump-availability`
- Linear branch: `feature/riotbox-221-align-help-scene-jump-availability-wording`
- PR: `#211`
- Merge commit: `be920d2`
- Labels: `ux`
- Follow-ups: `RIOTBOX-222`

## Why This Ticket Existed

Footer and suggested gestures distinguished an available Scene jump from a jump that waits for more scene material. The Help primary gesture list needed the same availability wording so first-run guidance would not contradict the Jam screen.

## What Shipped

- Reused the Scene jump availability label for Help primary gestures.
- Kept normal `y: scene jump` Help wording when Scene jump availability is not known-unavailable.
- Extended the single-Scene Jam regression to cover Help wording.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_single_scene_jump_waiting_cue`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Help wording slice only; no keymap, Scene selection policy, or broad Help redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

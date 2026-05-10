# `RIOTBOX-223` Model Scene jump availability in Jam view

- Ticket: `RIOTBOX-223`
- Title: `Model Scene jump availability in Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-223/model-scene-jump-availability-in-jam-view`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-223-scene-jump-availability-view`
- Linear branch: `feature/riotbox-223-model-scene-jump-availability-in-jam-view`
- PR: `#213`
- Merge commit: `36d5820`
- Labels: `ux`
- Follow-ups: `RIOTBOX-224`

## Why This Ticket Existed

The Scene jump surfaces explained the known too-few-scenes case across Suggested, Footer, Help, and Overview, but the TUI still inferred that condition directly from `scene_count` in several helpers. That repeated policy knowledge in presentation code and made future wording drift likely.

## What Shipped

- Added explicit `SceneJumpAvailabilityView` to the shared Jam view model.
- Derived `Ready` and `WaitingForMoreScenes` next to the existing `next_scene` projection.
- Updated Jam Suggested, Footer, Help, and Overview helpers to consume the view-model availability state.
- Documented that Scene jump availability belongs in the shared Jam view model.
- Extended core Jam view assertions for ready and waiting Scene jump states.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core view::jam::tests::`
- `cargo test -p riotbox-app renders_jam_shell_with_single_scene_jump_waiting_cue`
- `git diff --check`
- `rg -n "Scene jump availability|no-queueable-Scene|view-model availability" docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- View-model contract slice only; no Scene selection policy, user-facing copy, audio behavior, or broad Jam refactor changed.
- The fast-forward merge means the final feature commit is also the merge commit on `main`.

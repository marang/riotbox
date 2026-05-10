# `RIOTBOX-218` Clarify disabled Scene jump suggestion

- Ticket: `RIOTBOX-218`
- Title: `Clarify disabled Scene jump suggestion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-218/clarify-disabled-scene-jump-suggestion`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-218-disabled-scene-jump-suggestion`
- Linear branch: `feature/riotbox-218-clarify-disabled-scene-jump-suggestion`
- PR: `#208`
- Merge commit: `28119b0`
- Labels: `ux`
- Follow-ups: `RIOTBOX-219`

## Why This Ticket Existed

After `RIOTBOX-216`, the Jam view could tell when there was no queueable next Scene target. The primary suggested gesture still risked implying that `[y] jump` would do something meaningful when only one or no Scene target was available.

## What Shipped

- Rendered `[y] jump waits for 2 scenes` when no queueable Scene jump exists because the session has too few scenes.
- Kept the generic `[y] jump` fallback for unknown target data.
- Added focused Jam shell regression coverage for the single-Scene cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_jam_shell_with_single_scene_jump_waiting_cue`
- `cargo test -p riotbox-app renders_jam_shell_with_first_run_onramp`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI wording slice only; no Scene selection policy, source analysis, audio behavior, or broad onboarding flow changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

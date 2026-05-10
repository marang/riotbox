# `RIOTBOX-238` Surface TR-909 support accent cue in diagnostics

- Ticket: `RIOTBOX-238`
- Title: `Surface TR-909 support accent cue in diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-238/surface-tr-909-support-accent-cue-in-diagnostics`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-238-tr909-accent-diagnostics`
- Linear branch: `feature/riotbox-238-surface-tr-909-support-accent-cue-in-diagnostics`
- PR: `#228`
- Merge commit: `a46adba`
- Labels: `ux`, `review-followup`
- Follow-ups: `RIOTBOX-239`

## Why This Ticket Existed

`RIOTBOX-236` made `scene_target` source-support subtly audible, but diagnostics still only showed context/profile. The TUI needed a compact cue explaining whether the Scene-target support accent is active without adding a new control path or transition-engine promise.

## What Shipped

- Derived a compact TR-909 support accent label from runtime render state.
- Surfaced `accent scene`, `accent off fallback`, and `accent off` in existing Log/Inspect diagnostics.
- Split the Log TR-909 render line so the accent cue remains visible in the narrow render card.
- Added runtime-view and UI regression assertions for the new cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app tr909_render -- --nocapture`
- `cargo test -p riotbox-app committed_scene_select_projects_target_scene_into_tr909_source_support -- --nocapture`
- `cargo test -p riotbox-app source_support_render_profile_tracks_current_source_section -- --nocapture`
- `cargo test -p riotbox-app renders_jam_shell_inspect_snapshot -- --nocapture`
- `cargo test -p riotbox-app renders_log_shell_snapshot_with_action_trust_history -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Diagnostic slice only; no audio behavior, keybinding, broad TUI redesign, or Scene selection policy changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

# `RIOTBOX-231` Surface TR-909 source-support context in runtime diagnostics

- Ticket: `RIOTBOX-231`
- Title: `Surface TR-909 source-support context in runtime diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-231/surface-tr-909-source-support-context-in-runtime-diagnostics`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-231-tr909-support-context-diagnostics`
- Linear branch: `feature/riotbox-231-surface-tr-909-source-support-context-in-runtime-diagnostics`
- PR: `#221`
- Merge commit: `2ea783e`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-232`

## Why This Ticket Existed

`RIOTBOX-230` made projected Scene targets influence TR-909 `SourceSupport`, but the runtime diagnostics only showed the selected profile. Operators needed to see whether that profile came from the Scene target or the transport-bar fallback.

## What Shipped

- Added typed TR-909 source-support context labels: `scene_target`, `transport_bar`, and `unset`.
- Carried the context through core policy projection and app render-state projection.
- Surfaced context in existing TR-909 Log and Inspect diagnostics.
- Added runtime warning coverage for inconsistent profile/context state.
- Added focused regression assertions for Scene-target and transport-bar context wording.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core source_support_profile`
- `cargo test -p riotbox-audio render_profile_labels_stay_stable`
- `cargo test -p riotbox-app runtime_view_surfaces_tr909_render_diagnostics`
- `cargo test -p riotbox-app committed_scene_select_projects_target_scene_into_tr909_source_support`
- `cargo test -p riotbox-app renders_log_shell_snapshot_with_action_trust_history`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Diagnostic-only slice; no audio behavior, Scene selection policy, realtime callback path, or broad TUI redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

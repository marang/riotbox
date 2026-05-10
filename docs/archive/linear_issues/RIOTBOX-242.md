# `RIOTBOX-242` Add TR-909 accent diagnostics to TUI screenshot baseline

- Ticket: `RIOTBOX-242`
- Title: `Add TR-909 accent diagnostics to TUI screenshot baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-242/add-tr-909-accent-diagnostics-to-tui-screenshot-baseline`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-242-tr909-accent-baseline`
- Linear branch: `feature/riotbox-242-add-tr-909-accent-diagnostics-to-tui-screenshot-baseline`
- PR: `#232`
- Merge commit: `9524d45`
- Labels: `ux`, `benchmark`
- Follow-ups: `RIOTBOX-243`

## Why This Ticket Existed

The runtime, UI, recipe, and TUI spec slices already agreed on TR-909 support accent cues. The existing render diagnostics baseline still predated `accent scene` and `accent off fallback`, so future TUI work lacked a small review artifact for preserving that visible wording.

## What Shipped

- Refreshed `docs/screenshots/jam_tr909_render_diagnostics_baseline.txt`.
- Added Log examples for `render source_support | accent scene` and `render source_support | accent off fallback`.
- Added Jam Inspect tuple examples for `scene_target` and `transport_bar` source-support contexts.
- Reiterated that the accent cue is diagnostic-only, not a transition engine or arranger promise.

## Verification

- `git diff --check`
- `rg -n "RIOTBOX-242|accent scene|accent off fallback|transition engine|transport-bar fallback" docs/screenshots/jam_tr909_render_diagnostics_baseline.txt`
- `cargo test -p riotbox-app renders_log_shell_snapshot_with_action_trust_history -- --nocapture`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/baseline-only slice; no runtime behavior, audio behavior, layout redesign, broad Jam simplification, or recipe wording changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

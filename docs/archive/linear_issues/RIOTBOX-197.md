# `RIOTBOX-197` Surface source-window span in Log W-30 diagnostics

- Ticket: `RIOTBOX-197`
- Title: `Surface source-window span in Log W-30 diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-197/surface-source-window-span-in-log-w-30-diagnostics`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-197-log-source-window-span`
- Linear branch: `feature/riotbox-197-surface-source-window-span-in-log-w-30-diagnostics`
- PR: `#187`
- Merge commit: `e665267`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-198`

## Why This Ticket Existed

Capture exposed source-window provenance and the README explained `.../src` versus `.../fallback`, but Log is the primary truth screen for committed actions. Users needed a compact Log cue showing which source excerpt backed the current W-30 cue.

## What Shipped

- Added a compact `win <start>-<end>s <source>` cue to the W-30 Log lane when the focused capture has source-window metadata.
- Preserved the existing `cap <id> | <trigger>` fallback line for captures without source-window metadata.
- Added focused UI regression coverage for the visible Log cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_log_w30_source_window_when_available`
- `cargo test -p riotbox-app renders_log_shell_snapshot_with_action_trust_history`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Presentation-only slice; no audio behavior or persistence changed.
- The Log cue is intentionally shortened so it remains visible in the narrow W-30 lane.

# `RIOTBOX-195` Surface capture source-window span in Capture provenance

- Ticket: `RIOTBOX-195`
- Title: `Surface capture source-window span in Capture provenance`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-195/surface-capture-source-window-span-in-capture-provenance`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-195-capture-source-window-provenance`
- Linear branch: `feature/riotbox-195-surface-capture-source-window-span-in-capture-provenance`
- PR: `#185`
- Merge commit: `4ba6dd0`
- Labels: `Core`, `ux`
- Follow-ups: `RIOTBOX-196`

## Why This Ticket Existed

W-30 source-backed preview now depends on `CaptureRef.source_window`, but the Capture screen did not make the captured source span easy to inspect before auditioning or promoting.

## What Shipped

- Added a compact `win <source> <start>-<end>s` line to Capture provenance when the latest capture has source-window metadata.
- Preserved the low-noise legacy path for captures without source windows.
- Added a focused Capture shell snapshot test for the new provenance cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_capture_provenance_source_window_when_available`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_capture_context`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This is presentation-only and does not change source-window persistence or audio rendering.
- The cue is intentionally short so it fits inside the existing Capture provenance box.

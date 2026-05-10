# `RIOTBOX-200` Surface source-window shorthand in recent Capture list

- Ticket: `RIOTBOX-200`
- Title: `Surface source-window shorthand in recent Capture list`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-200/surface-source-window-shorthand-in-recent-capture-list`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-200-capture-recent-source-window`
- Linear branch: `feature/riotbox-200-surface-source-window-shorthand-in-recent-capture-list`
- PR: `#190`
- Merge commit: `c49fad4`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-201`

## Why This Ticket Existed

Capture provenance showed the latest source-window span and Log showed the current W-30 source cue, but the Recent Captures list still only showed capture id, target, and origin count.

## What Shipped

- Added a compact source-window time-span shorthand to Recent Captures rows when metadata exists.
- Preserved the previous target/origin row for captures without source-window metadata.
- Added focused Capture screen regression coverage.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app renders_recent_capture_source_window_shorthand_when_available`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Presentation-only slice; no audio behavior or session model changed.
- The shorthand intentionally favors a readable source span over adding more dense metadata to the narrow list row.

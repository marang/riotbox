# `RIOTBOX-202` Centralize compact source-window formatting for TUI surfaces

- Ticket: `RIOTBOX-202`
- Title: `Centralize compact source-window formatting for TUI surfaces`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-202/centralize-compact-source-window-formatting-for-tui-surfaces`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-202-source-window-format-helpers`
- Linear branch: `feature/riotbox-202-centralize-compact-source-window-formatting-for-tui-surfaces`
- PR: `#192`
- Merge commit: `f9b21ff`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-203`

## Why This Ticket Existed

Source-window cues appeared in Capture provenance, W-30 Log diagnostics, and Recent Captures with intentionally different compact shapes, but formatting logic was duplicated across the UI code.

## What Shipped

- Added small TUI-local helpers for source-window span formatting.
- Preserved the existing visible strings for Capture provenance, W-30 Log, and Recent Captures.
- Kept the focused source-window UI regression tests green across the refactor.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app source_window`
- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Refactor only; no model, persistence, or audio behavior changed.
- The helpers preserve surface-specific ordering while sharing the span formatter.

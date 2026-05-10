# `RIOTBOX-258` Centralize Jam semantic style helpers

- Ticket: `RIOTBOX-258`
- Title: `Centralize Jam semantic style helpers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-258/centralize-jam-semantic-style-helpers`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-258-centralize-jam-semantic-style-helpers`
- Linear branch: `feature/riotbox-258-centralize-jam-semantic-style-helpers`
- PR: `#248`
- Merge commit: `3256ca5`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-259`

## Why This Ticket Existed

`RIOTBOX-256` found that cyan/yellow/green/dark-gray semantic styles were encoded directly across footer, Capture, landed-result, Scene post-commit, pending intent, and timing rail helpers. The colors were consistent, but each new hierarchy slice increased drift risk.

## What Shipped

- Added small local semantic style helpers for primary controls, pending cues, confirmations, warnings, and low-emphasis context.
- Migrated existing Jam/Capture hierarchy renderers to those helpers without changing visible text or palette.
- Kept the existing style-focused assertions intact.

## Verification

- `cargo test -p riotbox-app styles_define -- --nocapture`
- `cargo test -p riotbox-app capture_pending_do_next_styles_define_pending_hierarchy -- --nocapture`
- `cargo test -p riotbox-app next_panel_promotes_timing_rail_above_landed_history -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI maintainability slice only; no palette redesign, theme support, layout changes, audio behavior, or scheduler behavior changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

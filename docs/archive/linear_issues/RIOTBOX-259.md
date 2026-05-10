# `RIOTBOX-259` Style Jam suggested gesture key tokens

- Ticket: `RIOTBOX-259`
- Title: `Style Jam suggested gesture key tokens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-259/style-jam-suggested-gesture-key-tokens`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-259-style-jam-suggested-gesture-key-tokens`
- Linear branch: `feature/riotbox-259-style-jam-suggested-gesture-key-tokens`
- PR: `#249`
- Merge commit: `b8b7ba8`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-260`

## Why This Ticket Existed

`RIOTBOX-258` centralized primary-control, pending, confirmation, warning, and low-emphasis styles. The Jam suggested-gestures panel still read mostly as plain text even though it is one of the main perform-facing surfaces.

## What Shipped

- Styled bracketed key tokens in the Jam suggested-gestures panel with the primary-control semantic style.
- Kept all suggested-gesture text and fallback states unchanged.
- Added focused tests for key-token styling and rendered text stability.

## Verification

- `cargo test -p riotbox-app suggested_gesture_key_tokens_use_primary_control_style -- --nocapture`
- `cargo test -p riotbox-app suggested_gesture_lines_style_start_key_token -- --nocapture`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no keymap changes, new gestures, theme support, or broader Jam redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

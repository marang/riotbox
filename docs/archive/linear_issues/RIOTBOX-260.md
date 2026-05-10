# `RIOTBOX-260` Style Jam help overlay key tokens

- Ticket: `RIOTBOX-260`
- Title: `Style Jam help overlay key tokens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-260/style-jam-help-overlay-key-tokens`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-260-style-jam-help-overlay-key-tokens`
- Linear branch: `feature/riotbox-260-style-jam-help-overlay-key-tokens`
- PR: `#250`
- Merge commit: `fc4c2c7`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-261`

## Why This Ticket Existed

`RIOTBOX-259` styled key tokens in the suggested-gestures panel. The Help overlay still taught many of the same primary controls as plain text, so the two perform-facing guidance surfaces could drift visually.

## What Shipped

- Styled Help overlay `key: action` prefixes with the existing primary-control semantic style.
- Styled bracketed Capture help tokens such as `[p]` and `[w]`.
- Kept Help text, keymap behavior, and popup layout unchanged.

## Verification

- `cargo test -p riotbox-app help_key_prefixes_use_primary_control_style -- --nocapture`
- `cargo test -p riotbox-app help_primary_gesture_line_styles_key_prefixes_without_rewriting_text -- --nocapture`
- `cargo test -p riotbox-app renders_help_overlay_with_first_run_guidance -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no Help copy rewrite, keymap changes, popup layout changes, theme support, or broader Jam redesign changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

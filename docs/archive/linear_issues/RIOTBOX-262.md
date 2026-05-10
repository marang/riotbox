# `RIOTBOX-262` Style Jam footer gesture key tokens

- Ticket: `RIOTBOX-262`
- Title: `Style Jam footer gesture key tokens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-262/style-jam-footer-gesture-key-tokens`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-262-style-jam-footer-gesture-key-tokens`
- Linear branch: `feature/riotbox-262-style-jam-footer-gesture-key-tokens`
- PR: `#252`
- Merge commit: `5544bab`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-263`

## Why This Ticket Existed

Suggested gestures and Help used primary-control emphasis for key tokens, and the TUI spec recorded that contract. The Jam footer still styled only `Primary:` / `Scene:` labels while the actual gesture keys remained flat text.

## What Shipped

- Styled key prefixes in the Jam footer `Primary:` gesture list.
- Added a dedicated `Advanced:` footer renderer that styles advanced gesture key prefixes.
- Kept footer text, keymap behavior, and layout unchanged.

## Verification

- `cargo test -p riotbox-app footer_line_styles_define_first_visual_hierarchy -- --nocapture`
- `cargo test -p riotbox-app footer_advanced_line_styles_gesture_key_prefixes -- --nocapture`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no footer copy rewrite, keymap changes, layout redesign, or theme support changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

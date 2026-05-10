# `RIOTBOX-264` Style Jam footer top key legend tokens

- Ticket: `RIOTBOX-264`
- Title: `Style Jam footer top key legend tokens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-264/style-jam-footer-top-key-legend-tokens`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-264-style-jam-footer-top-key-legend-tokens`
- Linear branch: `feature/riotbox-264-style-jam-footer-top-key-legend-tokens`
- PR: `#254`
- Merge commit: `be96e67`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-265`

## Why This Ticket Existed

Suggested gestures, Help, and the footer gesture rows already used primary-control emphasis for key tokens. The top Jam footer `Keys:` legend still rendered every navigation and transport key as flat text, making live controls harder to scan.

## What Shipped

- Rendered the Jam footer top `Keys:` legend through a focused helper that preserves the exact existing text.
- Styled the legend key tokens, including `Tab`, `space`, `[ ]`, and `r`, with the existing primary-control semantic style.
- Added focused coverage for rendered-text stability and token styling.

## Verification

- `cargo test -p riotbox-app footer_keys_line_styles_top_legend_key_tokens -- --nocapture`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- TUI styling slice only; no footer copy reduction, navigation redesign, keymap behavior, or theme support changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

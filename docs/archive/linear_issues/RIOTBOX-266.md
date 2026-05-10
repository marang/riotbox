# `RIOTBOX-266` Compress Jam footer top key legend copy

- Ticket: `RIOTBOX-266`
- Title: `Compress Jam footer top key legend copy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-266/compress-jam-footer-top-key-legend-copy`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-266-compress-jam-footer-top-key-legend-copy`
- Linear branch: `feature/riotbox-266-compress-jam-footer-top-key-legend-copy`
- PR: `#256`
- Merge commit: `8eb2605`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-267`

## Why This Ticket Existed

The footer top `Keys:` legend had key-token emphasis, but still spent horizontal space spelling out every screen key separately. That kept the always-visible Jam footer text-heavy.

## What Shipped

- Compressed the always-visible Jam footer `Keys:` legend from per-screen labels to `1-4 screens`.
- Shortened inspect/perform and refresh footer labels while keeping detailed explanations in Help.
- Preserved key token styling and added focused coverage for ingest and load/inspect footer text.
- Recorded the compact-footer rule in the TUI spec.

## Verification

- `cargo test -p riotbox-app footer_keys_line -- --nocapture`
- `cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot -- --nocapture`
- `git diff --check`
- `rg -n "footer copy compact|1-4 screens|i inspect|r re-ingest|detailed key explanations" crates/riotbox-app/src/ui.rs docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI copy slice only; no keymap behavior, footer layout redesign, new controls, or theme/color changes shipped.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

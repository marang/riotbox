# `RIOTBOX-265` Document footer key legend token emphasis in TUI spec

- Ticket: `RIOTBOX-265`
- Title: `Document footer key legend token emphasis in TUI spec`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-265/document-footer-key-legend-token-emphasis-in-tui-spec`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-265-document-footer-key-legend-token-emphasis-in-tui-spec`
- Linear branch: `feature/riotbox-265-document-footer-key-legend-token-emphasis-in-tui-spec`
- PR: `#255`
- Merge commit: `7498a88`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-266`

## Why This Ticket Existed

`RIOTBOX-264` styled the top Jam footer `Keys:` legend tokens while preserving the text contract. The TUI spec already covered perform guidance, Help prefixes, and bracketed key tokens, but it did not explicitly name footer top key legends.

## What Shipped

- Documented that footer top key legends may emphasize only their key token with the existing primary-control treatment.
- Kept the rule narrow so monochrome text remains authoritative and no keymap or layout behavior is implied.

## Verification

- `git diff --check`
- `rg -n "footer top key legends|Tab switch|space play/pause|key-token emphasis" docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/spec-only slice; no runtime behavior, UI implementation, footer copy reduction, or new color semantics changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

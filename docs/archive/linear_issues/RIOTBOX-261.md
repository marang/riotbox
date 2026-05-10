# `RIOTBOX-261` Document Jam key-token emphasis contract

- Ticket: `RIOTBOX-261`
- Title: `Document Jam key-token emphasis contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-261/document-jam-key-token-emphasis-contract`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-261-document-jam-key-token-emphasis-contract`
- Linear branch: `feature/riotbox-261-document-jam-key-token-emphasis-contract`
- PR: `#251`
- Merge commit: `35d27db`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-262`

## Why This Ticket Existed

`RIOTBOX-259` and `RIOTBOX-260` styled perform-facing key tokens in Suggested gestures and Help using the primary-control semantic style. The TUI spec needed to record that key-token emphasis is allowed for guidance surfaces while preserving monochrome-readable text.

## What Shipped

- Documented perform guidance key-token emphasis in the TUI spec.
- Covered bracketed keys such as `[c]` / `[Space]` and Help-style `key: action` prefixes.
- Kept the contract explicit that text, keymap behavior, and layout remain unchanged.

## Verification

- `git diff --check`
- `rg -n 'perform guidance key tokens|bracketed key tokens|Help-style|key-token emphasis' docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Spec-only slice; no runtime behavior, UI implementation changes, theme support, or broad spec rewrite changed.
- The branch was updated with current `main` before merge, so the final fast-forward commit on `main` is the feature-branch head.

# `RIOTBOX-253` Document Scene pending intent hierarchy in TUI spec

- Ticket: `RIOTBOX-253`
- Title: `Document Scene pending intent hierarchy in TUI spec`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-253/document-scene-pending-intent-hierarchy-in-tui-spec`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-253-document-scene-pending-intent-hierarchy-in-tui-spec`
- Linear branch: `feature/riotbox-253-document-scene-pending-intent-hierarchy-in-tui-spec`
- PR: `#243`
- Merge commit: `4016f1b`
- Labels: `ux`
- Follow-ups: `RIOTBOX-254`

## Why This Ticket Existed

`RIOTBOX-252` styled the Scene pending intent line so target, boundary, and energy direction are scannable after `y` / `Y`. The TUI spec needed to record that pending Scene intent is part of the semantic hierarchy contract, not a diagnostic dump.

## What Shipped

- Documented pending Scene intent cues in `docs/specs/tui_screen_spec.md`.
- Added emphasis guidance for armed Scene verbs, target Scene ids, boundary labels, and energy direction.
- Preserved the monochrome-readable sentence contract.
- Kept the cue framed as performance intent rather than diagnostics.

## Verification

- `git diff --check main...HEAD`
- `rg -n 'pending Scene intent|energy rise|Scene ids|live gesture is armed' docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Spec-only slice; no runtime behavior, screenshot baselines, new color semantics, or broad TUI spec rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

# `RIOTBOX-174` Introduce first Jam footer color/emphasis tokens

- Ticket: `RIOTBOX-174`
- Title: `Introduce first Jam footer color/emphasis tokens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-174/introduce-first-jam-footer-coloremphasis-tokens`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-174-jam-footer-emphasis`
- PR: `#164`
- Merge commit: `e2731b9`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-175`

## Why This Ticket Existed

After the footer wording reductions, the Jam footer still had no visual hierarchy: primary controls, Scene timing, status, and warnings all read with the same visual weight.

## What Shipped

- Added styled footer line helpers for primary controls, active Scene cues, status diagnostics, clear-state text, and warnings.
- Added a focused unit test for the first footer visual hierarchy tokens.
- Documented the small semantic terminal palette in the TUI screen spec.

## Notes

- This changed visual emphasis only; footer text, layout, keymap, and audio/runtime behavior stayed unchanged.

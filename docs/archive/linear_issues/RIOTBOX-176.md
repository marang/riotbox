# `RIOTBOX-176` Clarify audible capture and audition feedback path from first-use notes

- Ticket: `RIOTBOX-176`
- Title: `Clarify audible capture and audition feedback path from first-use notes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-176/clarify-audible-capture-and-audition-feedback-path-from-first-use`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-176-capture-audition-clarity`
- PR: `#166`
- Merge commit: `a707e1d`
- Labels: `ux`, `TUI`
- Follow-ups: `Next near-term UX slice to keep first-use guidance moving`

## Why This Ticket Existed

First-use feedback made it clear that capture could still feel blind: Riotbox could store or promote material before the user-facing screen made the audible handoff obvious.

## What Shipped

- Added a compact `hear ...` line to the Capture screen so stored captures explicitly point to `[p]->[w]` before they are playable W-30 hits.
- Distinguished stored captures, W-30 pad targets, and scene targets so scene captures do not advertise W-30 audition keys.
- Updated the jam recipe notes and TUI spec to document the stored-capture-to-promotion-to-hit/audition path.

## Notes

- This clarified the existing UI contract only; it did not add a sampler editor, new DSP, Ghost behavior, or a new capture audio path.

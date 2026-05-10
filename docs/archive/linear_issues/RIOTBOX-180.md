# `RIOTBOX-180` Add a Capture help cue for the Do Next path

- Ticket: `RIOTBOX-180`
- Title: `Add a Capture help cue for the Do Next path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-180/add-a-capture-help-cue-for-the-do-next-path`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-180-capture-help-cue`
- PR: `#170`
- Merge commit: `2183a03`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-181`

## Why This Ticket Existed

The Capture screen had a clearer `Do Next` hierarchy, but the help overlay still did not teach that path directly.

## What Shipped

- Added a contextual `Capture path` block when the help overlay is opened from the Capture screen.
- Pointed the user at `Do Next`, the `hear ... stored` promotion rule, and Log confirmation.
- Added a focused shell rendering test for the Capture help block.
- Recorded the contextual-help rule in the TUI screen spec.

## Notes

- This changed help guidance only; it did not change the Capture screen layout, sampler behavior, or audio engine behavior.

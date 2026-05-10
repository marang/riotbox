# `RIOTBOX-178` Reduce Capture screen first-action density after audible handoff cue

- Ticket: `RIOTBOX-178`
- Title: `Reduce Capture screen first-action density after audible handoff cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-178/reduce-capture-screen-first-action-density-after-audible-handoff-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-178-capture-first-action-density`
- PR: `#168`
- Merge commit: `3798cf4`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-179`

## Why This Ticket Existed

After the audible capture handoff cue landed, the Capture screen still made the first useful action compete with provenance, pinned state, and routing internals.

## What Shipped

- Replaced the top-row Capture `Provenance` card with a `Do Next` card focused on capture, promote, hit, and audition steps.
- Kept provenance visible in the lower Capture row.
- Renamed routing details to `Advanced Routing` so diagnostics no longer read as the primary first-action path.
- Updated Capture snapshot tests plus recipe and TUI spec wording for the new hierarchy.

## Notes

- This was a presentation hierarchy change only; capture records, promotion semantics, W-30 behavior, and audio output stayed unchanged.

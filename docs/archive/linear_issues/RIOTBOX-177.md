# `RIOTBOX-177` Add a quantized timing rail for queued live gestures

- Ticket: `RIOTBOX-177`
- Title: `Add a quantized timing rail for queued live gestures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-177/add-a-quantized-timing-rail-for-queued-live-gestures`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-177-quantized-timing-rail`
- PR: `#167`
- Merge commit: `f085c5b`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-178`

## Why This Ticket Existed

First-use feedback asked what the "right musical time" means and how Riotbox waits for it. Scene-specific timing cues already existed, but the Jam screen did not give the same compact waiting cue for general queued live gestures.

## What Shipped

- Added a generic `wait [..>] next ...` timing rail to the Jam `Next` panel for the first queued action.
- Preserved the existing Scene-specific `pulse ...` line for jump/restore while covering non-Scene gestures such as MC-202 phrase actions.
- Added focused UI tests for Next-Bar and Next-Phrase rail rendering plus boundary-width helper coverage.
- Updated Jam recipes and the TUI screen spec to teach the compact timing rail.

## Notes

- This reused existing transport, queue, and quantization data only; no scheduler, audio callback, or commit semantics changed.

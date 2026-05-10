# `RIOTBOX-158` Surface queued scene energy direction in the compact footer cue

- Ticket: `RIOTBOX-158`
- Title: `Surface queued scene energy direction in the compact footer cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-158/surface-queued-scene-energy-direction-in-the-compact-footer-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-158-surface-queued-scene-energy-direction-in-the-compact-footer`
- PR: `#148`
- Merge commit: `c0c5d95`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-159`

## Why This Ticket Existed

The compact queued-scene footer cue showed pulse and trail context, but still hid whether the pending move meant a musical rise, drop, or hold.

## What Shipped

- Added a bounded `rise` / `drop` / `hold` hint to the compact queued Scene cue whenever the pending scene energy move could be derived.
- Kept the existing `pulse -> trail` reading path intact while making the queued footer more musically legible at a glance.

## Notes

- This tightened the default Jam footer without reopening the broader footer-density problem.

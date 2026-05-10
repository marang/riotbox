# `RIOTBOX-163` Tighten restore wake-up wording around the first landed jump

- Ticket: `RIOTBOX-163`
- Title: `Tighten restore wake-up wording around the first landed jump`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-163/tighten-restore-wake-up-wording-around-the-first-landed-jump`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-163-tighten-restore-wake-up-wording-around-the-first-landed-jump`
- PR: `#153`
- Merge commit: `ad233da`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-164`, `RIOTBOX-165`

## Why This Ticket Existed

The restore-ready seam was already explicit, but the earlier wake-up-only state still sounded too active and too close to the ready state.

## What Shipped

- Reworded the wake-up-only restore guidance in the Jam suggestions and Help overlay to `waits for one landed jump`.
- Refreshed the paired shell assertions so the wait-state and ready-state stayed clearly distinct.

## Notes

- This sharpened the negative side of the restore state machine without touching any queue or commit semantics.

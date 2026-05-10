# `RIOTBOX-171` Add first compact visual timing tick for Scene Brain queued actions

- Ticket: `RIOTBOX-171`
- Title: `Add first compact visual timing tick for Scene Brain queued actions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-171/add-first-compact-visual-timing-tick-for-scene-brain-queued-actions`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-171-scene-visual-tick`
- PR: `#161`
- Merge commit: `987b101`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-172`

## Why This Ticket Existed

The Scene Brain cue ladder was readable but still text-heavy; the queued footer cue needed a first small timing shape without opening a larger timing widget.

## What Shipped

- Reused the existing ASCII countdown marker in the compact queued Scene footer cue.
- Updated focused shell assertions and the current Scene Brain benchmark quotes.

## Notes

- This kept the existing text cue but replaced the word `pulse` with the actual compact tick marker.

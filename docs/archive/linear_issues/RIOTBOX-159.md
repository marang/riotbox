# `RIOTBOX-159` Tighten Scene Brain restore-ready guidance around the active restore target

- Ticket: `RIOTBOX-159`
- Title: `Tighten Scene Brain restore-ready guidance around the active restore target`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-159/tighten-scene-brain-restore-ready-guidance-around-the-active-restore`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-159-tighten-scene-brain-restore-ready-guidance-around-the-active`
- PR: `#149`
- Merge commit: `1cf60d3`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-160`, `RIOTBOX-161`

## Why This Ticket Existed

The restore-ready copy still used a generic “bring it back” phrase even though the shell already knew which concrete scene the restore target pointed to.

## What Shipped

- Tightened the restore-ready cue so the player could read the active restore target directly from the default Jam shell.
- Updated the matching help and recipe wording so restore-ready guidance named the same target more explicitly.

## Notes

- This stayed intentionally copy-only and did not add any new restore behavior.

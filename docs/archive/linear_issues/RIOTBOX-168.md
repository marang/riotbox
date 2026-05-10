# `RIOTBOX-168` Carry scene/energy labels into the post-landed Scene Brain cue

- Ticket: `RIOTBOX-168`
- Title: `Carry scene/energy labels into the post-landed Scene Brain cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-168/carry-sceneenergy-labels-into-the-post-landed-scene-brain-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-168-carry-sceneenergy-labels-into-the-post-landed-scene-brain`
- PR: `#158`
- Merge commit: `0cdc17f`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-169`

## Why This Ticket Existed

The restore-ready and live/restore contrast cues already used compact `scene/energy` labels, but the post-landed Scene cue still fell back to scene-only names.

## What Shipped

- Updated the post-landed Scene cue to use the same compact `scene/energy` labels where the target labels are already derivable.
- Kept the existing `next [Y]` / `[c] capture` guidance and refreshed focused shell assertions around the wrapped cue text.

## Notes

- This keeps the immediate after-landing cue aligned with the rest of the current Scene Brain readability language.

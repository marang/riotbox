# `RIOTBOX-173` Reduce Scene Brain footer tick wording density after readability baselines

- Ticket: `RIOTBOX-173`
- Title: `Reduce Scene Brain footer tick wording density after readability baselines`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-173/reduce-scene-brain-footer-tick-wording-density-after-readability`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-173-scene-footer-density`
- PR: `#163`
- Merge commit: `5af61ca`
- Labels: `ux`, `TUI`
- Follow-ups: `None`

## Why This Ticket Existed

After the Scene footer gained a compact timing tick, the queued line still carried extra wording and punctuation that made the performance footer denser than necessary.

## What Shipped

- Shortened the Jam footer prefix from `Scene cue:` to `Scene:`.
- Reduced queued Scene timing copy to `rise [===>] | 2 trail` while keeping intent, boundary, tick, and trail cues.
- Updated focused shell assertions, benchmark baselines, jam recipes, and the portable skill-path workflow note.

## Notes

- This changed wording only; no Scene behavior, timing model, or audio runtime path changed.

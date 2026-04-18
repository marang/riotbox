# `RIOTBOX-131` Add one compact scene-specific help cue for queued jump and restore actions

- Ticket: `RIOTBOX-131`
- Title: `Add one compact scene-specific help cue for queued jump and restore actions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-131/add-one-compact-scene-specific-help-cue-for-queued-jump-and-restore`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-131-scene-help-cue`
- Linear branch: `feature/riotbox-131-add-one-compact-scene-specific-help-cue-for-queued-jump-and`
- Assignee: `Markus`
- Labels: `None`
- PR: `#124`
- Merge commit: `1baecc465d88225e3db701259d2eb9a4cc693a0d`
- Deleted from Linear: `Not deleted`
- Verification: `just ci`, branch diff review, GitHub Actions `Rust CI` run `#340`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-133`, `RIOTBOX-134`

## Why This Ticket Existed

Queued scene timing had become legible in the shell itself, but the help overlay still did not point players at that seam. Riotbox needed one bounded help follow-up that explains what to read on `Jam` and where to confirm the result without turning help into a second diagnostics surface.

## What Shipped

- added a compact `Scene timing` block to the help overlay when a queued scene jump or restore is pending
- pointed the player at the existing `launch/restore`, `pulse`, and `live <> restore` cues on `Jam`
- pointed the player at `Log` as the place to confirm the landed `trail ...` result
- added focused UI tests for pending scene jump and restore help states

## Notes

- this slice changed the help surface only; it did not add new scene semantics or timing widgets
- the help cue stays conditional so it only appears when the scene seam is actually relevant

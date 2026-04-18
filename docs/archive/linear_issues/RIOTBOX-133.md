# `RIOTBOX-133` Add one compact queued-scene footer cue on Jam when jump or restore is pending

- Ticket: `RIOTBOX-133`
- Title: `Add one compact queued-scene footer cue on Jam when jump or restore is pending`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-133/add-one-compact-queued-scene-footer-cue-on-jam-when-jump-or-restore-is`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-133-scene-footer-cue`
- Linear branch: `feature/riotbox-133-add-one-compact-queued-scene-footer-cue-on-jam-when-jump-or`
- Assignee: `Markus`
- Labels: `None`
- PR: `#125`
- Merge commit: `b18f7d9ea760e8c679960e610912d7466ef11ec7`
- Deleted from Linear: `Not deleted`
- Verification: `just ci`, branch diff review, GitHub Actions `Rust CI` run `#342`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-134`, `RIOTBOX-135`

## Why This Ticket Existed

The queued-scene seam had become readable in `Jam` and explainable in `Help`, but the default footer still stayed silent about it. Riotbox needed one tiny always-visible reminder that only appears when a scene jump or restore is pending, without turning the footer into a second diagnostics surface.

## What Shipped

- replaced the normal footer `Advanced:` row with a compact queued-scene cue while `jump` or `restore` is pending
- kept the cue on an already visible footer line instead of adding a new footer block
- added focused snapshot expectations for pending scene jump and restore shell states

## Notes

- the cue is intentionally terse and reminder-like, not a full explanation
- this slice changed shell guidance only; it did not alter Scene Brain semantics or timing behavior

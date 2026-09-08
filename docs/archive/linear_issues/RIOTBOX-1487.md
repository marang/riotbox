# `RIOTBOX-1487` [P1] Preserve the last loadable Session/Source Graph pair after a failed save

- Ticket: `RIOTBOX-1487`
- Title: `[P1] Preserve the last loadable Session/Source Graph pair after a failed save`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1487/p1-preserve-the-last-loadable-sessionsource-graph-pair-after-a-failed`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-05`
- Started: `2026-09-08`
- Finished: `2026-09-08`
- Branch: `feature/riotbox-1487-safe-session-graph-save`
- Linear branch: `feature/riotbox-1487-p1-preserve-the-last-loadable-sessionsource-graph-pair-after`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `review-followup`
- PR: `#1508 (https://github.com/marang/riotbox/pull/1508)`
- Merge commit: `c7f98f361d8bf28555d10850a3836d929d9f0698`
- Deleted from Linear: `2026-09-08`
- Verification: `All local CI stages passed across staged runs; independent review and GitHub rust-ci passed.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1488, RIOTBOX-1489, RIOTBOX-1490 complete the requested stability round; RIOTBOX-1491 remains deferred.`

## Why This Ticket Existed

A failed external Graph/Session save could invalidate the last loadable Session.

## What Shipped

- Preserved immutable exact-hash Graph generations before alias replacement and Session publication.
- Added exact-generation restore, Session-only integrity checks, collision guards, and failure regressions.

## Notes

- Single writer and ordinary I/O/process-interruption guarantee only; no power-loss or multiwriter claim. One unrelated sidecar startup test passed on isolated retry.

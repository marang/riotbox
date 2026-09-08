# `RIOTBOX-1488` [P2] Normalize external Graph references relative to the Session directory

- Ticket: `RIOTBOX-1488`
- Title: `[P2] Normalize external Graph references relative to the Session directory`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1488/p2-normalize-external-graph-references-relative-to-the-session`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-05`
- Started: `2026-09-08`
- Finished: `2026-09-08`
- Branch: `feature/riotbox-1488-session-relative-graph-paths`
- Linear branch: `feature/riotbox-1488-p2-normalize-external-graph-references-relative-to-the`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `review-followup`
- PR: `#1510 (https://github.com/marang/riotbox/pull/1510)`
- Merge commit: `d0c84e57fe5bef755b76db5bdc26d651ca0236bf`
- Deleted from Linear: `2026-09-08`
- Verification: `31 combined Graph/persistence tests, full combined just ci, independent review and GitHub rust-ci passed.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1489 completes source integrity; RIOTBOX-1491 remains deferred.`

## Why This Ticket Existed

Cwd-relative external Graph paths were later resolved relative to the Session, breaking reload without an override.

## What Shipped

- Anchored runtime destinations and stored Graph references relative to the Session directory.
- Added explicit hash-checked legacy repair, symlink-parent and changed-cwd restore regressions.

## Notes

- RBX-369 preserves absolute references and rejects guessed automatic legacy repair.

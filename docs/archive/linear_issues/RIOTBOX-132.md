# `RIOTBOX-132` Archive RIOTBOX-128 through RIOTBOX-131 after the current Scene Brain docs/help batch

- Ticket: `RIOTBOX-132`
- Title: `Archive RIOTBOX-128 through RIOTBOX-131 after the current Scene Brain docs/help batch`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-132/archive-riotbox-128-through-riotbox-131-after-the-current-scene-brain`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-132-archive-scene-guidance-batch`
- Linear branch: `feature/riotbox-132-archive-riotbox-128-through-riotbox-131-after-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#127`
- Merge commit: `8d837375e973f37933de8f2571d0d72d15963ca8`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#346`
- Docs touched: `docs/archive/linear_issues/index.md`, `docs/archive/linear_issues/RIOTBOX-128.md`, `docs/archive/linear_issues/RIOTBOX-129.md`, `docs/archive/linear_issues/RIOTBOX-130.md`, `docs/archive/linear_issues/RIOTBOX-131.md`
- Follow-ups: `RIOTBOX-135`, `RIOTBOX-136`

## Why This Ticket Existed

The first Scene Brain docs/help batch had already been merged and archived in repo history, but the archive-follow-up ticket itself was still only living in active Linear. Riotbox needed one more tiny repo-ops pass so that even the archive catch-up work remains deletable from Linear without losing historical traceability.

## What Shipped

- added the repo archive entry for `RIOTBOX-132`
- preserved the PR link, merge commit, verification, and project mapping for the archive-follow-up slice itself
- updated the archive index so the repo-ops chain remains complete and searchable

## Notes

- this was a meta archive slice only; it did not change product or runtime behavior
- the follow-up is intentionally small but keeps the archive trail consistent instead of leaving recursive repo-ops tickets undocumented

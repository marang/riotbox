# `RIOTBOX-128` Archive RIOTBOX-122 through RIOTBOX-127 after scene readability and workflow slices

- Ticket: `RIOTBOX-128`
- Title: `Archive RIOTBOX-122 through RIOTBOX-127 after scene readability and workflow slices`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-128/archive-riotbox-122-through-riotbox-127-after-scene-readability-and`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-128-archive-scene-readability-batch`
- Linear branch: `feature/riotbox-128-archive-riotbox-122-through-riotbox-127-after-scene`
- Assignee: `Markus`
- Labels: `None`
- PR: `#122`
- Merge commit: `74bd1903964ed8f8c97ebcf1bb873fe2bf02a50f`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#336`
- Docs touched: `docs/archive/linear_issues/index.md`, `docs/archive/linear_issues/RIOTBOX-122.md`, `docs/archive/linear_issues/RIOTBOX-123.md`, `docs/archive/linear_issues/RIOTBOX-124.md`, `docs/archive/linear_issues/RIOTBOX-126.md`, `docs/archive/linear_issues/RIOTBOX-127.md`
- Follow-ups: `RIOTBOX-132`

## Why This Ticket Existed

The recent Scene Brain readability and workflow-cleanup tickets had already landed on `main`, but they were still only preserved in active Linear. Riotbox needed one bounded repo-ops slice that moved those finished tickets into the repo archive before later free-tier cleanup.

## What Shipped

- added archive markdown entries for `RIOTBOX-122`, `RIOTBOX-123`, `RIOTBOX-124`, `RIOTBOX-126`, and `RIOTBOX-127`
- recorded project mapping, PR links, merge commits, and verification for each archived ticket
- updated the archive index so the newly archived readability and workflow slices stay searchable in repo history

## Notes

- this was a repo-history preservation slice only; it did not change product or runtime behavior
- the batch also served as the first archive follow-up after the shift to numbered `P00X | ...` Linear projects

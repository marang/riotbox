# `RIOTBOX-753` Document context-light search and branch cleanup workflow

- Ticket: `RIOTBOX-753`
- Title: `Document context-light search and branch cleanup workflow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-753/document-context-light-search-and-branch-cleanup-workflow`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-753-context-light-workflow`
- Linear branch: `feature/riotbox-753-document-context-light-search-and-branch-cleanup-workflow`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#746 (https://github.com/marang/riotbox/pull/746)`
- Merge commit: `429d1a8b5018c3e2e44fc7da31a4ecd91d6082ff`
- Deleted from Linear: `2026-05-10`
- Verification: `git diff --check`; default `rg` skips archived tickets; explicit `rg --no-ignore` still finds archived tickets; April archive split preserved 340 ticket headings and 340 ticket metadata entries.
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`, `docs/archive/linear_issues/README.md`, `docs/archive/linear_issues/index.md`
- Follow-ups: `None`

## Why This Ticket Existed

Riotbox needed a durable workflow rule that keeps routine agent searches from loading old Linear archives, raw planning transcripts, generated artifacts, or local audio data into context by accident.

The same slice also captured the new closeout rule that merged feature branches should be deleted after the PR is merged and local `main` is synced.

## What Shipped

- Added `.rgignore` for default context-light searches.
- Added context-hygiene rules to `AGENTS.md` and `docs/workflow_conventions.md`.
- Shortened the command shortlist in `AGENTS.md` and pointed agents to `just --list` / `Justfile` for the full catalog.
- Documented explicit `rg --no-ignore` archive searches.
- Split the oversized April 2026 Linear archive into smaller ticket-number range files and left `2026-04.md` as a short month index.

## Notes

- The split uses ticket-number buckets, not line-count buckets.
- Default searches now skip `docs/archive/linear_issues/`; archive history remains available through explicit searches.

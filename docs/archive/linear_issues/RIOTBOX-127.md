# `RIOTBOX-127` Refresh AGENTS current-state snapshot and point to live docs

- Ticket: `RIOTBOX-127`
- Title: `Refresh AGENTS current-state snapshot and point to live docs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-127/refresh-agents-current-state-snapshot-and-point-to-live-docs`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-127-agents-current-state`
- Linear branch: `feature/riotbox-127-refresh-agents-current-state-snapshot-and-point-to-live-docs`
- Assignee: `Markus`
- Labels: `None`
- PR: `#119`
- Merge commit: `bbec42541935641e33990041356769cb46b79e97`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#328`
- Docs touched: `AGENTS.md`
- Follow-ups: `None`

## Why This Ticket Existed

`AGENTS.md` still described Riotbox as if only `riotbox-core` and a minimal Jam view existed. That stale snapshot was starting to mislead implementation agents about what was already real in the repo, so Riotbox needed one small cleanup pass that kept `AGENTS.md` as an operating brief instead of a drifting changelog.

## What Shipped

- replaced the stale `Current State` snapshot in `AGENTS.md` with a shorter high-level summary of the real workspace and product spine
- pointed detailed status tracking toward the live docs that are updated more often
- kept the file focused on operating rules and source-of-truth references instead of stale implementation inventory

## Notes

- this was a docs-only repo-ops slice with no runtime or shell changes
- the refresh intentionally favors high-level accuracy over an ever-growing implementation checklist inside `AGENTS.md`

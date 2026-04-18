# `RIOTBOX-117` Align archived ticket markdowns with phase-project roadmap structure

- Ticket: `RIOTBOX-117`
- Title: `Align archived ticket markdowns with phase-project roadmap structure`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-117/align-archived-ticket-markdowns-with-phase-project-roadmap-structure`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-117-archive-phase-projects`
- Linear branch: `feature/riotbox-117-align-archived-ticket-markdowns-with-phase-project-roadmap`
- Assignee: `Markus`
- Labels: `None`
- PR: `#110`
- Merge commit: `06a2fe11baba330c8fbdee6d21af9a7a0ccbfb9f`
- Deleted from Linear: `Not deleted`
- Verification: branch diff review, `git diff --check -- docs/archive/linear_issues`, GitHub Actions `Rust CI` run `#313`
- Docs touched: `docs/archive/linear_issues/README.md`, `docs/archive/linear_issues/TEMPLATE.md`, `docs/archive/linear_issues/index.md`
- Follow-ups: `RIOTBOX-125`

## Why This Ticket Existed

Linear’s roadmap structure had moved from one milestone-heavy legacy container to numbered phase projects, but the repo archive still described completed work under the old `Riotbox MVP Buildout` project label. The archive needed to match the new phase-project structure so deleted Linear tickets would remain historically honest and searchable.

## What Shipped

- remapped archived ticket markdowns so their `Project` field now points to the new `P00X | ...` phase-project structure
- kept milestone or legacy phase context as a separate archive field instead of overloading the project field
- updated the archive README and template so future ticket archives follow the new phase-project meaning consistently

## Notes

- this was a repo-ops slice only; it changed archive metadata, not product behavior
- the old Linear milestone container was already superseded by phase projects, so this change was about historical clarity rather than roadmap semantics

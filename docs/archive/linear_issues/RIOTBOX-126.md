# `RIOTBOX-126` Codify non-blocking PR gate handling in repo workflow

- Ticket: `RIOTBOX-126`
- Title: `Codify non-blocking PR gate handling in repo workflow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-126/codify-non-blocking-pr-gate-handling-in-repo-workflow`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-126-non-blocking-pr-gates`
- Linear branch: `feature/riotbox-126-codify-non-blocking-pr-gate-handling-in-repo-workflow`
- Assignee: `Markus`
- Labels: `None`
- PR: `#116`
- Merge commit: `d5f367b2b394be4189eea4b33395f8e43a058a49`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, branch diff review, GitHub Actions `Rust CI` run `#325`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

The repo had already settled into a working pattern where open or green PRs should not stall the main implementation lane, but that rule still lived mostly in practice rather than in the written workflow. Riotbox needed one bounded repo-ops slice that made the non-blocking PR rule explicit.

## What Shipped

- added an explicit workflow rule that PRs and in-flight CI runs are merge gates, not reasons to idle roadmap work
- documented that PR state should be checked periodically because the current tooling has polling, not event-driven notifications
- aligned the main agent operating brief and workflow conventions around the same non-blocking implementation rule

## Notes

- this was a docs-only workflow slice; it did not change product behavior
- the rule is intentionally pragmatic: keep implementing bounded next slices while periodically rechecking merge gates

# `RIOTBOX-757` Add Linear archive generator helper

- Ticket: `RIOTBOX-757`
- Title: `Add Linear archive generator helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-757/add-linear-archive-generator-helper`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-757-add-linear-archive-generator-helper`
- Linear branch: `feature/riotbox-757-add-linear-archive-generator-helper`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#751 (https://github.com/marang/riotbox/pull/751)`
- Merge commit: `48f8e44a9c26e50ef9f7bbf8d4027fa064310e41`
- Deleted from Linear: `2026-05-10`
- Verification: `scripts/archive_linear_issue_smoke.sh`; `scripts/closeout_ticket_smoke.sh`; `git diff --check HEAD~1..HEAD`; `just ci`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`, `docs/archive/linear_issues/README.md`
- Follow-ups: `None`

## Why This Ticket Existed

Ticket closeout still required manually creating the per-ticket archive markdown and updating indexes before the existing closeout helper could safely delete Linear issues and merged branches.

## What Shipped

- Added scripts/archive_linear_issue.py to fetch Linear metadata, optional GitHub PR metadata, and write per-ticket archive markdown.
- Updated monthly and root archive indexes from the generator.
- Added a network-free archive generator smoke check.
- Hardened closeout_ticket.sh so generator TODO placeholders are not accepted as closeout-ready handoffs.

## Notes

- None

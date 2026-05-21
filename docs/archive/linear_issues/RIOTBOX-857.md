# `RIOTBOX-857` Clarify automatic next-ticket continuation in workflow conventions

- Ticket: `RIOTBOX-857`
- Title: `Clarify automatic next-ticket continuation in workflow conventions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-857/clarify-automatic-next-ticket-continuation-in-workflow-conventions`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-857-clarify-automatic-next-ticket-continuation`
- Linear branch: `feature/riotbox-857-clarify-automatic-next-ticket-continuation-in-workflow`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#852 (https://github.com/marang/riotbox/pull/852)`
- Merge commit: `083518dc0b713e24d2405e29c375537f9b1ce8cb`
- Verification: `git diff --check; GitHub Actions Rust CI passed for PR #852; main synced after merge.`
- Docs touched: `docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

Workflow docs contained stale short-form wait-for-merge wording that contradicted the active continuation rule.

## What Shipped

- Updated docs/workflow_conventions.md so merge remains the closeout boundary but clean PRs do not pause the main implementation lane; strengthened automatic next-ticket continuation wording; fixed Linear subsection numbering.

## Notes

- None

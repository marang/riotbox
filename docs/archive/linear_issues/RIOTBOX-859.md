# `RIOTBOX-859` Consolidate autonomous workflow documentation

- Ticket: `RIOTBOX-859`
- Title: `Consolidate autonomous workflow documentation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-859/consolidate-autonomous-workflow-documentation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-859-consolidate-autonomous-workflow-documentation`
- Linear branch: `feature/riotbox-859-consolidate-autonomous-workflow-documentation`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#853 (https://github.com/marang/riotbox/pull/853)`
- Merge commit: `09d44130792115dfd052f80d8e36571e94b17181`
- Verification: `git diff --check; search checks for stale wait/duplicate headings; GitHub Actions Rust CI passed for PR #853; main synced after merge.`
- Docs touched: `AGENTS.md, docs/workflow_conventions.md, docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Workflow rules were duplicated across AGENTS.md, docs/workflow_conventions.md, and roadmap wording, which had allowed stale continuation guidance to survive.

## What Shipped

- Made docs/workflow_conventions.md the canonical operational workflow source; reduced AGENTS.md to hard guardrails and a canonical pointer; clarified roadmap product-loop ownership; marked the workflow short version as a reminder rather than a second source of truth; preserved do-not-amend guidance in the canonical workflow doc.

## Notes

- None

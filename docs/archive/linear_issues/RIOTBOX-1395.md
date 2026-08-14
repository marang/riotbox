# `RIOTBOX-1395` Workflow: Archive and delete old Done Linear tickets

- Ticket: `RIOTBOX-1395`
- Title: `Workflow: Archive and delete old Done Linear tickets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1395/workflow-archive-and-delete-old-done-linear-tickets`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-08-14`
- Finished: `2026-08-14`
- Branch: `None; direct admin cleanup on main per ticket scope`
- Linear branch: `feature/riotbox-1395-workflow-archive-and-delete-old-done-linear-tickets`
- Assignee: `Markus`
- Labels: `workflow`
- PR: None
- Merge commit: `9994147e40b870d9956f48057a791dbeb64894a4`
- Deleted from Linear: `2026-08-14`
- Verification: `archive helper smoke; exact per-ticket metadata/index checks; git diff --check; active Done/Canceled counts both zero`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

## Why

Several older Done tickets still remain visible in Linear. This creates noise and makes active P023 / product-quality work harder to scan.

## Scope

* Audit old Done Riotbox Linear issues that are already shipped / merged.
* For each issue, ensure the repo archive exists under `docs/archive/linear_issues/RIOTBOX-123.md` and month/index files are updated.
* Delete archived Done issues from Linear using the existing workflow scripts.
* Do not mix this admin cleanup with product implementation branches.

## Acceptance

* Old Done ticket backlog is reduced in a deliberate admin block.
* No shipped context is lost from repo archives.
* Active product tickets remain visible and uncluttered.

## What Shipped

- Added and validated the missing durable repo archives for terminal Riotbox tickets.
- Removed 52 active Done tickets and 3 active Canceled tickets through the token-authenticated Linear issueDelete path.
- Preserved all active and backlog issues while reducing the active terminal-ticket count to zero.

## Notes

- Administrative cleanup only; no product, audio, source, runtime, schema, threshold, or algorithm behavior changed.

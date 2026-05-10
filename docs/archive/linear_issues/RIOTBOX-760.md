# `RIOTBOX-760` Fix closeout optional timeout status reporting

- Ticket: `RIOTBOX-760`
- Title: `Fix closeout optional timeout status reporting`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-760/fix-closeout-optional-timeout-status-reporting`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-760-fix-closeout-optional-timeout-status-reporting`
- Linear branch: `feature/riotbox-760-fix-closeout-optional-timeout-status-reporting`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#754 (https://github.com/marang/riotbox/pull/754)`
- Merge commit: `d8b7d9ef70baa388b9ab17e75a38a4949989aa5b`
- Deleted from Linear: `2026-05-10`
- Verification: `scripts/closeout_ticket_smoke.sh`; `bash -n scripts/closeout_ticket.sh scripts/closeout_ticket_smoke.sh`; `git diff --check`; `scripts/archive_linear_issue_smoke.sh`; `GitHub Rust CI success on PR #754`
- Docs touched: `scripts/closeout_ticket.sh`, `scripts/closeout_ticket_smoke.sh`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-759 bounded optional MemPalace status, but real output showed a wrong status-0 diagnostic because Bash captured the status after the timeout if statement instead of from the command itself.

## What Shipped

- Captured optional command exit status directly from timeout or command execution.
- Preserved non-blocking closeout behavior for optional MemPalace timeout or failure.
- Added CLOSEOUT_MEM_STATUS_COMMAND test override and smoke coverage for the timeout diagnostic.

## Notes

- None

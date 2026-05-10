# `RIOTBOX-759` Bound optional MemPalace status during ticket closeout

- Ticket: `RIOTBOX-759`
- Title: `Bound optional MemPalace status during ticket closeout`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-759/bound-optional-mempalace-status-during-ticket-closeout`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-759-bound-optional-mempalace-status-during-ticket-closeout`
- Linear branch: `feature/riotbox-759-bound-optional-mempalace-status-during-ticket-closeout`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#753 (https://github.com/marang/riotbox/pull/753)`
- Merge commit: `f244cb2b632ba224d15e1d6ecd183b8ba08ddf08`
- Deleted from Linear: `2026-05-10`
- Verification: `scripts/closeout_ticket_smoke.sh`; `bash -n scripts/closeout_ticket.sh scripts/closeout_ticket_smoke.sh`; `git diff --check`; `scripts/archive_linear_issue_smoke.sh`; `GitHub Rust CI success on PR #753`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

Real ticket closeout showed that optional MemPalace status could block cleanup for minutes while re-mining, even after archive handoff, branch deletion, and Linear deletion had already succeeded.

## What Shipped

- Added --mem-status-timeout to scripts/closeout_ticket.sh with a 120-second default.
- Made optional MemPalace status timeout or failure non-blocking for closeout completion.
- Documented that MemPalace status is optional maintenance and must not block cleanup.
- Extended closeout smoke coverage for the bounded mem-status dry-run path.

## Notes

- None

# `RIOTBOX-756` Add repo closeout helper for archive, Linear deletion, and branch cleanup

- Ticket: `RIOTBOX-756`
- Title: `Add repo closeout helper for archive, Linear deletion, and branch cleanup`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-756/add-repo-closeout-helper-for-archive-linear-deletion-and-branch`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-756-closeout-helper`
- Linear branch: `feature/riotbox-756-add-repo-closeout-helper-for-archive-linear-deletion-and`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#750 (https://github.com/marang/riotbox/pull/750)`
- Merge commit: `6d5e8c2c339dff1976a38b6fdf6a63ccf570c134`
- Deleted from Linear: `2026-05-10`
- Verification: `bash -n scripts/closeout_ticket.sh scripts/closeout_ticket_smoke.sh`; `scripts/closeout_ticket_smoke.sh`; dry-run closeout against `RIOTBOX-755`; `git diff --check`; GitHub Rust CI success on PR #750.
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

Ticket closeout repeatedly required the same manual cleanup: verify the repo archive handoff, delete the completed Linear issue, delete merged feature branches, and optionally refresh MemPalace status. Repeating that manually was error-prone and consumed agent context.

## What Shipped

- Added `scripts/closeout_ticket.sh` as a dry-run-first closeout helper.
- Required exact per-ticket archive file, ticket metadata, and archive index entry before cleanup actions.
- Reused `scripts/linear_issue_delete.sh` for Linear deletion instead of duplicating GraphQL.
- Required `--pr` and GitHub merged-PR verification before executing branch deletion.
- Added `scripts/closeout_ticket_smoke.sh` to prove missing archive handoff fails and dry-run closeout remains non-mutating.
- Documented the helper in `AGENTS.md` and `docs/workflow_conventions.md`.

## Notes

- `shellcheck` was not installed locally, so validation used `bash -n`, smoke coverage, dry-run coverage, and CI.
- MemPalace stays optional status/maintenance, not the canonical archive or deletion gate.

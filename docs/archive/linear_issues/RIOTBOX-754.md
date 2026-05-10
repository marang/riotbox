# `RIOTBOX-754` Use one archive file per Linear ticket

- Ticket: `RIOTBOX-754`
- Title: `Use one archive file per Linear ticket`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-754/use-one-archive-file-per-linear-ticket`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-754-one-archive-file-per-ticket`
- Linear branch: `feature/riotbox-754-use-one-archive-file-per-linear-ticket`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#747 (https://github.com/marang/riotbox/pull/747)`
- Merge commit: `8753d6455b3d2e294ffbbd302848bcd0d754e669`
- Deleted from Linear: `2026-05-10`
- Verification: `git diff --check`; month indexes contain no grouped ticket headings/table rows; April split exposes 463 exact ticket metadata entries; May split exposes 273 exact ticket metadata entries; exact file checks work for `RIOTBOX-475.md` and `RIOTBOX-619.md`.
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`, `docs/archive/linear_issues/README.md`, `docs/archive/linear_issues/index.md`, `docs/archive/linear_issues/2026-04.md`, `docs/archive/linear_issues/2026-05.md`
- Follow-ups: `None`

## Why This Ticket Existed

The Linear archive still had grouped April and May archive files. That made deletion checks less direct and encouraged broad archive reads when only exact ticket handoff proof was needed.

## What Shipped

- Changed archive rules to one file per Linear ticket.
- Split grouped April and May archive entries into individual `RIOTBOX-*.md` files.
- Kept `2026-04.md` and `2026-05.md` as month indexes only.
- Documented exact deletion checks using file existence or `- Ticket:` metadata.

## Notes

- MemPalace remains focused on live docs/specs/reviews rather than old Linear ticket history.
- Completed Linear deletion checks can now use exact file presence for all split archived tickets.

# Archive And Cleanup

Parent: [Riotbox Workflow Conventions](../workflow_conventions.md)

Use after merge/cancel for retention, deletion, and branch cleanup. The parent
owns closeout order; [Linear Lifecycle](./linear_lifecycle.md) owns issue state,
and [GitHub, PR, Review, And CI](./github_pr_ci.md) owns merge and local sync.

## Retention Model

Linear is the active execution surface, not the canonical long-term archive.

Because the workspace runs on the free tier, completed issues should not
accumulate forever.

Keep active work, the near backlog, and recent operational history in Linear;
keep durable ticket history in the repo archive. Riotbox has no active semantic
memory: `just decision-search "query"` is bounded exact/term search. Any future
semantic layer requires its own measured decision and must exclude the archive.

Before deleting a completed Linear issue, preserve its useful context in repo
markdown under `docs/archive/linear_issues/`. Do that archive update as part of
the normal ticket closeout path, not as a separate default `Archive ...` ticket.

## Archive Shape

Recommended archive shapes:

- one file per ticket for all archived Linear tickets
- monthly files such as `2026-05.md` are indexes, not grouped content archives

Naming and formatting rules:

- use `RIOTBOX-123.md` for one-ticket archive files
- use `YYYY-MM.md` for monthly index files
- use ISO dates in all metadata fields: `YYYY-MM-DD`
- keep the metadata block field order consistent with the archive template
- use stable final-status values such as:
  - `Done`
  - `Canceled`
  - `Duplicate`
  - `Superseded`

Each archived ticket entry should include at least:

- ticket id and title
- Linear project
- phase or milestone
- final status such as done, canceled, duplicate, or superseded
- created date
- implementation start date when known
- final status date such as merged, done, canceled, or deleted
- actual repo feature branch when one existed
- status date or merge point
- why the ticket existed
- what shipped
- PR link
- merge commit
- follow-up tickets or bounded open questions

Use the canonical [archive template](../archive/linear_issues/TEMPLATE.md) for
field order and optional metadata instead of copying a second template here.

## Linear Deletion Gate

For shipped work, do not delete a Linear ticket until:

- the PR is merged
- the issue is marked done
- the repo archive entry exists

For non-shipping terminal work (`Canceled`, `Duplicate`, or `Superseded`), a
merged PR is not required. Delete only after:

- the terminal status and reason are recorded in the repo archive
- the Linear issue is already in its non-shipping terminal state
- no open PR targets the ticket branch
- no active worktree or implementation branch still owns the ticket
- any intentionally retained branch is named and justified in the archive

Verify archive presence by exact file or metadata check, not by reading or
semantically searching the whole archive:

```bash
test -f docs/archive/linear_issues/RIOTBOX-123.md
rg --no-ignore -n '^- Ticket: `RIOTBOX-123`' docs/archive/linear_issues
```

Do not use semantic memory as the deletion gate; exact filesystem / metadata
checks are more reliable for cleanup decisions.

## Archive And Closeout Helpers

When generating the archive entry, prefer the repo-local helper:

```bash
scripts/archive_linear_issue.py --ticket RIOTBOX-123 --pr 99 --why "..." --shipped "..."
```

The helper fetches Linear metadata and optional GitHub PR metadata, writes
`docs/archive/linear_issues/RIOTBOX-123.md`, and updates the monthly and root
archive indexes. It requires explicit `--why` and at least one `--shipped`
entry unless `--allow-placeholders` is intentionally used for a draft.
Placeholder drafts are not valid closeout handoffs.

When deleting, prefer the repo-local helper:

```bash
scripts/linear_issue_delete.sh RIOTBOX-123
```

For repeated cleanup, prefer the repo-local closeout helper:

```bash
scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99
```

The archive and closeout helpers default to dry-run; execute them only when the
branch is ready for the corresponding mutation. The closeout helper's shipping
path must only be executed after the PR is merged, the issue is marked done,
and the archive handoff exists:

```bash
scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99 --delete-linear --delete-remote-branch --delete-local-branch --execute
```

The helper should use token auth via `LINEAR_API_TOKEN`. Do not treat pasted
browser session cookies as the normal cleanup path.

## GitHub Branch Cleanup

After a PR is merged and local `main` is synced, delete both the remote and
local feature branches unless they are intentionally long-lived. Do branch
cleanup as part of ticket closeout, alongside the Linear archive/delete path.

Prefer deleting the exact branch used by the merged PR:

```bash
git push origin --delete feature/riotbox-123-example
git branch -d feature/riotbox-123-example
```

If a squash merge prevents safe `git branch -d`, use the closeout helper's
verified local-branch deletion path after confirming the exact PR merge and
archive handoff; do not infer merge state from ancestry alone.

Safety rules:

- never delete `main`, release branches, protected branches, or an active branch with an open PR
- for squash-merged PRs, do not rely only on `git branch --merged`; squash merges can leave branch tips outside `main` even when the PR content is already merged
- if doing a bulk cleanup, first verify there are no open PRs and then delete only stale non-`main` heads that are known merged, archived, or otherwise obsolete

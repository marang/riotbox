# `RIOTBOX-1090` Workflow: Make Linear-first priority unambiguous

- Ticket: `RIOTBOX-1090`
- Title: `Workflow: Make Linear-first priority unambiguous`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1090/workflow-make-linear-first-priority-unambiguous`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1090-linear-first-workflow-priority`
- Linear branch: `feature/riotbox-1090-workflow-make-linear-first-priority-unambiguous`
- Assignee: `Markus`
- Labels: None
- PR: `#1066 (https://github.com/marang/riotbox/pull/1066)`
- Merge commit: `76314686cbe7c1878b37fd8a4b0d7c560a306928`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1066; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Shipped in PR #1066.

Goal:

Make the instruction priority explicit so autonomous continuation can no longer be read as permission to skip Linear issue creation/state transitions.

Shipped:

* `docs/workflow_conventions.md` now states workflow priority order: preserve user work; keep canonical state in repo docs/Git/Linear; keep Linear issue state before Git branches; keep PR/CI/review/merge gates; then continue autonomously.
* Added explicit `No Ticket, No Branch` rule.
* Normal slice flow now starts with step 0: create or pick exactly one Linear issue before any branch work.
* Continuation phrases like "weiter" and "warte nicht" are documented as changing only whether the agent continues, not the workflow order.
* `AGENTS.md` mirrors the hard Linear-first rule.

Validation:

* `git diff --check`
* GitHub rust-ci green on PR #1066.

PR: [marang/riotbox#1066](https://linear.app/riotbox/review/clarify-linear-first-workflow-priority-bb46c3f0dbe3)

## What Shipped

- Closed the bounded scope: Workflow: Make Linear-first priority unambiguous.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.

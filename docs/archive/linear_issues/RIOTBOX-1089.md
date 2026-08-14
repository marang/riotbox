# `RIOTBOX-1089` P016: Define stem package export action contract

- Ticket: `RIOTBOX-1089`
- Title: `P016: Define stem package export action contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1089/p016-define-stem-package-export-action-contract`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1089-stem-package-export-contract`
- Linear branch: `feature/riotbox-1089-p016-define-stem-package-export-action-contract`
- Assignee: `Markus`
- Labels: None
- PR: `#1070 (https://github.com/marang/riotbox/pull/1070)`
- Merge commit: `885b2992fdee459178dc2156ea3d7250e036167a`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1070; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Contract-first P016 slice before any stem-package implementation.

Goal:

Specify the `export.stem_package` ActionCommand, Session receipt shape, observer lifecycle, and QA gate requirements without claiming working stem export yet.

Acceptance:

* Action Lexicon documents command params, target scope, queue/commit semantics, non-undoability, and result fields.
* Session spec documents required artifact roles, source/capture lineage, audio metrics, fallback comparison, and QA gates.
* Audio QA workflow spec states non-silence, role labeling, hash stability, and lineage gates.
* No code path claims stem export readiness until implementation tickets land.

Why it matters:

Software avoids a second export truth, and musicians will not see a stem export option before Riotbox can prove the stems are real and usable.

## What Shipped

- Closed the bounded scope: P016: Define stem package export action contract.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.

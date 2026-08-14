# `RIOTBOX-1088` P016: Validate export receipt lineage during replay planning

- Ticket: `RIOTBOX-1088`
- Title: `P016: Validate export receipt lineage during replay planning`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1088/p016-validate-export-receipt-lineage-during-replay-planning`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1088-export-receipt-lineage-replay-validation`
- Linear branch: `feature/riotbox-1088-p016-validate-export-receipt-lineage-during-replay-planning`
- Assignee: `Markus`
- Labels: None
- PR: `#1069 (https://github.com/marang/riotbox/pull/1069)`
- Merge commit: `456baae2e3b6f8a0a3390742013a3b0f88b58e1c`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1069; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Bounded P016 replay-hardening slice.

Goal:

Tighten replay/export validation so typed artifact lineage fields are checked for obvious invalid state without introducing file rewrites or hidden restore behavior.

Acceptance:

* Replay validation rejects blank source graph refs and blank timing-grid identities where present.
* Missing optional lineage remains allowed for older receipts.
* Errors are typed and point at receipt/artifact identity.
* Tests cover valid current product-mix receipt, legacy missing lineage, and invalid blank lineage.

Why it matters:

Software catches corrupted export receipts earlier, and musicians avoid trusting exports whose lineage data is malformed.

Implementation:

PR #1069 validates present source-graph refs for non-blank source id and graph hash, and present timing-grid refs for non-blank source id and non-blank provided hypothesis id. Missing optional lineage remains valid for older receipts. Local verification passed: focused replay-validation tests and compact `just ci`.

## What Shipped

- Closed the bounded scope: P016: Validate export receipt lineage during replay planning.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.

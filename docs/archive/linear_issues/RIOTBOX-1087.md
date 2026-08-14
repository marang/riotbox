# `RIOTBOX-1087` P016: Add proof artifact entries to product export receipts

- Ticket: `RIOTBOX-1087`
- Title: `P016: Add proof artifact entries to product export receipts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1087/p016-add-proof-artifact-entries-to-product-export-receipts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1087-product-export-proof-artifact-entry`
- Linear branch: `feature/riotbox-1087-p016-add-proof-artifact-entries-to-product-export-receipts`
- Assignee: `Markus`
- Labels: None
- PR: `#1068 (https://github.com/marang/riotbox/pull/1068)`
- Merge commit: `a50561f1e4c8f717ecc777f66f3e15e56f905185`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1068; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Bounded P016 receipt-completeness slice.

Goal:

Represent the current product-export proof JSON as a typed `artifact_set[]` entry instead of keeping proof identity only in legacy `proof_path`.

Acceptance:

* Product export receipts keep legacy `proof_path` for older readers.
* Artifact set includes a `product_export_proof` JSON entry with local path and hash.
* Observer lifecycle surfaces the proof artifact entry.
* Replay/restore validation continues to use Session/Core receipt truth.
* Validation includes focused export receipt tests and `just ci`.

Why it matters:

Software can reason about all exported files as artifact-set entries, and musicians get a clearer receipt pack instead of one WAV artifact plus separate hidden proof path.

Implementation:

PR #1068 adds a typed `product_export_proof`/`json` artifact-set entry, hashes the copied proof JSON during product export, and exposes the proof artifact through the existing observer receipt lifecycle. Local verification passed: focused core/app/observer tests and compact `just ci`.

## What Shipped

- Closed the bounded scope: P016: Add proof artifact entries to product export receipts.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.

# `RIOTBOX-1094` P016: Add reserved stem_package export scope to receipts

- Ticket: `RIOTBOX-1094`
- Title: `P016: Add reserved stem_package export scope to receipts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1094/p016-add-reserved-stem-package-export-scope-to-receipts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1094-reserved-stem-package-export-scope`
- Linear branch: `feature/riotbox-1094-p016-add-reserved-stem_package-export-scope-to-receipts`
- Assignee: `Markus`
- Labels: None
- PR: `#1074 (https://github.com/marang/riotbox/pull/1074)`
- Merge commit: `7ba500f7e227053f94a6f47c1c61757900fad2e7`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1074; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Bounded P016 receipt-model slice.

Goal:

Allow Session export receipts to represent `export_scope: stem_package` as a typed value while keeping runnable stem export blocked.

Acceptance:

* `ExportScope` includes `StemPackage` with stable snake_case serialization and musician label.
* `default_export_scope()` remains `ProductMix` for old receipts.
* Product-mix contracts and receipts continue to serialize as `product_mix`.
* Tests cover stem-package scope serialization/deserialization and label.
* Docs state the scope is reserved and does not remove unsupported-scope readiness blocks.

Why it matters:

Software can model future stem-package receipts explicitly without inferring scope from artifact roles. Musicians still do not see a stem export option until stronger gates and implementation land.

## What Shipped

- Closed the bounded scope: P016: Add reserved stem_package export scope to receipts.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.

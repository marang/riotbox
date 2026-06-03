# `RIOTBOX-1120` P016: Add stem-package queue guard before runnable export

- Ticket: `RIOTBOX-1120`
- Title: `P016: Add stem-package queue guard before runnable export`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1120/p016-add-stem-package-queue-guard-before-runnable-export`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1120-p016-add-stem-package-queue-guard-before-runnable-export`
- Linear branch: `feature/riotbox-1120-p016-add-stem-package-queue-guard-before-runnable-export`
- Assignee: `Markus`
- Labels: `Core`, `workflow`
- PR: `#1099 (https://github.com/marang/riotbox/pull/1099)`
- Merge commit: `02e566c245569d0ab03aa109581464711ab1a1a5`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; cargo test -p riotbox-core queue::tests; cargo test -p riotbox-app reserved_stem_package_export_queue_attempt_is_rejected_without_receipt; cargo test -p riotbox-core; cargo test -p riotbox-app; scripts/run_compact.sh /tmp/riotbox-1120-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md`
- Follow-ups: `RIOTBOX-1121`

## Why This Ticket Existed

P016 needed queue-level guardrails for the reserved export.stem_package action before any writer work so stem package attempts cannot remain ambiguously pending or imply a completed package.

## What Shipped

- Added Core enqueue_if_no_pending_command with duplicate-pending proof for ExportStemPackage, added a reserved stem-package queue attempt that records an explicit rejected queue-history action, and proved it creates no pending action, Session export receipt, Session action-log export entry, or writer side effect.

## Notes

- No audible behavior changed; structured listening review was not applicable.

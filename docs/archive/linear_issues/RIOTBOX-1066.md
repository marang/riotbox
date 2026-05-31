# `RIOTBOX-1066` Add export receipt replay and missing-artifact fixtures

- Ticket: `RIOTBOX-1066`
- Title: `Add export receipt replay and missing-artifact fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1066/add-export-receipt-replay-and-missing-artifact-fixtures`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1066-export-receipt-replay-fixtures`
- Linear branch: `feature/riotbox-1066-add-export-receipt-replay-and-missing-artifact-fixtures`
- Assignee: `Markus`
- Labels: None
- PR: `#1042 (https://github.com/marang/riotbox/pull/1042)`
- Merge commit: `4513dc0c3432eb59351a1316b6e388b823a9839a`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core export_receipt_replay_validation`; `cargo test -p riotbox-app export_receipt_hydration_preflight`; `cargo test -p riotbox-app recovery_surface_reports_export_receipt_artifact_availability`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1042`
- Docs touched: `None`
- Follow-ups: `Continue P016 export workflow slices from roadmap`

## Why This Ticket Existed

Export receipts should be replay/restore-visible without silently rewriting files or hiding missing artifacts.

## What Shipped

- Core replay validates export.product_mix receipt metadata without filesystem writes.
- App preflight reports missing exported mix/proof artifacts before restore/recovery trusts the receipt.
- Recovery artifact availability includes export receipts beside capture artifacts.
- Focused fixtures cover present export artifacts, missing export artifact reporting, missing proof identity, and replay metadata validation.

## Notes

- Full stem package, DAW export, live recording export, and replay-time artifact rewriting remain out of scope.

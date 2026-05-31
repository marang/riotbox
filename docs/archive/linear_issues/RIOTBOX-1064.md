# `RIOTBOX-1064` Implement export.product_mix action and export receipt model

- Ticket: `RIOTBOX-1064`
- Title: `Implement export.product_mix action and export receipt model`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1064/implement-exportproduct-mix-action-and-export-receipt-model`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1064-export-product-mix-action`
- Linear branch: `feature/riotbox-1064-implement-exportproduct_mix-action-and-export-receipt-model`
- Assignee: `Markus`
- Labels: None
- PR: `#1040 (https://github.com/marang/riotbox/pull/1040)`
- Merge commit: `d79f2c271f0088c029931d1eee70fc39fb5a6073`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core: pass`; `cargo test -p riotbox-app product_mix_export: pass`; `just product-export-reproducibility-smoke: pass`; `just ci: pass`; `GitHub rust-ci on PR #1040: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-1063 defined the first bounded P016 export action boundary. Riotbox needed a typed export action and durable receipt model before widening export polish.

## What Shipped

- Added ActionCommand::ExportProductMix / export.product_mix with full_grid_mix ProductExport params.
- Added Session export receipts tied to the creating action id with artifact/proof paths, hashes, readiness status, and unsupported-scope flags.
- Added Jam app export transaction that validates product-export proof, streams artifact hashing, copies artifact/proof, and commits only after success.
- Rejected failed proof/hash/write attempts without writing a receipt.
- Updated action lexicon, session file, and audio QA specs for the implemented first P016 export action.

## Notes

- None

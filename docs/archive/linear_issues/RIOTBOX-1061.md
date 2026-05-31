# `RIOTBOX-1061` Define first P016 export readiness contract from product-export proof

- Ticket: `RIOTBOX-1061`
- Title: `Define first P016 export readiness contract from product-export proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1061/define-first-p016-export-readiness-contract-from-product-export-proof`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1061-export-readiness-contract`
- Linear branch: `feature/riotbox-1061-define-first-p016-export-readiness-contract-from-product`
- Assignee: `Markus`
- Labels: None
- PR: `#1037 (https://github.com/marang/riotbox/pull/1037)`
- Merge commit: `ca1e0e0548bcb4a0552f665b216b1e65f68ede23`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-core export_readiness -- --nocapture: pass`; `git diff --check: pass`; `just product-export-reproducibility-smoke: pass`; `just ci: pass`; `GitHub rust-ci on PR #1037: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P016 needed to start from the existing deterministic product-export proof seam instead of jumping directly to full DAW/stem export.

## What Shipped

- Added riotbox-core::export_readiness with ProductExportReproducibilityProof and ExportReadinessContract.
- Locked the first P016 boundary to the existing feral-grid-demo / full_grid_mix product-export proof.
- Added unsupported-scope flags for stem package export, live recording export, DAW export, and host-audio soak.
- Added fixture-backed core tests for happy path, unsupported scopes, mismatched hash rejection, unknown boundary rejection, and unknown pack-id rejection.
- Updated docs/specs/audio_qa_workflow_spec.md.

## Notes

- None

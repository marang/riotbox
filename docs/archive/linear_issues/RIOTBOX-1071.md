# `RIOTBOX-1071` Add typed export artifact-set contract to product receipts

- Ticket: `RIOTBOX-1071`
- Title: `Add typed export artifact-set contract to product receipts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1071/add-typed-export-artifact-set-contract-to-product-receipts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1071-export-artifact-set-contract`
- Linear branch: `feature/riotbox-1071-add-typed-export-artifact-set-contract-to-product-receipts`
- Assignee: `Markus`
- Labels: None
- PR: `#1046 (https://github.com/marang/riotbox/pull/1046)`
- Merge commit: `6dc38276f5904653d4a75e1d08b7ba87e269ed6d`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core export_receipt -- --nocapture`; `cargo test -p riotbox-app export -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1046`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1072`

## Why This Ticket Existed

Wider P016 exports need typed per-artifact identity in Session/Core before stems, live recording, or DAW packages can safely be claimed.

## What Shipped

- Added typed artifact_set entries to export receipts with role, local path or URI, media type, sha256, and optional audio metrics.
- Populated current product-mix receipts with a one-entry full-grid WAV artifact set while preserving legacy receipt fields.
- Extended replay validation and observer snapshots to carry typed artifact identity.
- Updated Action Lexicon and Session File specs for the shipped artifact-set contract.

## Notes

- No stem package, DAW session, live recording, host-audio export, or new export state system shipped in this ticket.

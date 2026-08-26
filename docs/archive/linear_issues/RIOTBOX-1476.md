# `RIOTBOX-1476` Complete musician product-mix export proof handoff

- Ticket: `RIOTBOX-1476`
- Title: `Complete musician product-mix export proof handoff`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1476/complete-musician-product-mix-export-proof-handoff`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-26`
- Started: `2026-08-26`
- Finished: `2026-08-26`
- Branch: `feature/riotbox-1476-complete-musician-product-mix-export-proof-handoff`
- Linear branch: `feature/riotbox-1476-complete-musician-product-mix-export-proof-handoff`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1486 (https://github.com/marang/riotbox/pull/1486)`
- Merge commit: `123f19bb`
- Deleted from Linear: `2026-08-26`
- Verification: `cargo test -p riotbox-app (655/655 primary suite plus binary/integration suites); just product-export-reproducibility-smoke; just ci; GitHub rust-ci`
- Docs touched: `README.md; docs/execution_roadmap.md; docs/research_decision_log.md (RBX-348); docs/specs/action_lexicon_spec.md; docs/specs/session_file_spec.md; docs/specs/tui_screen_spec.md; docs/reviews/riotbox_1476_product_mix_export_proof_handoff_2026-08-26.md`
- Follow-ups: `RIOTBOX-1036 retains the wider stem and live-recording export workflow; DAW, host-capture, and new renderer work remain separate.`

## Why This Ticket Existed

The Jam E control queued the existing immediate product-mix action but never executed its proof-backed file side effect, allowing a musician request to remain pending and allowing unverified proof lineage.

## What Shipped

- Interactive source/session launches accept one paired proof and destination handoff; E now completes or visibly rejects the existing export.product_mix action outside the realtime callback.
- The proof source hash must match the active Source Graph before files or Session lineage are written; every failure path clears pending state without a receipt.
- The writer preserves existing files, treats only a complete hash-identical destination bundle as idempotent, and the reusable just product-export-handoff producer publishes a contained reproducibility-validated full_grid_mix bundle.

## Notes

- No audio-producing behavior changed. Exported bytes remain hash-identical to the validated proof artifact, so human listening was not required.

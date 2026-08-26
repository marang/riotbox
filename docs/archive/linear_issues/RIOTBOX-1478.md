# `RIOTBOX-1478` Migrate proven MC-202 source-expression origin into product stems

- Ticket: `RIOTBOX-1478`
- Title: `Migrate proven MC-202 source-expression origin into product stems`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1478/migrate-proven-mc-202-source-expression-origin-into-product-stems`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-26`
- Started: `2026-08-26`
- Finished: `2026-08-27`
- Branch: `feature/riotbox-1478-migrate-proven-mc-202-source-expression-origin-into-product-stems`
- Linear branch: `feature/riotbox-1478-migrate-proven-mc-202-source-expression-origin-into-product`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1490 (https://github.com/marang/riotbox/pull/1490)`
- Merge commit: `b767a382000b5454e8c414b973c24d0d509a6666`
- Deleted from Linear: `2026-08-27`
- Verification: `Feral Rust 44/44; observer/audio correlation 61/61; product-stem mutation suite 13/13; synthetic v2 handoff and source-free audio QA smokes; cargo clippy -D warnings; just ci; GitHub rust-ci`
- Docs touched: `README.md; docs/execution_roadmap.md; docs/jam_recipes.md; docs/research_decision_log.md; action/session/audio/source-timing/audio-QA specs; docs/reviews/riotbox_1478_mc202_source_expression_origin_2026-08-27.md`
- Follow-ups: `RIOTBOX-1036 retains the wider musician stem-package/live-recording/DAW workflow; next bounded P016 slice should connect the reserved export.stem_package action and Session receipt to active Source Graph lineage and the v2 handoff without using the synthetic CI fixture as product evidence.`

## Why This Ticket Existed

The existing source-derived MC-202 phrase plan was applied in rendering, but legacy primitive_renderer provenance prevented the reconstructable product-stem handoff from honestly representing all three stems as source-derived.

## What Shipped

- Migrated proven MC-202 source-expression evidence into a fail-closed v2 product-stem handoff and observer/export proof; rejected missing, unapplied, weak, fallback, primitive-boundary, hash, and reconstruction evidence; preserved the approved full-mix bytes and kept release/musician-export readiness false.

## Notes

- No DSP, phrase-planning algorithm, threshold, registered Development source, Holdout, commercial reference, or playback changed. Full-mix SHA-256 remained 612bbd6ad874c5308f639753ac28f42c61b7ca386759be64a0bc9c9b41e3a828.
- Decision `RBX-350` supersedes the legacy `RBX-092` origin boundary for the version-2 handoff; version 1 remains historical evidence and is not reinterpreted.

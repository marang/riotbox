# `RIOTBOX-1463` P023: Qualify dense-break foundation chop v1 through the product spine

- Ticket: `RIOTBOX-1463`
- Title: `P023: Qualify dense-break foundation chop v1 through the product spine`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1463/p023-qualify-dense-break-foundation-chop-v1-through-the-product-spine`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-25`
- Started: `2026-08-25`
- Finished: `2026-08-25`
- Branch: `feature/riotbox-1463-dense-break-foundation-chop-v1`
- Linear branch: `feature/riotbox-1463-p023-qualify-dense-break-foundation-chop-v1-through-the`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1456 (https://github.com/marang/riotbox/pull/1456)`
- Merge commit: `bf427760e020721bb0cbd282e40a717bb4410f01`
- Deleted from Linear: `2026-08-25`
- Verification: `focused source-timing queue/commit/restore/replay/invalid-value tests; exact three-source product qualification; formal hash-bound human review; full just ci; final 645-test riotbox-app suite; strict Clippy; GitHub rust-ci (PR #1456)`
- Docs touched: `docs/benchmarks/dense_break_foundation_chop_product_qualification_v1.json, docs/benchmarks/dense_break_foundation_chop_product_qualification_v2.json, docs/benchmarks/dense_break_foundation_chop_product_qualification_v3.json, docs/reviews/riotbox_1463_dense_break_foundation_chop_product_rejection_2026-08-25.md, docs/execution_roadmap.md, docs/phase_definition_of_done.md`
- Follow-ups: `A clarity-preserving successor requires a new Linear-first mechanism version and Decision frozen before fresh source access.`

## Why This Ticket Existed

The provisionally kept Dense foundation chop needed exact product-spine, replay, callback, source-diversity, and formal listening qualification before any musician-facing promotion.

## What Shipped

- Preserved the complete frozen v1/v2/v3 qualification audit, accepted the three-source technical matrix as negative evidence, and removed the musically rejected low-clarity chop from the product before merge.
- Persisted the exact positive finite musician-confirmed BPM through Action params, Session runtime state, restore, and Core replay; matching revert clears it and invalid persisted values fail closed.

## Notes

- Formal product listening found the hook clear and source recognizable but rejected v1 for substantial low-frequency and clarity loss with an unintended radio-like character.
- Dense remains non-demo-ready. No Holdout, source-general, hardness, quality, demo, release, automatic-arrangement, or P023-completion claim follows.

# `RIOTBOX-1508` Replace Core Jam view and projection-test include shells with semantic modules

- Ticket: `RIOTBOX-1508`
- Title: `Replace Core Jam view and projection-test include shells with semantic modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1508/replace-core-jam-view-and-projection-test-include-shells-with-semantic`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1508-semantic-core-jam-view`
- Linear branch: `feature/riotbox-1508-replace-core-jam-view-and-projection-test-include-shells`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1561 (https://github.com/marang/riotbox/pull/1561)`
- Merge commit: `bf65b983af7c79e088d960b1f446c7f9a802ec0b`
- Deleted from Linear: `2026-09-30`
- Verification: `470 Core regressions before/after, identical leaf-name multiset; 203 complete items preserved, 198 normalized exact and five inspected Rustfmt-only blocks; all 35 public type shapes unchanged. Core/App all-target warning-free; full just ci and native CI 36780046907 both jobs green. Sequential solo review, zero outstanding findings.`
- Docs touched: `docs/reviews/riotbox_1508_semantic_core_jam_view_2026-09-30.md`, `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`
- Follow-ups: `Remaining include migrations stay in the existing inventory; separate RIOTBOX-1509 investigates sporadic native Sidecar startup failures.`

## Why This Ticket Existed

Replace the planned Core Jam projection include namespace with semantic module ownership.

## What Shipped

- Twelve view/test includes removed; stable explicit public facade, nine private projection owners, real regression families and shared synthetic fixtures. Inventory 82 sites / 11 owners.

## Notes

- RBX-386; no state/schema, timing policy, DSP, labels/layout, source/audio/device or DAW change. First substantive ownership slice after merged/closed RIOTBOX-1507 checkpoint.

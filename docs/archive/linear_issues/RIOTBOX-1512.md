# `RIOTBOX-1512` Replace JamApp lane-projection include namespace with semantic render owners

- Ticket: `RIOTBOX-1512`
- Title: `Replace JamApp lane-projection include namespace with semantic render owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1512/replace-jamapp-lane-projection-include-namespace-with-semantic-render`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1512-semantic-lane-projections`
- Linear branch: `feature/riotbox-1512-replace-jamapp-lane-projection-include-namespace-with`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1568 (https://github.com/marang/riotbox/pull/1568)`
- Merge commit: `6d4942db1b8fce77c31e76265611b60208e756f3`
- Deleted from Linear: `2026-10-01`
- Verification: `Full source-free just ci; final warning-free all-target App check with explicit log inspection; final App baseline/post identical 771 leaf/status entries (770 pass, same ignore); all 63 production definitions and six facade signatures retained; both inline chop regressions unchanged; include guard 56/8; solo code-review/Rust/self-review no unresolved findings; exact-head native run 36786876816 passed both jobs.`
- Docs touched: `docs/engineering/module_policy.md, docs/engineering/textual_include_allowlist.txt, docs/engineering/textual_include_inventory_2026-06-29.md, docs/research_decision_log.md, docs/reviews/riotbox_1512_semantic_lane_projections_2026-10-01.md`
- Follow-ups: `RIOTBOX-1513 Core TR-909 policy module migration; RIOTBOX-1509 remains Todo, no causal Windows startup fix claim.`

## Why This Ticket Existed

Expose existing Core/Session-to-audio projection dependencies without mixed lane includes or another product state owner.

## What Shipped

- Semantic TR-909, MC-202, shared scene-context and W-30 preview/material/resample owners; existing source phrase child and six JamApp-only entrypoints retained; no sound-policy or sample-preparation changes.

## Notes

- RBX-389 records ownership only. All 63 definitions retained: 56 strict normalized matches and seven individually inspected Rustfmt expression blocks. Initial unused-import warning and premature warning-free statement corrected and documented. No real source/holdout/reference/device/DAW/playback or musical claim. Architecture cadence three substantive slices since RIOTBOX-1507.

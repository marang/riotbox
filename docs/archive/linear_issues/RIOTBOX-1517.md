# `RIOTBOX-1517` Replace lane recipe QA includes with semantic, source-free Rust owners

- Ticket: `RIOTBOX-1517`
- Title: `Replace lane recipe QA includes with semantic, source-free Rust owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1517/replace-lane-recipe-qa-includes-with-semantic-source-free-rust-owners`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1517-semantic-lane-recipe`
- Linear branch: `feature/riotbox-1517-replace-lane-recipe-qa-includes-with-semantic-source-free`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1578 (https://github.com/marang/riotbox/pull/1578)`
- Merge commit: `674191dd5b7d014f5e5b4f9703bd2ff801f0027e`
- Deleted from Linear: `2026-10-01`
- Verification: `75 retained definitions (65 production, ten regressions): 74 strict normalized, one inspected Rustfmt expression brace. Ten executed test leaves/statuses exact; 13 CLI byte/status cases exact; 84 fresh synthetic file hashes exact, including 32 WAVs. Final all-target/build/tests and full source-free just ci warning/error-free. Exact head d2ff84b6518aef3661f2b0c6bfe34276d0bdc3af native 36799218094 Rust/Windows Sidecar passed, zero reviews/comments.`
- Docs touched: `docs/reviews/riotbox_1517_semantic_lane_recipe_2026-10-01.md`, `docs/engineering/module_policy.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1518 owns newly demonstrated W-30 non-finite duration defect before remaining QA migrations; RIOTBOX-1509 causal Windows diagnosis remains open. Architecture cadence now one since verified 1516 checkpoint closeout.`

## Why This Ticket Existed

Six lane recipe QA includes hid CLI/case/render/measurement/report dependencies on the trusted Recipe-2 path.

## What Shipped

- Single existing binary with semantic private owners; data/config/signal leaves avoid cycles. Complete original controls, algorithms, literals, thresholds, regression bodies and primitive promotion boundary retained. Removed six include sites; inventory 44/six owners to 38/five.

## Notes

- RBX-392; solo sequential Rust/correctness/architecture/workflow/audio-boundary review and self-review: initial imports fixed, no remaining migration finding. No real sources/holds/commercial audio/directories/device/DAW/playback or musical/source/human pass. Core/Session/runtime/frozen Stage-A untouched.

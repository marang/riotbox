# `RIOTBOX-1285` Make tonal-hook MC-202 candidate source-composed instead of template-only

- Ticket: `RIOTBOX-1285`
- Title: `Make tonal-hook MC-202 candidate source-composed instead of template-only`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1285/make-tonal-hook-mc-202-candidate-source-composed-instead-of-template`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1285-tonal-mc202-source-composed`
- Linear branch: `feature/riotbox-1285-make-tonal-hook-mc-202-candidate-source-composed-instead-of`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1260 (https://github.com/marang/riotbox/pull/1260)`
- Merge commit: `906239215ba3c7ef085da101c0a002861dba4bfe`
- Deleted from Linear: `2026-06-18`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio --bin feral_grid_pack tonal_hold_contour_keeps_mc202_support_reviewable -- --nocapture; just professional-output-listening-pack-smoke; just mc202-producer-grade-closeout-smoke; just pro-pressure-source-matrix-smoke; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `docs/execution_roadmap.md; docs/plans/mc202_source_phrase_planning_plan.md; docs/benchmarks/mc202_producer_grade_closeout_v1_2026-06-18.md`
- Follow-ups: `None`

## Why This Ticket Existed

Tonal-hook MC-202 review candidate still carried primitive/template-only status, which kept the producer-grade closeout from representing all current MC-202 candidates as source-composed.

## What Shipped

- Raised Hold/Neutral source-contour MC-202 pressure support, rebalanced tonal restore pressure, tightened professional-pack and closeout gates so dense/tonal/sparse candidates are source-composed and not primitive/template-only, and added validator coverage for ambiguous source-composed/primitive states.

## Notes

- None

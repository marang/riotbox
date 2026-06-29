# `RIOTBOX-1303` P023: Select MC-202 candidates by production-impact score

- Ticket: `RIOTBOX-1303`
- Title: `P023: Select MC-202 candidates by production-impact score`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1303/p023-select-mc-202-candidates-by-production-impact-score`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1303-p023-select-mc-202-candidates-by-production-impact-score`
- Linear branch: `feature/riotbox-1303-p023-select-mc-202-candidates-by-production-impact-score`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1277 (https://github.com/marang/riotbox/pull/1277)`
- Merge commit: `a84830654a5480ea14b51919c2badc7fa2fa8d25`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt`; `cargo test -p riotbox-app committed_mc202_selection_prefers_source_production_impact_dimensions -- --nocapture`; `cargo test -p riotbox-app mc202 -- --nocapture`; `cargo test -p riotbox-audio mc202 -- --nocapture`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1303-just-ci.log just ci`; `GitHub rust-ci passed on PR #1277`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

The MC-202 source phrase composer could expose useful scorecard dimensions, but winner selection was still dominated by family-specific feature buckets and role bias. Producer-grade behavior needs the selected candidate to be chosen from the same low-end, answer, hook, destructive, memory, grid, and role-fit dimensions that QA and replay can inspect.

## What Shipped

- Candidate scoring now includes an explicit production-impact score across low-end impact, source-grid confidence, answer contrast, hook avoidance, phrase memory, destructive usefulness, and role fit.
- Selected-candidate provenance now records production-impact score and selected dimension values for reviewable Session/replay evidence.
- FillPickupInstigator now carries answer/pickup contrast so transient pickup sources can beat sparse answer material when that is the stronger stage gesture.
- A pressure-vs-pickup source regression proves different source evidence selects different MC-202 candidate families and produces measurably different rendered output.

## Notes

- Human/demo promotion remains blocked; this improves source-composed selection quality but does not claim a human musical pass.

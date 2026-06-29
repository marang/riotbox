# `RIOTBOX-1312` P023: Make MC-202 closeout prove source-derived bass-or-answer role evidence

- Ticket: `RIOTBOX-1312`
- Title: `P023: Make MC-202 closeout prove source-derived bass-or-answer role evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1312/p023-make-mc-202-closeout-prove-source-derived-bass-or-answer-role`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1312-p023-make-mc-202-closeout-prove-source-derived-bass-or-answer-role`
- Linear branch: `feature/riotbox-1312-p023-make-mc-202-closeout-prove-source-derived-bass-or`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1286 (https://github.com/marang/riotbox/pull/1286)`
- Merge commit: `04b5618db6a9f689cb3bf4290fbdd2cf18a0e651`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_mc202_producer_grade_closeout.py`; `just mc202-producer-grade-closeout-smoke`; `just ci`; `GitHub Actions rust-ci passed for PR #1286`
- Docs touched: `docs/benchmarks/mc202_producer_grade_closeout_v1_2026-06-18.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

The MC-202 producer-grade closeout could show source-composed evidence while dense or tonal candidates had no source-derived bass movement. That can be musically valid when the MC-202 role is answer/stab/restraint rather than bass pressure, but the closeout needed to prove that role explicitly so generic source-composed flags cannot hide missing bass evidence.

## What Shipped

- Added per-candidate mc202_role_evidence to the MC-202 producer-grade closeout.
- Sparse bass-pressure candidates now require source-derived bass movement, static-distance separation, and frequency-span movement.
- Dense/non-dense and tonal candidates now require source-derived pressure-answer or hook-restraint/stab-answer evidence, including role-order derivation, scripted-distance separation, audibility, and pressure lift.
- Mutation fixtures now reject missing, stale, unsupported, or source-family-inappropriate MC-202 role evidence.
- Benchmark and roadmap docs now state that generic source-composed evidence is not enough for this closeout.

## Notes

- Role evidence remains quality_proof=false; it tightens technical closeout evidence but does not replace structured human listening for producer-grade/demo promotion.

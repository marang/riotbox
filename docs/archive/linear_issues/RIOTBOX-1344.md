# `RIOTBOX-1344` P023: Strengthen sparse MC-202 bass movement from producer fix routing

- Ticket: `RIOTBOX-1344`
- Title: `P023: Strengthen sparse MC-202 bass movement from producer fix routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1344/p023-strengthen-sparse-mc-202-bass-movement-from-producer-fix-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1344-p023-strengthen-sparse-mc-202-bass-movement-from-producer`
- Linear branch: `feature/riotbox-1344-p023-strengthen-sparse-mc-202-bass-movement-from-producer`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1308 (https://github.com/marang/riotbox/pull/1308)`
- Merge commit: `51b0a9809d4bd6833c503c1be4892ba581e904e6`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/validate_pro_pressure_source_matrix.py; just professional-source-wav-pack-smoke; just mc202-producer-grade-closeout-smoke; just demo-bank-promotion-fixtures; just sound-quality-readiness-report-smoke; just professional-output-suite-smoke; just pro-pressure-source-matrix-smoke; just audio-qa-ci; just ci; GitHub rust-ci success on PR #1308.`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md; docs/research_decision_log.md`
- Follow-ups: `Next producer-fix implementation candidate after sparse is no longer bass_movement: dense answer_bite or tonal hook_restraint/mix_bus from the MC-202 producer-grade closeout.`

## Why This Ticket Existed

Sparse MC-202 bass pressure was source-composed but still close enough to the old movement floor that producer routing kept treating it as weak bass movement.

## What Shipped

- Raised sparse MC-202 bass movement from the 10 Hz diagnostic floor to a 12 Hz producer floor, expanded source-ranked sparse frequencies only when source-derived span is too narrow, exposed sparse_bass_movement_span_margin_hz proof, hardened source-WAV/matrix/closeout/suite/readiness gates, and changed producer routing so sparse emits bass_movement only for real metric weakness.

## Notes

- No human listening verdict was added; human_verdict remains unverified. The automated gate now proves stronger sparse bass movement and routing hygiene, not final musician approval.

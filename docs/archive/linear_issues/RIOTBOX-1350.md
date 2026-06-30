# `RIOTBOX-1350` P023: Let MC-202 closeout consume structured listening verdict labels

- Ticket: `RIOTBOX-1350`
- Title: `P023: Let MC-202 closeout consume structured listening verdict labels`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1350/p023-let-mc-202-closeout-consume-structured-listening-verdict-labels`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1350-p023-let-mc-202-closeout-consume-structured-listening`
- Linear branch: `feature/riotbox-1350-p023-let-mc-202-closeout-consume-structured-listening`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1314 (https://github.com/marang/riotbox/pull/1314)`
- Merge commit: `fcdc6e4da9a8633d29253e17948226330716d7d7`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_mc202_producer_grade_closeout.py`; `just mc202-producer-grade-closeout-smoke`; `just professional-output-listening-verdict-import-fixtures`; `just audio-qa-ci`; `just ci`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`, `docs/research_decision_log.md`
- Follow-ups: `Continue P023 by turning weak/fail human labels and weak-output routing into the next concrete production-fix implementation slices.`

## Why This Ticket Existed

Make the MC-202 producer-grade review queue executable by consuming exact human listening labels without weakening quality boundaries.

## What Shipped

- MC-202 closeout accepts an optional structured label corpus and resolves queue entries only when source id, source family, professional review-pack schema, and candidate WAV hash match.
- Resolved labels carry human verdict and reviewer context while keeping quality_proof and automated_musical_approval false.
- Weak or fail labels resolve review work but remain producer-grade blockers; unresolved tonal/sparse entries stay in structured_human_verdict_missing.
- Closeout smoke now imports a dense-break pass verdict and proves only dense_beat03_130 resolves.

## Notes

- None

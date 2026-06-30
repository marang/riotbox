# `RIOTBOX-1345` P023: Strengthen dense MC-202 answer bite from producer fix routing

- Ticket: `RIOTBOX-1345`
- Title: `P023: Strengthen dense MC-202 answer bite from producer fix routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1345/p023-strengthen-dense-mc-202-answer-bite-from-producer-fix-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1345-p023-strengthen-dense-mc-202-answer-bite-from-producer-fix`
- Linear branch: `feature/riotbox-1345-p023-strengthen-dense-mc-202-answer-bite-from-producer-fix`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1309 (https://github.com/marang/riotbox/pull/1309)`
- Merge commit: `70634f04515574261f9b294bf08f71418a3a0964`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile affected MC-202 scripts; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1345-dense-pack-smoke; just mc202-producer-grade-closeout-smoke; just demo-bank-promotion-fixtures; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just sound-quality-readiness-report-smoke; just audio-qa-ci; just weak-output-fix-routing-fixtures; just ci; GitHub Rust CI/rust-ci`
- Docs touched: `docs/research_decision_log.md; docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `None`

## Why This Ticket Existed

Dense MC-202 answer_bite was still a default producer route instead of a measured producer floor.

## What Shipped

- Added Dense Answer Bite proof metrics and gates, preserved dense opening hook while selecting source-ranked non-scripted pressure bars, and routed dense answer_bite only when the measured floor fails.

## Notes

- Structured human listening remains human_verdict: unverified; automated gates prove routing/collapse resistance, not human taste approval.

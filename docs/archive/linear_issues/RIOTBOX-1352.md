# `RIOTBOX-1352` P023: Strengthen bass movement from weak-output routing

- Ticket: `RIOTBOX-1352`
- Title: `P023: Strengthen bass movement from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1352/p023-strengthen-bass-movement-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1352-p023-strengthen-bass-movement-from-weak-output-routing`
- Linear branch: `feature/riotbox-1352-p023-strengthen-bass-movement-from-weak-output-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1316 (https://github.com/marang/riotbox/pull/1316)`
- Merge commit: `5cf52ba766aa9693a50e7237ec6fa09bd43d8171`
- Deleted from Linear: `2026-06-30`
- Verification: `py_compile; just pro-pressure-source-matrix-smoke; just professional-source-wav-pack-smoke; just professional-output-suite-smoke; just weak-output-fix-routing-fixtures; just mc202-producer-grade-closeout-smoke; just sound-quality-readiness-report-smoke; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still named bass_movement after sparse bass pressure became source-derived, so the old 12 Hz / 0.28 gates were letting barely moving low-end diagnostics pass.

## What Shipped

- Raised sparse bass movement span to 15 Hz and low-band share to 0.32 across matrix, source-WAV, professional suite, MC-202 closeout, producer-fix routing, source-composed review gate, sound-readiness report, docs, and roadmap. Strengthened sparse render balance with wider source-derived contour, heavier sub pressure, less midrange masking, stronger restore impact, and preserved source identity. Verified source-matrix sparse minima: span 15.857 Hz, low-band share 0.340, lift 2.991x, restore/pressure 1.145x, bass dominance 0.490.

## Notes

- None

# `RIOTBOX-1299` Strengthen source-character selection survival proof

- Ticket: `RIOTBOX-1299`
- Title: `Strengthen source-character selection survival proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1299/strengthen-source-character-selection-survival-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-28`
- Started: `2026-06-28`
- Finished: `2026-06-28`
- Branch: `feature/riotbox-1299-source-character-selection-survival`
- Linear branch: `feature/riotbox-1299-strengthen-source-character-selection-survival-proof`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1273 (https://github.com/marang/riotbox/pull/1273)`
- Merge commit: `c799debbfc84c5f3f285619a023f7bc55a2440cd`
- Deleted from Linear: `2026-06-28`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py scripts/generate_sound_quality_readiness_report.py scripts/validate_pro_pressure_source_matrix.py scripts/route_weak_output_fixes.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1299-dense-smoke; just dense-break-weak-source-character-fixture-smoke artifacts/audio_qa/local-riotbox-1299-weak-source-smoke; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1299-matrix; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1299-source-wav-final; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1299-edge; just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1299-routing; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1299-suite; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1299-readiness; git diff --check; just ci; GitHub rust-ci passed on PR #1273`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still identified source-character loss as a source-selection production fix; Riotbox needed a stricter proof that transformed source identity survives with margin, not only barely over the floor.

## What Shipped

- Added rebuild_only_source_character_survival_margin with a 0.10 gate; propagated it through dense, matrix, source-WAV, edge, professional-suite, and readiness diagnostics; strengthened the real weak-source WAV fixture; routed margin failures to source_selection; improved tonal-hook rebuild-only source character with source-derived W-30/riff mid-focus shaping.

## Notes

- Observed readiness aggregation min survival 0.8290883691505337 and min margin 0.1290883691505338. Evidence remains diagnostic and quality_proof false until structured listening review promotes it.

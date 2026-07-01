# `RIOTBOX-1359` P023: Strengthen bass movement from weak-output routing

- Ticket: `RIOTBOX-1359`
- Title: `P023: Strengthen bass movement from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1359/p023-strengthen-bass-movement-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1359-p023-strengthen-bass-movement-from-weak-output-routing`
- Linear branch: `feature/riotbox-1359-p023-strengthen-bass-movement-from-weak-output-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1323 (https://github.com/marang/riotbox/pull/1323)`
- Merge commit: `4dc7d006734d26ea1a1ccae257a70ac080db2de7`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_professional_output_suite_contract.py scripts/generate_sound_quality_readiness_report.py scripts/mc202_source_composed_review_gate.py scripts/mc202_producer_fix_routing.py scripts/generate_mc202_producer_grade_closeout.py; just professional-source-wav-pack-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just mc202-producer-grade-closeout-smoke; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1359-readiness; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1323`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Fresh P023 readiness after RIOTBOX-1358 still routed bass_movement as a top weak-output category; sparse bass movement was passing with little headroom over the old 15 Hz span gate.

## What Shipped

- Sparse-bass-pressure diagnostics now require sparse_bass_movement_frequency_span_hz >= 17.0 across generator, matrix, professional source-WAV, professional suite, MC-202 closeout/review, producer-fix routing, and readiness reporting. The generator uses the same floor for source-feature contour expansion, so sparse matrix/source-WAV cases now render 18.6 Hz movement span while bass remains strongest.

## Notes

- This remains diagnostic/scripted evidence only: quality_proof=false and human_verdict=unverified; it improves technical pressure-shape proof but does not claim human musical pass.

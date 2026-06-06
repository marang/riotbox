# `RIOTBOX-1226` Derive arrangement role order from source section candidates

- Ticket: `RIOTBOX-1226`
- Title: `Derive arrangement role order from source section candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1226/derive-arrangement-role-order-from-source-section-candidates`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1226-derive-arrangement-role-order-from-source-section-candidates`
- Linear branch: `feature/riotbox-1226-derive-arrangement-role-order-from-source-section-candidates`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1199 (https://github.com/marang/riotbox/pull/1199)`
- Merge commit: `f11375b59d44094985414c86c3e6a90dc9f5968a`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1226-dense-smoke-4`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1226-matrix-smoke-5`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1226-source-wav-smoke-2`; `just destructive-variation-professional-smoke artifacts/audio_qa/local-riotbox-1226-destructive-smoke-1`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1226-suite-smoke-1`; `scripts/run_compact.sh /tmp/riotbox-1226-audio-qa-ci-final.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-1226-just-ci-final.log just ci`; `gh pr checks 1199 --watch`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224`

## Why This Ticket Existed

The P022 diagnostics had already moved hook/chop selection, destructive cues, and sparse bass movement toward source-derived proof, but the first six arrangement bars still came from a fixed source-family role recipe. That allowed different sources to share the same rehearsed hook/chop/pressure shape even when other production choices reacted to the source.

## What Shipped

- Added source/W-30 bar-candidate arrangement selection for eligible dense-break, tonal-hook, and sparse-bass-pressure sources.
- Exposed arrangement role source-derived proof, candidate count, old scripted role-order distance, and section score span in dense reports and downstream matrix/source/suite summaries.
- Added positive suite assertions and a negative dense report mutation rejecting non-source-derived arrangement role order.
- Tuned sparse pressure/restore body so source-derived pressure placement keeps lift and rebuild-only restore impact above existing gates.
- Documented the new P022 arrangement evidence boundary in benchmark docs, roadmap, and RBX-075.

## Notes

- This remains diagnostic scripted evidence with quality_proof false and human_verdict unverified because role vocabulary, destructive/restore tail, and mix recipe remain bounded.
- Branch review removed an artificial fallback swap so scripted-role collapse now fails instead of being adjusted into passing distance.

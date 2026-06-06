# `RIOTBOX-1222` Add source-derived hook/chop selection proof across dense and tonal sources

- Ticket: `RIOTBOX-1222`
- Title: `Add source-derived hook/chop selection proof across dense and tonal sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1222/add-source-derived-hookchop-selection-proof-across-dense-and-tonal`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1222-add-source-derived-hookchop-selection-proof-across-dense-and`
- Linear branch: `feature/riotbox-1222-add-source-derived-hookchop-selection-proof-across-dense-and`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1197 (https://github.com/marang/riotbox/pull/1197)`
- Merge commit: `870f80a62f9656643c1cbc0e65d3ca71a6cb0dbf`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_professional_output_suite.py scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1222-dense-smoke-review`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1222-suite-smoke-review-final`; `scripts/run_compact.sh /tmp/riotbox-1222-audio-qa-ci-final.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-1222-just-ci-after-review.log just ci`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224`

## Why This Ticket Existed

Dense-break and tonal-hook diagnostics still began W-30 hook/chop behavior from a fixed first-bar grain while claiming stronger source-aware behavior. The ticket existed to make hook/chop selection visibly source-derived, measurable, and rejectable when it collapses back to the old static choice.

## What Shipped

- Added a bounded hook_chop_policy that scans source/W-30 candidates and selects separate hook and chop offsets.
- Routed W-30 hook riff and dropout/stutter grain selection through the source-derived hook/chop policy.
- Exposed hook/chop candidate count, static-first-bar distance, and hook/chop offset contrast in dense reports, source matrix, professional-source WAV pack, and professional-output suite.
- Added positive and negative smoke assertions for dense and tonal hook/chop source-derived proof while preserving quality_proof: false and human_verdict: unverified.

## Notes

- The proof is diagnostic, not a musical approval gate; structured listening review is still required before claiming release-grade quality.
- The oversized Justfile assertions are intentionally left for RIOTBOX-1224 rather than refactored inside this slice.

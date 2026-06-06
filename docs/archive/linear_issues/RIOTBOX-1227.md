# `RIOTBOX-1227` Derive mix-bus treatment from source energy candidates

- Ticket: `RIOTBOX-1227`
- Title: `Derive mix-bus treatment from source energy candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1227/derive-mix-bus-treatment-from-source-energy-candidates`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1227-derive-mix-bus-treatment-from-source-energy-candidates`
- Linear branch: `feature/riotbox-1227-derive-mix-bus-treatment-from-source-energy-candidates`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1200 (https://github.com/marang/riotbox/pull/1200)`
- Merge commit: `1d3e8f1fd86b38eacf218ec21621bd426baff55f`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py`; `git diff --check`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1227-dense-smoke-2`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1227-matrix-smoke-2`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1227-source-wav-smoke-2`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1227-suite-smoke-1`; `just ci`; `just audio-qa-ci`; `GitHub Actions rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224 keeps the Justfile audio-QA validator extraction follow-up open`

## Why This Ticket Existed

P022 had source-derived hook/chop selection, destructive cues, sparse bass movement, and arrangement role order, but the professional-output mix bus still used fixed treatment constants. That could let Riotbox sound like the same pressure/restore recipe over different sources instead of reacting to source energy.

## What Shipped

- Added a bounded source-derived mix_treatment_policy for dense-break, tonal-hook, and sparse-bass-pressure diagnostics.
- Applied source-derived hook/chop/pressure/restore/final bus drive, slam, W-30 gain, break-snap gain, pressure peak, and restore bass gain in the dense performance render.
- Surfaced mix-treatment source-derived proof, candidate count, fixed-recipe distance, and output contrast through dense, matrix, source-WAV, and suite reports.
- Added positive gates and a negative mutation so non-source-derived or fixed-collapsed mix treatment fails CI.
- Updated P022 benchmark docs, roadmap text, and RBX-076 while keeping quality_proof false and human_verdict unverified.

## Notes

- An earlier parallel local audio-qa-ci run collided with just ci on default artifact directories; the serial audio-qa-ci rerun passed.

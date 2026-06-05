# `RIOTBOX-1216` Turn dense-break 8-bar order into source-aware arrangement policy

- Ticket: `RIOTBOX-1216`
- Title: `Turn dense-break 8-bar order into source-aware arrangement policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1216/turn-dense-break-8-bar-order-into-source-aware-arrangement-policy`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1216-turn-dense-break-8-bar-order-into-source-aware-arrangement`
- Linear branch: `feature/riotbox-1216-turn-dense-break-8-bar-order-into-source-aware-arrangement`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1191 (https://github.com/marang/riotbox/pull/1191)`
- Merge commit: `8928e74dd68392315714ba07a1a59520021b0ad8`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/route_weak_output_fixes.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1216-dense-smoke-2`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1216-matrix-smoke-2`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1216-source-wav-smoke-2`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1216-suite-smoke-1`; `scripts/run_compact.sh /tmp/riotbox-1216-audio-qa-ci-rerun.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-1216-just-ci-rerun.log just ci`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1216-matrix-smoke-review`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

The dense-break professional render had source-aware pressure/stutter/restore behavior, but the 8-bar role order was still effectively a fixed recipe. Riotbox needed a bounded arrangement policy so different source families can create different rise/cut/restore placements without claiming final musical proof.

## What Shipped

- Added a source-aware arrangement_policy alongside pressure_lift_policy in the dense-break performance pack.
- Rendered the full performance from bounded roles instead of hardcoded bar slots while preserving the dense-break Golden Path signature.
- Added a sparse-bass-pressure early-pressure arrangement signature and role-based pressure/restore proof.
- Extended pro-pressure source matrix, professional source WAV pack, and professional output suite key metrics with arrangement signatures.
- Added arrangement failure routing through the weak-output fix categories and CI smoke gates requiring at least two role-order signatures.
- Updated benchmark docs and roadmap to state the new diagnostic boundary: source-aware arrangement diversity exists, role grammar remains scripted, quality_proof is false, human_verdict is unverified.

## Notes

- Local parallel runs of audio-qa-ci and just ci can race on shared artifacts/audio_qa/local-* paths; final verification was rerun sequentially.

# `RIOTBOX-1295` Strengthen W-30 hook/chop policy for routed weak outputs

- Ticket: `RIOTBOX-1295`
- Title: `Strengthen W-30 hook/chop policy for routed weak outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1295/strengthen-w-30-hookchop-policy-for-routed-weak-outputs`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1295-w30-hook-chop-policy`
- Linear branch: `feature/riotbox-1295-strengthen-w-30-hookchop-policy-for-routed-weak-outputs`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1269 (https://github.com/marang/riotbox/pull/1269)`
- Merge commit: `7a273fe347888cdcb869532d9fa821967e342438`
- Deleted from Linear: `2026-06-18`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_output_suite.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/validate_professional_output_suite_contract.py; git diff --check; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1295-dense; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1295-source-wav; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1295-matrix; python3 scripts/generate_edge_source_professional_diagnostics.py --output artifacts/audio_qa/local-riotbox-1295-edge --date local-riotbox-1295-edge; just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1295-weak-routing; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1295-suite; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

W-30 hook/chop diagnostics needed to prove audible riff playback came from source-derived hit patterns, not only source-window selection.

## What Shipped

- Derived W-30 riff hit placement, velocity span, and reverse-hit proof from selected source offsets; gated dense/tonal reports across dense, matrix, source-WAV, and professional-output suite; kept pad/noise and bad-timing off the W-30 hook-riff path with texture/timing-cue behavior; updated P023 roadmap and audio QA spec.

## Notes

- Artifacts remain diagnostic-only with quality_proof=false and human_verdict=unverified; large QA scripts remain an extraction candidate for future P023 work.

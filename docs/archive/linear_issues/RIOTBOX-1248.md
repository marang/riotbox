# `RIOTBOX-1248` P023: Strengthen source-selection fixes for weak source-character survival

- Ticket: `RIOTBOX-1248`
- Title: `P023: Strengthen source-selection fixes for weak source-character survival`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1248/p023-strengthen-source-selection-fixes-for-weak-source-character`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1248-p023-source-selection-survival`
- Linear branch: `feature/riotbox-1248-p023-strengthen-source-selection-fixes-for-weak-source`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1223 (https://github.com/marang/riotbox/pull/1223)`
- Merge commit: `2d3154a2888a04346dddf9a0bd6e2802f249e806`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_output_suite.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/validate_professional_output_suite_contract.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1248-dense-break; just pro-pressure-source-matrix-smoke; just professional-source-wav-pack-smoke; just professional-output-suite-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak source-character failures needed to become source-window selection constraints instead of only post-hoc weak-output routing reports.

## What Shipped

- Added source-character scoring for hook/chop/riff candidates, biased dense-break source-window selection toward stronger source identity, gated source-character floor/span through dense, matrix, source-WAV, and professional-suite reports, and documented the diagnostic boundary.

## Notes

- Tonal hook ranking stayed stable after review because applying the dense ranking bias there weakened restore/pressure balance; tonal now exposes and gates the same source-character proof without changing its ranking.

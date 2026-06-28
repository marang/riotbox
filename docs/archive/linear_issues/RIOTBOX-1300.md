# `RIOTBOX-1300` Strengthen hook/chop policy professional proof

- Ticket: `RIOTBOX-1300`
- Title: `Strengthen hook/chop policy professional proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1300/strengthen-hookchop-policy-professional-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-28`
- Started: `2026-06-28`
- Finished: `2026-06-28`
- Branch: `feature/riotbox-1300-hook-chop-policy-proof`
- Linear branch: `feature/riotbox-1300-strengthen-hookchop-policy-professional-proof`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1274 (https://github.com/marang/riotbox/pull/1274)`
- Merge commit: `f76af4a77c02e06445dce7709d5998ff1aece800`
- Deleted from Linear: `2026-06-28`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_tonal_hook_professional.py scripts/generate_professional_source_wav_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py scripts/generate_sound_quality_readiness_report.py scripts/route_weak_output_fixes.py; just tonal-hook-professional-fixtures; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1300-source-wav-rerun; just professional-output-listening-pack-smoke artifacts/audio_qa/local-riotbox-1300-listening-rerun; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1300-dense; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1300-matrix; just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1300-routing; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1300-suite-rerun; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1300-readiness-rerun; git diff --check; just ci; GitHub rust-ci passed on PR #1274`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still identified chop_policy as a recurring production fix. Hook/chop diagnostics needed W-30 headroom proof so barely present hook material cannot pass as musician-useful source-derived output.

## What Shipped

- Added hook_chop_w30_to_source_margin >= 0.025 for dense/tonal generated diagnostics; added tonal fixture w30_contribution_margin >= 0.050; propagated margins through matrix, source-WAV, professional suite, and readiness; routed margin failures to chop_policy; raised tonal W-30 presence and balanced tonal MC-202 support to keep source-composed listening gates green.

## Notes

- Observed dense/matrix W-30 margin 0.04400001010296986, tonal margin 0.038052826026274716, tonal listening gate mc202_to_w30_rms_ratio 0.16466555303627634. Evidence remains diagnostic and quality_proof false.

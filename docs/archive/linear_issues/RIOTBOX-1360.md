# `RIOTBOX-1360` P023: Strengthen source-derived W-30 chop diversity

- Ticket: `RIOTBOX-1360`
- Title: `P023: Strengthen source-derived W-30 chop diversity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1360/p023-strengthen-source-derived-w-30-chop-diversity`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1360-p023-strengthen-source-derived-w30-chop-diversity`
- Linear branch: `feature/riotbox-1360-p023-strengthen-source-derived-w-30-chop-diversity`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1324 (https://github.com/marang/riotbox/pull/1324)`
- Merge commit: `a8c4379c377c733adfc542bd4b59c0484c4eaec6`
- Deleted from Linear: `2026-07-01`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_professional_output_suite_contract.py; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1360-pro-pressure-source-matrix; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1360-professional-source-wav; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1360-professional-output-suite; just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1360-readiness; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1324`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Fresh P023 readiness after RIOTBOX-1359 still routed chop_policy as a top weak-output category, with routed reasons around too few W-30 trigger patterns, too few unique source offsets, flat accent dynamics, weak dominance, and missing response signature.

## What Shipped

- Hook-forward W-30 diagnostics now require at least 6 source offsets, 10 riff hits, and hook_chop_riff_velocity_span >= 0.25. The generator targets 6 source-derived riff starts and up to 12 hits, while dense-break W-30 riff gain is trimmed so the richer riff does not mask snare/break pressure. Final suite evidence: dense/matrix 6 offsets, 12 hits, velocity span 0.576, W-30/source margin 0.126, dense snare margin 0.2245; tonal Source-WAV 6 offsets, 12 hits, velocity span 0.536, W-30/source margin 0.158.

## Notes

- This remains scripted diagnostic evidence only: quality_proof=false and human_verdict=unverified; it improves W-30 chop/riff diversity proof but does not claim human musical pass or demo readiness.

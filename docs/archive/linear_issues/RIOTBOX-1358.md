# `RIOTBOX-1358` P023: Strengthen W-30 hook/chop from weak-output routing

- Ticket: `RIOTBOX-1358`
- Title: `P023: Strengthen W-30 hook/chop from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1358/p023-strengthen-w-30-hookchop-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-01`
- Started: `2026-07-01`
- Finished: `2026-07-01`
- Branch: `feature/riotbox-1358-p023-strengthen-w-30-hookchop-from-weak-output-routing`
- Linear branch: `feature/riotbox-1358-p023-strengthen-w-30-hookchop-from-weak-output-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1322 (https://github.com/marang/riotbox/pull/1322)`
- Merge commit: `3853b7a3903b99889649cd8300743b8a91e162d3`
- Deleted from Linear: `2026-07-01`
- Verification: `python -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_professional_output_suite_contract.py; just professional-source-wav-pack-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just sound-quality-readiness-report-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass on PR #1322`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 weak-output routing still identified chop_policy as a top production-fix category after the W-30 lane became loud enough; tonal hook/chop replacement could widen source-character span by replacing a stronger source grain with a weaker one.

## What Shipped

- Tonal hook/chop replacement now preserves the selected source-character floor, and dense/tonal professional diagnostics require hook_chop_source_character_score_floor >= 0.64 across generator, source-WAV pack, professional suite contract, and matrix smoke. Tonal source-WAV floor rose to 0.644205 while span remained 0.101409.

## Notes

- This remains diagnostic/scripted evidence only: quality_proof=false and human_verdict=unverified; it does not claim human musical pass.

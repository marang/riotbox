# `RIOTBOX-1213` Replace pressure_lift fixed recipe with source-aware lift policy

- Ticket: `RIOTBOX-1213`
- Title: `Replace pressure_lift fixed recipe with source-aware lift policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1213/replace-pressure-lift-fixed-recipe-with-source-aware-lift-policy`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1213-replace-pressure_lift-fixed-recipe-with-source-aware-lift`
- Linear branch: `feature/riotbox-1213-replace-pressure_lift-fixed-recipe-with-source-aware-lift`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1188 (https://github.com/marang/riotbox/pull/1188)`
- Merge commit: `3897ca6a85f56f64e0980f6120bd8354083f1692`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1213-final-dense-smoke; just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1213-final-matrix-smoke; just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1213-final-source-smoke; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1213-professional-output-suite; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed on PR #1188.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Replace the pressure_lift fixed recipe boundary with a visible source-aware lift policy while keeping scripted diagnostic evidence honest.

## What Shipped

- Added PressureLiftPolicy classification for dense-break, tonal-hook, sparse-bass-pressure, and thin/uncertain source profiles; applied policy to lift mix, bar intensity, bass frequencies, report proof, matrix/professional-source summaries, smoke gates, benchmark docs, and roadmap status while preserving diagnostic/unverified evidence boundaries.

## Notes

- None

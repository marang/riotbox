# `RIOTBOX-1225` Derive destructive stutter/restore choices from source candidates

- Ticket: `RIOTBOX-1225`
- Title: `Derive destructive stutter/restore choices from source candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1225/derive-destructive-stutterrestore-choices-from-source-candidates`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1225-derive-destructive-stutterrestore-choices-from-source`
- Linear branch: `feature/riotbox-1225-derive-destructive-stutterrestore-choices-from-source`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1198 (https://github.com/marang/riotbox/pull/1198)`
- Merge commit: `cf7257b82d7f643be3f865e233b4da9a9427452d`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_pro_pressure_source_matrix.py scripts/generate_professional_source_wav_pack.py scripts/generate_professional_output_suite.py scripts/validate_destructive_variation_professional.py`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1225-dense-smoke-2`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1225-source-wav-smoke-2`; `just destructive-variation-professional-smoke artifacts/audio_qa/local-riotbox-1225-destructive-smoke-2`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1225-suite-smoke-2`; `scripts/run_compact.sh /tmp/riotbox-1225-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-1225-just-ci.log just ci`; `gh pr checks 1198 --watch`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`, `docs/benchmarks/professional_output_suite_v1_2026-06-04.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224`

## Why This Ticket Existed

The destructive dropout/stutter/restore path still had fixed scripted cue choices: a fixed stutter beat offset and a source-prefix restore transient. That meant source-aware hook/chop proof could improve while the most playable destructive gesture still behaved like a static trick across different sources.

## What Shipped

- Added a bounded destructive_gesture_policy that scans source/W-30 candidates and selects separate stutter and restore offsets for dense-break, tonal-hook, and sparse-bass-pressure source families.
- Surfaced destructive source-derived proof, candidate counts, fixed-choice distance, and stutter/restore offset contrast through dense reports, destructive-variation validation, pro-pressure matrix, professional source WAV pack, and professional output suite metrics.
- Added positive smoke assertions and negative report mutations that reject non-source-derived or fixed-choice destructive gestures for dense/tonal diagnostic evidence.
- Documented the P022 contract and RBX-074 decision that dense/tonal destructive evidence must be source-derived before it counts as stronger professional-output proof.

## Notes

- This remains diagnostic scripted evidence with quality_proof false and human_verdict unverified; it is stronger source-derived evidence, not a human musical-quality pass.
- RIOTBOX-1224 tracks extracting the now-oversized Justfile audio-QA JSON contracts into validators without weakening gates.

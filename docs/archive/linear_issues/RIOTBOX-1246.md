# `RIOTBOX-1246` P023: Strengthen W-30 hook/chop policy for weak hook outputs

- Ticket: `RIOTBOX-1246`
- Title: `P023: Strengthen W-30 hook/chop policy for weak hook outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1246/p023-strengthen-w-30-hookchop-policy-for-weak-hook-outputs`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1246-p023-hook-chop-policy`
- Linear branch: `feature/riotbox-1246-p023-strengthen-w-30-hookchop-policy-for-weak-hook-outputs`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1221 (https://github.com/marang/riotbox/pull/1221)`
- Merge commit: `ee932c1cf76f9782a51ec0539087b6d2759c8bb7`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1246-dense-break; just professional-source-wav-pack-smoke; just professional-output-suite-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1247, RIOTBOX-1248`

## Why This Ticket Existed

Strengthened weak tonal hook/chop output so W-30 source material carries a clearer first-two-bar riff/stab instead of barely clearing the old hook floor.

## What Shipped

- Added source-family-aware W-30 gain calibration; raised tonal-hook W-30/source diagnostic floor to 0.20; added visible Just gate; documented the diagnostic boundary.

## Notes

- Evidence remains diagnostic only: human_verdict unverified, not an automated musical pass.

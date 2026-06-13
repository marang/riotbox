# `RIOTBOX-1257` P023: Strengthen physical drum pressure after weak-output fixes

- Ticket: `RIOTBOX-1257`
- Title: `P023: Strengthen physical drum pressure after weak-output fixes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1257/p023-strengthen-physical-drum-pressure-after-weak-output-fixes`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1257-p023-drum-pressure`
- Linear branch: `feature/riotbox-1257-p023-strengthen-physical-drum-pressure-after-weak-output`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1231 (https://github.com/marang/riotbox/pull/1231)`
- Merge commit: `bd0d48f7`
- Deleted from Linear: `2026-06-13`
- Verification: `py_compile changed Python; dense-break smoke; destructive-variation smoke; professional-output suite smoke; readiness validation; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak-output routing still listed drum_pressure and dense-break pressure had too little explicit proof that pressure-lift retained break/snare transient instead of only low-band support.

## What Shipped

- Dense-break pressure gained source-family-limited TR-909/break-snap emphasis, a pressure-bar snap accent, restored Rise/Drop/Restore balance, and a new dense_break_pressure_transient_to_hook_ratio gate wired through dense reports, professional suite, readiness, Just targets, and Audio QA spec.

## Notes

- Evidence remains diagnostic only: human_verdict unverified and quality_proof false.

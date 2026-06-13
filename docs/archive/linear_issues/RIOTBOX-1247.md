# `RIOTBOX-1247` P023: Strengthen drum pressure for physical break impact

- Ticket: `RIOTBOX-1247`
- Title: `P023: Strengthen drum pressure for physical break impact`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1247/p023-strengthen-drum-pressure-for-physical-break-impact`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1247-p023-drum-pressure-impact`
- Linear branch: `feature/riotbox-1247-p023-strengthen-drum-pressure-for-physical-break-impact`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1222 (https://github.com/marang/riotbox/pull/1222)`
- Merge commit: `19b64e64fa54d2fc662a2e16e21bfd76f92bf0d3`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py; just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1247-dense-break; just professional-output-suite-smoke; just pro-pressure-source-matrix-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Dense-break output needed a bounded physical drum-pressure contract instead of only generic strongest-element proof.

## What Shipped

- Raised dense-break TR-909/break-snap drive, added dense snare dominance and physical drum-pressure proof fields/failure codes, added a negative dense drum-pressure smoke fixture, and documented the P023 diagnostic boundary.

## Notes

- The new proof remains diagnostic-only: quality_proof false and human_verdict unverified; a parallel audio-qa run showed an artifact-writer race, so audio-qa-ci was rerun isolated and passed.

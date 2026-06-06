# `RIOTBOX-1228` Derive dropout/restore tail shape from source candidates

- Ticket: `RIOTBOX-1228`
- Title: `Derive dropout/restore tail shape from source candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1228/derive-dropoutrestore-tail-shape-from-source-candidates`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1228-derive-dropoutrestore-tail-shape-from-source-candidates`
- Linear branch: `feature/riotbox-1228-derive-dropoutrestore-tail-shape-from-source-candidates`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1202 (https://github.com/marang/riotbox/pull/1202)`
- Merge commit: `ea7373de27777bea55e07b29c10c0f46ffea4a69`
- Deleted from Linear: `2026-06-06`
- Verification: `py_compile dense/matrix/source/suite scripts; git diff --check; dense-break-performance-pack-smoke; pro-pressure-source-matrix-smoke; professional-source-wav-pack-smoke; professional-output-suite-smoke; just ci (includes audio-qa-ci); GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224 continues Justfile/audio-QA validator extraction`

## Why This Ticket Existed

Dropout/restore tails were still one fixed destructive ending after hook/chop, cue selection, bass movement, arrangement, mix treatment, and pad/noise texture became source-derived.

## What Shipped

- Added bounded source-derived TailShapePolicy, applied it to dropout/stutter and restore rendering, surfaced tail_shape proof through dense/matrix/source-WAV/suite reports, gated fixed-collapse mutations, and documented RBX-078 while keeping quality_proof false and human_verdict unverified.

## Notes

- Tail proof remains diagnostic: quality_proof=false and human_verdict=unverified until structured listening review labels approve product-quality output.

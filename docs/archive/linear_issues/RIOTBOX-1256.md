# `RIOTBOX-1256` P023: Strengthen source-window selection from human weak-output candidates

- Ticket: `RIOTBOX-1256`
- Title: `P023: Strengthen source-window selection from human weak-output candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1256/p023-strengthen-source-window-selection-from-human-weak-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1256-p023-source-window-selection`
- Linear branch: `feature/riotbox-1256-p023-strengthen-source-window-selection-from-human-weak`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1230 (https://github.com/marang/riotbox/pull/1230)`
- Merge commit: `1692a142`
- Deleted from Linear: `2026-06-13`
- Verification: `py_compile changed Python; focused weak-routing/dense/source-matrix/source-wav/professional-suite/readiness checks; just audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Human weak-output candidates showed source character could be lost while still routing through generic production-fix flow; source-window selection needed a stricter product contract.

## What Shipped

- Source-character weighting is stronger in dense and tonal hook/chop selection, hook/chop source-character gates are raised from 0.55 to 0.60 across suite/readiness contracts, and human source_lost labels now prove source_selection routing.

## Notes

- Evidence remains diagnostic only: human_verdict unverified and quality_proof false.

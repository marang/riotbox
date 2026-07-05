# `RIOTBOX-1392` P023: Extract human listening label corpus fixtures

- Ticket: `RIOTBOX-1392`
- Title: `P023: Extract human listening label corpus fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1392/p023-extract-human-listening-label-corpus-fixtures`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1392-p023-extract-human-listening-label-corpus-fixtures`
- Linear branch: `feature/riotbox-1392-p023-extract-human-listening-label-corpus-fixtures`
- Assignee: `Markus`
- Labels: None
- PR: `#1356 (https://github.com/marang/riotbox/pull/1356)`
- Merge commit: `a0948c6de94e79dcd76e011355691d3e3a8a9c2f`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move human listening label corpus fixture checks out of Justfile and into a named validator without weakening the human-listening label corpus gate.

## What Shipped

- human-listening-label-corpus-fixtures now calls scripts/validate_human_listening_label_corpus_fixtures.py for valid corpus summary checks, verdict-count checks, source-family checks, and invalid bad-hash / weak-missing-reason rejection.

## Notes

- None

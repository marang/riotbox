# `RIOTBOX-1391` P023: Extract listening review label import fixtures

- Ticket: `RIOTBOX-1391`
- Title: `P023: Extract listening review label import fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1391/p023-extract-listening-review-label-import-fixtures`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1391-p023-extract-listening-review-label-import-fixtures`
- Linear branch: `feature/riotbox-1391-p023-extract-listening-review-label-import-fixtures`
- Assignee: `Markus`
- Labels: None
- PR: `#1355 (https://github.com/marang/riotbox/pull/1355)`
- Merge commit: `9385325ffcb802620e0555cbbcc135bebf7eb837`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move general listening-review label import fixtures out of Justfile and into a named validator without weakening the human-listening label import gate.

## What Shipped

- listening-review-label-import-fixtures now calls scripts/validate_listening_review_label_import_fixtures.py for valid weak-label import checks, human label corpus validation, and missing-metadata rejection.

## Notes

- None

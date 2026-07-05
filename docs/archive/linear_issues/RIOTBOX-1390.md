# `RIOTBOX-1390` P023: Extract professional output listening verdict import fixtures

- Ticket: `RIOTBOX-1390`
- Title: `P023: Extract professional output listening verdict import fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1390/p023-extract-professional-output-listening-verdict-import-fixtures`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1390-p023-extract-professional-output-listening-verdict-import`
- Linear branch: `feature/riotbox-1390-p023-extract-professional-output-listening-verdict-import`
- Assignee: `Markus`
- Labels: None
- PR: `#1354 (https://github.com/marang/riotbox/pull/1354)`
- Merge commit: `4227ad07ff9779215815596147ed1435ca4fb2cb`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move professional output listening verdict import fixtures out of Justfile and into a named validator without weakening the human-listening label import gate.

## What Shipped

- professional-output-listening-verdict-import-fixtures now calls scripts/validate_professional_output_listening_verdict_import_fixtures.py for keep-verdict import checks, human label corpus field checks, unverified-review rejection, and stale artifact-hash rejection.

## Notes

- None

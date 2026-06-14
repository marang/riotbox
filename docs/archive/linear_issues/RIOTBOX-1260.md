# `RIOTBOX-1260` P023: Move professional-output suite smoke assertions into validator

- Ticket: `RIOTBOX-1260`
- Title: `P023: Move professional-output suite smoke assertions into validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1260/p023-move-professional-output-suite-smoke-assertions-into-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-14`
- Finished: `2026-06-14`
- Branch: `feature/riotbox-1260-p023-professional-suite-validator-contract`
- Linear branch: `feature/riotbox-1260-p023-move-professional-output-suite-smoke-assertions-into`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1235 (https://github.com/marang/riotbox/pull/1235)`
- Merge commit: `613642e7ea3a68802d5d9019b0fe30549512cb35`
- Deleted from Linear: `2026-06-14`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The P023 professional-output suite smoke still carried large inline jq mutation assertions even though the spec requires large JSON contracts to live in named repo-local validators.

## What Shipped

- Moved seven professional-output negative mutation fixtures into validate_professional_output_suite_contract.py behind --mutation-fixtures and reduced the Justfile recipe to generate the suite and run the validator.

## Notes

- None

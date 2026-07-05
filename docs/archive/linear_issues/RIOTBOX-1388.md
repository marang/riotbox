# `RIOTBOX-1388` P023: Extract rendered weak professional output smoke validator

- Ticket: `RIOTBOX-1388`
- Title: `P023: Extract rendered weak professional output smoke validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1388/p023-extract-rendered-weak-professional-output-smoke-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1388-p023-extract-rendered-weak-professional-output-smoke`
- Linear branch: `feature/riotbox-1388-p023-extract-rendered-weak-professional-output-smoke`
- Assignee: `Markus`
- Labels: None
- PR: `#1352 (https://github.com/marang/riotbox/pull/1352)`
- Merge commit: `90a9c797b4813ecf7bc00e45ac0c4ba2d40215c5`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move rendered weak professional-output smoke checks out of Justfile and into a named validator without changing the negative diagnostic artifact.

## What Shipped

- rendered-weak-professional-output-fixtures now calls scripts/validate_rendered_weak_professional_outputs_smoke.py for report schema and boundary checks, required destructive failure codes, required rendered artifacts, quality-claim rejection, stale case-count mutation, missing failure-code mutation, and missing artifact rejection.

## Notes

- None

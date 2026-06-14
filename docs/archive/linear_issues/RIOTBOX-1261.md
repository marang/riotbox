# `RIOTBOX-1261` P023: Move sound-quality readiness smoke assertions into validator

- Ticket: `RIOTBOX-1261`
- Title: `P023: Move sound-quality readiness smoke assertions into validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1261/p023-move-sound-quality-readiness-smoke-assertions-into-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-14`
- Finished: `2026-06-14`
- Branch: `feature/riotbox-1261-p023-readiness-validator-contract`
- Linear branch: `feature/riotbox-1261-p023-move-sound-quality-readiness-smoke-assertions-into`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1236 (https://github.com/marang/riotbox/pull/1236)`
- Merge commit: `cfa9da39ed5f717becb6e4a6e6693e584c910f5c`
- Deleted from Linear: `2026-06-14`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The P023 sound-quality readiness smoke still had an oversized inline jq assertion for core release-blocker invariants.

## What Shipped

- Moved the large positive readiness-report contract into generate_sound_quality_readiness_report.py --validate-report and left the Justfile recipe to generate, validate, and run compact negative mutations.

## Notes

- None

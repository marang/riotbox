# `RIOTBOX-1386` P023: Extract sound-quality readiness smoke validator

- Ticket: `RIOTBOX-1386`
- Title: `P023: Extract sound-quality readiness smoke validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1386/p023-extract-sound-quality-readiness-smoke-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1386-p023-extract-sound-quality-readiness-smoke-validator`
- Linear branch: `feature/riotbox-1386-p023-extract-sound-quality-readiness-smoke-validator`
- Assignee: `Markus`
- Labels: None
- PR: `#1350 (https://github.com/marang/riotbox/pull/1350)`
- Merge commit: `87ab45795bc3980b3b36bc3e32d4bdff6e2a29bc`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move the oversized sound-quality readiness smoke checks out of Justfile and into a named validator without weakening P023 release-readiness proof.

## What Shipped

- sound-quality-readiness-report-smoke now calls scripts/validate_sound_quality_readiness_smoke.py for Markdown worklist checks and stale/missing-context mutation fixtures, while the recipe keeps only artifact generation and validator handoff.

## Notes

- None

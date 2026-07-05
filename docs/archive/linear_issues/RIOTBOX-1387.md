# `RIOTBOX-1387` P023: Extract weak-output routing smoke validator

- Ticket: `RIOTBOX-1387`
- Title: `P023: Extract weak-output routing smoke validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1387/p023-extract-weak-output-routing-smoke-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1387-p023-extract-weak-output-routing-smoke-validator`
- Linear branch: `feature/riotbox-1387-p023-extract-weak-output-routing-smoke-validator`
- Assignee: `Markus`
- Labels: None
- PR: `#1351 (https://github.com/marang/riotbox/pull/1351)`
- Merge commit: `d7034879051de7f08df4a2e89d45321b53c396a8`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move oversized weak-output routing smoke checks out of Justfile and into a named validator without weakening P023 production-fix routing proof.

## What Shipped

- weak-output-fix-routing-fixtures now calls scripts/validate_weak_output_fix_routing_smoke.py for routed-case checks, production-fix candidate integrity, Markdown assertions, stale-count mutations, unknown case/manifest fixtures, and duplicate-category rejection.

## Notes

- None

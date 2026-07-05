# `RIOTBOX-1389` P023: Extract destructive variation professional smoke validator

- Ticket: `RIOTBOX-1389`
- Title: `P023: Extract destructive variation professional smoke validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1389/p023-extract-destructive-variation-professional-smoke-validator`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1389-p023-extract-destructive-variation-professional-smoke`
- Linear branch: `feature/riotbox-1389-p023-extract-destructive-variation-professional-smoke`
- Assignee: `Markus`
- Labels: None
- PR: `#1353 (https://github.com/marang/riotbox/pull/1353)`
- Merge commit: `9fe3e3a65491725fb5a53124ff53b37d8005bc95`
- Deleted from Linear: `2026-07-05`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Move destructive-variation professional smoke checks out of Justfile and into a named validator without changing the diagnostic report semantics.

## What Shipped

- destructive-variation-professional-smoke now calls scripts/validate_destructive_variation_professional_smoke.py for report schema and evidence-boundary checks, destructive threshold comparisons, Markdown boundary checks, stale metric mutations, quality-claim rejection, and invalid flat-stutter failure-code checks.

## Notes

- None

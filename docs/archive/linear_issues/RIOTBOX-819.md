# `RIOTBOX-819` Validate Source Timing example expectation range schema

- Ticket: `RIOTBOX-819`
- Title: `Validate Source Timing example expectation range schema`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-819/validate-source-timing-example-expectation-range-schema`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-819-source-timing-expectation-range-schema`
- Linear branch: `feature/riotbox-819-validate-source-timing-example-expectation-range-schema`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#814 (https://github.com/marang/riotbox/pull/814)`
- Merge commit: `720d4f438b0e823b17105865e317fddbe8c0d807`
- Verification: `GitHub Actions Rust CI #1981 passed; local just ci passed; source-timing example report fixtures passed; source-timing readiness report gate passed.`
- Docs touched: `None`
- Follow-ups: `None from this slice.`

## Why This Ticket Existed

RIOTBOX-818 found that malformed numeric range expectations could silently weaken or confuse the Source Timing example report regression surface.

## What Shipped

- Added strict min/max range expectation schema validation, invalid empty/inverted/unknown-key fixtures, and wired them into the source-timing example report fixture gate.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.

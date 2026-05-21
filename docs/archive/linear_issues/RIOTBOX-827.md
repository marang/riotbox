# `RIOTBOX-827` Document strict Source Timing example expectation schema

- Ticket: `RIOTBOX-827`
- Title: `Document strict Source Timing example expectation schema`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-827/document-strict-source-timing-example-expectation-schema`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-827-source-timing-expectation-schema-doc`
- Linear branch: `feature/riotbox-827-document-strict-source-timing-example-expectation-schema`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#822 (https://github.com/marang/riotbox/pull/822)`
- Merge commit: `7a26e7fea4551449007e461f4c14ac3dd7bb378f`
- Verification: `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #2005 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing spec described optional example report expectations but did not record the stricter schema behavior added by RIOTBOX-826 for unknown keys and malformed numeric ranges.

## What Shipped

- Updated docs/specs/source_timing_intelligence_spec.md to state that unknown top-level expectation keys and malformed numeric range expectation objects must fail the fixture gate.

## Notes

- None

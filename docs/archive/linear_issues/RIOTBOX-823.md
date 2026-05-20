# `RIOTBOX-823` Assert Source Timing example report missing-source row behavior

- Ticket: `RIOTBOX-823`
- Title: `Assert Source Timing example report missing-source row behavior`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-823/assert-source-timing-example-report-missing-source-row-behavior`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-823-source-timing-report-missing-row-fixture`
- Linear branch: `feature/riotbox-823-assert-source-timing-example-report-missing-source-row`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#818 (https://github.com/marang/riotbox/pull/818)`
- Merge commit: `cea698a12e3955dab04fbc6b786f7341943b5e07`
- Verification: `python3 -m py_compile scripts/assert_source_timing_example_report_fixtures.py`; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #1993 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing example report promises fresh-clone safety for intentionally uncommitted local example WAVs, but the field-level fixture assertor did not yet assert expected missing-source rows.

## What Shipped

- Extended scripts/assert_source_timing_example_report_fixtures.py with a missing expected-source assertion proving status missing, expectation skipped, and rendered Markdown row presence while keeping existing report behavior unchanged.

## Notes

- None

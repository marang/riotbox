# `RIOTBOX-822` Assert Source Timing example report fixtures by fields

- Ticket: `RIOTBOX-822`
- Title: `Assert Source Timing example report fixtures by fields`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-822/assert-source-timing-example-report-fixtures-by-fields`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-822-source-timing-report-field-fixtures`
- Linear branch: `feature/riotbox-822-assert-source-timing-example-report-fixtures-by-fields`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#817 (https://github.com/marang/riotbox/pull/817)`
- Merge commit: `058224632cd8bbc73c7e8b2d32869de5911291e9`
- Verification: `python3 -m py_compile scripts/assert_source_timing_example_report_fixtures.py scripts/source_timing_example_probe_report.py scripts/source_timing_example_expectations.py`; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #1990 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing example report fixture gate relied on long full-row Markdown grep strings, making each report-column change harder to review and maintain.

## What Shipped

- Added scripts/assert_source_timing_example_report_fixtures.py to assert existing committed Source Timing report rows by ReportRow field, including positive rows, mismatch expectations, invalid range expectations, and Markdown render presence; simplified just source-timing-example-probe-report-fixtures to call the helper.

## Notes

- None

# `RIOTBOX-825` Validate Source Timing report probe fixtures inside the field assertor

- Ticket: `RIOTBOX-825`
- Title: `Validate Source Timing report probe fixtures inside the field assertor`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-825/validate-source-timing-report-probe-fixtures-inside-the-field-assertor`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-825-report-fixture-schema-validation`
- Linear branch: `feature/riotbox-825-validate-source-timing-report-probe-fixtures-inside-the`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#820 (https://github.com/marang/riotbox/pull/820)`
- Merge commit: `774a490431430d9aeb9d43dc9d34d182b62367a4`
- Verification: `python3 -m py_compile scripts/assert_source_timing_example_report_fixtures.py`; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #1999 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing example report fixture gate derived ReportRow assertions from committed probe JSON fixtures without also validating those fixtures against the existing probe JSON schema in the durable Just target.

## What Shipped

- Updated scripts/assert_source_timing_example_report_fixtures.py to run validate_source_timing_probe_json.validate_summary(...) for every positive report probe fixture before field assertions.

## Notes

- None

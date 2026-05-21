# `RIOTBOX-830` Add exact warning-code expectations to Source Timing example report

- Ticket: `RIOTBOX-830`
- Title: `Add exact warning-code expectations to Source Timing example report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-830/add-exact-warning-code-expectations-to-source-timing-example-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-830-source-timing-warning-codes-exact`
- Linear branch: `feature/riotbox-830-add-exact-warning-code-expectations-to-source-timing-example`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#825 (https://github.com/marang/riotbox/pull/825)`
- Merge commit: `35fe243e9a249e13df94d64f251aa81eb2e69edf`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; `python3 -m json.tool` on changed/new expectation fixtures; `just source-timing-example-probe-report-fixtures`; `just source-timing-example-probe-report-local`; `just ci`; GitHub Actions Rust CI #2014
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-828 found that Source Timing example expectations could only assert warning-code inclusion, so unexpected warnings could slip through rows that should be warning-free or exact.

## What Shipped

- Added warning_codes_exact, rejected mixing exact and include warning expectations, tightened committed/local example warning expectations, updated the Source Timing expectation contract, and added exact-mismatch plus invalid-mix fixture coverage.

## Notes

- None

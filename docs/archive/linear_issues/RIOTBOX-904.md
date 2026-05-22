# `RIOTBOX-904` Add phrase-count checks to Source Timing example expectations

- Ticket: `RIOTBOX-904`
- Title: `Add phrase-count checks to Source Timing example expectations`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-904/add-phrase-count-checks-to-source-timing-example-expectations`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-904-source-timing-example-phrase-count-expectations`
- Linear branch: `feature/riotbox-904-add-phrase-count-checks-to-source-timing-example`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#897 (https://github.com/marang/riotbox/pull/897)`
- Merge commit: `b7961d9f8e160d69b58d8d93321670d2de40a506`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py; just source-timing-example-probe-report-fixtures; just source-timing-example-probe-report-local /tmp/riotbox-904-source-timing-report-2.md; git diff --check; just ci; GitHub Rust CI #2234`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The local Source Timing example report rendered phrase count and phrase bars, but the expectation schema could not assert those values, so local report rows could still say ok if phrase evidence drifted.

## What Shipped

- Added exact integer expectation support for primary_phrase_count and primary_phrase_bar_count, updated committed and local example expectations, and tightened mismatch coverage.

## Notes

- None

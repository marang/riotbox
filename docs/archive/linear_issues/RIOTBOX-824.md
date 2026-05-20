# `RIOTBOX-824` Add weak and unavailable Source Timing example report fixtures

- Ticket: `RIOTBOX-824`
- Title: `Add weak and unavailable Source Timing example report fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-824/add-weak-and-unavailable-source-timing-example-report-fixtures`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-824-source-timing-report-weak-unavailable-fixtures`
- Linear branch: `feature/riotbox-824-add-weak-and-unavailable-source-timing-example-report`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#819 (https://github.com/marang/riotbox/pull/819)`
- Merge commit: `6e07b5b7efaccfcacd2acd25d2f5a506404716ee`
- Verification: `python3 scripts/validate_source_timing_probe_json.py scripts/fixtures/source_timing_example_probe_report/probe_weak_ambiguous_flat_pulse.json`; `python3 scripts/validate_source_timing_probe_json.py scripts/fixtures/source_timing_example_probe_report/probe_unavailable_silence.json`; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #1996 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The committed Source Timing example report gate covered ready and needs-review rows but did not prove weak ambiguous or unavailable degraded rows without uncommitted local example WAVs.

## What Shipped

- Added committed weak ambiguous and unavailable degraded Source Timing probe JSON fixtures and extended scripts/assert_source_timing_example_report_fixtures.py to assert their field-level report output.

## Notes

- None

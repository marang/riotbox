# `RIOTBOX-831` Expose downbeat offset in Source Timing example report

- Ticket: `RIOTBOX-831`
- Title: `Expose downbeat offset in Source Timing example report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-831/expose-downbeat-offset-in-source-timing-example-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-831-source-timing-downbeat-offset-report`
- Linear branch: `feature/riotbox-831-expose-downbeat-offset-in-source-timing-example-report`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#826 (https://github.com/marang/riotbox/pull/826)`
- Merge commit: `cc58ba2b6b0a84c988a9195e6f5c6eaf322aa887`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; `python3 -m json.tool scripts/fixtures/source_timing_example_probe_report/beat08_expectations.json`; `just source-timing-example-probe-report-fixtures`; `just source-timing-example-probe-report-local`; `just ci`; GitHub Actions Rust CI #2017
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing probe JSON already exposed primary_downbeat_offset_beats, but the compact example report hid the selected bar-phase offset from reviewers.

## What Shipped

- Added a Downbeat offset report column, rendered unavailable offsets as none, asserted committed fixture offsets, added primary_downbeat_offset_beats expectation support, and updated the Source Timing report contract.

## Notes

- None

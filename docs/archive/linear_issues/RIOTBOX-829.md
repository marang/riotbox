# `RIOTBOX-829` Catch malformed Source Timing example report fixture TypeErrors

- Ticket: `RIOTBOX-829`
- Title: `Catch malformed Source Timing example report fixture TypeErrors`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-829/catch-malformed-source-timing-example-report-fixture-typeerrors`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-829-source-timing-malformed-fixture-errors`
- Linear branch: `feature/riotbox-829-catch-malformed-source-timing-example-report-fixture`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#824 (https://github.com/marang/riotbox/pull/824)`
- Merge commit: `eea001213125435496dd24a1d0ccc8730c70d64e`
- Verification: `python3 -m py_compile scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; malformed fixture CLI smoke; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #2010
- Docs touched: `None`
- Follow-ups: `RIOTBOX-830`

## Why This Ticket Existed

RIOTBOX-828 found that malformed Source Timing example report fixture JSON could escape as Python tracebacks instead of the bounded CLI error path.

## What Shipped

- Caught TypeError in the Source Timing example report CLI bounded error path, added a malformed missing-source_path fixture, and wired a process-level fixture assertion proving the compact error prefix and no traceback.

## Notes

- None

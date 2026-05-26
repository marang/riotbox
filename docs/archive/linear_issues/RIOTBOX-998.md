# `RIOTBOX-998` Assert local example groove residual counts

- Ticket: `RIOTBOX-998`
- Title: `Assert local example groove residual counts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-998/assert-local-example-groove-residual-counts`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-998-assert-local-example-groove-residual-counts`
- Linear branch: `feature/riotbox-998-assert-local-example-groove-residual-counts`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: `#990 (https://github.com/marang/riotbox/pull/990)`
- Merge commit: `e381060440d221ca79ccbb984b151f2a6809f142`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-998-example-report-fixtures.log just source-timing-example-probe-report-fixtures`; `scripts/run_compact.sh /tmp/riotbox-998-example-report-local.log just source-timing-example-probe-report-local /tmp/riotbox-998-source-timing-report.md`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-998-just-ci.log just ci`; `GitHub Actions Rust CI run 26450329476 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Pin compact groove residual evidence in local Source Timing expectations so report drift is visible alongside beat, downbeat, and anchor evidence.

## What Shipped

- Added nested groove_evidence expectation support for primary_groove_residual_count.
- Added invalid groove expectation fixtures and mismatch coverage.
- Populated local example expectations with current groove residual counts.

## Notes

- No groove extraction/scoring, probe analyzer, readiness policy, UI, Session, realtime audio, or output behavior changed.

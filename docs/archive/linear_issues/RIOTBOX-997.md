# `RIOTBOX-997` Assert local example downbeat offsets

- Ticket: `RIOTBOX-997`
- Title: `Assert local example downbeat offsets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-997/assert-local-example-downbeat-offsets`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-997-assert-local-example-downbeat-offsets`
- Linear branch: `feature/riotbox-997-assert-local-example-downbeat-offsets`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: `#989 (https://github.com/marang/riotbox/pull/989)`
- Merge commit: `55a0b99915636f7c38e3a9b34300b52656706505`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/source_timing_example_probe_report.py scripts/assert_source_timing_example_report_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-997-example-report-fixtures.log just source-timing-example-probe-report-fixtures`; `scripts/run_compact.sh /tmp/riotbox-997-example-report-local.log just source-timing-example-probe-report-local /tmp/riotbox-997-source-timing-report.md`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-997-just-ci.log just ci`; `GitHub Actions Rust CI run 26449830703 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Pin selected downbeat offsets in local Source Timing examples so phase drift is visible while ambiguous rows remain manual-confirm.

## What Shipped

- Added primary_downbeat_offset_beats expectations to local real-source rows with available offsets.
- Kept Beat20 manual-confirm-only and ambiguous while asserting its selected offset as evidence, not confidence.
- Added fixture coverage proving downbeat-offset expectation mismatches fail.

## Notes

- No downbeat scoring, analyzer, readiness policy, UI, Session, realtime audio, or output behavior changed.

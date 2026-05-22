# `RIOTBOX-958` Expose beat/bar counts in Source Timing example probe report

- Ticket: `RIOTBOX-958`
- Title: `Expose beat/bar counts in Source Timing example probe report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-958/expose-beatbar-counts-in-source-timing-example-probe-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-958-expose-beatbar-counts-in-source-timing-example-probe-report`
- Linear branch: `feature/riotbox-958-expose-beatbar-counts-in-source-timing-example-probe-report`
- Assignee: `Markus`
- Labels: None
- PR: `#951 (https://github.com/marang/riotbox/pull/951)`
- Merge commit: `1c5babc6e8033042a6b2435048f7b6d7c5d96cc3`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check`; `python3 -m py_compile scripts/source_timing_example_probe_report.py scripts/source_timing_example_expectations.py scripts/assert_source_timing_example_report_fixtures.py scripts/validate_source_timing_probe_json.py`; `scripts/run_compact.sh /tmp/riotbox-958-report-fixtures-2.log just source-timing-example-probe-report-fixtures`; `scripts/run_compact.sh /tmp/riotbox-958-local-report-2.log just source-timing-example-probe-report-local /tmp/riotbox-958-source-timing-report-2.md`; `scripts/run_compact.sh /tmp/riotbox-958-probe-bin-tests-2.log cargo test -p riotbox-audio --bin source_timing_probe -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-958-probe-validator-2.log just source-timing-probe-json-validator-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-958-just-ci.log just ci`; `GitHub Actions Rust CI run 26303849147 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The local Source Timing example probe report showed stable timing labels and phrase evidence but hid beat/bar grid counts, forcing reviewers to infer how much selected-grid evidence backed a real-source row.

## What Shipped

- Added primary_beat_count and primary_bar_count to the source_timing_probe JSON summary using selected primary-hypothesis grids before top-level fallback.
- Rendered Beat count and Bar count columns in the Source Timing example report and pinned committed/local expectations.
- Tightened the probe JSON validator, refreshed the local benchmark note, and documented the expectation contract.

## Notes

- Probe/report/validator surface only; no analyzer scoring, ActionCommand, queue, Session/replay, JamAppState, app observer schema, realtime audio, or render behavior changed.

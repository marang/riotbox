# `RIOTBOX-769` Add optional expectations to Source Timing example report

- Ticket: `RIOTBOX-769`
- Title: `Add optional expectations to Source Timing example report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-769/add-optional-expectations-to-source-timing-example-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-769-add-optional-expectations-to-source-timing-example-report`
- Linear branch: `feature/riotbox-769-add-optional-expectations-to-source-timing-example-report`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#763 (https://github.com/marang/riotbox/pull/763)`
- Merge commit: `177ba6d1ae9c96e54bcb292442b8ca5696c37f9a`
- Deleted from Linear: `2026-05-11`
- Verification: `python3 -m py_compile scripts/source_timing_example_probe_report.py; just source-timing-example-probe-report-fixtures; missing-source expectations smoke; git diff --check; GitHub Actions Rust CI passed on PR #763`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The real-example Source Timing report was observational; optional expectations were needed to flag conservative timing-surface regressions when local example WAVs are present while still skipping cleanly in fresh clones.

## What Shipped

- Added --expectations support to the source timing example probe report, conservative field comparisons, pass and mismatch fixtures, missing-source skipped behavior, Just fixture coverage, and a Source Timing spec note.

## Notes

- None

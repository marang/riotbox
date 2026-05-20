# `RIOTBOX-821` Split Source Timing example report expectations before next expansion

- Ticket: `RIOTBOX-821`
- Title: `Split Source Timing example report expectations before next expansion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-821/split-source-timing-example-report-expectations-before-next-expansion`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-821-source-timing-report-expectations-module`
- Linear branch: `feature/riotbox-821-split-source-timing-example-report-expectations-before-next`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#816 (https://github.com/marang/riotbox/pull/816)`
- Merge commit: `8a335a3304a9a9487ecbec6bac36c2b05f3389a2`
- Verification: `python3 -m py_compile scripts/source_timing_example_probe_report.py scripts/source_timing_example_expectations.py`; `just source-timing-example-probe-report-fixtures`; `just source-timing-example-probe-report-local /tmp/riotbox-821-source-timing-report.md`; `just ci`; GitHub Actions Rust CI #1987 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing example report mixed CLI rendering with expectation parsing and comparison after the beat-evidence expansion, raising review cost before further P012 report work.

## What Shipped

- Extracted expectation loading, mismatch formatting, BPM/range/warning comparisons, and expectation schema validation into scripts/source_timing_example_expectations.py while preserving the existing Markdown report behavior and Source Timing probe contract.

## Notes

- None

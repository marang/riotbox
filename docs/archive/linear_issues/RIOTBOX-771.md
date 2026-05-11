# `RIOTBOX-771` Capture current Source Timing example report baseline

- Ticket: `RIOTBOX-771`
- Title: `Capture current Source Timing example report baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-771/capture-current-source-timing-example-report-baseline`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-771-capture-current-source-timing-example-report-baseline`
- Linear branch: `feature/riotbox-771-capture-current-source-timing-example-report-baseline`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#765 (https://github.com/marang/riotbox/pull/765)`
- Merge commit: `db3a85eec11f934e3ffece9b760b3f2a1b188682`
- Deleted from Linear: `2026-05-11`
- Verification: `just source-timing-example-probe-report-local artifacts/audio_qa/local/source_timing_example_probe_report_expected.md`; `git diff --check main...HEAD`; `manual Markdown hygiene check for tabs and trailing newline`
- Docs touched: `docs/benchmarks/README.md`, `docs/benchmarks/source_timing_example_probe_report_2026-05-11.md`
- Follow-ups: `None`

## Why This Ticket Existed

The optional local Source Timing expectations file needed a durable benchmark note showing the current real-example Beat/DH report table, while keeping the example WAVs outside Git and the check local-only.

## What Shipped

- Added a Source Timing example probe benchmark note with the current local report table and interpretation.
- Linked the benchmark note from docs/benchmarks/README.md.

## Notes

- This is a local review baseline, not a mandatory CI gate or arbitrary-audio detector claim.

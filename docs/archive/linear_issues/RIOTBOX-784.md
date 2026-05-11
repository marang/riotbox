# `RIOTBOX-784` Add Source Timing example report fixtures for locked and manual-confirm rows

- Ticket: `RIOTBOX-784`
- Title: `Add Source Timing example report fixtures for locked and manual-confirm rows`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-784/add-source-timing-example-report-fixtures-for-locked-and-manual`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-784-source-timing-example-report-fixtures`
- Linear branch: `feature/riotbox-784-add-source-timing-example-report-fixtures-for-locked-and`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#779 (https://github.com/marang/riotbox/pull/779)`
- Merge commit: `56f3a6cae6872b91cb399ff368528b68cd700bdb`
- Deleted from Linear: `2026-05-11`
- Verification: `python3 -m json.tool scripts/fixtures/source_timing_example_probe_report/beat08_source_timing_probe.json >/tmp/riotbox-784-beat08.json`; `python3 -m json.tool scripts/fixtures/source_timing_example_probe_report/beat08_expectations.json >/tmp/riotbox-784-expectations.json`; `just source-timing-example-probe-report-fixtures`; `cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The Source Timing example report needed committed fixture coverage for both the conservative short-loop manual-confirm row and the long stable locked-grid row so the musician-facing/debugging report stays aligned with the probe contract.

## What Shipped

- Updated the Beat08 report fixture to the current short_loop_manual_confirm boundary.
- Added the locked-grid probe fixture to the example report smoke and asserted both report rows through expectations.

## Notes

- No real example WAVs were committed and no detector thresholds changed.

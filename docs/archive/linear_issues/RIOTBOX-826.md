# `RIOTBOX-826` Reject unknown Source Timing example expectation keys

- Ticket: `RIOTBOX-826`
- Title: `Reject unknown Source Timing example expectation keys`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-826/reject-unknown-source-timing-example-expectation-keys`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-826-source-timing-expectation-keys`
- Linear branch: `feature/riotbox-826-reject-unknown-source-timing-example-expectation-keys`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#821 (https://github.com/marang/riotbox/pull/821)`
- Merge commit: `c28784c610d2b89023a13f36fef18fc07952f201`
- Verification: `python3 -m py_compile scripts/source_timing_example_expectations.py scripts/assert_source_timing_example_report_fixtures.py`; `just source-timing-example-probe-report-fixtures`; `just ci`; GitHub Actions Rust CI #2002 passed.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Source Timing example expectation objects rejected malformed numeric range keys, but typoed top-level expectation keys were silently ignored and could weaken regression coverage.

## What Shipped

- Added an explicit supported expectation-key whitelist, rejected unknown keys during expectation loading/comparison, and added an invalid unknown-key fixture to the report fixture gate.

## Notes

- None

# `RIOTBOX-898` Accept SparseOnsets in Source Timing fixture-report validator

- Ticket: `RIOTBOX-898`
- Title: `Accept SparseOnsets in Source Timing fixture-report validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-898/accept-sparseonsets-in-source-timing-fixture-report-validator`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-898-sparse-onsets-fixture-report-validator`
- Linear branch: `feature/riotbox-898-accept-sparseonsets-in-source-timing-fixture-report`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`, `timing`
- PR: `#891 (https://github.com/marang/riotbox/pull/891)`
- Merge commit: `c444d1b8e88faec9ddc53a80636fede5f2dae941`
- Deleted from Linear: `2026-05-21`
- Verification: `python3 -m py_compile scripts/validate_source_timing_fixture_report_json.py; just source-timing-fixture-report-json-validator-fixtures; git diff --check origin/main...HEAD; GitHub Rust CI #2214 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Keep the Source Timing fixture-report JSON validator aligned with the SparseOnsets warning introduced by the current P012 timing model.

## What Shipped

- Added SparseOnsets to the fixture-report warning whitelist and covered it in the valid object-issue fixture while preserving invalid warning-code rejection.

## Notes

- None

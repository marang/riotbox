# `RIOTBOX-993` Make grid-use fixture phrase counts explicit

- Ticket: `RIOTBOX-993`
- Title: `Make grid-use fixture phrase counts explicit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-993/make-grid-use-fixture-phrase-counts-explicit`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-993-make-grid-use-fixture-phrase-counts-explicit`
- Linear branch: `feature/riotbox-993-make-grid-use-fixture-phrase-counts-explicit`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#985 (https://github.com/marang/riotbox/pull/985)`
- Merge commit: `c0a5a0189d31db1aaa9fa8876cf9a2fc7fe44ff4`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-993-grid-use-fixtures.log python3 scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-993-grid-use-just.log just source-timing-grid-use-contract-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-993-just-ci.log just ci`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Make Source Timing grid-use fixture phrase-count evidence explicit in the case table instead of hiding it behind helper branching.

## What Shipped

- Added explicit primary_phrase_count and primary_phrase_bar_count fields to GridUseCase.
- Populated all existing grid-use cases with the current expected phrase-count evidence.
- Changed apply_timing_fields to copy phrase counts from the case instead of deriving them from phrase_status.

## Notes

- GitHub Actions run 26448590989 failed in actions/checkout with HTTP 403 / account suspended before any project checks ran; local just ci passed before merge.

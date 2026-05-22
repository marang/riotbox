# `RIOTBOX-961` Make Source Timing grid-use fixture beat/bar counts explicit

- Ticket: `RIOTBOX-961`
- Title: `Make Source Timing grid-use fixture beat/bar counts explicit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-961/make-source-timing-grid-use-fixture-beatbar-counts-explicit`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-961-make-source-timing-grid-use-fixture-beatbar-counts-explicit`
- Linear branch: `feature/riotbox-961-make-source-timing-grid-use-fixture-beatbar-counts-explicit`
- Assignee: `Markus`
- Labels: None
- PR: `#954 (https://github.com/marang/riotbox/pull/954)`
- Merge commit: `05d8476f9c0cbaa46532f5e15d06393d780bafa6`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-961-grid-use-fixtures.log just source-timing-grid-use-contract-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-961-just-ci.log just ci`; `GitHub Actions Rust CI run 26304973819 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The P012 validator surface review found that grid-use contract fixtures inferred probe-only beat/bar counts in helper logic instead of owning them in the per-case data.

## What Shipped

- Added explicit primary_beat_count and primary_bar_count fields to GridUseCase.
- Replaced helper inference with direct per-case assignment while preserving generated fixture behavior.

## Notes

- Fixture readability cleanup only; no analyzer scoring, ActionCommand, queue, Session/replay, JamAppState, observer schema, realtime audio, or render behavior changed.

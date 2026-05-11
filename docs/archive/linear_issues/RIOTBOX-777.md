# `RIOTBOX-777` Add grid-use contract parity fixture matrix

- Ticket: `RIOTBOX-777`
- Title: `Add grid-use contract parity fixture matrix`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-777/add-grid-use-contract-parity-fixture-matrix`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-777-add-grid-use-contract-parity-fixture-matrix`
- Linear branch: `feature/riotbox-777-add-grid-use-contract-parity-fixture-matrix`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#771 (https://github.com/marang/riotbox/pull/771)`
- Merge commit: `f57ed59f0f152f1f5adfc917443f04af775e934d`
- Deleted from Linear: `2026-05-11`
- Verification: `python3 -m py_compile scripts/validate_source_timing_grid_use_contract_fixtures.py; just source-timing-grid-use-contract-fixtures; git diff --check; cargo fmt --check; just audio-qa-ci`
- Docs touched: `Justfile`
- Follow-ups: `None.`

## Why This Ticket Existed

The grid_use contract needed validator-side parity coverage across probe, listening manifest, and observer/audio summary JSON after Rust producers were unified in RIOTBOX-776.

## What Shipped

- Added a generated grid_use contract fixture matrix for locked_grid, short_loop_manual_confirm, manual_confirm_only, fallback_grid, and unavailable across three JSON validator surfaces, including one negative mismatch check per surface, and wired it into just audio-qa-ci.

## Notes

- None

# `RIOTBOX-832` Preserve downbeat offset in user-session observer snapshots

- Ticket: `RIOTBOX-832`
- Title: `Preserve downbeat offset in user-session observer snapshots`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-832/preserve-downbeat-offset-in-user-session-observer-snapshots`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-832-user-observer-downbeat-offset`
- Linear branch: `feature/riotbox-832-preserve-downbeat-offset-in-user-session-observer-snapshots`
- Assignee: `Markus`
- Labels: `benchmark`
- PR: `#827 (https://github.com/marang/riotbox/pull/827)`
- Merge commit: `1fb304ac4a3e263439d7021eb3cd0ee4662dfa6a`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-app user_session_observer_probe -- --nocapture`; `python3 -m py_compile scripts/validate_user_session_observer_ndjson.py`; manual locked observer probe plus `scripts/validate_user_session_observer_ndjson.py`; `just ci`; GitHub Actions Rust CI #2020
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-831 exposed primary_downbeat_offset_beats in the Source Timing example report, but user-session observer snapshots still omitted the selected primary downbeat offset from the control path.

## What Shipped

- Added primary_downbeat_offset_beats to observer source_timing snapshots, derived it from the primary timing hypothesis bar grid, kept unavailable offsets as null, validated the field in the observer NDJSON validator, updated observer tests, and documented the Source Timing snapshot contract.

## Notes

- None

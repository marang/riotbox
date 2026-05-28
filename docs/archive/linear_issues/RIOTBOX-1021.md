# `RIOTBOX-1021` Require observer source timing primary anchor cue in user-session snapshots

- Ticket: `RIOTBOX-1021`
- Title: `Require observer source timing primary anchor cue in user-session snapshots`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1021/require-observer-source-timing-primary-anchor-cue-in-user-session`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-28`
- Started: `2026-05-28`
- Finished: `2026-05-28`
- Branch: `feature/riotbox-1021-require-observer-source-timing-primary-anchor-cue-in-user`
- Linear branch: `feature/riotbox-1021-require-observer-source-timing-primary-anchor-cue-in-user`
- Assignee: `Markus`
- Labels: None
- PR: `#1004 (https://github.com/marang/riotbox/pull/1004)`
- Merge commit: `c9c795bfbab97b77c94132e2717024535511a6da`
- Deleted from Linear: `2026-05-28`
- Verification: `python3 -m py_compile scripts/validate_user_session_observer_ndjson.py; just user-session-observer-validator-fixtures; git diff --check; just ci; GitHub Rust CI #2535 success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Require observer source timing primary anchor cue in user-session snapshots.

## What Shipped

- Required non-empty snapshot.source_timing.primary_anchor_cue in the user-session observer NDJSON validator, refreshed Source Timing observer fixtures, added a missing-field validator gate, and documented the primary_anchor_cue control-path contract.

## Notes

- None

# `RIOTBOX-1013` P012: Reject contradictory user-session Source Timing count evidence

- Ticket: `RIOTBOX-1013`
- Title: `P012: Reject contradictory user-session Source Timing count evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1013/p012-reject-contradictory-user-session-source-timing-count-evidence`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1013-user-session-source-timing-count-evidence`
- Linear branch: `feature/riotbox-1013-p012-reject-contradictory-user-session-source-timing-count`
- Assignee: `Markus`
- Labels: `Bug`, `timing`, `workflow`
- PR: `#996 (https://github.com/marang/riotbox/pull/996)`
- Merge commit: `5269513b7b62eeb0a7fc9f917da9a65edca5b46f`
- Deleted from Linear: `2026-05-27`
- Verification: `python3 -m py_compile scripts/validate_user_session_observer_ndjson.py`; `just user-session-observer-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-1013-audio-qa-ci.log just audio-qa-ci`; `git diff --check`; `GitHub Actions Rust CI run 26495892935 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

User-session observer NDJSON validation only enforced phrase-lock count consistency, while the P012 contract also requires rejecting grid/bar statuses that contradict beat/bar counts.

## What Shipped

- Added user-session Source Timing count consistency checks for beat_status=grid and downbeat_status=bar_locked, preserving phrase-lock checks.
- Added negative Justfile smokes for zero beat_count and zero bar_count on the locked-grid observer fixture.

## Notes

- None

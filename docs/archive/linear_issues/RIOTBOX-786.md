# `RIOTBOX-786` Add invalid observer fixture for locked-grid Source Timing contradictions

- Ticket: `RIOTBOX-786`
- Title: `Add invalid observer fixture for locked-grid Source Timing contradictions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-786/add-invalid-observer-fixture-for-locked-grid-source-timing`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-786-invalid-observer-locked-grid-fixture`
- Linear branch: `feature/riotbox-786-add-invalid-observer-fixture-for-locked-grid-source-timing`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#781 (https://github.com/marang/riotbox/pull/781)`
- Merge commit: `4205cff2f5ba1114bc7f206f2588697989432dad`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-786-rebase-fixtures.log just user-session-observer-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-786-rebase-pycompile.log python3 -m py_compile scripts/validate_user_session_observer_ndjson.py`; `scripts/run_compact.sh /tmp/riotbox-786-rebase-fmt.log cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed the negative observer Source Timing contract: locked-grid snapshots must not still carry warning evidence.

## What Shipped

- Rejected locked observer Source Timing snapshots with warning fields and added an expected-failure locked-warning fixture to the user-session observer validator recipe.

## Notes

- No app runtime, Source Timing detector policy, or audio render behavior changed.

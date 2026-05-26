# `RIOTBOX-999` Assert unavailable local timing null metrics

- Ticket: `RIOTBOX-999`
- Title: `Assert unavailable local timing null metrics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-999/assert-unavailable-local-timing-null-metrics`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-999-assert-unavailable-local-timing-null-metrics`
- Linear branch: `feature/riotbox-999-assert-unavailable-local-timing-null-metrics`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: `#991 (https://github.com/marang/riotbox/pull/991)`
- Merge commit: `c3d286e7def848a59b5bec2d877d4235c1af8c4f`
- Deleted from Linear: `2026-05-26`
- Verification: `py_compile; source-timing example fixtures; local report; diff check; just ci; GitHub Rust CI 26451600396 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Unavailable local Source Timing rows should prove numeric timing evidence is absent, not only unavailable labels.

## What Shipped

- Allowed null numeric timing expectations and asserted unavailable local null BPM/score evidence.
- Added a null-metric mismatch fixture.

## Notes

- No analyzer, readiness, UI, Session, or audio-output behavior changed.

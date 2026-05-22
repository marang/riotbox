# `RIOTBOX-941` Reject contradictory source-timing phrase locks in user-session observer validator

- Ticket: `RIOTBOX-941`
- Title: `Reject contradictory source-timing phrase locks in user-session observer validator`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-941/reject-contradictory-source-timing-phrase-locks-in-user-session`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-941-user-session-phrase-lock-validator`
- Linear branch: `feature/riotbox-941-reject-contradictory-source-timing-phrase-locks-in-user`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#934 (https://github.com/marang/riotbox/pull/934)`
- Merge commit: `26174334df82f62b29cfcbd621c63ebf5ef3b377`
- Deleted from Linear: `2026-05-22`
- Verification: `just user-session-observer-validator-fixtures; just ci; GitHub Actions Rust CI run 26296884234 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-942 continues by applying the same phrase-evidence contradiction checks to listening manifests.`

## Why This Ticket Existed

User-session observer snapshots exposed phrase status/count evidence but validator accepted clear contradictions such as phrase_locked with zero phrase_count.

## What Shipped

- Added phrase status/count consistency checks to the user-session observer NDJSON validator and focused mutated fixture checks for locked, non-locked, and negative bar-count cases.

## Notes

- QA contract hardening only; no runtime behavior changed.

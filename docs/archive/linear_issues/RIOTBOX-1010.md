# `RIOTBOX-1010` P012: Replace replay-plan linear scans with indexed lookups

- Ticket: `RIOTBOX-1010`
- Title: `P012: Replace replay-plan linear scans with indexed lookups`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1010/p012-replace-replay-plan-linear-scans-with-indexed-lookups`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1010-p012-replace-replay-plan-linear-scans-with-indexed-lookups`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-core`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Replay plan construction used repeated linear membership scans that were unnecessary risk for longer sessions.

## What Shipped

- Replaced replay-plan linear membership scans with indexed lookups while preserving duplicate-id first-action behavior.

## Notes

- None

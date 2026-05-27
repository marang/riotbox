# `RIOTBOX-1009` P012: Add sidecar request timeout and degraded startup behavior

- Ticket: `RIOTBOX-1009`
- Title: `P012: Add sidecar request timeout and degraded startup behavior`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1009/p012-add-sidecar-request-timeout-and-degraded-startup-behavior`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1009-p012-add-sidecar-request-timeout-and-degraded-startup`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`, `workflow`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-sidecar`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The stdio sidecar client could block indefinitely waiting on ping or analysis responses from a hung Python sidecar.

## What Shipped

- Added bounded timeout behavior for sidecar stdio responses with structured timeout errors.

## Notes

- None

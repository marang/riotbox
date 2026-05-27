# `RIOTBOX-1004` P012: Surface source-audio load failures and monitor fallback

- Ticket: `RIOTBOX-1004`
- Title: `P012: Surface source-audio load failures and monitor fallback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1004/p012-surface-source-audio-load-failures-and-monitor-fallback`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1004-p012-surface-source-audio-load-failures-and-monitor-fallback`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`, `timing`, `ux`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-app`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Source monitor source/blend paths could silently collapse to generated Riotbox output when source audio failed to load or did not match the output shape.

## What Shipped

- Surfaced source-audio load failures and source-monitor fallback diagnostics so unavailable source audio is visible in runtime diagnostics and tests.

## Notes

- None

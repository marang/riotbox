# `RIOTBOX-1005` P012: Resolve external Source Graph paths relative to session files

- Ticket: `RIOTBOX-1005`
- Title: `P012: Resolve external Source Graph paths relative to session files`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1005/p012-resolve-external-source-graph-paths-relative-to-session-files`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1005-p012-resolve-external-source-graph-paths-relative-to-session`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`, `timing`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-app`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Relative external Source Graph paths were resolved against process cwd instead of the restored session file directory, weakening portable restore/replay.

## What Shipped

- Resolved relative external Source Graph paths against the session file directory while preserving absolute-path behavior.

## Notes

- None

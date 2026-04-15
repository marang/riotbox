# `RIOTBOX-36` Ticket Archive

- Ticket: `RIOTBOX-36`
- Title: `Align ingest graph storage with the MVP embedded-graph session contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-36/align-ingest-graph-storage-with-the-mvp-embedded-graph-session`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-13`
- Finished: `2026-04-13`
- Branch: `riotbox-36-embedded-graph-ingest`
- Assignee: `Markus`
- Labels: `Docs`, `Core`
- PR: `#29`
- Merge commit: `d041fc9`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-37`

## Why This Ticket Existed

The periodic review found that ingest was defaulting to external graph files even though the MVP session contract preferred embedded graph storage.

## What Shipped

- Made embedded graph storage the default ingest behavior.
- Kept external graph files only when explicitly requested.
- Updated tests so both explicit external and default embedded behavior were covered.

## Notes

- This was a contract-alignment cleanup, not a new ingest architecture.

# `RIOTBOX-875` Show Source Timing actionability in the compact Jam Trust line

- Ticket: `RIOTBOX-875`
- Title: `Show Source Timing actionability in the compact Jam Trust line`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-875/show-source-timing-actionability-in-the-compact-jam-trust-line`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-875-show-source-timing-actionability-in-compact-jam-trust-line`
- Linear branch: `feature/riotbox-875-show-source-timing-actionability-in-the-compact-jam-trust`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#869 (https://github.com/marang/riotbox/pull/869)`
- Merge commit: `0e512c6d453ee203f301e47880eb370973900fd9`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo test -p riotbox-app shell_state_jam_snapshot; cargo test -p riotbox-app shell_state_log_source; cargo test -p riotbox-app post_commit_help_restore; git diff --check; just ci; GitHub Rust CI success`
- Docs touched: `docs/reviews/p012_jam_source_timing_surface_review_2026-05-21.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The P012 Jam / Source timing surface review found that Source Timing actionability was shared but not yet visible in the compact Jam Trust panel.

## What Shipped

- Added a compact Jam Trust action line sourced from SourceTimingSummaryView.actionability while keeping the readiness line stable and avoiding TUI wrapping.

## Notes

- None

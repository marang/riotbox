# `RIOTBOX-963` Show compact Source Timing counts in Jam readiness line

- Ticket: `RIOTBOX-963`
- Title: `Show compact Source Timing counts in Jam readiness line`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-963/show-compact-source-timing-counts-in-jam-readiness-line`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-963-show-compact-source-timing-counts-in-jam-readiness-line`
- Linear branch: `feature/riotbox-963-show-compact-source-timing-counts-in-jam-readiness-line`
- Assignee: `Markus`
- Labels: None
- PR: `#956 (https://github.com/marang/riotbox/pull/956)`
- Merge commit: `2e75b3adf308e38f7414d0bf82d78f359daeb670`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-app ui::tests::shell_state_jam_snapshot -- --nocapture; cargo test -p riotbox-app --lib ui::tests -- --nocapture; cargo fmt --check; git diff --check; just ci; GitHub Actions Rust CI run 26305685052 success`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-964 continues Jam Source Timing clock partial/unavailable evidence`

## Why This Ticket Existed

Expose compact beat/bar/phrase count evidence directly in the Jam readiness line.

## What Shipped

- Added compact pN:bB/R/P Source Timing evidence chip from SourceTimingSummaryView and updated Jam/Source UI snapshots.

## Notes

- None

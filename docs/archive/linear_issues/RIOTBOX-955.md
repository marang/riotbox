# `RIOTBOX-955` Use primary hypothesis grid counts in Jam Source Timing summary

- Ticket: `RIOTBOX-955`
- Title: `Use primary hypothesis grid counts in Jam Source Timing summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-955/use-primary-hypothesis-grid-counts-in-jam-source-timing-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-955-use-primary-hypothesis-grid-counts-in-jam-source-timing`
- Linear branch: `feature/riotbox-955-use-primary-hypothesis-grid-counts-in-jam-source-timing`
- Assignee: `Markus`
- Labels: None
- PR: `#948 (https://github.com/marang/riotbox/pull/948)`
- Merge commit: `d19fd8ad1d68ca62c174c1b3cfdb9771987190d5`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-core view::jam::source_timing_summary_tests -- --nocapture`; `cargo test -p riotbox-app ui::tests::shell_state_source -- --nocapture`; `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-955-just-ci.log just ci`; `GitHub Actions Rust CI run 26302289772 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The shared Jam Source Timing summary already used the selected primary timing hypothesis for anchors, groove, and downbeat phase, but beat/bar/phrase counts still came only from top-level TimingModel grids.

## What Shipped

- Derived beat, bar, and phrase counts from the selected primary timing hypothesis when it has grid evidence.
- Kept fallback to top-level TimingModel grids when the selected hypothesis lacks that grid.
- Updated core summary tests and Source UI expectations so the fixture-backed primary-hypothesis bar grid shows as bars 1.

## Notes

- View-model summary slice only; no analyzer, ActionCommand, queue, Session/replay, JamAppState, realtime audio, observer schema, or render behavior changed.

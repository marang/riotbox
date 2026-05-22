# `RIOTBOX-956` Use primary hypothesis grids for short-loop grid-use policy

- Ticket: `RIOTBOX-956`
- Title: `Use primary hypothesis grids for short-loop grid-use policy`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-956/use-primary-hypothesis-grids-for-short-loop-grid-use-policy`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-956-use-primary-hypothesis-grids-for-short-loop-grid-use-policy`
- Linear branch: `feature/riotbox-956-use-primary-hypothesis-grids-for-short-loop-grid-use-policy`
- Assignee: `Markus`
- Labels: None
- PR: `#949 (https://github.com/marang/riotbox/pull/949)`
- Merge commit: `10aeb237cde3bc3b570e88f630df1b45b0e968c9`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-core source_graph::probe_candidate_tests::readiness_report_tests -- --nocapture`; `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-956-just-ci.log just ci`; `GitHub Actions Rust CI run 26302797963 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-955 made Jam Source Timing summary prefer selected primary-hypothesis grids, but the short-loop grid-use policy still only inspected top-level TimingModel grids.

## What Shipped

- Updated short-loop grid-use detection to use selected primary-hypothesis beat/bar/phrase counts before falling back to top-level grids.
- Added a regression test that clears top-level grids while keeping primary-hypothesis short-loop evidence and still expects ShortLoopManualConfirm.
- Documented selected-primary-hypothesis grid precedence for Jam summary and grid-use policy in the Source Timing spec.

## Notes

- Core policy/spec slice only; no analyzer, ActionCommand, queue, Session/replay, JamAppState, realtime audio, observer schema, or render behavior changed.

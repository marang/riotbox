# `RIOTBOX-925` Split footer style and suggested gesture UI tests

- Ticket: `RIOTBOX-925`
- Title: `Split footer style and suggested gesture UI tests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-925/split-footer-style-and-suggested-gesture-ui-tests`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-925-split-footer-suggested-gesture-tests`
- Linear branch: `feature/riotbox-925-split-footer-style-and-suggested-gesture-ui-tests`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#918 (https://github.com/marang/riotbox/pull/918)`
- Merge commit: `4bb34cbe8fc3c72a70194d6048d3fcfb547f2bf9`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; git diff --check; scripts/run_compact.sh /tmp/riotbox-925-just-ci-final.log just ci; GitHub Actions Rust CI run 26289564766 passed`
- Docs touched: `docs/archive/linear_issues/RIOTBOX-925.md; docs/archive/linear_issues/2026-05.md; docs/archive/linear_issues/index.md`
- Follow-ups: `RIOTBOX-926 continues the P015 UI test ownership split by separating W-30 preview/source-readiness tests`

## Why This Ticket Existed

The P015 review found footer/suggested gesture/fixture types mixed in one UI test shard; splitting it reduces review cost without changing Jam behavior.

## What Shipped

- Split the mixed footer_gesture_fixture_types.rs shard into shared imports, footer/help/capture style-token tests, suggested gesture cue tests, and regression fixture types.

## Notes

- Test ownership split only; no ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.

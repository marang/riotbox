# `RIOTBOX-917` Split oversized shell state log/source UI tests by surface

- Ticket: `RIOTBOX-917`
- Title: `Split oversized shell state log/source UI tests by surface`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-917/split-oversized-shell-state-logsource-ui-tests-by-surface`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-917-split-shell-state-log-source-tests`
- Linear branch: `feature/riotbox-917-split-oversized-shell-state-logsource-ui-tests-by-surface`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#910 (https://github.com/marang/riotbox/pull/910)`
- Merge commit: `33587a98ba4736757157292985a4f18dd1e211d5`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-917-just-ci.log just ci; GitHub Actions Rust CI run 26286188056 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-918 continues P015 TUI module ownership work`

## Why This Ticket Existed

P015 Productization Alpha needs UI tests split by surface before mixed test files become expensive to review and extend.

## What Shipped

- Split the oversized mixed shell_state_log_source UI test file into key, Log, Source, and Capture/source-window test files while preserving assertions and ui::tests names.

## Notes

- No production code, ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.

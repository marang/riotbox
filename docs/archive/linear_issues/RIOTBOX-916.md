# `RIOTBOX-916` Extract Jam footer lane perform lines into semantic module

- Ticket: `RIOTBOX-916`
- Title: `Extract Jam footer lane perform lines into semantic module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-916/extract-jam-footer-lane-perform-lines-into-semantic-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-916-footer-lane-lines-module`
- Linear branch: `feature/riotbox-916-extract-jam-footer-lane-perform-lines-into-semantic-module`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#909 (https://github.com/marang/riotbox/pull/909)`
- Merge commit: `2ea75192c11ed98d92af3bc3e03797e76a05faa2`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-916-just-ci.log just ci; GitHub Actions Rust CI run 26285840631 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-917 continues P015 test surface split work`

## Why This Ticket Existed

P015 Productization Alpha needs the Jam TUI helpers to remain reviewable as musician-facing surfaces grow.

## What Shipped

- Extracted MC-202, W-30, and TR-909 Jam footer perform/inspect line helpers into footer_lane_perform_lines without changing rendered TUI behavior.

## Notes

- No ActionCommand, queue/session/replay, JamAppState, or audio-producing behavior changed.

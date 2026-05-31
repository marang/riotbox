# `RIOTBOX-1070` Split riotbox-app event loop controls before more key handlers

- Ticket: `RIOTBOX-1070`
- Title: `Split riotbox-app event loop controls before more key handlers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1070/split-riotbox-app-event-loop-controls-before-more-key-handlers`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1070-event-loop-control-split`
- Linear branch: `feature/riotbox-1070-split-riotbox-app-event-loop-controls-before-more-key`
- Assignee: `Markus`
- Labels: None
- PR: `#1048 (https://github.com/marang/riotbox/pull/1048)`
- Merge commit: `57248725cbd77e064382717811fdeadefe931347`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-app export -- --nocapture`; `cargo test -p riotbox-app shell_state_keys -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1048`
- Docs touched: `None`
- Follow-ups: `Derive next concrete P016 implementation tickets from RIOTBOX-1036.`

## Why This Ticket Existed

The TUI event loop was at 496 lines after export key handling landed; splitting a semantic control branch keeps future export work reviewable.

## What Shipped

- Extracted Product Mix export key-control handling into product_export_control.rs.
- Preserved queue-result handling, status strings, observer recording, and key behavior.
- Reduced event_loop.rs from 496 to 484 lines.

## Notes

- No new export scope behavior, key binding, observer behavior, or audio behavior shipped.

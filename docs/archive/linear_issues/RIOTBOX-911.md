# `RIOTBOX-911` Convert TUI Source timing panel shard into a semantic module

- Ticket: `RIOTBOX-911`
- Title: `Convert TUI Source timing panel shard into a semantic module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-911/convert-tui-source-timing-panel-shard-into-a-semantic-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-911-source-timing-panel-module`
- Linear branch: `feature/riotbox-911-convert-tui-source-timing-panel-shard-into-a-semantic-module`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#904 (https://github.com/marang/riotbox/pull/904)`
- Merge commit: `b6a41803cfd15c2047bce298aaaff1d587db78c8`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app renders_source_shell_snapshot_with; git diff --check; scripts/run_compact.sh /tmp/riotbox-911-ci.log just ci; GitHub Rust CI #2255 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Continue leaf-first TUI include-shell conversion with the small Source timing panel boundary.

## What Shipped

- Converted source_timing_panel.rs from textual include shard into ui::source_timing_panel module, exposing only source_timing_lines while keeping panel helpers private.

## Notes

- Screen-output preserving TUI module-ownership conversion only; no visual redesign, Source Timing behavior, runtime behavior, ActionCommand, Session/replay, lane, or audio-output behavior changed.

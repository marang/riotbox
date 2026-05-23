# `RIOTBOX-969` Add source monitor mode to action and session contracts

- Ticket: `RIOTBOX-969`
- Title: `Add source monitor mode to action and session contracts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-969/add-source-monitor-mode-to-action-and-session-contracts`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-23`
- Branch: `feature/riotbox-969-add-source-monitor-mode-to-action-and-session-contracts`
- Linear branch: `feature/riotbox-969-add-source-monitor-mode-to-action-and-session-contracts`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#961 (https://github.com/marang/riotbox/pull/961)`
- Merge commit: `371dff55dfa257acd0a4c0cb857ce1afc3af2bee`
- Deleted from Linear: `2026-05-23`
- Verification: `cargo test -p riotbox-core replay::executor::tests::structural -- --nocapture; cargo test -p riotbox-core session::tests::source_monitor -- --nocapture; cargo test -p riotbox-app jam_app::tests::source_monitor -- --nocapture; cargo test -p riotbox-app jam_app::side_effects::source_monitor -- --nocapture; cargo test -p riotbox-core; cargo test -p riotbox-app; git diff --check; just ci; post-rebase just ci; GitHub Actions Rust CI #2433`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-970 routes the monitor contract through the audio runtime policy; later slices add Source Map, seek, grid confirmation, and capture length UI.`

## Why This Ticket Existed

Turn the planned source/blend/riotbox monitor concept into typed Action/Session/Replay contracts before audio routing work.

## What Shipped

- Added typed SourceMonitorMode, persistent Session runtime state, source_monitor.set_mode action params/command, replay support, Jam queue path, commit side effect, runtime view, observer output, and regression coverage.

## Notes

- Contract/control slice only; no audio playback or Source Map rendering in this ticket.

# `RIOTBOX-906` Show phrase counts in the Source timing panel

- Ticket: `RIOTBOX-906`
- Title: `Show phrase counts in the Source timing panel`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-906/show-phrase-counts-in-the-source-timing-panel`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-906-source-timing-panel-phrase-counts`
- Linear branch: `feature/riotbox-906-show-phrase-counts-in-the-source-timing-panel`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#899 (https://github.com/marang/riotbox/pull/899)`
- Merge commit: `619222daab4e59e8e8333101546d20a3e74af35d`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app renders_source_shell_snapshot_with; git diff --check; scripts/run_compact.sh /tmp/riotbox-906-ci.log just ci; GitHub Rust CI #2240 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Show existing Source Timing phrase-count evidence directly in the musician-facing Source timing panel.

## What Shipped

- Source timing panel phrase segment now renders the phrase status with count, e.g. phrase uncertain(0), using compact off wording to avoid wrapping.

## Notes

- Display-only Source panel observability change; no detector, readiness/grid-use policy, ActionCommand, Session/replay, JamAppState, lane behavior, or audio-output behavior changed.

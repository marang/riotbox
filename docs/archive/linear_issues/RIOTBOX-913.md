# `RIOTBOX-913` Convert TUI first-run capture shard into a semantic module

- Ticket: `RIOTBOX-913`
- Title: `Convert TUI first-run capture shard into a semantic module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-913/convert-tui-first-run-capture-shard-into-a-semantic-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-913-first-run-capture-module`
- Linear branch: `feature/riotbox-913-convert-tui-first-run-capture-shard-into-a-semantic-module`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#906 (https://github.com/marang/riotbox/pull/906)`
- Merge commit: `06b419dcef4de966406bff1410d55066b853c833`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app renders_capture_shell_snapshot_with; cargo test -p riotbox-app first_run; cargo test -p riotbox-app renders_capture_pending_cues_panel_as_first_item_with_log_overflow; git diff --check; scripts/run_compact.sh /tmp/riotbox-913-ci.log just ci; GitHub Rust CI #2261 passed`
- Docs touched: `None`
- Follow-ups: `Continue with the next bounded TUI include shard from clean main; avoid adding more bulk to first_run_capture.rs without a semantic split.`

## Why This Ticket Existed

Continue the TUI include-shell cleanup by converting the musician-facing first-run/Capture shard into an explicit semantic module without changing rendered output.

## What Shipped

- ui::first_run_capture now owns first-run onramp and Capture render helpers behind explicit imports and pub(super) sibling surfaces; ui.rs no longer textually includes first_run_capture.rs.

## Notes

- Behavior-preserving module-boundary slice only. No ActionCommand, JamAppState, Session/replay, lane, runtime, Capture behavior, visual redesign, or audio-output behavior changed. first_run_capture.rs is 489 lines after imports and should be treated as near the soft file-size threshold.

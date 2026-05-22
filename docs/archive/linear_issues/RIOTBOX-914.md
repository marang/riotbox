# `RIOTBOX-914` Extract pending Capture cue helpers from first-run capture module

- Ticket: `RIOTBOX-914`
- Title: `Extract pending Capture cue helpers from first-run capture module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-914/extract-pending-capture-cue-helpers-from-first-run-capture-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-914-pending-capture-cues-module`
- Linear branch: `feature/riotbox-914-extract-pending-capture-cue-helpers-from-first-run-capture`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#907 (https://github.com/marang/riotbox/pull/907)`
- Merge commit: `1c61f38f0394a385ef08973968ac710c74cd1696`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app renders_capture_shell_snapshot_with; cargo test -p riotbox-app first_run; cargo test -p riotbox-app renders_capture_pending_cues_panel_as_first_item_with_log_overflow; git diff --check; scripts/run_compact.sh /tmp/riotbox-914-ci.log just ci; GitHub Rust CI #2264 passed`
- Docs touched: `None`
- Follow-ups: `Continue with the next bounded TUI include/module cleanup from clean main.`

## Why This Ticket Existed

Keep the new first-run Capture module below the soft review-size threshold by extracting its coherent pending Capture / W-30 audition cue responsibility.

## What Shipped

- ui::first_run_capture::pending_capture now owns pending Capture and W-30 audition do-next cue construction; first_run_capture.rs dropped from 489 to 405 lines while rendered Capture/first-run text stayed unchanged.

## Notes

- Behavior-preserving semantic split only. No ActionCommand, JamAppState, Session/replay, lane, runtime, Capture behavior, visual redesign, or audio-output behavior changed.

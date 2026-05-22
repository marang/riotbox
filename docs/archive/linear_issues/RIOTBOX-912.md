# `RIOTBOX-912` Convert TUI Source trust summary shard into a semantic module

- Ticket: `RIOTBOX-912`
- Title: `Convert TUI Source trust summary shard into a semantic module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-912/convert-tui-source-trust-summary-shard-into-a-semantic-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-912-source-trust-summary-module`
- Linear branch: `feature/riotbox-912-convert-tui-source-trust-summary-shard-into-a-semantic`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#905 (https://github.com/marang/riotbox/pull/905)`
- Merge commit: `8ac37ebe38e19812176fb08ed30edbb57ebc43b5`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app renders_source_shell_snapshot_with; cargo test -p riotbox-app source_timing; cargo test -p riotbox-app renders_more_musical_jam_shell_snapshot; cargo test -p riotbox-app jam_trust_warning_uses_shared_source_timing_priority; git diff --check; scripts/run_compact.sh /tmp/riotbox-912-ci-final.log just ci; GitHub Rust CI #2258 passed`
- Docs touched: `None`
- Follow-ups: `Continue with the next bounded TUI include shard, likely first_run_capture.rs, from clean main.`

## Why This Ticket Existed

Continue the TUI include-shell cleanup by converting the Source trust summary shard into an explicit semantic module without changing musician-facing output.

## What Shipped

- ui::source_trust_summary now owns Source trust, warning, timing readiness/help, clock, and energy-label helpers behind explicit imports and pub(super) sibling surfaces; ui.rs no longer textually includes source_trust_summary.rs.

## Notes

- Behavior-preserving module-boundary slice only. No ActionCommand, JamAppState, Session/replay, lane, runtime, Source Timing, visual redesign, or audio-output behavior changed.

# `RIOTBOX-910` Convert TUI recovery prompt shard into a semantic module

- Ticket: `RIOTBOX-910`
- Title: `Convert TUI recovery prompt shard into a semantic module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-910/convert-tui-recovery-prompt-shard-into-a-semantic-module`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-910-recovery-prompt-module`
- Linear branch: `feature/riotbox-910-convert-tui-recovery-prompt-shard-into-a-semantic-module`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#903 (https://github.com/marang/riotbox/pull/903)`
- Merge commit: `926cdf5b4a065a5ef1fd4a86775ba71be8cc9c40`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo check -p riotbox-app; cargo test -p riotbox-app recovery; cargo test -p riotbox-app renders_manual_recovery_prompt; git diff --check; scripts/run_compact.sh /tmp/riotbox-910-ci.log just ci; GitHub Rust CI #2252 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Make the first low-risk TUI include-shell leaf boundary explicit after the RIOTBOX-909 audit.

## What Shipped

- Converted recovery_prompt.rs from textual include shard into ui::recovery_prompt module, exposing only recovery_warning_line and recovery_help_lines while keeping helper internals private.

## Notes

- Screen-output preserving TUI module-ownership conversion only; no visual redesign, recovery behavior, runtime behavior, ActionCommand, Session/replay, lane, or audio-output behavior changed.

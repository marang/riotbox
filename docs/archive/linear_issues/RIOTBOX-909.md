# `RIOTBOX-909` Audit TUI include shell for later semantic module conversion

- Ticket: `RIOTBOX-909`
- Title: `Audit TUI include shell for later semantic module conversion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-909/audit-tui-include-shell-for-later-semantic-module-conversion`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-909-tui-include-shell-audit`
- Linear branch: `feature/riotbox-909-audit-tui-include-shell-for-later-semantic-module-conversion`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#902 (https://github.com/marang/riotbox/pull/902)`
- Merge commit: `5bc0bf6e0cb6617e97be4a39823b849c24c992bc`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; git diff --check; scripts/run_compact.sh /tmp/riotbox-909-ci.log just ci; GitHub Rust CI #2249 passed`
- Docs touched: `docs/reviews/tui_include_shell_audit_2026-05-22.md; docs/README.md; docs/reviews/README.md`
- Follow-ups: `RIOTBOX-910 converts the recovery prompt TUI shard into a semantic module`

## Why This Ticket Existed

Audit the TUI textual include shell separately from the audio-runtime module conversion and decide whether to keep, convert, or defer it.

## What Shipped

- Recorded the current TUI include-shell shape, recommended deferring broad conversion, and created RIOTBOX-910 as the first leaf-first module conversion follow-up.

## Notes

- Docs/review-only slice; no TUI screen output, runtime behavior, ActionCommand, Session/replay, JamAppState, lane, or audio-output behavior changed.

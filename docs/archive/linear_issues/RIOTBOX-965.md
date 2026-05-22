# `RIOTBOX-965` Label queued timing rail counters as transport context

- Ticket: `RIOTBOX-965`
- Title: `Label queued timing rail counters as transport context`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-965/label-queued-timing-rail-counters-as-transport-context`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-965-label-queued-timing-rail-counters-as-transport-context`
- Linear branch: `feature/riotbox-965-label-queued-timing-rail-counters-as-transport-context`
- Assignee: `Markus`
- Labels: None
- PR: `#958 (https://github.com/marang/riotbox/pull/958)`
- Merge commit: `1ef0cc66e38f24413cd73c4d6c4060bc692cb99f`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-app --lib ui::tests -- --nocapture; cargo fmt --check; git diff --check; just ci; GitHub Actions Rust CI run 26306679829 success`
- Docs touched: `docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-966 adds observer beat/bar counts to the lower-level generated Feral-grid summary index`

## Why This Ticket Existed

Clarify that queued timing rail beat/bar/phrase counters are transport context, not Source Timing trust evidence.

## What Shipped

- Labelled queued timing rail and Scene pulse counters as transport bN barN pN, preserved scheduler semantics, and updated the TUI spec.

## Notes

- None

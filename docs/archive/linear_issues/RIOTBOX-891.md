# `RIOTBOX-891` Show Source Timing actionability in Jam Trust readiness line

- Ticket: `RIOTBOX-891`
- Title: `Show Source Timing actionability in Jam Trust readiness line`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-891/show-source-timing-actionability-in-jam-trust-readiness-line`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-891-jam-trust-actionability`
- Linear branch: `feature/riotbox-891-show-source-timing-actionability-in-jam-trust-readiness-line`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#885 (https://github.com/marang/riotbox/pull/885)`
- Merge commit: `e45b9adf436d91d4c027f81a9d4da0f454bdc100`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; git diff --check; cargo test -p riotbox-app ui::tests; just ci; GitHub Rust CI #2197 passed`
- Docs touched: `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Put the shared Source Timing actionability phrase directly into the compact Jam Trust readiness line so musicians can read the next useful timing action without leaving Jam.

## What Shipped

- Updated source_timing_readiness_line to show cue, actionability, grid-use, and selected phase; removed the duplicate Trust action row; updated Jam perform/inspect render tests and TUI spec wording.

## Notes

- UI/spec-only slice; no ActionCommand, Session/replay, JamAppState, timing policy, lane behavior, or audio-producing behavior changed.

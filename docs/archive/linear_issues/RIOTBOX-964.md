# `RIOTBOX-964` Show partial Source Timing clock when beat or phrase grid is absent

- Ticket: `RIOTBOX-964`
- Title: `Show partial Source Timing clock when beat or phrase grid is absent`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-964/show-partial-source-timing-clock-when-beat-or-phrase-grid-is-absent`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-964-show-partial-source-timing-clock-when-beat-or-phrase-grid-is`
- Linear branch: `feature/riotbox-964-show-partial-source-timing-clock-when-beat-or-phrase-grid-is`
- Assignee: `Markus`
- Labels: None
- PR: `#957 (https://github.com/marang/riotbox/pull/957)`
- Merge commit: `9cb446590e2213aba8770ab5c55ab1f419c0a41c`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-app --lib ui::tests -- --nocapture; cargo fmt --check; git diff --check; just ci; GitHub Actions Rust CI run 26306288206 success`
- Docs touched: `docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-965 labels queued timing rail counters as transport context`

## Why This Ticket Existed

Prevent Jam source clock from overstating Source Timing confidence when beat or phrase grid evidence is absent.

## What Shipped

- Gated Jam source clock components through SourceTimingSummaryView counts, added partial clock labels such as source b- bar8 p-, added full-grid positive UI coverage, and updated the TUI spec.

## Notes

- None

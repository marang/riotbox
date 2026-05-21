# `RIOTBOX-869` Align Source timing panel with shared Jam timing summary

- Ticket: `RIOTBOX-869`
- Title: `Align Source timing panel with shared Jam timing summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-869/align-source-timing-panel-with-shared-jam-timing-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-869-align-source-timing-panel-with-shared-jam-timing-summary`
- Linear branch: `feature/riotbox-869-align-source-timing-panel-with-shared-jam-timing-summary`
- Assignee: `Markus`
- Labels: `TUI`, `timing`, `ux`
- PR: `#863 (https://github.com/marang/riotbox/pull/863)`
- Merge commit: `4d489ca46d0c7c5e2989603f0bf361e0d0f820d7`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo test -p riotbox-core source_timing_summary`; `cargo test -p riotbox-app source_timing_observer`; `cargo test -p riotbox-app shell_state_log_source`; `cargo test -p riotbox-app jam_scene_pending_onramp`; `git diff --check`; `just ci`; GitHub Rust CI
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Align the Source timing panel and observer snapshots with the shared Jam Source Timing summary so musician-facing timing trust language does not drift across Jam, Source, observer, and QA surfaces.

## What Shipped

- Added compact beat/downbeat/phrase status labels and counts to SourceTimingSummaryView.
- Routed app observer Source Timing status fields through the shared Jam summary instead of observer-local mapping helpers.
- Routed the Source timing panel through the shared summary for compact status labels while preserving readable TUI wording.
- Updated Source Timing and TUI specs to pin shared-summary ownership for compact timing status fields.

## Notes

- None

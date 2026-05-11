# `RIOTBOX-778` Surface Source Timing grid-use cue in Jam/Source summary

- Ticket: `RIOTBOX-778`
- Title: `Surface Source Timing grid-use cue in Jam/Source summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-778/surface-source-timing-grid-use-cue-in-jamsource-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-778-surface-source-timing-grid-use-cue-in-jamsource-summary`
- Linear branch: `feature/riotbox-778-surface-source-timing-grid-use-cue-in-jamsource-summary`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#772 (https://github.com/marang/riotbox/pull/772)`
- Merge commit: `da45bb925955d51320164df8c4cef0e699e9705b`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-core source_timing_summary -- --nocapture`; `cargo test -p riotbox-app source_timing -- --nocapture`; `cargo test -p riotbox-app --lib -- --nocapture`; `just ci`
- Docs touched: `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 had stable grid_use labels in source timing probe JSON, Feral grid manifests, observer/audio summaries, and validators, but the musician-facing Jam/Source timing summary did not expose that same cue. This risked UI and QA language drifting apart.

## What Shipped

- Added a shared Core helper that derives SourceTimingGridUse from TimingModel.
- Added grid_use to SourceTimingSummaryView and covered unavailable, locked, manual-only, short-loop/manual-confirm, and fallback states.
- Surfaced grid_use in Jam readiness/help lines and Source timing mode lines while preserving the Source warning row.
- Updated TUI snapshot tests and documented the shared grid_use display contract.

## Notes

- No ActionCommand, queue path, session/replay state, analyzer threshold, lane behavior, or audio rendering behavior changed.

# `RIOTBOX-278` Project Capture recent and provenance rows into Jam view

- Ticket: `RIOTBOX-278`
- Title: `Project Capture recent and provenance rows into Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-278/project-capture-recent-and-provenance-rows-into-jam-view`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-278-project-capture-recent-and-provenance-rows-into-jam-view`
- Linear branch: `feature/riotbox-278-project-capture-recent-and-provenance-rows-into-jam-view`
- PR: `#268`
- Merge commit: `01a172b`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-279`

## Why This Ticket Existed

The Capture screen still rendered `Recent Captures` and `Provenance` by directly scanning `session.captures` in the TUI. Those panels are visible Capture UX, so the renderer should consume Jam view rows while the view model owns session-to-display projection.

## What Shipped

- Added recent Capture row and latest provenance line projections to `CaptureSummaryView`.
- Updated Capture `Recent Captures` and `Provenance` panels to render from Jam view fields.
- Preserved current source-window/provenance wording and refreshed tests after direct session mutations.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core builds_minimal_jam_view_model`
- `cargo test -p riotbox-app source_window`
- `cargo test -p riotbox-app renders_capture_shell_snapshot_with_capture_context`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI projection slice only; no Capture persistence, audio behavior, or layout behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

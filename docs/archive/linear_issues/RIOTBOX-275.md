# `RIOTBOX-275` Project Capture handoff readiness into Jam view

- Ticket: `RIOTBOX-275`
- Title: `Project Capture handoff readiness into Jam view`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-275/project-capture-handoff-readiness-into-jam-view`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-275-project-capture-handoff-readiness-into-jam-view`
- Linear branch: `feature/riotbox-275-project-capture-handoff-readiness-into-jam-view`
- PR: `#265`
- Merge commit: `0066991`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-276`

## Why This Ticket Existed

`RIOTBOX-273` added `src` / `fallback` handoff wording in the Capture UI, but the TUI still derived that readiness by inspecting the latest session capture directly. The confidence cue belongs in the Jam view projection so the perform-facing Capture surface does not keep learning more session internals.

## What Shipped

- Added typed `CaptureHandoffReadinessView` to `CaptureSummaryView`.
- Updated Capture `Do Next` and heard-path copy to render `src` / `fallback` from the Jam view projection.
- Recorded the projection rule in `docs/research_decision_log.md`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core builds_minimal_jam_view_model`
- `cargo test -p riotbox-app renders_capture_handoff_source_readiness_for_w30_targets`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- UI projection slice only; no audio behavior, persistence, source cache, or Capture layout behavior changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

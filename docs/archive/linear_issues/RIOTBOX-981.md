# `RIOTBOX-981` Show Source Map capture range from capture length intent

- Ticket: `RIOTBOX-981`
- Title: `Show Source Map capture range from capture length intent`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-981/show-source-map-capture-range-from-capture-length-intent`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-981-show-source-map-capture-range-from-capture-length-intent`
- Linear branch: `feature/riotbox-981-show-source-map-capture-range-from-capture-length-intent`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#973 (https://github.com/marang/riotbox/pull/973)`
- Merge commit: `91383f5f5b2c3f4fab621a3832d87e84d16454fe`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-981-rebased-source-map.log cargo test -p riotbox-core source_map -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-981-rebased-source-shell.log cargo test -p riotbox-app renders_source_shell_snapshot -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-981-rebased-ci.log just ci`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make the Source Map show what the current capture length intent would capture before a musician commits c, without creating a sample-editor selection model.

## What Shipped

- Added capture_range_row to SourceMapView.
- Projected capture range from Session transport position, Source Timing readiness, and runtime capture length intent.
- Rendered the compact marker row on the Source screen while leaving fallback/unconfirmed timing unavailable.

## Notes

- Rebased onto current main after RIOTBOX-980 merged, retargeted PR #973 to main, then reran focused projection/render tests and just ci.

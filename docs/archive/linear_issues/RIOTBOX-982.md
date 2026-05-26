# `RIOTBOX-982` Expose Source Map capture range in observer snapshots

- Ticket: `RIOTBOX-982`
- Title: `Expose Source Map capture range in observer snapshots`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-982/expose-source-map-capture-range-in-observer-snapshots`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-982-expose-source-map-capture-range-in-observer-snapshots`
- Linear branch: `feature/riotbox-982-expose-source-map-capture-range-in-observer-snapshots`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#974 (https://github.com/marang/riotbox/pull/974)`
- Merge commit: `17b2c8b2fbc33e15921eaa3d09b4dd07bc46267f`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-982-rebased-observer.log cargo test -p riotbox-app --bin riotbox-app source_timing_observer -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-982-rebased-ci.log just ci`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Expose the same Source Map capture-range projection to observer/QA streams so visible capture intent can be checked without terminal snapshot scraping.

## What Shipped

- Added a top-level source_map block to observer snapshots sourced from SourceMapView.
- Exposed mode, trust label, rows, playhead column, capture range row/availability, current region, navigation hint, and capture hint.
- Added observer tests for bar-grid timing with capture range and untrusted timing without bar-accurate range.

## Notes

- Rebased onto current main after RIOTBOX-981 merged, retargeted PR #974 to main, then reran observer test and just ci.

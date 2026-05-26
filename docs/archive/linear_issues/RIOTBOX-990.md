# `RIOTBOX-990` Back Source Map block rows with typed source energy buckets

- Ticket: `RIOTBOX-990`
- Title: `Back Source Map block rows with typed source energy buckets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-990/back-source-map-block-rows-with-typed-source-energy-buckets`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-990-back-source-map-block-rows-with-typed-source-energy-buckets`
- Linear branch: `feature/riotbox-990-back-source-map-block-rows-with-typed-source-energy-buckets`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#982 (https://github.com/marang/riotbox/pull/982)`
- Merge commit: `6c6a8d58919e054300a5d23a8d75535ae5c9c418`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-990-rebased-source-map.log cargo test -p riotbox-core source_map -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-990-rebased-legacy.log cargo test -p riotbox-core legacy_source_graph_without_source_map_evidence_still_loads -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-990-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/source_graph_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make compact Source Map block rows read from a durable typed source-data contract instead of only coarse section labels.

## What Shipped

- Added default-compatible source_map.buckets evidence to SourceGraph.
- Added SourceMapBucket and SourceMapPeakClass for compact energy/loudness and transient emphasis.
- Made SourceMapView prefer bucket-backed energy and peak rows while preserving section and anchor/asset fallback.

## Notes

- Rebased onto current main after RIOTBOX-989 merged, retargeted PR #982 to main, then reran Source Map/legacy tests and just ci.

# `RIOTBOX-992` Prove decoded Source Map buckets reach observer ingest surface

- Ticket: `RIOTBOX-992`
- Title: `Prove decoded Source Map buckets reach observer ingest surface`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-992/prove-decoded-source-map-buckets-reach-observer-ingest-surface`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-992-prove-decoded-source-map-buckets-reach-observer-ingest`
- Linear branch: `feature/riotbox-992-prove-decoded-source-map-buckets-reach-observer-ingest`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#984 (https://github.com/marang/riotbox/pull/984)`
- Merge commit: `008152db396891f4eb3391120f6aa30738e64f01`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-992-rebased-ingest.log cargo test -p riotbox-app --bin riotbox-app ingest_observer_source_map_uses_decoded_bucket_evidence -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-992-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prove decoded Source Map bucket evidence reaches the musician-facing Jam/observer Source Map surface after normal app ingest, not only sidecar-client parsing.

## What Shipped

- Added focused app-bin ingest test using a generated rising-amplitude WAV.
- Ran the stdio sidecar ingest path through JamAppState::analyze_source_file_to_json.
- Asserted parsed buckets, multi-level bucket-backed energy contour, peak evidence, and observer snapshot parity with SourceMapView.

## Notes

- Rebased onto current main after RIOTBOX-991 merged, retargeted PR #984 to main, then reran focused ingest test and just ci.

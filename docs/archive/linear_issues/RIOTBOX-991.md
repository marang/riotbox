# `RIOTBOX-991` Populate Source Map buckets from decoded WAV sidecar analysis

- Ticket: `RIOTBOX-991`
- Title: `Populate Source Map buckets from decoded WAV sidecar analysis`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-991/populate-source-map-buckets-from-decoded-wav-sidecar-analysis`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-991-populate-source-map-buckets-from-decoded-wav-sidecar`
- Linear branch: `feature/riotbox-991-populate-source-map-buckets-from-decoded-wav-sidecar`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#983 (https://github.com/marang/riotbox/pull/983)`
- Merge commit: `3a48750e76d0ac8dd1f80bf4d68236e727b55578`
- Deleted from Linear: `2026-05-26`
- Verification: `python3 -m py_compile python/sidecar/json_stdio_sidecar.py`; `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-991-rebased-sidecar-test.log cargo test -p riotbox-sidecar stdio_sidecar_can_analyze_a_real_source_file_path -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-991-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make source_map.buckets real decoded-WAV ingest evidence instead of only fixture data.

## What Shipped

- Added deterministic Source Map bucket extraction to json_stdio_sidecar.py.
- Emitted bucket time span, RMS-derived energy class, local peak/positive-flux peak class, confidence, and provider provenance.
- Added Rust sidecar-client assertions that generated WAV analysis parses 32 Source Map buckets.

## Notes

- Rebased onto current main after RIOTBOX-990 merged, retargeted PR #983 to main, then reran sidecar test and just ci.

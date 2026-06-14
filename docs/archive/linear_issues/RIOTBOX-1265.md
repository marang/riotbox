# `RIOTBOX-1265` MC-202 measured source phrase evidence from real audio

- Ticket: `RIOTBOX-1265`
- Title: `MC-202 measured source phrase evidence from real audio`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1265/mc-202-measured-source-phrase-evidence-from-real-audio`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-14`
- Finished: `2026-06-14`
- Branch: `feature/riotbox-1265-measured-source-phrase-evidence`
- Linear branch: `feature/riotbox-1265-mc-202-measured-source-phrase-evidence-from-real-audio`
- Assignee: `Markus`
- Labels: None
- PR: `#1240 (https://github.com/marang/riotbox/pull/1240)`
- Merge commit: `5590999e8f5736b04c06b7bad7720b86359e1cb1`
- Deleted from Linear: `2026-06-14`
- Verification: `cargo fmt`; `cargo test -p riotbox-core mc202_source_phrase_features --quiet`; `cargo test -p riotbox-audio measured_phrase_features --quiet`; `cargo test -p riotbox-sidecar stdio_sidecar_can_analyze_a_real_source_file_path --quiet`; `cargo test -p riotbox-app ingest_observer_source_map_uses_decoded_bucket_evidence --quiet`; `cargo test -p riotbox-core --quiet`; `cargo test -p riotbox-audio --quiet`; `cargo test -p riotbox-sidecar --quiet`; `cargo test -p riotbox-app --quiet`; `python3 -m py_compile python/sidecar/json_stdio_sidecar.py`; `cargo clippy --all-targets --all-features -- -D warnings`; `git diff --check`; `just ci`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`, `docs/research_decision_log.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: `RIOTBOX-1266 must generate source-backed MC-202 candidate families from these measured features; RIOTBOX-1264 remains open until producer-grade listening-reviewed quality is met.`

## Why This Ticket Existed

MC-202 source-composed planning needed real measured phrase-level audio evidence instead of relying on Source Graph labels, tags, anchors, or fingerprints.

## What Shipped

- Added persisted PhraseAudioFeatures to SourceGraph with legacy JSON compatibility and source-graph spec coverage.
- Added riotbox-audio phrase analysis over SourceAudioCache for low-band RMS, low/mid ratio, low-band movement, transient/offbeat density, roughness, brightness, hook-restraint hint, confidence, and provenance.
- Updated the Python sidecar analyze_source_file path to emit measured phrase audio features for decoded WAV ingest.
- Changed MC-202 feature derivation to prefer trusted measured audio evidence and mark weak measured provenance as untrusted.

## Notes

- None

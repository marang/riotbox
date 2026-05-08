# P012 Source Timing Contract Review - 2026-05-08

## Scope

Review-codebase checkpoint for `P012 | Source Timing Intelligence`, focused on the current timing contract seams:

- Source Graph timing model, probe candidate reports, and warning/policy labels
- shared Jam source timing presentation summary
- user-session observer `source_timing` snapshot
- observer/audio correlation summary, validators, and fixture smokes
- Source Timing Intelligence, TUI, Audio QA, and observer/audio summary docs

This review intentionally did not cover unrelated P011 replay/recovery areas or audio rendering internals outside the observer/audio timing contract.

## Reviewed Paths

- `crates/riotbox-core/src/source_graph/timing*.rs`
- `crates/riotbox-core/src/source_graph/timing_probe_candidates/`
- `crates/riotbox-core/src/view/jam/source_timing_summary.rs`
- `crates/riotbox-app/src/observer.rs`
- `crates/riotbox-app/src/source_timing_cues.rs`
- `crates/riotbox-app/src/bin/observer_audio_correlate/`
- `crates/riotbox-app/src/bin/user_session_observer_probe/`
- `scripts/validate_observer_audio_summary_json.py`
- `scripts/validate_user_session_observer_ndjson.py`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/specs/tui_screen_spec.md`
- `docs/specs/audio_qa_workflow_spec.md`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`

## Findings

No concrete defects found in the reviewed contract surface.

## Boundary Assessment

- Source Graph remains the durable timing truth. The reviewed observer and QA paths do not introduce a second timing model.
- `SourceTimingSummaryView` is the shared presentation contract for musician-facing cue, timing quality, degraded policy, primary warning, and compact anchor evidence.
- Observer snapshots correctly keep raw beat/downbeat/phrase counts and full warning lists as Source Graph diagnostics while using the shared Jam summary for presentation fields.
- Observer/audio correlation validates both control-path timing evidence and generated manifest-side output evidence without requiring exact anchor-count equality across paths.
- The current contract still clearly marks the detector as a bounded skeleton, not a production-grade arbitrary-audio beat/downbeat detector.

## Context Budget

The reviewed Rust files stayed within the current soft file budget. The largest files in scope were:

- `crates/riotbox-app/src/bin/user_session_observer_probe/probe_scenarios.rs`: 482 lines
- `crates/riotbox-app/src/bin/observer_audio_correlate/tests.rs`: 449 lines
- `crates/riotbox-core/src/view/jam/source_timing_summary.rs`: 406 lines

No mechanical split is recommended from this checkpoint.

## Residual Risks

- P012 still lacks production-grade arbitrary-audio BPM/downbeat detection; this is already documented as future P012+ work.
- Current observer/audio proofs are strong contract and fixture checks, but they are not a substitute for later full musician-facing listening gates across arbitrary sources.
- The shared cue mappings exist in Rust and validator scripts. They are currently consistent, but future label additions should update code, docs, fixtures, and validators in one slice.

## Verification

- `cargo test -p riotbox-core source_timing_summary -- --nocapture`
- `cargo test -p riotbox-app --bin observer_audio_correlate observer_source_timing -- --nocapture`
- `cargo test -p riotbox-app --bin user_session_observer_probe feral_grid -- --nocapture`
- `just observer-audio-summary-validator-fixtures`
- `just user-session-observer-validator-fixtures`
- `just observer-audio-correlate-json-fixture`

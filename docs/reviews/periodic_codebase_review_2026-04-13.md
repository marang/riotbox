# Periodic Codebase Review 2026-04-13

Scope: repo root current state after `RIOTBOX-28` through `RIOTBOX-32`

This review follows the periodic `review-codebase` cadence. It is not a diff review. The focus is current architecture boundaries, drift risk, and near-term maintainability.

## Findings

- **Location**: [crates/riotbox-app/src/bin/riotbox-app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/bin/riotbox-app.rs:117), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:321)
- **Category**: scope
- **Severity**: major
- **Title**: TUI event loop currently owns transport advancement and commit timing
- **Description**: The terminal event loop both advances transport time and commits queued actions. `run_event_loop(...)` drives `tick_delta_beats(...)` on every UI poll tick, and `JamAppState::advance_transport_by(...)` converts that UI cadence into beat/bar/phrase boundaries. This couples musical timing to terminal redraw/input polling, which conflicts with the repo’s realtime boundary rules and will force a rewrite once audio or a real scheduler becomes authoritative.
- **Suggestion**: Move transport advancement and boundary emission into a dedicated runtime or scheduler layer. The TUI should render snapshots and dispatch intents, not be the source of musical clock progression.

- **Location**: [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:373), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:598), [docs/specs/tui_screen_spec.md](/home/markus/Dev/riotbox/docs/specs/tui_screen_spec.md:199)
- **Category**: scope
- **Severity**: major
- **Title**: Pending TR-909 state is persisted as if it were already committed
- **Description**: `queue_tr909_fill(...)` flips `fill_armed_next_bar` inside persisted `session.runtime_state` immediately, even though the fill action itself remains only in the in-memory queue until commit. `JamAppState::save(...)` writes the session but does not persist the pending queue. Saving and reloading after queueing a fill can therefore recreate committed lane state without the queued action that should justify it. That violates the TUI rule that pending actions must not be represented as committed state.
- **Suggestion**: Either derive “armed” display state from the pending queue or persist the pending queue explicitly. Do not mutate persisted lane state before the corresponding action commits.

- **Location**: [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:622), [docs/specs/session_file_spec.md](/home/markus/Dev/riotbox/docs/specs/session_file_spec.md:133)
- **Category**: scope
- **Severity**: major
- **Title**: Ingest path hardcodes external graph storage against the current MVP session contract
- **Description**: `session_from_ingested_graph(...)` always writes `GraphStorageMode::External` and leaves `embedded_graph` empty. The current session-file spec explicitly says MVP should start with embedded graph storage unless graph size becomes a real problem. The implementation therefore adds file-coupling and operational complexity that the current contract does not require.
- **Suggestion**: Make embedded graph storage the default ingest path for MVP sessions. Keep external graph storage as an explicit later option once size or workflow constraints justify it.

- **Location**: [crates/riotbox-core/src/session.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/session.rs:21), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:193), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:964)
- **Category**: scope
- **Severity**: major
- **Title**: App layer collapses the plural session graph model to a single active source graph
- **Description**: `SessionFile` already models `source_refs` and `source_graph_refs` as plural collections, but `JamAppState` stores only one `Option<SourceGraph>`, and `resolve_source_graph(...)` loads only the first graph ref unless an explicit external path is supplied. That narrows the app contract below the core session contract and risks future multi-source or multi-graph work requiring invasive refactors.
- **Suggestion**: Either explicitly freeze Riotbox MVP to a single-source session contract everywhere, or add an app-level active-source selector while preserving the plural core model.

- **Location**: [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:203), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:622), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:771), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:964)
- **Category**: scope
- **Severity**: suggestion
- **Title**: `jam_app.rs` is becoming the repo’s main god-object seam
- **Description**: `jam_app.rs` now owns session/file loading, sidecar ingest, transport math, action queue orchestration, committed side effects, capture promotion, graph resolution, and a large body of fixtures/tests. The current file is still understandable, but its responsibilities are expanding faster than the rest of the app boundary, which will make future device-lane work and review granularity harder.
- **Suggestion**: Split the module before the next wave of device-facing features. A pragmatic first cut would separate `ingest`, `runtime`, `capture_flow`, and `storage` concerns, with test fixtures moved into local helpers or dedicated test modules.

## Open Questions

- The core session model is already plural for sources and source graphs. If MVP is intentionally single-source only, that should be made explicit in the core spec and model instead of only in app behavior.
- The current queue is intentionally in-memory. If that remains true, any runtime-state fields that reflect queued intent need stricter discipline to avoid save/reload drift.

## Recommended Follow-up Order

1. Move the terminal-driven transport clock toward an explicit runtime/scheduler boundary.
2. Fix pending-vs-committed TR-909 fill state so save/reload semantics stay honest.
3. Align ingest graph storage with the current session-file contract.
4. Decide whether Riotbox MVP is truly single-source or whether the app should respect the plural session contract now.

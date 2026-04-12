# Riotbox Whole-Codebase Review

Date: 2026-04-13  
Ticket: `RIOTBOX-23`  
Scope:

- `crates/riotbox-core`
- `crates/riotbox-app`
- `crates/riotbox-audio`
- `crates/riotbox-sidecar`
- `python/sidecar`

Verification used during review:

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Findings

### 1. Terminal cleanup is not guaranteed on early UI startup failure

Severity: high

Files:

- [crates/riotbox-app/src/bin/riotbox-app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/bin/riotbox-app.rs:67)

Why it matters:

- `run_terminal_ui(...)` enables raw mode before the alternate-screen terminal is fully constructed.
- If `EnterAlternateScreen` or `Terminal::new(...)` fails, the function returns immediately and never reaches the cleanup path at lines 80-82.
- That leaves the user in raw mode and/or the alternate screen, which is a real CLI usability failure and makes debugging harder.

Evidence:

- Raw mode is enabled at line 71.
- Alternate-screen entry happens at line 73.
- `Terminal::new(backend)?` can still fail at line 76.
- Cleanup only happens after `run_event_loop(...)` returns normally through the later path.

Recommendation:

- Move the terminal lifecycle into a small RAII guard or wrapper that always restores raw mode and leaves the alternate screen in `Drop`.

### 2. App load/save path ignores the session graph-storage contract

Severity: high

Files:

- [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:219)
- [crates/riotbox-core/src/session.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/session.rs:61)

Why it matters:

- The core session model explicitly supports both `Embedded` and `External` graph storage through `GraphStorageMode`, `embedded_graph`, and `external_path`.
- `JamAppState::from_json_files(...)` ignores that contract and always requires a separate graph JSON file to be passed in and loaded from disk.
- `JamAppState::save(...)` mirrors the same assumption and only writes the out-of-band graph file.

Evidence:

- `SourceGraphRef` supports both storage modes in `session.rs`.
- `from_json_files(...)` unconditionally calls `load_source_graph_json(&source_graph_path)` at line 226.
- `save(...)` only writes `files.source_graph_path` when `self.source_graph` exists at lines 321-327.

Consequence:

- A valid `SessionFile` with an embedded graph cannot be loaded by the app even though the core contract says it is valid.
- This is already contract drift between `riotbox-core` and `riotbox-app`.

Recommendation:

- Make app loading resolve the graph from `SessionFile` first, honoring `storage_mode`, and only use an explicit graph path as a fallback/override when that is part of the launch mode.

### 3. Sidecar request IDs exist in the protocol but are never validated by the client

Severity: medium

Files:

- [crates/riotbox-sidecar/src/protocol.rs](/home/markus/Dev/riotbox/crates/riotbox-sidecar/src/protocol.rs:9)
- [crates/riotbox-sidecar/src/client.rs](/home/markus/Dev/riotbox/crates/riotbox-sidecar/src/client.rs:95)
- [crates/riotbox-sidecar/src/client.rs](/home/markus/Dev/riotbox/crates/riotbox-sidecar/src/client.rs:165)

Why it matters:

- The transport protocol includes `request_id` on every request and response payload.
- The client sends unique request IDs, but it never checks that the response it received actually matches the request it just issued.
- Today the client is single-request and sequential, so this is latent rather than catastrophic, but the current code is already carrying a contract it does not enforce.

Evidence:

- `PingPayload`, `AnalyzeSourceFilePayload`, `SourceGraphBuiltPayload`, and `SidecarErrorPayload` all carry `request_id`.
- `ping()`, `build_source_graph_stub()`, and `analyze_source_file()` write requests and accept the next parsed response by type only.
- `read_response()` at lines 165-173 returns any decoded response without checking correlation.

Consequence:

- A stale line, unexpected out-of-band response, or future progress/event message could be mis-associated with the wrong request.
- That makes the current transport harder to extend safely.

Recommendation:

- Validate `request_id` in the client before accepting the payload, and surface a distinct mismatch error.

### 4. Reloaded transport state loses bar and phrase position information

Severity: medium

Files:

- [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:392)
- [crates/riotbox-core/src/transport.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/transport.rs:24)

Why it matters:

- `TransportClockState` carries beat, bar, and phrase indices because commit-boundary metadata is supposed to be explicit.
- When the app reconstructs runtime transport from a saved session, it only restores `position_beats`; `bar_index` and `phrase_index` are reset to `0`.
- That means the persisted session and the in-memory transport clock disagree immediately after load.

Evidence:

- `transport_clock_from_session(...)` sets `beat_index` from `position_beats.floor()` but hardcodes `bar_index: 0` and `phrase_index: 0`.
- `CommitBoundaryState` explicitly includes `bar_index` and `phrase_index`, and queue commit summaries include them in user-visible result strings.

Consequence:

- Any future commit or UI path that trusts the reloaded `runtime.transport` before an external timing refresh arrives will carry incorrect boundary metadata.
- This weakens replay and restore correctness at exactly the point where the architecture says determinism matters.

Recommendation:

- Either persist enough transport timing detail in the session to reconstruct bar/phrase indices correctly, or mark those fields unknown/unavailable rather than silently resetting them to zero.

## Test Gaps

- No test currently exercises terminal cleanup on startup failure in the TUI binary path.
- No app-level test currently proves that embedded-graph sessions load correctly.
- No sidecar-client test currently asserts request/response ID correlation.
- No test currently covers transport reconstruction fidelity after save/load.

## Recommendation Before Next Feature Slice

- Fix findings 1 and 2 before taking the next implementation ticket.
- Treat findings 3 and 4 as short follow-up hardening slices if they are not addressed in the same pass.

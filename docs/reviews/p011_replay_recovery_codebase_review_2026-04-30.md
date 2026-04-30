# P011 Replay / Recovery Codebase Review

Date: 2026-04-30
Scope:

- `crates/riotbox-core/src/replay/`
- `crates/riotbox-app/src/jam_app/restore_replay.rs`
- `crates/riotbox-app/src/jam_app/recovery*.rs`
- P011 replay / recovery app test shards

## Verdict

No architecture blocker found.

The current replay/recovery path still follows the intended architecture:
action log stays canonical, snapshots act as payload-backed accelerators, and
the app restore runner delegates to the core target-suffix executor instead of
creating a second replay model.

## Findings

### 1. Large artifact replay test shards are now review-cost hotspots

- Location: `crates/riotbox-app/src/jam_app/tests/w30_artifact_replay.rs:1`
- Location: `crates/riotbox-app/src/jam_app/tests/w30_capture_to_pad_replay.rs:1`
- Category: scope
- Severity: minor

Both files now sit at roughly the soft file-size boundary and mix multiple
artifact families in one shard. `w30_artifact_replay.rs` covers loop-freeze,
promote-resample, and promote-capture-to-pad resample replay.
`w30_capture_to_pad_replay.rs` covers direct W-30 capture, bar-group capture,
and loop capture. The tests are valuable, but future additions will make these
files expensive for branch review and agent context.

Suggestion: split only by real responsibility, not mechanically. Good boundaries
would be `w30_resample_artifact_replay.rs`, `w30_source_capture_replay.rs`, and
`capture_loop_replay.rs`, with shared fixture builders for persisted capture
metadata and fallback-output assertions.

### 2. Snapshot-payload restore parity tests duplicate anchor construction

- Location: `crates/riotbox-app/src/jam_app/tests/w30_replay.rs:194`
- Location: `crates/riotbox-app/src/jam_app/tests/tr909_replay.rs:169`
- Location: `crates/riotbox-app/src/jam_app/tests/mc202_restore_replay.rs:22`
- Category: scope
- Severity: minor

The new parity probes intentionally repeat the same pattern: build committed
plan, compute action cursors, materialize an anchor runtime state, construct a
snapshot payload, restore to a target cursor, and assert report/runtime/output
convergence. The pattern is correct, but the repeated boilerplate makes it easy
for future probes to accidentally skip one assertion class or use subtly
different cursor math.

Suggestion: add a small app-test helper for payload-backed restore probes. It
should not hide the musical assertions, but it should centralize action cursor
lookup, anchor materialization, snapshot payload construction, and basic restore
report identity assertions.

## Positive Checks

- `crates/riotbox-core/src/replay/target_execution.rs:52` keeps payload
  hydration explicit and applies only the existing target suffix executor after
  the payload is installed.
- `crates/riotbox-app/src/jam_app/restore_replay.rs:35` keeps the app boundary
  narrow: hydrate through core, replace session with the reported hydrated
  session, then refresh derived runtime/view state.
- The new device-family probes cover control path and output path: W-30 preview,
  TR-909 render, and MC-202 render now all have snapshot-payload parity evidence.

## Recommended Follow-Ups

- Add a small restore-parity test helper before the next wave of device-family
  parity probes.
- Split the two W-30 artifact replay test shards when the next artifact replay
  change touches either file.
- Keep P011 focused on one replay/recovery seam at a time; do not expand into
  automatic recovery selection until the manual recovery path remains explicit
  and testable.

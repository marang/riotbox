# Riotbox Session File Spec

Version: 0.1  
Status: Draft  
Audience: realtime, session, TUI, Ghost, QA

---

## 1. Purpose

This document defines the Riotbox session format.

It exists so that:

- a session can be saved and restored deterministically
- action history, source analysis, and live state remain coherent
- Ghost, TUI, and device lanes share the same persistence model
- future migrations happen intentionally instead of by accident

---

## 2. Core Rule

A Riotbox session is not just a preset dump.

It is the minimum complete state needed to restore:

- source references
- analysis references
- action history
- live configuration
- capture lineage needed for replay-safe continuation

If a state change matters to musical behavior after reload, it belongs in the session model or in a referenced artifact.

---

## 3. Session Goals

- deterministic restore
- explicit versioning
- replay-safe action history
- compact enough for routine save/load
- resilient to partial provider upgrades
- compatible with future migrations

---

## 4. Top-Level Shape

Canonical shape:

```text
SessionFile {
  session_version
  session_id
  created_at
  updated_at
  app_version
  source_refs
  source_graph_refs
  runtime_state
  action_log
  snapshots
  captures
  ghost_state
  notes
}
```

---

## 5. Session Identity

Required fields:

- `session_version`
- `session_id`
- `created_at`
- `updated_at`
- `app_version`

Rules:

- `session_version` governs migration
- `session_id` must be stable for the life of the session
- `app_version` records the writing version but does not replace explicit schema versioning

---

## 6. Source References

MVP note:

- Riotbox MVP currently supports exactly one active source per session
- `source_refs` remains plural in the schema so later migrations can grow without rewriting the top-level shape
- app/runtime code should reject sessions that silently contain multiple active sources during MVP

```text
SourceRef {
  source_id
  path_hint
  content_hash
  duration_sec
  decode_profile
}
```

Rules:

- session restore should prefer `content_hash` verification over path trust
- `path_hint` may help the UI, but path alone must not be the authority

---

## 7. Source Graph References

The session does not need to inline every raw analysis payload.

It must preserve enough to restore or validate the graph used by the session.

```text
SourceGraphRef {
  source_id
  graph_version
  graph_hash
  storage_mode
  embedded_graph
  external_path
  provenance
}
```

Allowed `storage_mode` values:

- `embedded`
- `external`

MVP preference:

- start with `embedded` unless graph size becomes a real problem

MVP note:

- Riotbox MVP currently supports exactly one active source graph reference per session
- that graph reference must match the single active `source_ref`
- plural shape is retained for forward compatibility, not to imply current multi-source support in the app/runtime

---

## 8. Runtime State

`runtime_state` is the live musical state that must exist after load even before replay begins.

```text
RuntimeState {
  transport
  macro_state
  lane_state
  mixer_state
  scene_state
  lock_state
  pending_policy
  undo_state
}
```

### 8.1 Transport

- tempo basis
- playback position if persisted
- current scene

### 8.2 Macro state

Examples:

- source retain
- chaos
- 202 touch
- W-30 grit
- 909 slam

### 8.3 Lane state

Per lane, store only the state required to reproduce behavior:

- MC-202 role, phrase references, and committed phrase variant
- W-30 preview intent plus bank/pad focus and current capture reference
- TR-909 takeover, pattern, and reinforcement state

### 8.4 Mixer state

- per-bus levels
- sends
- mute / solo if relevant
- profile flags that alter render behavior

### 8.5 Scene state

- active scene
- scene list or scene references
- restore pointers

### 8.6 Lock state

- locked objects
- lock owner or actor if relevant

### 8.7 Undo state

Undo state stores bounded restore snapshots for committed moves whose audible state must roll back.

Current MVP use:

- MC-202 commit-time lane snapshots keyed by action id
- previous role, phrase reference, phrase variant, and touch

Rules:

- snapshots must be explicit session state, not callback-local memory
- snapshots are only for undo restore, not a second arrangement or phrase system
- undo must refresh the typed render projection after applying a snapshot

---

## 9. Action Log

The session must persist the replay-relevant action history.

```text
ActionLog {
  actions[]
  commit_records[]
  replay_policy
}
```

Each action must follow the Action Lexicon contract.

Minimum stored fields:

- action ID
- actor
- command
- params
- target scope
- requested time
- quantization
- final status
- committed time if committed
- undo payload or undo reference

Commit records preserve replay-relevant musical boundary metadata separately
from human-readable result summaries:

- action ID
- boundary kind
- beat index
- bar index
- phrase index
- scene ID when known
- commit sequence within that boundary
- committed timestamp

Rules:

- uncommitted or transient UI noise does not belong in the durable log by default
- committed musical state changes do
- budget and replay logic must consume structured commit records, not parse
  result-summary strings

---

## 10. Snapshots

Snapshots provide coarse restore points in addition to action replay.

```text
Snapshot {
  snapshot_id
  created_at
  label
  action_cursor
  payload?
}

SnapshotPayload {
  payload_version
  snapshot_id
  action_cursor
  runtime_state
}
```

Rules:

- snapshots should not replace the action log
- snapshots accelerate restore and allow safer rollback
- snapshot payloads, when present, should be typed and versioned rather than
  external state dumps
- the first payload boundary should carry replay-relevant `RuntimeState` at the
  snapshot cursor and no separate action, lane, or arrangement model
- snapshot restore must validate payload version, `snapshot_id`, and
  `action_cursor` before using the payload as an anchor
- session load validation must reject present payloads whose `snapshot_id` or
  `action_cursor` does not match the owning snapshot; missing payloads remain
  valid until a snapshot-anchored restore path explicitly requires one
- snapshot payload hydration should happen before the existing replay suffix
  executor runs; the payload must not execute actions itself
- the current save path may fill missing payloads only for existing snapshots at
  the latest action cursor, because the top-level `runtime_state` represents only
  latest materialized state

---

## 11. Captures

Captured material must be representable without losing provenance.

```text
CaptureRef {
  capture_id
  type
  source_origin_refs
  source_window?
  lineage_capture_refs
  resample_generation_depth
  created_from_action
  storage_path
  assigned_target
  notes
}
```

Examples:

- W-30 pad capture
- loop promotion
- internal resample

Minimum provenance:

- source object references when available
- source audio window metadata when the capture maps directly back to the loaded source
- explicit capture-to-capture lineage when the material is internally reused
- resample generation depth for internally derived material
- generating action ID
- resulting assigned pad or bank if applicable

`source_window` is optional for backward compatibility and for captures that are derived from internal resampling rather than a direct source range. When present, it should preserve source id, start/end seconds, and start/end source frames so later raw playback can resolve audio without guessing from UI state.

For committed source-backed captures loaded from a session file, `storage_path` should be backed by a real PCM WAV artifact relative to the session file directory unless it is absolute. Artifact writing belongs to the non-realtime app commit path; the audio callback must never write capture files.

For internally printed W-30 resample captures, `storage_path` should point to the printed bus artifact rather than the source-window input artifact. Such captures should preserve input ownership through `lineage_capture_refs` and `resample_generation_depth`; `source_window` should be omitted unless the printed result is intentionally still a literal source-window copy. This keeps reload and later pad playback pointed at the exact printed audio instead of reconstructing it from source metadata.

---

## 12. Ghost State

Ghost state should be persisted only to the extent that it affects deterministic continuation or user trust.

```text
GhostState {
  mode
  budget_state
  suggestion_history
  lock_awareness_state
}
```

MVP rule:

- persist accepted or active constraints
- Watch-mode suggestions are read-only proposal objects until a later Assist flow accepts them.
- Session `suggestion_history` should store compact user-facing records for continuity, not raw Ghost reasoning or queue/action objects.
- do not persist opaque internal deliberation blobs

---

## 13. Notes and User Metadata

Optional user-facing notes may include:

- session title
- tags
- short comments
- favorite captures

These are secondary and must never replace core replay state.

---

## 14. Versioning and Migration

Versioning is mandatory from the first saveable session.

Rules:

- every session file must carry an explicit schema version
- breaking changes require migration code or explicit incompatibility handling
- graph and action schema versions must be validated during load

If load detects mismatch, Riotbox should:

- explain the mismatch
- offer degraded load where safe
- refuse silent corruption

---

## 15. Save Rules

Save must be:

- atomic enough to avoid half-written sessions
- explicit about embedded vs external artifacts
- stable under repeated writes

MVP expectation:

- one session save path
- one clear artifact layout
- no hidden autosave complexity before the format is stable
- current JSON saves serialize first, write beside the target, then rename into place; this is the MVP crash-safety seam, not a full multi-file transaction

---

## 16. Load Rules

Load must validate:

- schema version
- source hash compatibility when possible
- source graph compatibility
- action log readability
- referenced capture existence where required

Load may degrade only when the resulting state remains honest and usable.

Current MVP crash-recovery boundary:

- truncated or partial session JSON must fail with an explicit parse error
- load must not silently repair or replace the requested file with guessed state
- adjacent valid session files can still be loaded manually, but automatic fallback selection is not part of MVP yet

### 16.1 MVP crash recovery policy

The MVP recovery model is explicit manual recovery, not hidden automatic repair.

Current save behavior:

- session JSON is serialized before touching the target path
- the serialized payload is written to a hidden sibling temp file named like `.<target-file-name>.tmp-<pid>-<nonce>`
- the temp file is renamed over the target only after the write succeeds
- if rename fails, the temp file is best-effort removed and the existing target remains the authority

Orphan temp-file policy:

- hidden sibling files matching `.<target-file-name>.tmp-*` are treated as interrupted writes for that target
- load must not choose an orphan temp file automatically
- UI or CLI diagnostics may list orphan temp files as recovery clues, but they must be clearly marked as untrusted candidates
- deleting orphan temp files is safe only after the user has confirmed the canonical target and any desired manual backup have been inspected

Autosave policy:

- autosave is not automatic in MVP
- when autosave lands, it should use an explicit sibling name such as `session.autosave.json` or timestamped `session.autosave.<ISO-UTC>.json`
- autosave files must use the same schema validation and replay/commit-record checks as normal session files
- autosave must not overwrite the canonical session without explicit user action

Manual fallback selection:

- if the requested session fails to parse or validate, Riotbox should fail the load with the concrete path and error
- a user may explicitly retry another candidate path, such as a manual backup or future autosave file
- the app must not silently fall back to an adjacent file because that would hide which replay truth was actually loaded

First implementation seam:

- add a non-mutating recovery-candidate scanner that reports canonical target status, orphan temp files, and explicit autosave candidates
- keep the scanner separate from `load_session_json` so normal load remains deterministic and side-effect free
- project scanner results into an explicit manual recovery surface before adding any automatic recovery action

Current implementation:

- `scan_session_recovery_candidates` reports the canonical target plus matching hidden temp and autosave siblings
- candidates are parse-checked as missing, parseable session JSON, invalid session JSON, or unreadable
- the scanner is read-only and does not load, replace, delete, or choose a recovery candidate
- `JamAppState::scan_session_recovery_surface` converts scanner output into TUI/CLI-facing labels, trust levels, details, and action hints
- the recovery surface keeps `selected_candidate` empty and states that Riotbox did not choose, load, replace, or delete any candidate
- load-mode TUI may attach a recovery surface when the requested canonical session loads successfully but adjacent manual candidates exist
- the TUI recovery prompt is guidance only: it must not select, load, replace, delete, or promote a candidate
- operator-facing snapshot-payload and unsupported-suffix label guidance lives in `docs/recovery_notes.md`

---

## 17. MVP Requirements

Session v1 must support:

- one source file
- one embedded Source Graph
- macro and lane state
- action log for committed actions
- basic snapshots
- capture references
- Ghost watch / assist state sufficient for continuity

It does not yet need:

- multiple simultaneous source files
- distributed asset storage
- collaborative session metadata
- performance-take libraries

---

## 18. Validation Requirements

Required validation:

- save / load smoke tests
- deterministic restore from same session file
- action-log replay consistency
- migration guard tests
- missing-asset error-path tests

---

## 19. Open Follow-Ups

This draft should be followed by:

1. exact on-disk layout
2. session migration policy
3. snapshot frequency policy
4. recovery prompt for failed canonical loads, likely as a CLI/manual report before the full TUI can start
5. autosave strategy after the format stabilizes

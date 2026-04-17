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

- MC-202 role and phrase references
- W-30 bank or pad assignment
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

---

## 9. Action Log

The session must persist the replay-relevant action history.

```text
ActionLog {
  actions[]
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

Rules:

- uncommitted or transient UI noise does not belong in the durable log by default
- committed musical state changes do

---

## 10. Snapshots

Snapshots provide coarse restore points in addition to action replay.

```text
Snapshot {
  snapshot_id
  created_at
  label
  state_ref
  action_cursor
}
```

Rules:

- snapshots should not replace the action log
- snapshots accelerate restore and allow safer rollback

---

## 11. Captures

Captured material must be representable without losing provenance.

```text
CaptureRef {
  capture_id
  type
  source_origin_refs
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
- explicit capture-to-capture lineage when the material is internally reused
- resample generation depth for internally derived material
- generating action ID
- resulting assigned pad or bank if applicable

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

---

## 16. Load Rules

Load must validate:

- schema version
- source hash compatibility when possible
- source graph compatibility
- action log readability
- referenced capture existence where required

Load may degrade only when the resulting state remains honest and usable.

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
4. autosave strategy after the format stabilizes

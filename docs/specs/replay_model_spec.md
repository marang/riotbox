# Riotbox Replay Model Spec

Version: 0.1  
Status: Replay Spike Result  
Audience: core, session, audio, TUI, Ghost, QA

Derived from:
- `docs/specs/session_file_spec.md`
- `docs/specs/action_lexicon_spec.md`
- `docs/specs/validation_benchmark_spec.md`

---

## 1. Purpose

This document defines the deterministic replay model for Riotbox.

It exists to answer four questions before runtime work scales:

- what must be replayed
- what may be reconstructed
- which timing metadata is authoritative
- how snapshots relate to full action replay

---

## 2. Core Rule

Riotbox must have one replay truth.

That truth is:

- frozen source references
- frozen Source Graph references
- durable committed action history
- optional snapshots that accelerate restore but do not replace the action log

Replay may not depend on:

- rerunning analysis as if it were stable
- re-asking Ghost what it wanted
- hidden wall-clock timing behavior
- ambient mutable process state

---

## 3. Deterministic Replay Goal

Given:

- the same session file
- the same Source Graph
- the same captures and referenced artifacts
- the same replay-relevant actions

Riotbox should restore the same musical state and produce materially equivalent behavior.

For MVP, this means deterministic structural behavior first.

It does not yet promise sample-perfect offline render identity across future audio backends.

---

## 4. Replay Inputs

The following are replay inputs and must be treated as authoritative:

- source content hash and source identity
- Source Graph identity and provenance
- session runtime state
- action log
- snapshots
- capture references and provenance
- active profile / preset identity where behavior depends on it

The following are **not** authoritative replay inputs:

- requested-at wall clock alone
- transient pending UI state
- rejected Ghost suggestions
- provider availability at restore time

---

## 5. What Must Be Replayed

Must be replayed or represented as durable committed state:

- scene launches and restores
- lane mutations
- quantized capture and promotion events
- lock changes that affect later behavior
- Ghost-accepted actions
- profile or preset changes that alter behavior
- snapshot save points and their action cursor

Rule:

If the user would hear, trust, save, or depend on the change after reload, it belongs in replay truth.

---

## 6. What May Be Reconstructed

The following may be reconstructed from durable replay truth:

- derived Jam view state
- summary counters and warnings
- TUI-focused presentation objects
- non-authoritative caches

The following may be reconstructed only if their inputs are frozen:

- app-level Jam state
- source summaries
- replay-time convenience indexes

---

## 7. Action Log Authority

The action log is the replay spine.

For replay purposes, the authoritative dimensions of an action are:

- command
- params
- target scope and references
- final committed or rejected state
- quantization target
- commit order
- committed-at metadata if present
- undoability semantics where relevant

### 7.1 Requested time vs committed time

Decision:

- `requested_at` is diagnostic and UX-relevant
- `committed_at` is replay-relevant

If the action was queued and committed later, replay must care about the commit boundary, not the initial human request time.

### 7.2 Action ordering

Decision:

- durable log order is authoritative
- replay should process actions in durable committed order

If two actions commit in the same musical window, their stored order must remain stable.

This means the system should eventually expose an explicit commit sequence or durable ordering guarantee instead of relying on incidental vector order alone.

---

## 8. Commit Boundary Authority

Replay should reproduce **musical** commit timing, not human wall-clock latency.

Authoritative timing concept:

- quantization boundary
- ordered commit sequence within that boundary

Helpful but secondary metadata:

- `requested_at`
- wall-clock timestamps

Implication:

Future runtime work should preserve enough metadata to say:

- this action committed on bar N or phrase N
- and it committed in sequence position K within that boundary

---

## 9. Snapshots

Snapshots are replay accelerators, not a second truth system.

Decision:

- snapshots are valid restore anchors
- action log remains canonical

Recommended restore algorithm:

1. load session and Source Graph
2. pick the most recent valid snapshot at or before the target cursor
3. hydrate runtime state from that snapshot or base runtime state
4. replay all committed actions after the snapshot cursor
5. rebuild derived app and TUI state

### 9.1 Snapshot rules

- every snapshot must reference an action cursor
- snapshots should be taken at stable musical points
- snapshots must not contain hidden state that the action log cannot eventually explain

### 9.2 Runtime state relation

Decision:

- `runtime_state` in the session file is the latest materialized state for fast load
- snapshots provide additional anchored restore points

Practical consequence:

`runtime_state` and the latest snapshot should not drift semantically.

---

## 10. Non-Determinism Sources

The main threats to deterministic replay are:

- provider drift in analysis
- random mutation logic
- Ghost suggestion generation
- callback-timing accidents
- hidden capture-side state

### 10.1 Analysis

Decision:

- replay does not rerun analysis as part of normal restore
- replay consumes the frozen Source Graph used by the session

### 10.2 Ghost

Decision:

- accepted Ghost actions replay as normal actions
- Ghost proposals themselves are not replay truth unless accepted

### 10.3 Randomness

Decision:

- future randomized behavior must either:
  - log its concrete chosen params, or
  - log a stable seed plus stable algorithm version

For MVP, concrete committed params are preferred over clever seed magic.

### 10.4 Captures

Decision:

- captures are durable replay artifacts, not recomputed intentions
- replay should reference the capture artifact and its provenance, not attempt to recreate it from scratch unless explicitly designed to do so later

---

## 11. MVP Replay Recommendation

For MVP, use a hybrid replay model:

1. persist latest runtime state
2. persist committed action log
3. persist snapshots with action cursor
4. persist Source Graph directly or by frozen reference
5. persist captures as referenced artifacts

On restore:

- trust frozen analysis and captured artifacts
- use runtime state for fast hydration
- use snapshots plus action replay for verification and deterministic continuation

This is the best tradeoff between:

- fast load
- replay trust
- implementation simplicity
- future debugging

---

## 12. Follow-Up Implementation Implications

This spike implies the following future changes or clarifications:

1. `Action` should eventually gain explicit replay-order metadata beyond incidental array order.
2. snapshot representation should become more explicit than the current lightweight placeholder.
3. future runtime code should expose stable musical boundary identity for commits.
4. capture persistence should remain artifact-first, not recompute-first.
5. validation should include replay-from-snapshot and replay-from-zero-path checks.

---

## 13. Validation Impact

The replay model should drive future validation:

- save / load should preserve state identity
- replay from latest snapshot and full replay from origin should converge
- accepted Ghost actions must replay identically to user actions
- changed provider availability should not change restore of existing sessions

---

## 14. Decision Summary

Short version:

- replay truth is frozen inputs plus committed actions plus snapshots
- committed musical order matters more than request time
- snapshots accelerate replay but do not replace the action log
- analysis and Ghost are not rerun as truth during normal restore
- captures are referenced artifacts, not recomputed guesses

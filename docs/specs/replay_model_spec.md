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

Current implementation:

- `ActionLog.commit_records` stores one structured commit record per committed action.
- Each commit record is keyed by action id and stores the commit boundary plus commit sequence within that boundary.
- Replay and budget logic should consume these structured commit records instead of parsing result summaries or relying only on incidental action vector order.
- A replay-plan builder may consume the existing action log and commit records to produce deterministic committed-order entries, but it must not become a second action, persistence, or repair system.
- Snapshot-vs-origin replay-plan comparisons may select a suffix from the origin plan by using the existing snapshot `action_cursor`.
- Snapshot anchor selection should pick the latest valid snapshot at or before the target action cursor and reject out-of-range cursors instead of silently falling back.
- Target replay planning may combine the origin plan, selected snapshot anchor, and target-limited suffix, but execution and runtime hydration remain separate responsibilities.
- Target replay dry-run summaries may expose selected anchor metadata, target cursor, and suffix action scope for QA and future UI/debug seams, but they must not execute actions or mutate runtime state.
- Latest-snapshot replay convergence summaries may compare the full committed origin against the latest snapshot-to-end suffix for QA, including whether no snapshot forces full replay.
- Session restore rebuilds the app-runtime `last_commit_boundary` from the latest structured commit record so fresh app state does not lose the most recent musical boundary context.

### 7.3 Minimal replay executor boundary

The replay executor is intentionally narrow. It applies only deterministic actions from committed replay-plan entries whose state mutation is explicit enough to replay without consulting UI summaries or log text.

Current supported structural commands:

- `transport.play`
- `transport.pause`
- `transport.stop`
- `transport.seek`
- `lock.object`
- `unlock.object`
- `ghost.set_mode`

Current supported musical commands:

- `mc202.set_role`
- `mc202.generate_follower`
- `mc202.generate_answer`
- `mc202.generate_pressure`
- `mc202.generate_instigator`
- `mc202.mutate_phrase`
- `tr909.set_slam`
- `tr909.fill_next`
- `tr909.reinforce_break`
- `tr909.takeover`
- `tr909.scene_lock`
- `tr909.release`
- `scene.launch`
- `scene.restore`
- `w30.live_recall`
- `w30.trigger_pad`
- `w30.audition_raw_capture`
- `w30.audition_promoted`
- `w30.swap_bank`
- `w30.browse_slice_pool`
- `w30.step_focus`
- `w30.apply_damage_profile`
- `w30.loop_freeze` for explicitly persisted artifact-backed captures
- `promote.resample` for explicitly persisted artifact-backed resample captures

Rules:

- The executor consumes replay-plan entries, not UI summaries or parsed log text.
- Unsupported commands fail explicitly instead of being silently ignored.
- Invalid params fail explicitly instead of guessing defaults.
- Whole-plan application is all-or-nothing: if any entry fails, the session passed to the executor is not mutated.
- Single-entry application may mutate the passed session and should be used only when the caller already accepts that boundary.
- This executor does not perform audio rendering, capture artifact creation, non-allowlisted W-30/TR-909 side effects, Ghost reasoning, source analysis, or snapshot hydration.
- MC-202 replay currently covers the deterministic phrase-lane state needed by downstream projection: role, phrase reference, phrase variant, and MC-202 touch.
- TR-909 replay currently covers the deterministic support-lane state needed by downstream projection: slam, fill, reinforce, takeover, scene-lock, release, pattern references, reinforcement mode, and takeover profile.
- Scene replay currently covers the deterministic active-scene / restore-scene state carried by committed scene launch and restore actions. The minimal executor updates transport current scene, scene active scene, and restore pointer while deliberately staying Source Graph-free. A separate graph-aware replay boundary can hydrate `last_movement` from the pre-action scene state, committed boundary, committed scene action, and frozen Source Graph context.
- W-30 replay currently covers a deterministic cue subset whose committed actions already carry explicit target state: live recall, trigger, audition, bank swap, slice-pool browse, focus step, and damage profile. It updates preview mode, focused bank/pad, last capture, and W-30 grit only.
- `w30.loop_freeze` and `promote.resample` are the first artifact-backed W-30 replay suffixes allowed through the executor. They may only hydrate from explicit persisted capture identity: source capture target, exactly one produced capture, stable storage path, and valid source-window or lineage/depth identity. Replay points `last_capture` at the produced artifact capture so app projection can use the existing cached WAV artifact; it does not regenerate capture audio.
- Other artifact-producing W-30 / capture / promote actions remain intentionally unsupported and must reject without leaving partially applied state.
- Current convergence coverage materializes a snapshot anchor by replaying the safe prefix in tests, then applies the selected suffix and compares the resulting structural state against origin replay; this proves the executor path, not real snapshot payload hydration.
- Replay dry-run and latest-snapshot convergence summaries expose unsupported committed commands for both full-origin and selected-suffix scope so QA can distinguish "needs replay" from "cannot replay this command family yet" without executing or mutating state.
- App runtime diagnostics surface unsupported replay commands as read-only warnings so restore/debug views can explain replay incompleteness without executing unsupported actions.
- App runtime restore-readiness view data exposes read-only status, selected anchor, selected-anchor payload readiness, target suffix scope, and unsupported command labels from the latest-snapshot convergence summary. These labels are presentation data only; they must not execute replay, repair the session, or become a second replay model.
- Target replay suffix execution is available as a core helper for already-hydrated anchor state. It builds the target plan, applies only the selected suffix, preserves all-or-nothing mutation, and reports anchor plus applied action ids. Snapshot payload hydration remains caller-owned and is not implied by this helper.
- App-level target-suffix QA currently proves this helper can feed downstream TR-909 render projection and source-backed W-30 preview projection from test-owned hydrated anchor state. The app restore-runner keeps that explicit caller-owned suffix path, and also has a separate snapshot-payload restore method that hydrates a cloned session through the core payload helper before refreshing app projection. Neither method is normal app startup behavior or automatic recovery selection. App recovery code can also request a read-only target dry-run summary for the current session before mutating restore state.
- Broader musical replay must expand this allowlist command by command with tests that prove both control-path and output-path behavior where audible state is affected. The MC-202 expansion includes app-level render parity proving replayed MC-202 state projects to the same audible render as the committed app path and differs from the preceding phrase. The first TR-909 expansion includes app-level render parity proving replayed TR-909 state projects to the same audible render as the committed app path and that fill, slam, takeover, and release remain distinguishable at the output seam. The first W-30 expansion includes app-level preview parity proving replayed slice-pool cue state projects to the same source-backed W-30 preview render as the committed app path and differs from the previous recall preview. Scene replay now has graph-aware movement parity for the launch/restore subset, proving replayed movement intent projects to the same TR-909/MC-202 render state and mixed output as the committed app path.

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

Runtime work must preserve enough metadata to say:

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
- session restore must reject snapshots whose action cursor points beyond the persisted action log
- session restore must reject commit records whose action id does not exist in the persisted action log
- session restore must reject commit records whose action id exists but is not `Committed`
- session restore must reject commit records whose `committed_at` is missing or differs from the referenced action
- session restore must reject duplicate commit records for the same action id
- session restore must reject zero or duplicate commit-record sequence numbers within the same commit boundary

### 9.2 Runtime state relation

Decision:

- `runtime_state` in the session file is the latest materialized state for fast load
- snapshots provide additional anchored restore points

Practical consequence:

`runtime_state` and the latest snapshot should not drift semantically.

### 9.3 Snapshot payload hydration boundary

The smallest safe implementation boundary for real snapshot hydration is a typed
payload that carries replay-relevant `RuntimeState` at the snapshot's
`action_cursor`.

This boundary must stay data-first:

- a snapshot payload is an accelerator, not a new replay truth
- the action log and structured commit records remain canonical
- hydration must not parse UI text, result summaries, log lines, or TUI labels
- hydration must not run Ghost, rerun source analysis, or recreate capture audio
- hydration must not apply a target suffix itself

Minimal restore flow:

1. load and validate the `SessionFile`
2. build the target replay plan and select the latest valid snapshot anchor
3. validate the selected snapshot payload
4. clone the session and replace only `runtime_state` with the payload runtime state
5. apply the existing target-suffix executor to that hydrated clone
6. rebuild derived app/runtime/TUI projection only after suffix application succeeds

Payload validation must reject:

- missing payloads for restore paths that require a snapshot anchor
- payload schema versions the runtime does not support
- payload `snapshot_id` or `action_cursor` mismatches
- snapshot cursors beyond the persisted action log
- runtime payloads that claim state after the snapshot cursor
- referenced capture or source graph ids that cannot be resolved from the same session

The caller owns anchor hydration. The existing target-suffix helper remains a
suffix executor for already-hydrated anchor state. If snapshot payload hydration
fails, the runtime may surface a read-only recovery/debug prompt, but it must not
silently fall back to a different anchor or repair the session without an
explicit recovery path.

Session load validation rejects present payloads whose identity or cursor does
not match the owning snapshot before app state is constructed. Missing payloads
remain load-valid because payloads are optional unless a selected restore path
requires a snapshot anchor.

The first implementation should copy the current `RuntimeState` shape into a
versioned snapshot payload and prove that latest-runtime restore and
snapshot-payload-plus-suffix restore converge for one replay-safe session. Broader
capture/resample artifact hydration remains out of scope for that first slice.

Current producer boundary:

- saving a session may materialize payloads only for existing explicit snapshots
  whose `action_cursor` equals the current action-log length
- save must not invent new snapshots or choose snapshot frequency policy
- older snapshots whose cursor predates the latest runtime state must keep their
  existing payload state instead of receiving the current `runtime_state`

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
- W-30 artifact hydration may only use an artifact when the session identity is explicit: `capture_id`, `storage_path`, `created_from_action`, direct `source_window` for source-backed captures, and lineage/depth fields for internally derived captures
- if that identity is missing, ambiguous, or points at an unavailable artifact, replay/recovery should reject with a clear unsupported or missing-artifact diagnostic rather than synthesize a replacement silently

Current implementation boundary:

- `plan_w30_artifact_replay_hydration` is the core contract seam for
  artifact-producing W-30 replay actions such as `w30.loop_freeze` and
  `promote.resample`
- it only plans hydration from explicit session identity; it does not read files,
  decode WAV data, mutate runtime state, or apply the replay suffix
- it locates the produced capture through `created_from_action`, rejects missing
  or ambiguous produced captures, requires the referenced source capture to exist
  in the same session, requires a non-empty `storage_path`, and requires
  source-window identity for source-backed captures or lineage/depth identity for
  resample-derived captures
- the replay executor can hydrate the first narrow artifact-backed suffixes,
  `w30.loop_freeze` and `promote.resample`, by using the contract to point W-30
  state at the persisted produced capture; other artifact-producing W-30 /
  capture / promote actions remain unsupported until they get their own explicit
  hydrator

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

1. snapshot representation should become more explicit than the current lightweight placeholder.
2. capture persistence should remain artifact-first, not recompute-first.
3. validation should include replay-from-snapshot and replay-from-zero-path checks.
4. Ghost accepted-action fixtures should continue proving that accepted Ghost actions persist as normal actions plus structured commit records.

---

## 13. Validation Impact

The replay model should drive future validation:

- save / load should preserve state identity
- replay from latest snapshot and full replay from origin should converge
- accepted Ghost actions must replay identically to user actions
- restore-from-zero fixtures should prove commit records, latest boundary context, and future action-id reservation survive fresh app startup
- changed provider availability should not change restore of existing sessions

---

## 14. Decision Summary

Short version:

- replay truth is frozen inputs plus committed actions plus snapshots
- committed musical order matters more than request time
- snapshots accelerate replay but do not replace the action log
- analysis and Ghost are not rerun as truth during normal restore
- captures are referenced artifacts, not recomputed guesses

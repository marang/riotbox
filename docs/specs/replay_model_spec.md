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
- source-timing grid trust together with the exact accepted BPM; replay of
  `source_timing.confirm_grid` restores both and replay of the matching revert
  clears both without rewriting Source Graph analysis

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

RIOTBOX-1599 / RBX-457 places optional legacy MC-202 plan resolution at the
actual missing-plan entry in the existing Core replay traversal. The resolver
observes immutable pre-entry Session/entry metadata and returns only a plan;
Core still owns execution, Undo snapshots, order, suffix/anchor selection,
matching timing invalidation and transactional installation. Persisted
`Some`, including degraded/stay-out plans, bypasses derivation. Default Core
interfaces remain no-backfill; there is no general replay hook system. App must
not overwrite a Core target/payload result by reconstructing the whole prefix.
An authoritative anchor `None`, empty suffix or reconfirmation without a new
MC-202 action cannot regenerate old material. Keep the existing App source-ID
trust check, unchanged source selector and separate unvalidated-constructor
compatibility contract in the [Session spec](./session_file_spec.md).

RBX-458 corrects the pre-existing MC-202 SetRole replay touch discrepancy:
SetRole assigns its clamped/default touch, matching the live writer; Generate
and Mutate retain their existing maximum-with-current-touch behavior. This
applies to ordinary Core replay as well as legacy resolution, so the next
missing plan receives the actual preceding performer state. Persisted plans
still bypass selection. No source-selection arithmetic or thresholds change.

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

RIOTBOX-1593 / RBX-454 declares that guarantee through the existing required
`ReplayPolicy::DeterministicCommittedOrderV2`, selected by fresh Session/ActionLog
defaults. The `commit_records` array is append chronology, not a set to sort by
musical coordinates, ActionId or timestamps. Every committed/undone action needs
its one record, in agreement with the committed action-vector subsequence;
structural and failed-result records participate before executable filtering.
Sequence numbers remain local to the complete boundary, increase on revisits,
and need not be contiguous. Wall-clock rollback cannot reverse chronology;
action/record timestamp equality and presence remain required.

Legacy `DeterministicPreferred`/`SnapshotOnly` values retain their historical
interpretation and roundtrip identity. Loading/saving never opts them into V2.
Backward navigation requiring append chronology is explicitly unsupported in a
legacy history: refuse the ordinary gesture before enqueue and fail replay
closed with an upgrade-required error, not a new musical ordering guess.
An explicit conservative Core upgrade changes only the policy after validating
complete history, legacy relations, all-record canonical order, append/action
agreement and sequences. Ambiguity refuses without mutation or repair. Older
enum decoders reject the new value; no ignored optional field can downgrade it.
The frozen generated loaded-recorder V1 producer pins its historical policy.

RBX-455 makes that unchanged conservative Core operation reachable through the
exclusive `--replay-chronology-upgrade-copy` metadata CLI. It creates only an
explicit new sibling file; never automatically migrates load/save, adopts an
ambiguous history, rewrites records/cursors or hydrates referenced material.
The [Session copy contract](./session_file_spec.md#9-action-log) owns exact-byte
fidelity, no-clobber publication and operator feedback. Already-V2 input is
validated and reported as a copy, not a new chronology upgrade.

RIOTBOX-1597 / RBX-456 requires live mutation to respect that declared policy.
Non-append commits stage prospective metadata through the existing Queue and
Core planner before original mutation; no second validator or silent upgrade.
Legacy Undo stages its exact normalization, restored typed snapshot, target and
marker/record before changing original state or ID capacity. Ordinary failed
drains preserve legitimate earlier pending enqueue. Immediate toggle stages its
new draft too; timing entrances validate their prospective boundary/timestamp
before changing clock or pending acknowledgment. Preserve capacity-error
precedence, V2 domain/rollback behavior and existing empty-drain semantics.
Typed App ReplayAdmission reports policy/Core failure visibly and nonfatally for
interactive use; recorder errors retain unconditional stop/abort. SnapshotOnly
does not gain a migration promise. This guards live declared history, not all
Session/source contracts or external audio delivery as a single transaction.

V2 structural Undo uses per-domain candidate stacks and the existing Core domain
predicates: a non-undoable mutation invalidates only overlapping older candidates.
An eligible typed action remains undoable in reverse committed order; timestamp
monotonicity is not its order proof. Every Undone target needs its valid later
marker. These stacks are transient validation, never a second persisted history.

Current implementation:

- `ActionLog.commit_records` stores one structured commit record per committed action.
- Each commit record is keyed by action id and stores the commit boundary plus commit sequence within that boundary.
- Replay and budget logic should consume these structured commit records instead of parsing result summaries or relying only on incidental action vector order.
- A replay-plan builder may consume the existing action log and commit records to produce deterministic committed-order entries, but it must not become a second action, persistence, or repair system.
- A commit record whose action is `undone` remains valid historical commit
  evidence and is omitted from the executable replay plan only when that
  command declares implemented typed undo semantics. An `undone` status on a
  command without that typed contract is invalid instead of silently skipping
  state whose inverse is unknown. Commit records that reference any other
  non-committed status remain invalid.
- A committed `undo.last` marker has its own Immediate commit record and typed
  target action id. It is structural replay evidence rather than an executable
  audio side effect. Replay accepts it only when it is successful, Immediate,
  non-undoable, uniquely consumes the latest prior active successful action
  with typed restoration, and that target is successful, `Undoable`, and owns
  its historical commit record. The plan then omits the marker itself together
  with the successfully undone target.
- Commit sequence numbers are allocated against the existing persisted maximum
  for the complete boundary. A second queue drain, structural Undo, or direct
  side-effect commit at the same paused transport point must continue the same
  sequence rather than restart at one.
- Core checks capacity for the whole intended batch before consuming queue
  state. An exhausted boundary returns typed `CommitSequenceExhausted`; no
  rollover, renumber, saturation, fabricated record or panic is permitted.
  Existing valid MAX-sequence history and other boundaries remain usable.
  Core assigns final absolute sequences during the admitted drain; App retains
  `record one action → its pipeline` order rather than preappending a batch.
  Undo and side-effect writers admit their own one-record capacity before
  snapshots, identity allocation or publication. A transport toggle instead
  admits all existing ordinary Immediate-ready actions plus its prospective
  action, because its existing behavior drains that whole batch.
- Replaying a still-committed action with typed undo semantics rebuilds that
  action's pre-state undo snapshot before applying its side effect, so restored
  runtime state retains the same live undo capability as the original commit.
- Live Undo selects only an accepted typed target with exactly one historical
  commit record and a persisted pre-state snapshot. Newer committed mutations
  form barriers only for overlapping typed restore domains; in particular, a
  source-grid revert prevents an older MC-202 snapshot from restoring a
  source-derived plan that replay correctly leaves cleared.
- Snapshot and historical-target safety is cursor-relative. A snapshot from
  before the matching typed undo marker remains unsafe because it contains the
  now-undone state; a snapshot at or after that committed marker is a valid
  anchor. Later tail actions replay normally from that post-undo anchor.
- RIOTBOX-1605 / RBX-466 makes preset activation an overlapping writer for
  Source Monitor mode and MC-202 lane snapshots (the latter includes touch).
  It is not a TR-909 Fill-window writer. The shared Core predicate governs
  future live Undo for all policies and existing V2 marker validation. Already
  persisted V2 markers crossing such a preset are invalid relations and fail
  closed; do not rewrite actions, infer a new policy or adopt their divergent
  materialized state. This corrects the existing V2 conflict contract without
  changing its chronology or adding a policy version. Historical Legacy Core
  stack interpretation stays unchanged; explicit upgrade must refuse an
  unprovable V2 relation without mutation. Independent-domain live Undo under
  Legacy remains subject to its existing prospective global-stack admission.
- Cursor safety is a transient Core projection, never persisted replay truth
  (RBX-426). After the existing history-validation gate, a forward sweep records
  the first unresolved `Undone` action in action-vector order for each cursor.
  Only a prior typed, committed, accepted Undo marker with a commit record
  resolves its target; legacy untyped markers remain untrusted. Candidate
  snapshots share that index rather than repeating action/marker/record scans.
  Plain histories need no prefix allocation or record index, and log-end replay
  without eligible nonzero anchors needs no safety index at all. Complete cursor
  validation, first-error identity, latest-on-tie selection and the log-end
  target exception stay unchanged. `just replay-cursor-benchmark` measures the
  public planners using generated histories only; timing is not a CI threshold.
- A request may reach its quantized commit boundary yet produce
  `ActionResult.accepted == false` when the product-side precondition cannot be
  proven. Its action and commit record remain historical evidence, but the
  action is omitted from executable replay because the live side effect is an
  atomic no-op. Replay must not reconstruct a rejected partial mutation.
- Snapshot-vs-origin replay-plan comparisons may select a suffix from the origin plan by using the existing snapshot `action_cursor`.
- Snapshot anchor selection should pick the latest valid snapshot at or before the target action cursor and reject out-of-range cursors instead of silently falling back.
- Target replay planning may combine the origin plan, selected snapshot anchor, and target-limited suffix, but execution and runtime hydration remain separate responsibilities.
- Target replay dry-run summaries may expose selected anchor metadata, target cursor, and suffix action scope for QA and future UI/debug seams, but they must not execute actions or mutate runtime state.
- Latest-snapshot replay convergence summaries may compare the full committed origin against the latest snapshot-to-end suffix for QA, including whether no snapshot forces full replay.
- Session restore rebuilds the app-runtime `last_commit_boundary` from the latest structured commit record so fresh app state does not lose the most recent musical boundary context.
- MC-202 source phrase plans stored on `ActionCommitRecord` are replay truth for
  MC-202 phrase actions. The replay executor restores that persisted plan
  directly from the commit record, and clears stale MC-202 source phrase state
  when an older commit record has no trusted plan. SourceGraph-based
  reconstruction may still be used by higher app restore helpers, but it is not
  allowed to invent fallback audio or preserve stale source phrase state when
  the commit record does not carry a plan.
- Replay preserves a historical MC-202 plan whose requested role and candidate
  family disagree, but projection must classify it as non-source-derived and
  remain silent/degraded. Replay must not reinterpret the requested role,
  replace the family, or resurrect the mismatch as audible output.
- The shared dense-break `LivePerformancePolicy` is derived projection, not
  replay truth. After the Source Graph, timing confirmation, and committed
  source phrase plan are restored, app projection must derive the same lane
  hierarchy, MC-202 intent, explicit bass ownership, and bounded live mixer
  floors. Replay must not restore a stale cached policy or infer bass ownership
  from the requested MC-202 role alone.

### 7.3 Minimal replay executor boundary

The replay executor is intentionally narrow. It applies only deterministic actions from committed replay-plan entries whose state mutation is explicit enough to replay without consulting UI summaries or log text.

Current supported structural commands:

- `transport.play`
- `transport.pause`
- `transport.stop`
- `transport.seek`
- `preset.activate`
- `lock.object`
- `unlock.object`
- `ghost.set_mode`

Live Jam/TUI Play/Pause input reaches this executor through committed
`transport.play` / `transport.pause` actions, not through app-local transport
mutation. Both commands commit immediately at the current clock boundary, and
replay must restore the same Session transport state without consulting the
key label, status message, or observer text.

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
- `promote.capture_to_pad` for existing captures and explicit W-30 pad targets
- `promote.capture_to_scene` for existing captures and explicit scene targets
- `capture.now` for explicitly persisted source-window-backed loop captures
- `capture.loop` for explicitly persisted source-window-backed loop captures
- `capture.bar_group` for explicitly persisted source-window-backed pad captures
- `w30.live_recall`
- `w30.trigger_pad`
- `w30.audition_raw_capture`
- `w30.audition_promoted`
- `w30.swap_bank`
- `w30.browse_slice_pool`
- `w30.step_focus`
- `w30.apply_damage_profile`
- `w30.hook_turnaround`
- `w30.pitch_dive`
- `w30.filter_slam`
- `w30.capture_to_pad` for explicitly persisted source-window-backed pad captures
- `w30.loop_freeze` for explicitly persisted artifact-backed captures
- `promote.resample` for explicitly persisted artifact-backed resample captures

Rules:

- The executor consumes replay-plan entries, not UI summaries or parsed log text.
- Unsupported commands fail explicitly instead of being silently ignored.
- Invalid params fail explicitly instead of guessing defaults.
- Whole-plan application is all-or-nothing: if any entry fails, the session passed to the executor is not mutated.
- Single-entry application may mutate the passed session and should be used only when the caller already accepts that boundary.
- This executor does not perform audio rendering, capture artifact creation, non-allowlisted W-30/TR-909 side effects, Ghost reasoning, source analysis, or snapshot hydration.
- MC-202 replay currently covers the deterministic phrase-lane state needed by
  downstream projection: role, phrase reference, phrase variant, and MC-202
  touch. App restore/projection also rehydrates committed MC-202 source phrase
  plans from `ActionCommitRecord.mc202_source_phrase_plan` when present, so
  source-derived MC-202 output can survive action-log and snapshot replay
  without rerunning musical selection. Older logs without that persisted plan
  may reconstruct through the same MC-202 source-phrase side-effect path only
  when trusted Source Graph / source-timing evidence is available; otherwise the
  lane must remain degraded/silent rather than synthesize fallback audio.
- TR-909 replay currently covers the deterministic support-lane state needed by downstream projection: slam, fill, reinforce, takeover, scene-lock, release, pattern references, reinforcement mode, and takeover profile.
- Scene replay currently covers the deterministic active-scene / restore-scene state carried by committed scene launch and restore actions. The minimal executor updates transport current scene, scene active scene, restore pointer, and swaps the paired active/restore audio-projection snapshots while deliberately staying Source Graph-free. A separate graph-aware replay boundary hydrates `last_movement` from the pre-action scene state, committed boundary, committed scene action, and frozen Source Graph context; launches install that derived projection, while restores recover the previously paired projection instead of projecting the inverse transition as new musical state.
- W-30 replay currently covers a deterministic cue subset whose committed actions already carry explicit target state: live recall, trigger, audition, bank swap, slice-pool browse, focus step, damage profile, hook turnaround, pitch dive, and filter slam. It updates preview mode, focused bank/pad, last capture, W-30 grit where applicable, and the typed articulation profile/capture/committed start beat. Pitch Dive and Filter Slam replay restore the persisted `pitch_dive_v1` or `filter_slam_v1` profile; replay does not rerun source selection or retune either frozen curve. Live recall, trigger, raw/promoted audition, and damage-profile replay clear a preceding timed articulation exactly as the live commit path does, so ordinary W-30 re-entry cannot remain trapped behind a completed destructive exit. Damage audio projection is separately derived from the latest committed `w30.apply_damage_profile` mutation for the exact active capture: positive intensity applies the persistent profile and zero intensity explicitly bypasses it. Replay must not infer damage from a different capture or from UI state.
- `capture.now` and `capture.loop` replay hydrate an already persisted source-window-backed loop capture produced by the committed action and point W-30 `last_capture` / live-recall preview state at it. They do not regenerate capture audio from the Source Graph.
- `capture.bar_group` replay hydrates an already persisted source-window-backed pad capture produced by the committed action and points W-30 `last_capture` / live-recall preview state at it. It does not regenerate capture audio from the Source Graph or assign it to a pad.
- `w30.capture_to_pad` replay hydrates an already persisted source-window-backed pad capture produced by the committed action and points W-30 focus/last-capture/live-recall state at it. It does not regenerate capture audio from the Source Graph.
- `promote.capture_to_pad` replay assigns an existing `CaptureRef` to an explicit W-30 pad target and updates W-30 lane focus. It does not create or regenerate capture audio.
- `promote.capture_to_scene` replay assigns an existing `CaptureRef` to an explicit scene target and updates the latest W-30 capture pointer. It does not create or regenerate capture audio, launch a scene, or change scene audio rendering.
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
- session restore must reject duplicate action ids in the persisted action log
- session restore must reject commit records whose action id exists but is not
  `Committed`, except for historical `Undone` targets whose command has typed
  restoration semantics
- session restore must reject commit records whose `committed_at` is missing or differs from the referenced action
- session restore must reject duplicate commit records for the same action id
- session restore must reject zero or duplicate commit-record sequence numbers within the same commit boundary
- session restore/replay validation must reject a committed undo marker whose
  typed target is missing, not the latest still-active typed target, not earlier
  in the log, not `Undone`, unsuccessful, non-undoable, already consumed, or
  lacks typed restoration semantics; marker quantization, boundary, result,
  policy, and structured commit evidence must also satisfy the typed contract
- legacy untyped markers never establish a safe post-undo snapshot boundary;
  restore degrades committed typed actions without their pre-state snapshot to
  `NotUndoable`, and rejects legacy undone Source Monitor or TR-909 Fill state
  whose historical rollback cannot be proven

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

RIOTBOX-1607 / RBX-470 makes the selected V1 payload's explicit provenance
checkable before installation. Prefix membership means an action's stored vector
index is strictly below the **anchor** cursor, not the target cursor, ActionId,
timestamp, or transport position. History retains its declared policy's record
admission: V2 requires complete validated records; legacy partial record coverage
is not implicitly upgraded. The referenced action must have a committed timestamp,
a committed (or supported typed-Undone) status, no rejected result,
and the correct owner command/domain. Confirmed timing, capture length, all three
Scene movement slots and all three Undo snapshot families are covered. Timing,
length and movement identities must match their owner's persisted parameters;
Undo snapshots hold pre-action values, not the owner's result.

Resolve reached CaptureIds and SourceIds uniquely from this Session. Follow
typed capture lineage without cycles; a present capture producer must be a
prefix capture-producing action. An absent producer remains legacy/import-valid.
Resolve explicit SectionIds in already supplied Session-bound or embedded graph
metadata; absent required metadata is a typed refusal, never implicit file I/O.
The declared storage mode owns graph authority: an embedded graph cannot be
overridden by a supplied same-SourceId graph; a conflicting supplied embedded
context is refused before traversal or resolution. All supplied graphs must have
passed existing Session graph-identity admission before either public hydration
API is called; Core does not duplicate that hash validator or load external files.
Explicit Scene source bindings use the same resolution and reject duplicates.
Legacy absent MC-202 sections and absent/empty binding lists retain their distinct
meanings. Do not invent Bank/Pad or custom Scene registries: initial focus and
bare Scenes require no owning action.

The bounded check does not prove arbitrary runtime equality to a replayed prefix.
Opaque phrase/pattern/provenance strings, scalar sound decisions, timestamps and
beat positions are not action identities. It does not replay skipped history,
replan phrases, load graph/audio files, validate unrelated historical payloads,
migrate metadata or change save/load header admission. A typed refusal precedes
any suffix resolver and preserves the input Session and App/queue projections.

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
- export receipt replay validation consumes Session/Core receipt truth only:
  legacy receipts may omit optional artifact lineage, but present
  `source_graph_ref` and `timing_grid_ref` identities must be non-blank so
  corrupted export evidence is rejected before it is trusted
- stem-package receipt replay/recovery validation consumes the claimed stem
  roles from the `stem_package_artifact_set_evidence` QA gate and validates
  explicit `artifact_set[]` identities for every claimed stem WAV plus
  manifest/proof JSON. Missing identity and missing local file are distinct
  diagnostics; neither path may regenerate, rewrite, or infer package artifacts
  during restore.
- Explicit-duration live-master DAW V3 contracts (RBX-424) are validated
  from Core/Session metadata: committed export action, matching receipt
  version/duration, and exactly one pinned ready V3 recording source of the
  same duration, audio identity and recorded tempo. New eight-/sixteen-bar
  receipt placement and tempo spans must match 32/64 beats. Missing, ambiguous
  or inconsistent identity fails closed. Historical DAW V1/V2 restore policy
  remains unchanged; no replay or metadata-only restore opens source audio or
  regenerates the external archive.
- arrangement / DAW placement receipt validation consumes
  `arrangement_placement_refs[]` from Session receipts only. Missing placement
  evidence, blank scene refs, invalid bar ranges, and invalid beat ranges are
  distinct from missing local artifact files, and replay/recovery must not
  synthesize arrangement placement from observer state or current app runtime.
- DAW tempo-map receipt validation consumes `daw_tempo_map_ref` from Session
  receipts only. Missing tempo-map evidence, blank source refs, invalid tempo,
  and invalid beat ranges are distinct from missing placement evidence and
  missing local artifact files. Replay/recovery must not synthesize tempo-map
  evidence from observer state or current app runtime.
- DAW JSON package receipt validation consumes `artifact_set[]` entries and the
  `daw_session_json_package_integrity` QA gate from Session receipts only.
  Manifest, tempo-map, proof JSON paths/hashes, and package integrity status are
  receipt evidence, not inferred from observer snapshots, current destination
  folders, or app-local report state during replay.

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
- Capture audio identity travels with the existing Session `CaptureRef` through
  replay; planning metadata does not attest WAV bytes. App hydration must verify
  the persisted hash before supplying cached audio. Legacy adoption is an
  explicit offline Session migration with typed provenance, not a replay action
  or a reason to regenerate missing audio (RBX-375).
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
  state at the persisted produced capture
- `promote.capture_to_pad` and `promote.capture_to_scene` can replay existing
  capture assignment metadata when the target is explicit; they do not create or
  regenerate capture audio
- `capture.now` and `capture.loop` can replay existing produced `CaptureRef`
  values when they have `created_from_action`, non-empty storage identity,
  source-window identity, and `CaptureType::Loop`; they do not recreate capture
  audio
- `capture.bar_group` can replay an existing produced `CaptureRef` when it has
  `created_from_action`, non-empty storage identity, source-window identity, and
  `CaptureType::Pad`; it does not recreate the capture or assign it to a pad
- `w30.capture_to_pad` can replay an existing produced `CaptureRef` when it has
  `created_from_action`, non-empty storage identity, source-window identity, and
  an explicit W-30 pad target; it does not recreate the capture
- other artifact-producing W-30 / capture / promote actions remain unsupported
  until they get their own explicit hydrator

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

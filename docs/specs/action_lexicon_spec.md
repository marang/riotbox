# Riotbox Action Lexicon Spec

Version: 0.1  
Status: Draft  
Audience: realtime, TUI, Ghost, session, QA

---

## 1. Purpose

This document defines the action vocabulary used by Riotbox.

It exists so that:

- user actions
- Ghost actions
- undo / redo
- replay
- TUI state
- quantized scheduling

all refer to the same action model.

---

## 2. Core Rule

No subsystem invents its own action semantics.

Every action that changes session state must eventually map to:

- one command name
- one parameter schema
- one commit policy
- one undo model
- one actor type

---

## 3. Action Object

Canonical action shape:

```text
Action {
  id
  actor
  command
  params
  target_scope
  requested_at
  quantization
  status
  committed_at
  result
  undo_payload
  explanation
}
```

Commit-boundary metadata is not inferred from result text. When an action
commits, the session `ActionLog` records a structured commit record keyed by
action id with the musical boundary, commit sequence within that boundary, and
commit timestamp.

### 3.1 Actor types

- `user`
- `ghost`
- `system`

### 3.2 Status states

- `requested`
- `queued`
- `pending_commit`
- `committed`
- `rejected`
- `undone`
- `failed`

---

## 4. Quantization Model

Allowed quantization targets:

- `immediate`
- `next_beat`
- `next_half_bar`
- `next_bar`
- `next_phrase`
- `next_scene`

Rules:

- hard musical changes default to `next_bar` or stricter
- destructive rebuild actions should default to `next_phrase`
- `immediate` is reserved for safe state changes that do not destabilize playback

---

## 5. Target Scope

Every action must declare its scope:

- `global`
- `scene`
- `lane_mc202`
- `lane_w30`
- `lane_tr909`
- `mixer`
- `ghost`
- `session`

Optional target references:

- scene ID
- bank ID
- pad ID
- loop ID
- lane object ID

---

## 6. Action Families

### 6.1 Transport

- `transport.play`
- `transport.pause`
- `transport.stop`
- `transport.seek`

`transport.seek` may be used for musician-facing Source Map bar and phrase
navigation. The shell may expose typed intents such as previous / next bar and
previous / next phrase, but the committed product truth remains a structured
`transport.seek` action with `ActionParams::Transport { position_beats }`.
Seek preserves the current play / pause state, clamps at source bounds by
default, and must be replayable.

### 6.1.1 Source monitor and timing trust

Source monitor and timing-trust actions:

- `source_monitor.set_mode`
- `source_timing.confirm_grid`
- `source_timing.revert_grid`

`source_monitor.set_mode` changes the persisted listening mode between `source`,
`blend`, and `riotbox`.

`source_timing.confirm_grid` records that the user accepted the currently
selected timing hypothesis after source audition. It uses `session` scope,
commits immediately, and carries structured `source_id` plus optional
`hypothesis_id` parameters so replay / restore can distinguish analyzed
confidence from user-accepted musical trust.

`source_timing.revert_grid` removes a matching user confirmation through the
same session-scope action path without deleting the original Source Timing
evidence. It carries the confirmed `source_id` plus optional `hypothesis_id` so
replay can distinguish the trust-state removal from analysis changes.

### 6.2 Mutation

- `mutate.scene`
- `mutate.lane`
- `mutate.loop`
- `mutate.pattern`
- `mutate.hook`

### 6.3 Capture and promotion

- `capture.now`
- `capture.loop`
- `capture.bar_group`
- `capture.set_length`
- `promote.capture_to_pad`
- `promote.capture_to_scene`
- `promote.resample`

Capture actions that target source material should carry structured musical
length intent. `capture.set_length` is a session-scope immediate action with
typed `CaptureLengthIntent` values: `one_beat`, `one_bar`, `four_bars`, and
`phrase`. It changes the length used by subsequent source-window capture without
creating a hidden TUI-local selector.

`capture.bar_group` may omit explicit `bars`; when it does, commit uses
`runtime_state.capture.length_intent`. `one_beat`, `one_bar`, and `four_bars`
derive duration from Source Timing meter / beat evidence. `phrase` captures to
the end of the matching phrase span when phrase evidence is usable and otherwise
falls back visibly to `four_bars`.

### 6.4 Scene control

- `scene.launch`
- `scene.restore`
- `scene.regenerate`
- `scene.reinterpret`

### 6.5 Device control

- `mc202.generate_follower`
- `mc202.generate_answer`
- `mc202.generate_pressure`
- `mc202.generate_instigator`
- `mc202.mutate_phrase`
- `mc202.set_role`
- `w30.capture_to_pad`
- `w30.live_recall`
- `w30.trigger_pad`
- `w30.audition_raw_capture`
- `w30.audition_promoted`
- `w30.swap_bank`
- `w30.browse_slice_pool`
- `w30.step_focus`
- `w30.apply_damage_profile`
- `w30.loop_freeze`
- `tr909.fill_next`
- `tr909.set_slam`
- `tr909.reinforce_break`
- `tr909.takeover`
- `tr909.scene_lock`
- `tr909.release`

`w30.browse_slice_pool` normally cycles through captures assigned to the
current W-30 pad. A Feral-ready Source Graph may bias that choice toward a
non-current capture whose `source_origin_refs` match a `CaptureCandidate` asset
or supported `HookFragment`. This remains the same queued
`w30.browse_slice_pool` action; Feral policy changes target selection, not the
Action Lexicon or commit semantics.

TR-909 source-support render projection may consume the same Feral Source Graph
evidence to choose a stronger bounded support profile, for example lifting
neutral `steady_pulse` support into `break_lift`. This changes render policy
only; it does not add a new drum action, arranger, or commit path.

MC-202 follower / leader render projection may consume the same Feral Source
Graph evidence to choose `answer_space` hook response and a sparser note budget
when supported hook or capture material suggests room should be left for a
break-rebuild move. This changes render policy only; it does not add a new
MC-202 action or phrase-generation path.

MC-202 role and phrase-intent labels currently remain stable serialized labels
for existing actions, sessions, TUI, observer output, and QA artifacts. New
behavior must not add a second MC-202 action path to escape those labels. The
typed migration plan is documented in
`docs/reviews/mc202_typed_contract_migration_plan_2026-05-10.md`; implementation
should move behavior branching to typed conversion helpers while preserving the
existing action commands and compatibility labels.

### 6.6 Structural / safety

- `lock.object`
- `unlock.object`
- `snapshot.save`
- `snapshot.load`
- `undo.last`
- `redo.last`
- `restore.source`

### 6.7 Ghost

- `ghost.set_mode`
- `ghost.accept_suggestion`
- `ghost.reject_suggestion`
- `ghost.execute_tool`

---

## 7. Required Fields Per Action

Every action must define:

- command name
- actor
- target scope
- quantization
- parameter schema
- validation rules
- expected result shape
- undo policy

Optional:

- explanation
- confidence
- source references

---

## 8. Initial MVP Action Set

The MVP should treat these as first-class:

- `transport.play`
- `transport.pause`
- `mutate.scene`
- `mutate.lane`
- `capture.now`
- `promote.capture_to_pad`
- `scene.launch`
- `scene.restore`
- `lock.object`
- `unlock.object`
- `snapshot.save`
- `snapshot.load`
- `undo.last`
- `ghost.set_mode`
- `ghost.accept_suggestion`
- `ghost.reject_suggestion`

Everything else may exist later, but this set should be enough to support the MVP product spine.

---

## 9. Undo Rules

### 9.1 Undoable by default

All state-changing committed actions should be undoable unless explicitly marked otherwise.

### 9.2 Undo payload requirements

Undo payload must include enough information to:

- restore previous state
- restore target references
- restore scene or bank assignment
- preserve replay determinism

### 9.3 Not undoable

If an action is not undoable, that must be explicit in its schema and disallowed for MVP-critical live flows.

---

## 10. Replay Rules

Replay-critical actions must:

- use stable command names
- use deterministic parameter serialization
- log quantization and commit time
- preserve object references or resolvable IDs
- avoid hidden side effects

Ghost actions are replayable only if their executed action and explanation are both logged.

---

## 11. TUI Requirements

The TUI must be able to render:

- requested action
- pending commit action
- committed action
- rejected or failed action
- actor identity
- undo availability

This is why action semantics cannot remain implicit.

---

## 12. Ghost Requirements

Ghost suggestions and executions must refer only to action names defined here.

Ghost-specific constraints:

- no hidden private commands
- no sidecar-only magical mutations
- explanation text must point to the actual action executed
- Ghost action budgets must apply at action-family level

---

## 13. Open Questions

- exact parameter schemas for scene vs lane mutation
- whether `capture.now` and `capture.loop` should stay separate
- whether `scene.regenerate` should be distinct from `mutate.scene`
- how object locking is represented in action params vs session state

---

## 14. Next Step

After this draft:

1. define the canonical action JSON / serde schema
2. enumerate params for the MVP action set
3. connect quantization behavior to the Audio Core Spec
4. connect logging requirements to the Session File Spec

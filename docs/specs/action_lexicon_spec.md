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
- `promote.capture_to_pad`
- `promote.capture_to_scene`
- `promote.resample`

### 6.4 Scene control

- `scene.launch`
- `scene.restore`
- `scene.regenerate`
- `scene.reinterpret`

### 6.5 Device control

- `mc202.generate_follower`
- `mc202.generate_answer`
- `mc202.set_role`
- `w30.capture_to_pad`
- `w30.swap_bank`
- `w30.apply_damage_profile`
- `tr909.fill_next`
- `tr909.set_slam`
- `tr909.reinforce_break`
- `tr909.takeover`
- `tr909.scene_lock`
- `tr909.release`

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

# Riotbox Ghost API Spec

Version: 0.1  
Status: Draft  
Audience: Ghost, realtime, TUI, session, QA

---

## 1. Purpose

This document defines the Ghost integration contract for Riotbox.

It exists so that:

- Ghost acts through the same action model as the user
- Ghost remains bounded, inspectable, and replay-safe
- realtime and session systems know exactly what Ghost may read and write
- future assistive behavior does not create a shadow architecture

---

## 2. Core Rule

Ghost does not invent hidden behavior.

Ghost may only:

- read approved state surfaces
- propose approved actions
- execute approved tools that resolve to approved actions

Ghost may not:

- bypass the Action Lexicon
- mutate session state outside logged actions
- create parallel arrangement or mix logic that the core does not understand

---

## 3. Scope

The Ghost API defines:

- Ghost modes
- readable context surfaces
- executable tool surfaces
- proposal and execution flow
- budgets, locks, and safety limits
- logging and explainability requirements

It does **not** define:

- underlying model implementation
- external cloud behavior
- prompt-writing strategy as a product feature

---

## 4. Ghost Modes

Initial modes:

- `off`
- `watch`
- `assist`
- `perform`

MVP support:

- `watch`
- `assist`

Rules:

- `perform` must remain disabled until replay safety and action safety are proven
- mode must be visible in the TUI at all times

---

## 5. Read Surfaces

Ghost may read only explicit view models or structured state surfaces.

Minimum readable surfaces:

- transport state
- current scene and section context
- macro state
- lane summaries
- pending actions
- recent committed actions
- Source Graph summary and selected candidate surfaces
- locks and budgets
- audio health warnings

Ghost should prefer summaries over raw dumps.

If a new state surface is exposed, it must be named and justified.

---

## 6. Write Surfaces

Ghost does not write arbitrary state.

Ghost may write only through:

- proposal objects
- accepted action requests
- explicitly allowed tool outputs that resolve to action requests

All durable Ghost effects must appear in:

- the action log
- the session model where relevant
- the TUI log view

---

## 7. Tool Model

Ghost tools are bounded operations that return structured output.

Canonical shape:

```text
GhostTool {
  tool_name
  description
  allowed_modes
  input_schema
  output_schema
  side_effect_policy
  action_mapping
}
```

Rules:

- every tool must declare whether it is suggestion-only or execution-capable
- every execution-capable tool must map to one or more Action Lexicon commands
- tools must not hide secondary side effects

---

## 8. Initial MVP Tool Families

The canonical registry id is the durable contract. Rust proposal JSON stores the
short `tool_name` enum in `snake_case`; code must be able to map that enum back
to the canonical registry id.

### 8.1 Observation tools

- `ghost.inspect_jam_state`
- `ghost.inspect_source_summary`
- `ghost.inspect_recent_actions`
- `ghost.inspect_health`

### 8.2 Suggestion tools

- `ghost.suggest_scene_mutation`
- `ghost.suggest_capture`
- `ghost.suggest_macro_shift`
- `ghost.suggest_restore`

### 8.3 Execution tools

- `ghost.request_action`
- `ghost.accept_suggested_action`
- `ghost.reject_suggested_action`

MVP rule:

- direct multi-step autonomous tools are out of scope

### 8.4 MVP Registry

| Registry id | Rust proposal `tool_name` | Family | Allowed modes | Side-effect policy | Action mapping |
| --- | --- | --- | --- | --- | --- |
| `ghost.inspect_jam_state` | `inspect_jam_state` | Observation | `watch`, `assist` | Read-only | None |
| `ghost.inspect_source_summary` | `inspect_source_summary` | Observation | `watch`, `assist` | Read-only | None |
| `ghost.inspect_recent_actions` | `inspect_recent_actions` | Observation | `watch`, `assist` | Read-only | None |
| `ghost.inspect_health` | `inspect_health` | Observation | `watch`, `assist` | Read-only | None |
| `ghost.suggest_capture` | `suggest_capture` | Suggestion | `watch`, `assist` | Emits proposal only; accepted in `assist` through the normal action queue | `capture.now` |
| `ghost.suggest_scene_mutation` | `suggest_scene_mutation` | Suggestion | `watch`, `assist` | Emits proposal only; accepted in `assist` through the normal action queue | `scene.mutate` |
| `ghost.suggest_macro_shift` | `suggest_macro_shift` | Suggestion | `watch`, `assist` | Emits proposal only; accepted in `assist` through the normal action queue | Lane or mixer action from proposal payload |
| `ghost.suggest_restore` | `suggest_restore` | Suggestion | `watch`, `assist` | Emits proposal only; accepted in `assist` through the normal action queue | `scene.restore` |
| `ghost.request_action` | n/a | Execution request | `assist` | Creates a normal `ActionDraft` only after explicit approval and safety checks | Any allowed Action Lexicon command carried by the approved request |
| `ghost.accept_suggested_action` | n/a | Decision | `assist` | Marks the proposal accepted and queues the mapped action; no direct musical mutation | Action mapped by the accepted proposal |
| `ghost.reject_suggested_action` | n/a | Decision | `watch`, `assist` | Marks the proposal rejected; no musical mutation | None |

Registry rules:

- proposal `tool_name` values must remain stable because they persist into
  sessions and review artifacts
- execution registry ids do not currently appear as `GhostWatchTool` variants
  because they are host/app control operations around proposals, not proposal
  producers
- adding a new Ghost tool requires updating this registry, the Rust mapping, and
  at least one serialization or UI/control-path regression

---

## 9. Proposal Object

Ghost suggestions must use one explicit proposal shape.

```text
GhostWatchSuggestion {
  proposal_id
  mode
  tool_name
  summary
  rationale
  suggested_action
  confidence
  safety
  blockers
  created_at
}
```

Rules:

- `watch` suggestions are read-only comments; they must not queue or execute actions
- `summary` must be short enough for the Jam screen
- `rationale` must be explainable in plain language
- `suggested_action`, when present, must name an Action Lexicon command, target, quantization, and plain-language intent, but it is still only an intent until accepted by a later Assist flow
- `safety` must distinguish safe suggestions, suggestions that need Assist acceptance, and blocked suggestions
- blockers such as locks or low-confidence conditions must be explicit

---

## 10. Execution Flow

Canonical MVP flow:

1. Ghost reads allowed state.
2. Ghost emits a proposal.
3. User sees summary and rationale.
4. User accepts or rejects.
5. Accepted suggestions with a safe `suggested_action` become normal `ActionDraft`s with `ActorType::Ghost`.
6. Accepted actions enter the normal queue.
7. Commit occurs on the normal quantized boundary.
8. Result appears in log and replay state.

Rules:

- no silent auto-commit in `watch`
- accepting a suggestion is only effective in `assist`; `watch` remains read-only
- rejecting a suggestion only marks that proposal as rejected
- `assist` may prepare actions, but acceptance remains explicit for MVP-critical changes
- accepted suggestions must use the existing Action Queue; no Ghost-specific commit path may mutate musical state directly

---

## 11. Budgets and Locks

Ghost must respect explicit safety controls.

### 11.1 Budgets

Examples:

- max actions per phrase
- max destructive actions per scene
- max capture promotions per window
- max simultaneous pending Ghost actions

### 11.2 Locks

Ghost must honor:

- locked scenes
- locked pads or banks
- locked lanes
- locked mixer regions

If blocked, Ghost must surface the reason instead of retrying invisibly.

---

## 12. Quantization and Safety

Ghost actions follow the same quantization rules as user actions.

Rules:

- destructive mutations default to conservative boundaries
- immediate Ghost execution is reserved for safe non-musical state changes
- Ghost may not exceed lane-specific or scene-specific safety rules

---

## 13. Explainability

Explainability is mandatory.

Every Ghost proposal or executed action should expose:

- what it wants to do
- why it wants to do it
- what it is targeting
- when it will commit
- why it was blocked or rejected if not executed

Opaque “AI felt like it” behavior is not acceptable.

---

## 14. Session Requirements

Ghost-relevant session state should include:

- current mode
- active budgets
- lock-aware status where relevant
- suggestion history when needed for continuity
- each suggestion's explicit decision state: suggested, accepted, or rejected

Accepted suggestion state is only a durable intent marker until the accepted action enters the normal Action Lexicon queue. Rejected suggestion state is a durable user decision and must not mutate musical runtime state.

Do not persist raw internal reasoning traces unless they are deliberately promoted into user-facing logs.

---

## 15. MVP Requirements

Ghost API v1 must support:

- `watch` and `assist`
- structured proposals
- explicit accept / reject flow
- action-log integration
- lock and budget enforcement
- TUI-visible rationale

It does not yet need:

- autonomous performance mode
- long-horizon planning
- hidden multi-scene orchestration
- cloud-assisted execution

---

## 16. Validation Requirements

Required validation:

- Ghost cannot bypass locks
- accepted Ghost actions serialize into the action log correctly
- rejected Ghost actions do not mutate durable musical state beyond the suggestion decision marker
- Ghost suggestions render clearly in the TUI
- Ghost action replay remains deterministic

---

## 17. Open Follow-Ups

This draft should be followed by:

1. exact Ghost tool registry
2. proposal-to-view-model contract
3. budget defaults per mode
4. escalation path for future `perform` mode

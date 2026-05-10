# Riotbox Architecture Drift Guardrails

Date: 2026-05-10

Audience: coding agents, reviewers, and maintainers

Suggested repo path: `docs/reviews/riotbox_drift_guardrails_2026-05-10.md`

## Purpose

This note freezes the practical review findings from the May 2026 Riotbox codebase inspection into durable working rules.

The goal is not to slow Riotbox down. The goal is to keep the project from drifting into generic AI-assisted glue code while it grows from prototype into a reliable musical instrument.

This document should be treated as an agent-facing guardrail. Canonical project truth remains in:

- `AGENTS.md`
- `docs/prd_v1.md`
- `docs/execution_roadmap.md`
- `docs/workflow_conventions.md`
- `docs/specs/`
- `docs/reviews/`
- Linear and Git history where applicable

If this document conflicts with a more specific product or architecture spec, update the specific spec and this document together instead of silently choosing one.

---

## Current Assessment

Riotbox is not AI slop today.

It is a controlled, AI-assisted Brownfield project with a real product spine, typed contracts, explicit queue and commit semantics, replay/session state, Source Graph contracts, and meaningful audio/QA evidence.

The strongest anti-slop signals are:

- typed core contracts instead of stringly feature glue
- one central Action Lexicon rather than many hidden action paths
- explicit queue and quantized commit semantics
- explicit Session and Source Graph models
- test and QA recipes that check state, replay, observer output, generated fixtures, audio metrics, and reproducibility
- honest documentation about prototype status and remaining gaps

The main drift risk is not the core model. The main drift risk is the app orchestration layer, especially `crates/riotbox-app/src/jam_app*`.

`jam_app` is currently a necessary facade, but it can become a feature gravity well if new slices keep adding queue functions, match arms, summary strings, side effects, runtime mirrors, and state fields without consolidation.

---

## Drift Model

### Green zones

These areas currently show strong architecture discipline:

- `riotbox-core` action contracts
- queue and commit behavior
- Session file model
- Source Graph model
- fixture-backed source timing contracts
- audio QA and observer correlation harnesses

### Yellow zones

These areas need active pressure management:

- `JamAppState` growth
- `jam_app` facade imports and orchestration coupling
- repeated queue-draft construction across lanes
- repeated side-effect log-result mutation
- repeated string labels that may become semantic state
- mechanical `include!` splits that preserve behavior but do not create true module boundaries

### Red flags

Any new slice should be stopped for review if it introduces one of these patterns:

- a second action system
- a second persistence/session model
- a second queue or commit path
- a second replay truth
- a hidden Ghost-only arrangement state
- a hidden Feral-only architecture branch
- realtime audio code that blocks on I/O, analysis, model calls, or heavy UI work
- features that only change UI text or logs when the claimed product change should be audible, replayable, or stateful
- new string identifiers that start controlling behavior without a typed contract or documented transition plan
- new large helper modules that hide coupling instead of reducing it

---

## Mandatory Rule For Every New `ActionCommand`

Every new `ActionCommand` must be mapped before implementation is considered complete.

A new action is not done until it has an explicit answer for all five surfaces:

1. **Queue path**
   - Where is the action drafted?
   - What actor creates it?
   - What quantization applies?
   - What target scope and params are used?

2. **Commit and side-effect path**
   - Where does it commit?
   - Which side-effect module owns it?
   - Does it alter Session state, runtime state, capture artifacts, or only logs?

3. **Session / replay consequence**
   - What state must survive restore?
   - What state must be represented in `SessionFile`?
   - Does deterministic replay need an assertion?

4. **User-visible or observer surface**
   - What does the musician see, hear, or inspect?
   - Does the observer or runtime view need to expose it?
   - Is a summary string enough, or is real state/audio evidence required?

5. **Test / QA proof**
   - Which unit tests cover the action?
   - Which integration or replay tests cover it?
   - If audio-producing, which buffer regression, offline render comparison, listening manifest, or observer/audio gate proves it?

If one of these surfaces is intentionally not applicable, the PR must say why.

---

## `jam_app` Guardrails

`JamAppState` may remain the app facade, but it must not become the product architecture.

When adding new behavior, prefer one of these patterns:

- add behavior to a lane-specific queue module when the change is about user intent
- add behavior to a lane-specific side-effect module when the change is about committed session mutation
- add behavior to runtime projection or runtime view modules when the change is about rendering state to a view
- add behavior to capture helpers or persistence helpers when the change is about artifacts or storage
- add behavior to core when the state affects replay, restore, capture lineage, source timing, or product contracts

Do not add new state to `JamAppState` unless all are true:

- the state is truly app-runtime state rather than Session truth
- the state does not need to survive restore
- the state does not affect deterministic replay
- there is no better typed home in `SessionFile`, Source Graph, queue state, or audio runtime state
- the PR explains why the facade owns it

A new field on `JamAppState` should trigger a short review question:

> Is this app coordination state, or are we hiding product truth outside the Session/Core model?

---

## Queue-Draft Construction Rule

Repeated action draft construction is acceptable while the project is young, but it should not grow unbounded.

When three or more lane queue functions repeat the same shape, consider extracting a small helper.

Good helper shape:

- narrow
- typed
- boring
- no new product semantics
- names the lane, action, quantization, scope, params, and explanation clearly

Bad helper shape:

- generic action factory that hides product intent
- macro-heavy construction
- helper that accepts too many strings
- helper that bypasses target scope or quantization clarity

Suggested future helper direction:

```rust
// Example shape only; do not add mechanically.
fn lane_mutation_draft(
    actor: ActorType,
    command: ActionCommand,
    quantization: Quantization,
    scope: TargetScope,
    intensity: f32,
    target_id: Option<String>,
    explanation: impl Into<String>,
) -> ActionDraft
```

The goal is not abstraction for its own sake. The goal is to prevent repeated agent-generated queue code from diverging silently.

---

## Side-Effect Result Rule

Side-effect modules currently repeat this pattern:

- find the committed action in `session.action_log.actions`
- mutate its `ActionResult`
- write a musician-facing summary

That is acceptable today, but future slices should prefer a shared helper when the repetition continues.

The helper should remain explicit and small. It should not hide whether a side effect accepted, rejected, failed, or only summarized an action.

Suggested direction:

```rust
// Example shape only; do not add mechanically.
fn update_logged_action_result(
    session: &mut SessionFile,
    action_id: ActionId,
    accepted: bool,
    summary: impl Into<String>,
)
```

Any helper must preserve the current behavior that side effects update the already-committed action rather than creating a second log event.

---

## Stringly-State Rule

String labels are allowed for musician-facing summaries, artifact labels, and stable display references.

String labels become suspicious when they start controlling behavior.

If a string value is used for branching, replay, restore, QA validation, generated artifacts, or cross-module behavior, ask whether it should become a typed enum or a documented contract.

Examples that should be watched:

- MC-202 role names
- MC-202 phrase variants
- W-30 damage profile labels
- W-30 loop-freeze labels
- TR-909 profile labels
- source timing warning/status strings
- observer/audio manifest fields

The project already uses strong enums in many places. New semantic state should follow that style unless there is a documented reason not to.

---

## Audio-Producing Slice Rule

Do not close an audio-producing slice with only UI, log, or summary proof.

A slice that claims audible behavior must include at least one proof path from this set:

- buffer regression test
- offline render comparison
- generated listening manifest validation
- observer/audio correlation
- source-vs-control metric check
- reproducibility smoke
- documented manual listening note when the relevant automated seam is not operational yet

The proof must demonstrate that the affected seam is:

- not silent
- not fallback-collapsed
- inside expected metrics
- connected to the intended action, lane, source, or render state

If the current harness cannot prove the claim yet, the PR must say that explicitly and should add the smallest useful harness improvement when feasible.

---

## `include!` Split Rule

Mechanical `include!` splits are acceptable when used to preserve behavior and reduce review/context cost.

They are not a substitute for durable module ownership.

A future refactor should convert a textually included file into a real module only when all are true:

- the semantic boundary is clear
- visibility becomes clearer
- tests still map naturally to the behavior
- the change reduces review cost
- the change does not obscure the public contract

Do not split or unsplit files only to satisfy line counts.

---

## Feature vs Cleanup Balance

Riotbox should not spend unlimited time on cleanup-only slices.

Use this balance:

- keep cleanup slices small and behavior-preserving
- finish active import/coupling cleanup lanes when they are already in progress
- after several cleanup-only PRs, return to a product-path slice unless a real drift blocker remains
- prefer cleanup that unlocks safer future feature work
- avoid cleanup that only changes aesthetics or file layout

A good cleanup PR should answer:

> What future drift or review failure does this prevent?

---

## Slop Audit Cadence

Run a broader drift/slop audit every 5 to 10 finished feature branches, or sooner after a cluster of agent-generated slices.

The audit should check for:

- new `use super::*` or broad import surfaces
- repeated queue draft construction
- repeated side-effect log-result mutation
- new `ActionCommand` values without all five required surfaces
- new fields on `JamAppState`
- new Session-adjacent state stored outside Session/Core without explanation
- new string constants that control behavior
- new feature claims with only log/UI proof
- new generated artifacts or validators not wired into `just ci` or the appropriate QA recipe
- tests that assert implementation text but not product behavior
- audio slices with no audible or metric proof
- Shadow System risk around Ghost, Feral, capture, replay, source timing, or persistence

Record meaningful findings in `docs/reviews/` and update `AGENTS.md`, `docs/workflow_conventions.md`, or the relevant spec when the finding changes how agents should work.

---

## Spec Kit Guardrail

Spec Kit may be used as a draft and analysis tool for bounded feature slices.

Spec Kit must not become a second source of truth for Riotbox.

Recommended stance:

> Riotbox owns the contracts; Spec Kit may draft the branch plan.

Allowed uses:

- clarify a bounded feature slice
- produce a task list for a branch
- check for ambiguity before coding
- run an analysis pass against a planned change

Disallowed uses unless explicitly approved:

- replacing `docs/specs/`
- replacing `AGENTS.md`
- replacing Linear as the operations layer
- generating a new architecture path for Ghost, Feral, replay, source timing, capture, or audio runtime
- running implementation blindly on realtime/audio-critical code

Every Spec Kit-generated plan must be checked against the five-surface `ActionCommand` rule and the audio-producing slice rule before implementation.

---

## Suggested Addition To `AGENTS.md`

The following block can be added to `AGENTS.md` under Architecture Rules or Review gate.

```markdown
### Architecture drift / AI-slop guardrail

Riotbox may use coding agents, but agents must not create generic glue-code drift.

For every new `ActionCommand`, explicitly account for:

1. queue path
2. commit / side-effect path
3. Session / replay consequence
4. user-visible or observer surface
5. test / QA proof

Do not close a slice when the claimed product change only appears in UI text or logs. If a feature is supposed to affect sound, replay, capture lineage, source timing, restore, or exported artifacts, include a concrete proof path.

Before adding state to `JamAppState`, ask whether the state belongs in Session/Core instead. `JamAppState` is an app facade, not a second product truth.

Avoid new `use super::*` imports in app modules. Prefer explicit imports so coupling remains reviewable.

String labels may be used for display and artifact names. If a string starts controlling behavior across modules, turn it into a typed contract or document why it remains a string.
```

---

## Suggested Addition To `docs/workflow_conventions.md`

The following block can be added to the workflow conventions document.

```markdown
## Drift review checklist

For each finished feature branch, reviewers should check whether the branch introduced hidden architecture drift.

Minimum questions:

- Did this add or change an `ActionCommand`?
  - If yes, are queue, commit/side-effect, Session/replay, user/observer surface, and QA proof all covered?
- Did this add state to `JamAppState`?
  - If yes, why is it app-runtime state rather than Session/Core truth?
- Did this add a new lane, Ghost, Feral, capture, replay, source timing, or persistence path?
  - If yes, does it reuse the existing contracts instead of creating a shadow system?
- Did this claim an audible product change?
  - If yes, where is the audio, metric, observer/audio, listening manifest, or reproducibility proof?
- Did this introduce string values that now control behavior?
  - If yes, should they become enums or documented contract fields?
- Did this increase repeated queue or side-effect patterns?
  - If yes, is a small helper now warranted?

Record recurring findings in `docs/reviews/` and promote durable rules into `AGENTS.md` or the relevant spec.
```

---

## Suggested PR Description Addition

Add this section to PR descriptions for non-trivial feature branches.

```markdown
## Drift Check

- New `ActionCommand`: yes/no
- Queue path covered: yes/no/n/a
- Commit or side-effect path covered: yes/no/n/a
- Session/replay consequence covered: yes/no/n/a
- User-visible or observer surface covered: yes/no/n/a
- Test/QA proof included: yes/no/n/a
- Added `JamAppState` state: yes/no
- Added or changed audio-producing behavior: yes/no
- Shadow-system risk reviewed: yes/no
```

---

## Agent Instruction Summary

When working on Riotbox, prefer:

- smaller typed contracts over generic helpers
- one action truth over hidden feature-specific actions
- Session/Core truth over facade-local state
- explicit imports over broad parent imports
- real behavior tests over summary-string tests
- audio/metric proof over UI/log proof
- bounded cleanup over endless refactoring
- documented decisions over hidden assumptions

Riotbox should remain a musical instrument with explicit contracts, not a pile of plausible-looking agent-generated slices.

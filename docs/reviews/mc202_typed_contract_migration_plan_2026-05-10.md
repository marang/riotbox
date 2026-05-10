# MC-202 Typed Contract Migration Plan

Date: 2026-05-10
Ticket: RIOTBOX-747
Audience: coding agents, reviewers, maintainers

## Purpose

This note turns the drift-guardrail finding about MC-202 stringly state into a
bounded migration plan.

The goal is not to redesign the MC-202 lane. The goal is to keep the existing
queue, commit, replay, render, observer, and TUI surfaces on one shared contract
while moving behavior-controlling role and phrase intent labels out of ad-hoc
strings.

## Current String Contract

The current implementation uses these behavior-controlling role labels:

- `leader`
- `follower`
- `answer`
- `pressure`
- `instigator`

The current implementation also uses this phrase mutation label:

- `mutated_drive`

These strings appear in multiple behavior surfaces:

- MC-202 queue targets in `crates/riotbox-app/src/jam_app/mc202_queue.rs`
- committed MC-202 side effects in `crates/riotbox-app/src/jam_app/side_effects/mc202.rs`
- replay execution in `crates/riotbox-core/src/replay/executor.rs`
- persisted lane state in `crates/riotbox-core/src/session/version_types.rs`
- render projection in `crates/riotbox-app/src/jam_app/projection/tr909_projection.rs`
- TUI, observer, fixture, and recipe surfaces as display labels

Display labels are fine. Behavior branching on arbitrary strings is the drift
risk.

## Migration Rule

Do not add a second MC-202 action system, replay path, phrase engine, or render
model.

MC-202 typed migration must preserve the existing action commands:

- `mc202.set_role`
- `mc202.generate_follower`
- `mc202.generate_answer`
- `mc202.generate_pressure`
- `mc202.generate_instigator`
- `mc202.mutate_phrase`

The migration should introduce typed conversion at the contract boundary, then
move internal behavior to the typed values. Strings should remain only for JSON
compatibility, external display, and artifact labels until a session-version
migration is explicitly shipped.

## Proposed Types

Add core/session-facing enums with stable snake-case serialization labels:

```rust
#[serde(rename_all = "snake_case")]
enum Mc202RoleState {
    Leader,
    Follower,
    Answer,
    Pressure,
    Instigator,
}

#[serde(rename_all = "snake_case")]
enum Mc202PhraseIntentState {
    Base,
    MutatedDrive,
}
```

`Mc202PhraseVariantState::MutatedDrive` already exists. The eventual migration
can either rename/extend that enum into an intent contract or introduce the new
intent enum and keep the old field as compatibility input.

## Staged Migration

### Stage 1: Typed conversion helpers

Add a typed conversion layer in `riotbox-core`, with tests proving every current
role and mutation label round-trips through the expected stable label.

Expected helpers:

- parse role label into `Mc202RoleState`
- serialize `Mc202RoleState` back to the stable label
- derive default touch and phrase shape intent from role
- parse phrase mutation target into typed phrase intent

This stage should not change session JSON shape yet.

### Stage 2: Use typed values inside app side effects and replay

Replace direct string comparisons in commit and replay code with typed parsing at
the action boundary.

Important behavior to preserve:

- `mc202.set_role` still accepts existing `ActionTarget::object_id` and
  `ActionParams::Mutation.target_id` labels.
- old sessions still replay.
- unknown role labels reject or degrade explicitly instead of silently rendering
  idle in one surface and mutating state in another.
- commit summaries still use musician-facing labels.

### Stage 3: Use typed values in render projection

Render projection should branch on `Mc202RoleState` / phrase intent rather than
raw strings.

This reduces the chance that a future role string lands in Session/replay but
does not render, or renders but does not replay.

### Stage 4: Session-file migration

Only after Stage 1-3 are stable, decide whether the persisted `Mc202LaneState`
should store:

- compatibility strings plus typed accessors, or
- typed enum fields with a session-version migration for old JSON.

If the JSON shape changes, update `docs/specs/session_file_spec.md`, fixture
roundtrips, restore tests, and archive the migration decision.

## Required Proof For Implementation Slices

Every implementation slice in this migration must prove all relevant surfaces:

- queue path still writes the same command, quantization, target, params, and
  explanation for each gesture
- committed side effects update the same lane state and action result summaries
- replay of old action logs converges to the same lane state and render state
- render projection stays non-silent and keeps existing MC-202 role contrasts
- TUI and observer labels remain musician-readable
- session roundtrips keep backward compatibility

Minimum test targets:

- `cargo test -p riotbox-app mc202 -- --nocapture`
- `cargo test -p riotbox-core mc202 -- --nocapture` once core helpers exist
- affected replay/snapshot tests for MC-202 follower, answer, pressure,
  instigator, mutation, and undo
- lane recipe or observer/audio proof when render behavior is touched

## Do Not Do

- Do not rename labels casually. Stable labels are part of current session,
  replay, TUI, and QA evidence.
- Do not add a new `ActionCommand` for typed roles.
- Do not store typed MC-202 truth only in `JamAppState`.
- Do not let `phrase_ref` become the behavioral authority. It is a reference and
  display/provenance aid, not the source of phrase semantics.
- Do not close an audio/render slice with only UI or log proof.

## Recommended Next Ticket

Create a small implementation ticket for Stage 1:

> Add core MC-202 role/phrase intent conversion helpers with compatibility tests.

Keep that ticket behavior-preserving. It should introduce typed helpers and
tests only, then later slices can move side effects, replay, and projection to
consume the helpers.

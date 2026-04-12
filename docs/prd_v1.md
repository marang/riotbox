# Riotbox PRD v1

Version: 0.1  
Status: Draft  
Audience: product, realtime, MIR/ML, TUI, QA

Derived from:
- `plan/riotbox_masterplan.md`
- `plan/riotbox_liam_howlett_feral_addendum.md`

---

## 1. Purpose

This document defines the MVP product contract for Riotbox.

It is not a full implementation spec. It fixes:

- what the MVP must do
- what is explicitly out of scope
- which user flows matter first
- which acceptance criteria define "usable"
- which follow-up specs are required before implementation scales

---

## 2. Product Statement

Riotbox is a terminal-native audio instrument that transforms an input track into a controllable live performance object.

For the MVP, the product must let a user:

- load one track
- run a basic analysis pass
- generate a playable hybrid rebuild
- capture useful material into the instrument
- control key musical changes in realtime
- save and restore a deterministic session
- receive assistive Ghost suggestions without sacrificing control

The MVP is an instrument-first system, not a black-box generator and not a DAW replacement.

---

## 3. Product Principles

- Instrument before magic
- Realtime stability before model cleverness
- Jam-first before lab complexity
- Capture-first before offline perfection
- Determinism before hand-wavy "AI behavior"
- Profile as policy, not as forked architecture

---

## 4. Target Users

### 4.1 Primary User

Live performer / producer who wants to load a track, mutate it quickly, capture good moments, and keep musical control.

### 4.2 Secondary Users

- producer using Riotbox for loop and phrase extraction
- sound designer building banks and resamples
- exploratory user watching Ghost suggestions or performance behavior

---

## 5. MVP Problem To Solve

Current audio tools split across two weak extremes:

- remix tools that are too static
- generative tools that are too opaque

The Riotbox MVP must prove a middle path:

- audio in
- useful structure out
- immediate live control
- meaningful capture and recall
- explainable assistive behavior

---

## 6. MVP Scope

### 6.1 In Scope

- load one local audio file
- run basic analysis sufficient for a playable rebuild
- create a source-derived musical object usable in a jam context
- provide a Jam screen with key state visibility
- expose core macros for live manipulation
- enable quantized mutation
- enable loop or pad capture into the W-30 path
- enable basic TR-909 reinforcement
- enable basic MC-202 follower behavior
- provide undo for recent committed actions
- save and load sessions deterministically
- show Ghost suggestions in `watch` or `assist`

### 6.2 Explicitly Out of Scope

- cloud AI
- multi-user or network collaboration
- full DAW export workflow
- polished cross-platform packaging
- advanced granular or spectral processing
- full vocal editing suite
- plugin ecosystem
- autonomous Ghost performance as a headline MVP requirement

---

## 7. MVP Operating Modes

The MVP focuses on a constrained subset of the masterplan:

- primary mode: `hybrid`
- secondary support: limited `derivative`-leaning behavior where needed for capture and rebuild

The MVP does not need a full `de_novo` experience.

The `feral_rebuild` profile may exist in MVP only as a preset/policy layer over existing macros and scoring, not as a separate mode or separate architecture.

---

## 8. Core User Flows

### 8.1 Flow A: Track To Jam

1. User loads a track.
2. Riotbox analyzes core structure.
3. Jam screen becomes playable.
4. User starts playback and hears a source-informed rebuild.
5. User can shape energy, source retain, 202 touch, W-30 grit, and 909 slam.

Success condition:
- the instrument becomes musically usable within a few minutes

### 8.2 Flow B: Capture A Good Moment

1. User hears a useful loop, phrase, or fragment.
2. User captures it at a quantized boundary.
3. Captured material is promoted into the playable W-30 path.
4. User can recall or reuse it without losing flow.

Success condition:
- capture feels like a core musical action, not like debug tooling

### 8.3 Flow C: Assisted Mutation

1. User sees Ghost suggestion or invokes a suggestion pass.
2. Ghost suggests a bounded musical action.
3. User accepts or rejects it.
4. Accepted action lands on a safe quantization boundary and is undoable.

Success condition:
- Ghost is useful and legible without feeling autonomous or dangerous

### 8.4 Flow D: Save And Restore

1. User saves session state.
2. User reloads later.
3. Riotbox restores the session with the same source references, seeds, key state, and action history needed for deterministic rebuild.

Success condition:
- a saved session feels replayable, not approximate

---

## 9. Functional Requirements

### 9.1 Audio And Analysis

- support at least one practical local file input path
- decode audio into internal analysis-ready representation
- estimate enough structural information to support:
  - tempo/grid confidence
  - section overview
  - loop candidates
  - basic tonal or contour hints if available
- produce a stable source graph contract for downstream use

### 9.2 Realtime Core

- stable playback under normal laptop load
- quantized action commit model
- no analysis or Ghost failure may block the audio path
- pending actions must be visible before commit

### 9.3 Device Lanes

- TR-909 path must reinforce or partially replace drums in a controllable way
- MC-202 path must generate a usable follower line
- W-30 path must support at least basic capture and playback of slices or loops

### 9.4 TUI

- Jam screen is required
- page model may be partial, but Jam must expose the main live controls and state
- logs, confidence, and pending actions must be visible enough to build trust
- TUI must avoid parameter flood

### 9.5 Ghost

- Ghost must support `watch` and `assist`
- Ghost actions or proposals must reference available tools/actions only
- every accepted Ghost action must be logged and undoable
- Ghost must respect locks, budgets, and quantization rules

### 9.6 Persistence

- session save/load is required
- replay-critical state must be serializable
- versioning and migration strategy must be specified before session format is treated as stable

---

## 10. Feral Profile Handling In MVP

The Liam/Feral profile is included as a constrained extension to the MVP, not as an independent product surface.

Rules:

- no new top-level mode
- no separate session format
- no separate UI shell
- no separate Ghost system
- no hardcoded architecture branch

Allowed MVP-level feral support:

- preset or policy entry such as `feral_rebuild`
- profile-derived macro interpretation
- profile-derived scoring weights
- profile-derived scene tendencies
- profile-specific acceptance tests once the core contracts exist

Deferred feral depth:

- expanded damage profiles
- full rebake queue behavior
- wide break-variant families
- richer Ghost feral tooling

---

## 11. Acceptance Criteria

The MVP is acceptable when all statements below are true:

- a user can load a track and obtain a playable rebuild
- the Jam screen is sufficient for a short live interaction loop
- at least one useful capture workflow exists and feels intentional
- 202 and 909 contribute musically controllable behavior
- recent mutations are undoable
- sessions can be saved and restored with deterministic intent
- Ghost can make useful suggestions without breaking control or replayability
- the system remains usable if analysis confidence is imperfect

---

## 12. Non-Functional Requirements

- audio stability over novelty
- bounded CPU behavior
- no blocking model or analysis work in the realtime path
- visible confidence and pending action states
- graceful degradation when analysis is weak
- meaningful logs for operator trust and debugging

---

## 13. Open Questions Before PRD Freeze

- exact MVP-supported file formats
- minimum viable source graph fields
- minimum viable session schema
- exact first-class action list for quantized mutation
- which Ghost actions are MVP-safe enough to expose
- which feral behaviors are policy-only in v1 and which are deferred
- what counts as a "successful capture" in QA terms

---

## 14. Required Follow-Up Specs

These specs should be written next because they unblock implementation:

1. `Source Graph Spec`
2. `Session File Spec`
3. `Action Lexicon Spec`
4. `Audio Core Spec`
5. `TUI Screen Spec`
6. `Ghost API Spec`
7. `Preset & Style Spec`
8. `Validation Spec`

---

## 15. Delivery Guidance

Implementation should proceed in vertical slices, but only after the contracts above are explicit enough that teams do not invent their own incompatible assumptions.

The preferred early build order is:

1. stable skeleton
2. source graph vertical slice
3. session/action contract
4. jam-first TUI
5. device-lane MVP behavior
6. Ghost assist
7. feral profile expansion

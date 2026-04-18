# Jam First-Use Feedback 2026-04-18

Context:

- feedback captured from first manual learning runs against the current Jam shell
- intent is to preserve product/UX observations for later ticketing without interrupting the current core implementation path

## Primary Observations

- the current shell still feels more like an operator/diagnostic surface than a clearly playable first-run instrument
- first contact is over-explained by text and under-explained by hierarchy, audible feedback, and obvious next actions
- key ideas such as quantized commit timing, `fill`, and `capture` are not yet obvious enough from the shell alone
- `capture` currently feels partly blind because the user can store or promote material before the audible feedback path is strong enough to make that material feel owned

## Specific User Questions Worth Preserving

- what is the “right musical moment” for queued actions, and how should that timing be made visible?
- what exactly does `fill` do, and how is it different from `capture` in practical play?
- when captured material is surfaced on the `Capture` screen, should it already be audibly previewable?
- how can the shell reduce the current “everything is equally important” feeling?

## Recommended Later Ticket Bundles

### 1. Quantization / Commit Visualization

- add one clearer visual cue for next beat / next bar / next phrase timing
- make the “action is queued, action landed” boundary visible without requiring deep `Log` reading

### 2. Gesture Semantics / Learnability

- explain `fill`, `capture`, `restore`, and related terms more directly in the shell and docs
- keep the learning path focused on one believable first success loop instead of exposing the full vocabulary immediately

### 3. Audible Feedback / Audition

- strengthen preview/audition so capture and reuse stop feeling like blind state manipulation
- make it clearer when a gesture is workflow-only today versus when it should already change the heard result

### 4. Visual Hierarchy / Color System

- introduce a stronger color and emphasis system so warnings, next actions, active lane state, and secondary diagnostics stop competing equally
- continue reducing text density on the primary Jam surface

### 5. Capture UX

- clarify what was actually captured
- clarify when it becomes reusable
- connect capture ownership more directly to something the user can hear or explicitly preview

## Product Interpretation

The current issue is not that Riotbox is too weak. It is that the engine, state model, and learning surface are not yet equally mature. The right response is to keep the core path moving while preserving these notes for a later UX-focused pass instead of forcing premature polish into the current implementation slices.

# MC-202 MVP Exit Review 2026-04-26

Context:

- ticket: `RIOTBOX-313`
- phase: `P006 | MC-202 MVP`
- source criteria:
  - `docs/phase_definition_of_done.md`
  - `plan/riotbox_liam_howlett_feral_addendum.md`
  - `docs/specs/audio_qa_workflow_spec.md`

## Summary

The MC-202 lane is close to its first honest MVP, but it is not exit-clean until one replay/undo gap is closed.

What is now real:

- follower, answer, pressure, and instigator roles exist on the existing queue / commit seam
- MC-202 bass reaches the live audio render seam through typed render state
- touch is live controllable with `<` / `>`
- phrase mutation is quantized with `G`
- pressure, instigator, mutation, contour, and hook-response behavior have output-path proof
- musician-facing recipe replay proof landed in PR #302

Blocking gap:

- `undo` currently marks the latest undoable action as undone and appends an undo marker, but it does not restore the previous MC-202 lane state or prove the audible render state rolls back

Conclusion:

- Phase 4 is musically and architecturally close.
- Do not mark the MC-202 MVP fully done until MC-202 undo rollback is implemented with control-path and output-path proof.

## Phase 4 Criteria

| Criterion | Status | Evidence | Remaining Gap |
| --- | --- | --- | --- |
| usable follower basslines exist | Satisfied for MVP | `mc202.generate_follower`, typed `Mc202RenderState`, live audio runtime wiring, follower-vs-answer render proof | sound design is still first-pass, not final instrument character |
| sound parameters are live controllable | Satisfied for MVP | `<` / `>` update persisted `mc202_touch`, refresh render state, and have low-vs-high output proof | only touch exists as a live macro today |
| phrase mutation is quantized | Satisfied for MVP | `mc202.mutate_phrase` queues for `NextPhrase`, commits `mutated_drive`, and has signal-delta proof | only one mutation variant exists |
| the lane adds pressure without clutter | Satisfied for MVP | `pressure` role, sparse `pressure_cell`, note budget, hook `answer_space`, and listening-pack deltas | source-aware phrase scoring is still coarse |
| replay and undo remain intact | Partially satisfied | committed MC-202 state survives JSON roundtrip and action-log replay fixtures; recipe replay proof is in PR #302 | undo does not yet roll back MC-202 lane/render/audio state |

## Feral Addendum Criteria

| Criterion | Status | Evidence | Remaining Gap |
| --- | --- | --- | --- |
| answer role | Satisfied | `mc202.generate_answer` action, committed lane state, render shape, tests | deeper answer selection can improve later |
| pressure role | Satisfied | `mc202.generate_pressure`, sparse note budget, audio delta proof | pressure policy is deterministic and simple |
| instigate role | Satisfied | `mc202.generate_instigator`, push note budget, audio delta proof | future source-aware push scoring can refine it |
| hook-response rules instead of hook doubling | Satisfied for MVP | `Mc202HookResponse::AnswerSpace` for hook-like sections, sparse/offbeat response, audio delta proof | hook detection is section-label/tag based |
| contour follower with feral simplification | Satisfied for MVP | `Mc202ContourHint` from source/scene section context, render interval offsets, audio delta proof | no pitch tracking or extracted bassline yet |
| note budget against overplay | Satisfied for MVP | typed `Mc202NoteBudget` projected into render state and renderer gating | budget is phrase-shape based, not adaptive yet |

## Required Follow-Up

Create one bounded follow-up before closing `P006 | MC-202 MVP`:

- `RIOTBOX-314`: MC-202 undo rollback must restore the previous committed lane state, refresh the typed render state, and prove the rendered output returns to the previous audible seam.

Suggested acceptance:

- queue and commit at least two MC-202 states, then undo
- assert the action log contains the undone action and undo marker
- assert session lane state, `Mc202RenderState`, and Jam view return to the previous MC-202 role/phrase/touch/variant
- render before/after buffers and prove undo output matches the previous state more closely than the undone state

## Non-Blocking Follow-Ups

These should not block the first MC-202 MVP exit:

- richer source-aware phrase scoring beyond section labels and tags
- more mutation variants beyond `mutated_drive`
- stronger sound-design pass for the bass voice
- a TUI observer replay that drives the exact user key sequence instead of the app-level action path

## Review Notes

The important distinction is that MC-202 now has meaningful output proof, not only state/log proof. The remaining issue is narrower: Riotbox promises experimentation with undo, and for MC-202 that must mean the sounded lane state rolls back, not only that an action row changes to `undone`.

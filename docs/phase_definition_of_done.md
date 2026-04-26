# Riotbox Phase Definition of Done

Version: 0.1  
Status: Draft  
Audience: product, engineering, QA

---

## 1. Purpose

This document defines the minimum "done" criteria for each implementation phase.

A phase is not done when code exists. A phase is done only when:

- the promised output exists
- the required tests pass
- the required benchmarks were run
- the user-facing result is real

---

## 2. Global Done Rules

Every phase must satisfy:

- core deliverables implemented
- no unresolved blocker in the product spine for that phase
- relevant automated tests pass
- benchmark results recorded when applicable
- major decisions captured in `docs/research_decision_log.md`

---

## 3. Phase 0 - Sound Bible and Specification

Done when:

- vocabulary is stable enough for implementation
- MVP scope is explicit
- action lexicon exists
- execution roadmap exists
- validation and fixture docs exist
- no major core contract is still hand-wavy

---

## 4. Phase 1 - Core Skeleton

Done when:

- Riotbox starts and stops cleanly
- audio playback is stable in the baseline environment
- scheduler behavior is testable
- session state exists
- action log exists
- snapshot baseline exists
- Jam shell is visible
- callback timing and xrun metrics are observable

---

## 5. Phase 2 - Analysis Vertical Slice

Done when:

- a track can be loaded
- decode / normalize works
- beat / bar information exists
- sections exist
- first slice or loop candidates exist
- a Source Graph v1 is produced
- Jam screen shows useful analysis state
- analysis failure degrades visibly rather than silently

---

## 6. Phase 3 - TR-909 MVP

Done when:

- source drums can be reinforced audibly
- 909 can take over in a controlled way
- fills are triggerable live
- drum behavior remains quantized and stable
- reinforcement does not break replay or capture

---

## 7. Phase 4 - MC-202 MVP

Done when:

- usable follower basslines exist
- sound parameters are live controllable
- phrase mutation is quantized
- the lane adds pressure without clutter
- replay and undo remain intact

Current exit review:

- `docs/reviews/mc202_mvp_exit_review_2026-04-26.md`
- status: closed for MVP on 2026-04-26 after `RIOTBOX-314` added MC-202 undo rollback with control-path and output-path proof

---

## 8. Phase 5 - W-30 MVP

Done when:

- useful loops can be captured
- pads are playable
- internal bus resampling works
- captured material can be reused without leaving flow
- provenance for captured material is not lost

Current reviews:

- `docs/reviews/w30_mvp_gap_review_2026-04-26.md`
- `docs/reviews/w30_mvp_exit_review_2026-04-26.md`

---

## 9. Phase 6 - Scene Brain

Done when:

- a track yields multiple usable scenes
- scene changes sound musical
- restore logic works
- default arrange no longer feels like a static 8-bar loop
- scene actions remain replay-safe

---

## 10. Phase 7 - Ghost / AI Assist

Done when:

- Ghost can make useful suggestions
- Ghost can execute approved quantized actions safely
- Ghost respects locks and budgets
- Ghost actions are logged and explainable
- accepted Ghost actions remain undoable and replayable

---

## 11. Phase 8 - Pro Hardening

Done when:

- deterministic replay is trustworthy
- crash recovery is acceptable
- export outputs are reproducible
- long-run tests are acceptable
- benchmark regressions are visible
- a stage-style end-to-end run completes reliably

---

## 12. Feral Layer Done Criteria

The feral layer is done for MVP purposes when:

- harvest produces usable fragment candidates
- at least one break rebuild path is musically interesting
- hook-fragment handling exists without full-quote dependence
- resample reuse is real, not decorative
- feral scorecard metrics can be generated

---

## 13. Final Product Readiness

Riotbox is close to product-ready when:

- the full user spine works end-to-end
- benchmarks are stable enough to catch regressions
- validation corpus is in routine use
- Ghost watch / assist is dependable
- feral mode behaves as policy, not architecture drift
- repeated live-oriented runs do not expose structural instability

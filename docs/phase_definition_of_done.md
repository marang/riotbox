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

Current note:

- the initial Analysis v1 contract exists, but Source Timing Intelligence reopens the timing foundation as a stricter all-lane analysis track; these criteria define the next timing-ready analysis bar and do not retroactively invalidate later bounded MVP exits

Done when:

- a track can be loaded
- decode / normalize works
- beat / bar information exists
- beat, downbeat, bar, and phrase timing includes confidence and visible degraded-state behavior
- multiple timing hypotheses are preserved when half-time, double-time, or downbeat ambiguity is plausible
- source-grid drift reporting exists and is usable by downstream render checks
- sections exist
- first slice or loop candidates exist
- a Source Graph v1 is produced
- Jam screen shows useful analysis state
- analysis failure degrades visibly rather than silently
- TR/Kick-Bass, MC-202, and W-30 each have at least one timing-aware output-path proof with source-vs-output audio evidence

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
- status: closed for MVP on 2026-04-26 after `RIOTBOX-322` added duration-aware focused W-30 pad playback from committed capture artifacts with control-path and output-path proof

---

## 9. Phase 6 - Scene Brain

Done when:

- a track yields multiple usable scenes
- scene changes sound musical
- restore logic works
- default arrange no longer feels like a static 8-bar loop
- scene actions remain replay-safe

Current review:

- `docs/reviews/scene_brain_mvp_gap_review_2026-04-26.md`
- status: closed for bounded MVP on 2026-04-26 after `RIOTBOX-327` added replay-safe Scene movement state and bounded TR-909 / MC-202 render movement with mixed-output proof; this is not a full arranger or source-playback repositioner

---

## 10. Phase 7 - Ghost / AI Assist

Done when:

- Ghost can make useful suggestions
- Ghost can execute approved quantized actions safely
- Ghost respects locks and budgets
- Ghost actions are logged and explainable
- accepted Ghost actions remain undoable and replayable

Current review:

- `docs/reviews/p010_ghost_watch_assist_exit_review_2026-04-29.md`
- status: closed for bounded Watch / Assist MVP on 2026-04-29 after Ghost proposals, explicit accept/reject, lock awareness, pending/phrase/destructive budgets, normal action-queue commit, and structured commit metadata landed; autonomous `perform` mode remains a future escalation, not part of this phase exit

---

## 11. Phase 8 / P011 - Pro Hardening

Boundary:

- `P011 | Pro Hardening` is the final MVP-spine hardening project, not the first Post-MVP expansion project.
- The phase closes only when the bounded MVP spine can be trusted for repeated replay, recovery, QA, and export-oriented work.
- Post-MVP project phases such as Source Timing Intelligence and deeper musical expansion should start from this hardened spine instead of bypassing it.

Done when:

- deterministic replay is trustworthy
- crash recovery is acceptable
- export outputs are reproducible
- long-run tests are acceptable
- benchmark regressions are visible
- a stage-style end-to-end run completes reliably

Current review:

- `docs/reviews/p011_replay_hardening_checkpoint_2026-04-29.md`
- `docs/reviews/p011_replay_recovery_exit_checklist_2026-04-30.md`
- status: active after restore and commit-record validation hardening; not exit-ready because full replay execution, snapshot convergence, crash recovery, export reproducibility, and long-run/stage-run gates remain open

Current bounded gates:

- `just stage-style-jam-probe` exercises a longer generated W-30 source-diff output plus generated app-level multi-boundary observer/audio correlation evidence.
- `just stage-style-restore-diversity-probe` adds a richer replay-safe W-30 / TR-909 / MC-202 control path correlated with generated full-grid output evidence.
- `just stage-style-snapshot-convergence-smoke` proves a supported Scene / MC-202 / TR-909 stage-style suffix converges from a mid-run snapshot payload to the same final mix buffer as the committed path.
- `just stage-style-stability-smoke` / `just stage-style-stability-proof` repeats the richer generated run, requires stable full-mix hashes plus non-collapsed observer/audio summaries across repetitions, and validates a normalized proof manifest for run count, observer/audio evidence, commit-boundary coverage, and stable output hash.
- `just stage-style-stability-gate` is the stronger bounded variant with more repetitions and a longer generated source/grid budget.
- `just interrupted-session-recovery-probe` and `just missing-target-recovery-probe` cover generated file-backed recovery observer drills.
- `just offline-render-reproducibility-smoke` proves an existing deterministic source-backed W-30 render helper emits byte-stable WAV output for the same generated source.
- `just full-grid-export-reproducibility-smoke` / `just product-export-reproducibility-smoke` proves the deterministic Feral grid source-first plus generated-support pack validates and exports the same generated-support WAV hash twice from generated source material, then validates a normalized product-export proof that removes temp paths and compares stable manifest data plus audio artifact hashes.

Current boundary:

- These are bounded CI-safe probes, not host-audio soak tests, automatic recovery, full arrangement export, stem package export, live recording export, or DAW-style export coverage.
- P011 should remain active until the exit checklist can say replay, recovery, export reproducibility, and stage-style reliability are no longer smoke-level evidence.

---

## 12. Feral Layer Done Criteria

The feral layer is done for MVP purposes when:

- harvest produces usable fragment candidates
- at least one break rebuild path is musically interesting
- hook-fragment handling exists without full-quote dependence
- resample reuse is real, not decorative
- feral scorecard metrics can be generated

Current review:

- `docs/reviews/feral_policy_entry_audit_2026-04-26.md`
- `docs/reviews/p009_feral_policy_exit_review_2026-04-29.md`
- status: MVP-exit-clean for the bounded Feral policy layer; future autonomous promotion or quote-risk expansion must reuse the existing scorecard, lineage, action/result, and audio-QA seams rather than adding a separate Feral architecture

---

## 13. 1.0 / Stage-Ready Product Readiness

Riotbox is close to product-ready when:

- the full user spine works end-to-end
- benchmarks are stable enough to catch regressions
- validation corpus is in routine use
- Ghost watch / assist is dependable
- feral mode behaves as policy, not architecture drift
- repeated live-oriented runs do not expose structural instability

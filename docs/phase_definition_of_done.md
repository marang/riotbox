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
- status: closed for bounded P012 Source Timing foundation on 2026-05-28 after
  the all-lane source-grid output proof passed across generated Feral-grid
  observer/audio paths, Recipe 2 observer/audio, and Recipe 15 real-source
  auto/fallback proof; production-grade arbitrary-audio beat/downbeat detection
  remains deferred to later bounded P012+ work

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
- `docs/reviews/p011_exit_evidence_gate_review_2026-05-10.md`
- `docs/reviews/p011_exit_readiness_decision_2026-05-10.md`
- status: closed for bounded MVP-spine hardening on 2026-05-10 after the aggregate P011 evidence gate passed across replay, recovery, export reproducibility, and stage-style stability; host-audio soak, multi-hour endurance, full arrangement/stem export, deeper musical quality, and Source Timing Intelligence remain explicitly deferred to later projects

Current bounded gates:

- `just stage-style-jam-probe` exercises a longer generated W-30 source-diff output plus generated app-level multi-boundary observer/audio correlation evidence.
- `just p011-exit-evidence-manifest` validates a machine-checkable P011 exit evidence index across replay, recovery, export reproducibility, and stage-style stability categories while keeping known open boundaries explicit, including repo-local `just` recipe references for proof commands.
- `just p011-exit-evidence-gate` executes every bounded P011 evidence category from the manifest with global command deduplication; this is the CI entrypoint for category-level P011 exit evidence, not a claim of host-audio soak, full arrangement export readiness, or endurance coverage.
- `just p011-replay-evidence-gate` executes the bounded replay category from the P011 exit evidence index after validating the manifest, making replay evidence the first category promoted beyond index-only validation without claiming full all-category exit readiness.
- `just p011-recovery-evidence-gate` executes the bounded recovery category from the same P011 evidence index, covering generated recovery observer drills and recovery-surface tests while keeping automatic startup recovery and real interrupted-host-session rehearsal out of scope.
- `just p011-export-evidence-gate` executes the bounded export reproducibility category from the same P011 evidence index, covering the current deterministic Feral grid product-export seam without claiming full arrangement export readiness.
- `just p011-stage-style-evidence-gate` executes the bounded stage-style stability category from the same P011 evidence index, covering the generated repeated-run restore-diversity proof without claiming host-audio soak or multi-hour endurance coverage.
- `just stage-style-restore-diversity-probe` adds a richer replay-safe W-30 / TR-909 / MC-202 control path correlated with generated full-grid output evidence.
- `just stage-style-snapshot-convergence-smoke` proves a supported Scene / MC-202 / TR-909 stage-style suffix converges from a mid-run snapshot payload to the same final mix buffer as the committed path.
- `just stage-style-stability-smoke` / `just stage-style-stability-proof` repeats the richer generated run, requires stable full-mix hashes plus non-collapsed observer/audio summaries across repetitions, and validates a normalized proof manifest for run count, observer/audio evidence, commit-boundary coverage, and stable output hash.
- `just stage-style-stability-gate` is the stronger bounded variant with more repetitions and a longer generated source/grid budget.
- `just interrupted-session-recovery-probe` and `just missing-target-recovery-probe` cover generated file-backed recovery observer drills.
- `just offline-render-reproducibility-smoke` proves an existing deterministic source-backed W-30 render helper emits byte-stable WAV output for the same generated source.
- `just full-grid-export-reproducibility-smoke` / `just product-export-reproducibility-smoke` proves the deterministic Feral grid source-first plus generated-support pack validates and exports the same generated-support WAV hash twice from generated source material, then validates a normalized product-export proof that removes temp paths and compares stable manifest data plus audio artifact hashes.

Current boundary:

- These are bounded CI-safe probes, not host-audio soak tests, automatic recovery, full arrangement export, stem package export, live recording export, or DAW-style export coverage.
- The passing 2026-05-10 aggregate evidence gate is the P011 bounded-exit baseline. Keep it as a regression gate while primary implementation moves to P012.

---

## 12. P013 - All-Lane Musical Depth

Boundary:

- `P013 | All-Lane Musical Depth` is the bounded representative showcase
  musical-depth phase on top of the P012 timing foundation.
- It closes only when TR-909, W-30, MC-202, and generated-support mix behavior
  have concrete output proof without weakening P012 source-grid timing
  boundaries.
- It is not a finished arranger, full source-derived MC-202 phrase planner,
  final W-30 loop detector, or product taste oracle.

Done when:

- the P012 all-lane source-grid output proof still passes
- representative showcase musical-quality validation passes with at least one
  musically convincing candidate
- TR-909 support exposes source profile, kick pressure, source-grid alignment,
  groove timing, and source-accent dynamics proof
- W-30 source chop exposes source identity, loop closure, trigger/slice
  variation, source-grid alignment, and source-accent dynamics proof
- MC-202 support exposes audible pressure, phrase/bar variation, source-grid
  alignment, and bounded source-section contour proof while staying labeled as
  primitive support until phrase planning exists
- generated-support mix exposes all-lane mix movement proof so source-first and
  generated-support listening mixes are distinct and all three lanes contribute
- relevant P013 specs, validator fixtures, review notes, and roadmap state are
  updated

Current review:

- `docs/reviews/p013_exit_review_2026-05-29.md`
- status: closed for bounded representative all-lane musical depth on
  2026-05-29 after the P012 all-lane source-grid proof, representative showcase
  musical-quality gate, `just audio-qa-ci`, and `just ci` passed with TR-909,
  W-30, MC-202, and all-lane mix output proof. Full arrangement / scene system
  work moves to P014.

---

## 13. Arrangement / Scene System Done Criteria

The Arrangement / Scene System is done for the bounded P014 exit when:

- scene behavior extends Source Graph, Source Timing, Session scene state,
  Action Lexicon, queue / commit, replay, observer, and output QA instead of
  creating a second arranger
- manual `scene.launch` / `scene.restore` chains prove landed movement through
  Session, graph-aware replay, Jam projection, and non-collapsed lane/mix output
- Source Monitor scene repositioning only uses analyzer-locked or
  user-confirmed Source Timing, while manual-confirm pending, fallback,
  disabled, unavailable, or missing-BPM timing keep transport-position playback
- observer/audio QA exposes landed scene movement, bounded extension state,
  Source Monitor anchor evidence, and non-collapsed output metrics
- the Arrangement Scene contract explicitly keeps automatic scene-chain
  scheduling out of P014 until a later Action Lexicon, Session/replay,
  observer, and output-QA expansion exists

Current review:

- `docs/reviews/p014_exit_candidate_review_2026-05-30.md`
- status: exit candidate pending stacked PR CI / merge on 2026-05-31 after the
  local stacked P014 branches passed `just p014-scene-movement-observer-probe`,
  `just audio-qa-ci`, and `just ci`; formal closure waits for GitHub CI
  inspection and merge of the remaining stacked P014 PRs

---

## 14. Feral Layer Done Criteria

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

## 15. 1.0 / Stage-Ready Product Readiness

Riotbox is close to product-ready when:

- the full user spine works end-to-end
- benchmarks are stable enough to catch regressions
- validation corpus is in routine use
- Ghost watch / assist is dependable
- feral mode behaves as policy, not architecture drift
- repeated live-oriented runs do not expose structural instability

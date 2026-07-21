# Riotbox Phase Definition of Done

Version: 0.2
Status: Active
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
- structural refactors preserve behavior unless the phase explicitly owns a
  behavior change
- Rust module work follows `docs/engineering/module_policy.md`; textual
  `include!` migration must create semantic module ownership, not arbitrary
  line-count shards
- audio-producing work proves both control path and output path, and must not
  introduce musical fallback output on product paths
- scripted, fixture, primitive, fallback, or compatibility output remains
  labeled as non-quality proof until source-backed evidence and listening
  review justify promotion

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
- MC-202 source-derived phrase planning remains a P013+/P023 follow-up tracked
  by RIOTBOX-1035 and `docs/plans/mc202_source_phrase_planning_plan.md`; it is
  not retroactively required for the closed P013 baseline, but future claims
  that MC-202 bass / answer behavior is source-derived must satisfy that plan
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
- status: closed bounded P014 exit on 2026-05-31 after PRs #1019, #1021,
  #1022, #1023, and #1024 merged with GitHub `rust-ci` success; the stack
  passed `just p014-scene-movement-observer-probe`, `just audio-qa-ci`, and
  `just ci` before review, then the next implementation lane moved to P015

---

## 14. P015 - Productization Alpha

Boundary:

- `P015 | Productization Alpha` is the bounded Jam productization phase on top
  of the P012 Source Timing, P013 all-lane musical-depth, and P014
  Arrangement / Scene baselines.
- It closes when the proof-heavy Jam surfaces are understandable enough for a
  musician to read taste/proof, trust or distrust scene movement, and choose a
  safer next move without weakening the underlying proof contracts.
- It is not full product completion, automatic arranging, arbitrary-source
  polish, host-audio soak evidence, a product taste oracle, full DAW/stem export
  readiness, or autonomous Ghost performance.

Done when:

- P012 all-lane source-grid output proof still passes
- P013 representative musical-quality validation still passes
- P014 scene-movement observer/audio proof still passes
- P015 Jam taste/proof recipe proof passes
- Jam perform and inspect surfaces expose compact musician-facing taste/proof
  language while keeping detailed proof inspectable
- first-run next-move guidance and Recipe 16 respect scene/timing trust
  boundaries instead of promoting scene jump under cautious or unknown evidence
- Help overlay and docs explain the taste/proof path without hiding primary
  gesture guidance
- explicit P015 deferrals are recorded so later phases do not mistake bounded
  productization for full product completion

Current review:

- `docs/reviews/p015_exit_evidence_checklist_2026-05-31.md`
- `docs/reviews/p015_exit_review_2026-05-31.md`
- status: closed bounded Productization Alpha exit on 2026-05-31 after
  RIOTBOX-1037 and RIOTBOX-1050 through RIOTBOX-1058 landed Jam taste/proof,
  Recipe 16, first-run next-move, Help overlay, glossary, checklist, and exit
  evidence surfaces; the stack passed `just p015-jam-taste-recipe-proof`,
  `just p014-scene-movement-observer-probe`,
  `just p012-all-lane-source-grid-output-proof`,
  `just representative-source-showcase-musical-quality` after regenerating a
  stale local representative showcase, and `just ci` before review. P015 stays
  a closed regression baseline while P023 is the active implementation phase.

---

## 15. P023 - Usable Musical Alpha

Boundary:

- `P023 | Sound Excellence / Production Quality` remains the single active
  product priority. Its first bounded exit, `RIOTBOX-1396`, closed on
  2026-07-21; the next work expands from that accepted baseline.
- The first exit supports one trusted dense-break Golden Path through the exact
  live `riotbox-app` runtime and mixer. Offline binaries, scripted packs,
  fixtures, reports, and validators are diagnostic evidence only.
- P016, P021, and P022 remain subordinate unless a ticket names the exact
  post-exit P023 outcome it enables or blocker it removes.
- Broad positive source-family expansion and negative-source coverage did not
  block this first exit; they now proceed from the accepted dense-break
  baseline in the documented order.

Done when:

- one real dense-break source reaches a strong hook through the exact live
  product path within five minutes from a fresh session
- timing is trusted or explicitly confirmed and the same committed timing
  identity drives W-30, TR-909, MC-202, transport, Session, and replay
- W-30 plays recognizable transformed source audio, TR-909 supplies physical
  drum pressure, and MC-202 takes a source-related bass / answer / stay-out role
- one element clearly hits hardest, the hook appears within two bars, and at
  least four live gestures create immediate audible contrast including one
  destructive stop / drop / retrigger
- no silent collapse, clipping, source-fake output, or synthetic replacement
  fallback reaches the product output path
- capture, audition, promote / save, restart, recall, trigger, and deterministic
  replay preserve the accepted musical result
- README and one short Jam recipe state what to load, press, see, and hear
  without requiring Log or source-code knowledge
- a real structured listening review records strongest element, source
  recognition, hook, main failure / preferred direction, eight-bar replay
  value, and final `human_verdict: pass`

Current landed evidence:

- RIOTBOX-1333 provides the first human-passed exact-path W-30 element: trusted
  dense-break ingest, timing-aligned one-bar capture, committed promotion and
  trigger / damage actions, duration-aware callback-safe artifact playback,
  transient-derived chop retriggers, exact live-mixer offline simulation, and
  deterministic artifact replay. The separately auditioned normal hook and
  destructive variation both received `human_verdict: pass` on 2026-07-12.
- This closes the W-30 sampler slice only. It does not by itself close the P023
  Golden Path, which still requires the all-lane instrument, gesture, recipe,
  restart / recall, and aggregate structured-review exit evidence above.
- RIOTBOX-1400 establishes typed shared live-performance projection and explicit
  bass ownership across the exact W-30 / TR-909 / MC-202 render path, with the
  reproducible `just dense-break-live-path-smoke` gate. Human
  review accepted the stronger dense hook/instigator direction but rejected the
  Beat08 MC-202 bass candidate as boring and non-loopable. That source now
  resolves to `bass_owner=unassigned`; this is honest degraded role selection,
  not a P023 bass-quality pass. A true MC-202 bass-pressure pass still requires
  a trusted source with sufficient low-band movement evidence.
- RIOTBOX-1400 is therefore a `contract_enabler`, not an audible Golden Path
  completion. Its Beat03 all-lane v2 was stronger and punchier than v1 but
  remained human-rejected as less useful than the source: the render was quiet,
  lost substantial high-band energy, repeated its short unit nearly unchanged,
  and did not establish recognizable Riotbox character. It directly enables
  RIOTBOX-1401 to implement and human-review a source-derived eight-bar arc with
  hook establishment, pressure lift, destructive contrast, and changed return.
- RIOTBOX-1401 lands the curated `Feral Break Alpha` product preset and the
  documented five-minute Recipe 17 path through capture, raw audition,
  promotion, save, intentional quit, restart, live recall, trigger, and
  deterministic callback-path replay. The recipe also names the exact screen
  cues and expected eight-bar development without requiring Log or source-code
  knowledge.
- RIOTBOX-1402 corrects the V2 product/QA path to `Riotbox` monitoring and the
  documented `w -> s -> f -> y -> Y+D` order, then proves a real TUI and
  audio-callback take as `8 -> 8 -> 4 -> 4 -> 7.90757` beats before its explicit
  stop. The isolated 14.787-second artifact has SHA-256
  `327f9d4d00bd18c294bcf26f86c8b8a3b23f8e4f85474572735139d627d5ce61`,
  measures `-18.6 LUFS` and `-0.3 dBTP`, and does not clip. After exact-artifact
  preflight and fresh readiness confirmation, Markus gave it a direct `pass`:
  the elements are usable, though he would loop them differently.
- The structured classification keeps the break/pause as the strongest
  musical element (`silence`) from the earlier explicit same-recipe review,
  the source-backed chop as a clear within-two-bars hook, and the TR-909 as the
  hardest active drum/transient layer. Source character is transformed but
  present. Beat03 truthfully assigns bass owner `unassigned`; the exit therefore
  does not claim an MC-202 bass-pressure pass.

Status: first bounded P023 Usable Musical Alpha exit closed on 2026-07-21.
This is one human-passed dense-break Golden Path, not broad source-family,
finished arrangement, export, or release readiness.

Not sufficient for exit:

- a control-path, queue, log, or observer assertion without live output proof
- a nearest offline render seam without the exact live product path
- scripted or fixture-backed `pass` data
- another unverified review pack, readiness report, threshold, or validator
  when a review-ready candidate is already waiting for human listening

After exit:

- tonal and sparse positive sources may expand the human-passed live policy
- `weak_source` and `bad_timing` pass their family contract through reviewed
  correct degraded / unavailable / reject behavior when trusted generation is
  not possible; they do not need demo-ready music
- live readiness must not infer human coverage from fixture/calibration demo
  banks; a real bank is an explicit input and its human verdicts carry
  non-fixture reviewer plus hashed structured-review provenance
- P016 and the wider P021/P022/P023 quality ladder may resume from the accepted
  live baseline

---

## 16. Feral Layer Done Criteria

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

## 17. 1.0 / Stage-Ready Product Readiness

Riotbox is close to product-ready when:

- the full user spine works end-to-end
- benchmarks are stable enough to catch regressions
- validation corpus is in routine use
- Ghost watch / assist is dependable
- feral mode behaves as policy, not architecture drift
- repeated live-oriented runs do not expose structural instability

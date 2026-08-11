# Riotbox Research and Decision Log

Version: 0.1  
Status: Draft  
Audience: whole project

---

## 1. Purpose

This log exists to prevent repeated discussion, hidden assumptions, and undocumented architecture drift.

Use it for:

- bounded research spikes
- architecture decisions
- provider choices
- benchmark interpretations
- explicit rejections of alternatives

Do **not** use it as a diary.

Entry IDs are required for new durable decisions. Older unnumbered entries remain historical context and may be normalized in a dedicated cleanup, but new accepted decisions should use stable `RBX-###` headings.

---

## 2. Entry Template

```text
ID:
Date:
Topic:
Phase:
Question:
Options considered:
Decision:
Why:
Evidence:
Consequences:
Follow-up:
Status:
```

---

## 3. Initial Entries

### RBX-001

Date: 2026-04-12  
Topic: language and documentation baseline  
Phase: Phase 0  
Question: what should be considered canonical planning documentation?  
Decision: `docs/` holds implementation-facing specs; `plan/` holds strategy and long-form planning; the active feral addendum is `plan/riotbox_liam_howlett_feral_addendum.md`.  
Why: this keeps strategy, archive history, and implementation contracts separated.  
Consequences: future spec work should land in `docs/`, not in new planning sprawl under `plan/`.  
Status: accepted

### RBX-105

Date: 2026-06-30
Topic: Mix-bus impact must rise with source-masking headroom intact
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `mix_bus` weak-output fixes increase musical impact without hiding the transformed source?
Decision: generated-support mix fixes may raise TR-909/MC-202 impact only when source-first masking remains tightly bounded. The P023 professional-output suite now treats source-first generated/source ratio above `0.08` as masking, requires at least `0.04` source-first masking headroom, and raises the useful generated-support floor to `0.145` while preserving the `0.46` support/source ceiling.
Why: weak-output routing showed support could be too buried or source-masking depending on family. A mix-bus fix that only makes generated support louder risks hiding the W-30/source identity; a fix that only lowers support stays polite. The gate must require useful impact and preserved source character together.
Evidence: RIOTBOX-1354 strengthens drop-contour generated-support shaping and mirrors the stricter support/headroom contract through the professional-output generator, validator, audio QA spec, and readiness aggregation.
Consequences: future mix-bus work should report both musician payoff and masking risk. Scripted diagnostic evidence remains `quality_proof: false` and cannot be promoted as musical approval without structured listening.
Status: accepted

---

### RBX-106

Date: 2026-07-01
Topic: Source-character window promotion needs score lift and RMS retention
Phase: P023 Sound Excellence / Production Quality
Question: how should source-selection fixes prove they selected better source material instead of always rendering the beginning of a source or chasing a weak transient peak?
Decision: Feral grid source-character window selection may promote a later window only after scanning the available source, improving the source-character score, and preserving source energy with `rms_retention_ratio >= 0.98`. Professional-suite/readiness gates must require searched cases and at least one promotion, while family-specific diagnostics may keep longer requested windows when a short window would damage tonal-hook or bad-timing review contracts.
Why: prior suite runs reported source-window selection but every case scanned exactly one candidate and promoted nothing. That let source-selection blockers remain reporting artifacts instead of becoming an audible selection contract.
Evidence: RIOTBOX-1355 adds manifest retention/search fields, opens Feral grid search over available source audio, makes dense/sparse/pad diagnostics request shorter windows where selection is useful, keeps tonal-hook and bad-timing context when needed, and validates 8 searched cases / 7 promoted cases with observed retention at or above `1.0`.
Consequences: future source-selection work should preserve explicit search and retention evidence. Scripted diagnostic examples remain `quality_proof: false`; promotion metrics prove the selector is active and energy-preserving, not that the output has passed human musical review.
Status: accepted

---

### RBX-107

Date: 2026-07-01
Topic: Rendered TR-909 drum pressure must not pass as decorative support
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `drum_pressure` fixes prove the drum lane survives the rendered output path without masking the source?
Decision: Professional-output TR-909 rendered-pressure proof must require at least `0.05` support-mix TR-909 contribution for every source support profile. Drop-drive and break-lift profiles keep a `0.0030` low-band RMS floor; break-lift and steady-pulse profiles are strengthened in the kick-pressure renderer, and steady-pulse must reach at least `0.0017` low-band RMS. Existing source-first masking and generated-support ceilings remain active.
Why: weak-output routing still identified `drum_pressure` as a recurring production fix. The previous steady-pulse contribution floor of `0.035` allowed restrained tonal/pad material to pass while the drum lane risked becoming decorative.
Evidence: RIOTBOX-1356 raises break-lift and steady-pulse TR-909 kick body and tightens rendered-pressure floors. The professional-suite/readiness aggregate now reports minimum support contribution `0.057182`, minimum TR-909 low-band RMS `0.001890`, maximum source-first generated/source ratio `0.027283`, and maximum support/source ratio `0.376061`; the syncopated-source showcase also stays above the break-lift `0.0030` low-band floor.
Consequences: future drum-pressure changes should improve physical drum impact at the rendered seam and report masking headroom at the same time. These metrics remain diagnostic and do not replace human listening approval.
Status: accepted

---

### RBX-108

Date: 2026-07-01
Topic: Destructive gestures need dense-specific cut/restore impact without punishing sparse pressure
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `destructive_gesture` weak-output fixes strengthen stage-meaningful cuts while keeping sparse-bass-pressure diagnostics honest?
Decision: dense-break destructive diagnostics must use a tighter source-transient stutter and harder source/drum restore attack, and the destructive validator must require `dropout_to_stutter_rms_ratio <= 0.0065`, `dropout_silence_to_stutter_rms_ratio <= 0.0065`, `stutter_to_hook_transient_ratio >= 1.55`, `restore_to_hook_transient_ratio >= 1.60`, and `restore_to_pressure_rms_ratio >= 1.36`. The dense generator reports an active restore/pressure floor of `1.18`, while sparse-bass-pressure keeps the generic `1.12` floor because its pressure section is intentionally large.
Why: weak-output routing still identified destructive gesture weakness, but a single global restore/pressure floor punished sparse cases for doing the right thing: carrying heavy pressure before the cut. The product needs dense cuts and restores to hit harder without mislabeling strong sparse pressure as a failed destructive gesture.
Evidence: RIOTBOX-1357 updates the dense-break performance generator, destructive-variation validator, professional-output suite aggregator, suite contract, audio QA spec, and roadmap. The professional suite passes with destructive metrics `dropout_to_stutter_rms_ratio=0.004930`, `dropout_silence_to_stutter_rms_ratio=0.005194`, `stutter_to_hook_transient_ratio=1.574247`, `restore_to_hook_transient_ratio=1.654569`, `restore_to_pressure_rms_ratio=1.366160`, and `restore_to_dropout_silence_rms_ratio=261.537586`.
Consequences: future destructive-gesture work should keep dense impact and sparse pressure contracts separate, surface the active family floor in reports, and avoid treating scripted diagnostic renders as musical approval. `quality_proof` remains false until structured listening review accepts the result.
Status: accepted

---

### RBX-110

Date: 2026-07-01
Topic: Sparse bass movement must clear a stronger source-derived contour span
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `bass_movement` fixes make sparse bass pressure more musician-useful without adding fallback bass output?
Decision: sparse-bass-pressure diagnostics must require `sparse_bass_movement_frequency_span_hz >= 17.0` across the generator, professional source-WAV pack, professional-output suite, matrix smoke, MC-202 closeout, producer-fix routing, source-composed review gate, and sound-readiness report. The generator uses the same contract when expanding source-feature-derived sparse bass contours, so this is a rendered-output change, not only a stricter validator.
Why: after RIOTBOX-1358, readiness still routed `bass_movement` as a top weak-output category. The old `15.0 Hz` sparse contour floor let real sparse cases pass with little headroom, which can read as midrange phrase support instead of room-carrying low-end pressure.
Evidence: RIOTBOX-1359 raises the sparse contour span contract and passes `just professional-source-wav-pack-smoke` plus `just pro-pressure-source-matrix-smoke`. The sparse source-WAV case reports `sparse_bass_movement_frequency_span_hz=18.6`, `sparse_pressure_low_band_share=0.425415`, and `strongest_audible_element=bass`; matrix sparse cases report `18.6` span with bass dominance margins between about `0.484` and `0.533`.
Consequences: future bass-movement work should improve source-derived contour and pressure projection before raising simple gain. These diagnostics remain scripted, `quality_proof: false`, and `human_verdict: unverified`.
Status: accepted

---

### RBX-111

Date: 2026-07-01
Topic: W-30 hook/chop diversity must increase without masking drum pressure
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `chop_policy` fixes prove the first two bars are a source-derived riff instead of a repeated chop?
Decision: hook-forward generated diagnostics must require at least six source offsets, ten riff hits, and `hook_chop_riff_velocity_span >= 0.25`. The dense-break generator targets six source-derived riff starts and up to twelve source-derived hits, but trims dense W-30 riff-layer gain so richer chop density does not mask snare/break pressure.
Why: after RIOTBOX-1359, weak-output routing still named `chop_policy` as the top category and specifically called out too few W-30 trigger patterns, too few unique source offsets, flat accent dynamics, weak dominance, and missing response signature. Simply making W-30 louder would hide drum pressure; the fix needs more source-backed riff movement while preserving the physical snare/break hit.
Evidence: RIOTBOX-1360 raises the generator and professional Source-WAV / Matrix / Suite gates. Current suite evidence reports dense/matrix `hook_chop_riff_unique_source_offset_count=6`, `hook_chop_riff_hit_count=12`, `hook_chop_riff_velocity_span=0.576`, `hook_chop_w30_to_source_margin=0.126`, and `dense_break_snare_pressure_margin=0.2245`; tonal source-WAV reports `6` offsets, `12` hits, velocity span `0.536`, and W-30/source margin `0.158`.
Consequences: future chop-policy work should improve source-derived offset selection, hit placement, accent dynamics, and response signature before adding generic gain. These diagnostics remain scripted, `quality_proof: false`, and `human_verdict: unverified`; structured listening is still required before claiming musical approval.
Status: accepted

---

### RBX-112

Date: 2026-07-01
Topic: Sparse bass pressure must prove low-band projection, not only movement
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `bass_movement` fixes prove sparse pressure reads as physical low end rather than a moving midrange phrase?
Decision: sparse-bass-pressure diagnostics must keep the `17.0 Hz` source-derived movement span floor and also require `pressure_low_band_lift_ratio >= 2.70`, `sparse_pressure_low_band_share >= 0.36`, `sparse_pressure_low_to_mid_ratio >= 2.45`, and `strongest_audible_element_margin >= 0.20` across matrix, professional source-WAV, professional-output suite, and sound-readiness reporting. The sparse render path carries more sub pressure and less harmonic midrange support so this is a rendered-output change, not a validator-only gate.
Why: after RIOTBOX-1360, weak-output routing still reported `bass_movement`; the previous sparse pressure floors could pass moving bass evidence even when the pressure section risked reading as a melodic or midrange answer phrase instead of room-carrying bass pressure.
Evidence: RIOTBOX-1361 passes the professional source-WAV, matrix, suite, and readiness smokes with source-WAV sparse pressure lift/share/low-mid/dominance at `2.828` / `0.431` / `2.517` / `0.455`, matrix minimums at `3.076` / `0.377` / `2.513` / `0.493`, and readiness aggregate source-WAV/matrix lift at `2.765` / `3.002`.
Consequences: future bass-pressure work should improve source-derived contour and low-band projection together, and must not hide weak musicality behind generic gain. This remains scripted diagnostic evidence with `quality_proof: false` and `human_verdict: unverified` until structured listening accepts it.
Status: accepted

---

### RBX-113

Date: 2026-07-01
Topic: W-30 hook/chop response must prove transformed signature, not only diversity
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `chop_policy` fixes prove that stronger W-30 hook/chop output is a transformed response rather than a louder or more varied source copy?
Decision: dense-break and tonal-hook professional diagnostics must expose and gate `hook_chop_response_delta_ratio`, `hook_chop_response_correlation`, and `hook_chop_response_transient_ratio`. Current gates require response delta `>= 0.35`, source-copy correlation `<= 0.92`, and response transient ratio `>= 0.58` across dense, matrix, tonal source-WAV, professional-suite, readiness, and matrix smoke surfaces. The rendered W-30 riff layer may add source-derived shifted/reversed transient response material, but dense-break must trim the hook layer enough that snare/break pressure remains the strongest element.
Why: after RIOTBOX-1360, `chop_policy` still routed as a top production fix. Offset count, hit count, and velocity spread prove movement, but they do not by themselves reject a source copy or a hookless response. The musician-facing target is a hook/riff response worth retriggering without burying the physical break.
Evidence: RIOTBOX-1362 passes the professional source-WAV, matrix, suite, and readiness smokes with dense/matrix response delta/correlation/transient `1.038` / `0.619` / `3.632`, tonal source-WAV response `1.085` / `0.496` / `5.529`, and dense snare margin `0.2234` above the `0.22` floor.
Consequences: future chop-policy work should improve source-derived transformed response signature and listening verdicts before adding generic W-30 gain. These diagnostics remain scripted, `quality_proof: false`, and `human_verdict: unverified`; structured listening is still required before claiming musical approval.
Status: accepted

---

### RBX-114

Date: 2026-07-01
Topic: Source-selection edge material needs actionable demotion evidence
Phase: P023 / Sound Excellence / Production Quality
Question: is it enough for P023 diagnostics to mark bad-timing and pad/noise edge sources as blocked from promotion?
Decision: no. Edge-source promotion summaries must also carry demotion reasons, reason counts, and concrete review actions. Bad-timing material must point to timing confirmation before bar-locked moves; pad/noise material must point to texture audition before demo promotion; all diagnostic edge cases stay `quality_proof: false` and `human_verdict: unverified`.
Why: a plain blocked flag protects against false quality claims, but it does not tell the musician or implementer what must happen next. Source-selection risk should be visible as actionable product state, not a vague backlog bucket or a hidden fallback path.
Evidence: RIOTBOX-1363 adds source-selection demotion fields to edge-source diagnostics, lifts them through the professional-output suite and sound-quality readiness report, and validates the required reasons/actions in the suite and readiness gates.
Consequences: future source-selection fixes must preserve the no-promotion boundary for risky edge sources until source evidence, timing confidence, human verdict, and audible result are good enough. Demotion evidence is a routing/protection contract, not a musical approval gate.
Status: accepted

---

### RBX-115

Date: 2026-07-01
Topic: Weak-output priority must be reconciled with current professional evidence
Phase: P023 / Sound Excellence / Production Quality
Question: should stale/static weak-output fixtures keep driving the top P023 production category when current generated diagnostics already pass the related professional-suite gates?
Decision: no. Weak-output fixtures stay valuable as negative controls, but P023 readiness must reconcile their priority against current professional-output evidence. If `chop_policy` remains top only from hookless/static fixtures while dense, matrix, and tonal W-30 response gates pass, readiness marks it as stale fixture-only risk and surfaces the next current product category.
Why: otherwise the implementation loop can keep chasing old fixture failures after the actual W-30 response path has already improved, hiding the next audible product gap from musicians and implementers.
Evidence: RIOTBOX-1364 adds `current_evidence_reconciliation` to the sound-quality readiness report, validates the `chop_policy` stale-control state against current W-30 response metrics, and surfaces the current product top category in JSON/Markdown.
Consequences: negative fixtures must not be deleted or treated as pass, but they also must not outrank current professional-suite evidence without an explicit remaining-current-risk reason. Reconciliation remains diagnostic and does not claim musical approval or quality proof.
Status: accepted

---

### RBX-116

Date: 2026-07-01
Topic: Bass weak-output priority must be reconciled with current pressure evidence
Phase: P023 / Sound Excellence / Production Quality
Question: should old sparse-bass weak fixtures keep driving `bass_movement` priority when current professional-suite sparse pressure gates pass?
Decision: no. As with W-30 hook/chop, sparse-bass weak fixtures remain negative controls, but P023 readiness must reconcile them against current matrix and source-WAV sparse movement, low-band lift/share, low-to-mid ratio, and bass-dominance evidence. When those gates pass, `bass_movement` is marked stale fixture-only and the current product top category advances.
Why: stale weak-bass artifacts are useful regression examples, but they should not keep the implementation loop on a bass-pressure slice after current diagnostics already prove physical sparse low-end projection.
Evidence: RIOTBOX-1365 extends `current_evidence_reconciliation` to `bass_movement`, validates the stale-control state against current sparse pressure metrics, and surfaces the next current product top category in readiness JSON/Markdown.
Consequences: future P023 prioritization should distinguish negative controls from current product gaps for each recurring weak-output category. Reconciliation remains diagnostic and cannot claim quality proof or human musical approval.
Status: accepted

---

### RBX-117

Date: 2026-07-01
Topic: Destructive weak-output priority must be reconciled with current gesture evidence
Phase: P023 / Sound Excellence / Production Quality
Question: should old flat-stutter weak fixtures keep driving `destructive_gesture` priority when current professional-suite cut/stutter/restore gates pass?
Decision: no. As with W-30 hook/chop and sparse bass, destructive weak fixtures remain negative controls, but P023 readiness must reconcile them against current dropout-to-stutter silence, stutter-to-hook transient, restore-to-hook transient, restore-to-pressure, and restore-to-dropout-silence evidence. When those gates pass, `destructive_gesture` is marked stale fixture-only and the current product top category advances.
Why: stale flat-stutter artifacts are useful regression examples, but they should not keep the implementation loop on destructive gestures after current diagnostics already prove stage-meaningful cut/stutter/restore contrast.
Evidence: RIOTBOX-1366 extends `current_evidence_reconciliation` to `destructive_gesture`, validates the stale-control state against current destructive gesture metrics, and surfaces destructive gesture context in readiness JSON/Markdown.
Consequences: future P023 prioritization should continue separating negative controls from current product gaps across recurring weak-output categories. Reconciliation remains diagnostic and cannot claim quality proof or human musical approval.
Status: accepted

---

### RBX-118

Date: 2026-07-01
Topic: Mix-bus weak-output priority must be reconciled with current support evidence
Phase: P023 / Sound Excellence / Production Quality
Question: should old source-masked or support-buried weak fixtures keep driving `mix_bus` priority when current professional-suite mix-balance gates pass?
Decision: no. As with W-30 hook/chop, sparse bass, and destructive gestures, mix-bus weak fixtures remain negative controls, but P023 readiness must reconcile them against current generated-support/source RMS balance, source-first masking ceiling, source-first masking headroom, and support/source ceiling. When those gates pass, `mix_bus` is marked stale fixture-only and the current product top category advances.
Why: stale source-masking artifacts are useful regression examples, but they should not keep the implementation loop on mix-bus treatment after current diagnostics already prove useful generated support without burying source character.
Evidence: RIOTBOX-1367 extends `current_evidence_reconciliation` to `mix_bus`, validates the stale-control state against current mix-balance metrics, and surfaces mix headroom context in readiness JSON/Markdown.
Consequences: future P023 prioritization should continue separating negative controls from current product gaps across recurring weak-output categories. Reconciliation remains diagnostic and cannot claim quality proof or human musical approval.
Status: accepted

---

### RBX-119

Date: 2026-07-01
Topic: Source-selection priority must be actionable after stale controls reconcile
Phase: P023 / Sound Excellence / Production Quality
Question: what should readiness expose when `source_selection` becomes the current product priority after stale weak-output controls have been reconciled?
Decision: expose a concrete `source_selection_priority` detail in the P023 readiness report. It must include triggering case ids, primary case ids, source families, artifact refs, demotion reasons, required review actions, a non-generic software next step, and a musician-facing unavailable/degraded action. The detail remains diagnostic and must preserve `quality_proof: false` and `automated_musical_approval: false`.
Why: once old chop, bass, destructive, and mix controls are correctly demoted to stale regression controls, a generic “review source-window and source-character policy” bucket is too vague to drive implementation or musician trust. The next slice needs exact evidence and actions without promoting risky edge sources.
Evidence: RIOTBOX-1368 adds source-selection priority detail to sound-quality readiness, validates that it is non-generic and evidence-backed when `source_selection` is current top, and surfaces the priority in Markdown.
Consequences: future source-selection work should use this detail to decide whether to improve source-window/character choice, timing confirmation, texture review, or human verdict import. Edge sources remain diagnostic/unavailable for promotion until their explicit review actions resolve.
Status: accepted

---

### RBX-120

Date: 2026-07-02
Topic: Source-selection priority must become a family-specific source-window policy
Phase: P023 / Sound Excellence / Production Quality
Question: how should Riotbox implement the current `source_selection` priority without promoting edge sources or chasing the wrong transient peak?
Decision: apply `source_selection_policy` to the product dense source-window path and keep it separate from edge-source demotion. Dense-break selection may rescue the requested head to a stronger source-character/transient window only when high-band/stab tilt stays bounded; sparse-bass-pressure selection prioritizes low-band share and low/mid weight over generic transient score; tonal-hook full-window material may keep a one-candidate verified window when no extra searchable source duration exists. Candidate-count floors apply only when the search window is larger than the selected window.
Why: a generic source-character score picked a transient-rich dense window that weakened snare/stab separation and a sparse-bass window that read too much like midrange. Source selection must improve the musician-facing result for the source family, not just raise a single metric.
Evidence: RIOTBOX-1369 updates the dense performance generator, professional-output suite, suite contract, destructive-variation validator, sound-quality readiness report, audio-QA spec, and roadmap. `just dense-break-performance-pack-smoke`, `just professional-output-suite-smoke`, and `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1369-readiness` pass. The professional suite reports dense source selection `selected_start_seconds=1.5`, `candidate_count=7`, `score_lift=1.091212`, and `rms_retention_ratio=0.985980`; destructive review now also accepts source-referenced transient proof when a source-selected hook raises the hook baseline.
Consequences: future source-selection work must preserve family-specific scoring and keep edge-source `promotion_allowed: false` until timing, texture, and human verdict are trusted. `source_selection_policy` remains diagnostic with `quality_proof: false` and `automated_musical_approval: false`; it proves policy execution and output-path gates, not human musical approval.
Status: accepted

---

### RBX-121

Date: 2026-07-02
Topic: P023 readiness must reconcile destructive priority with source-referenced proof
Phase: P023 Sound Excellence / Production Quality
Question: should sound-quality readiness demote stale destructive-gesture weak fixtures when the current professional suite passes destructive gates via source-referenced transient evidence rather than hook-referenced ratios?
Decision: yes. Destructive readiness now uses the same transient contract as the professional suite: stutter and restore pass through hook-referenced ratios or source-referenced ratios when selected source hooks raise the hook baseline.
Why: otherwise stale flat-stutter controls can remain the current top product gap even after the current dense product output proves dropout/stutter silence, restore pressure, and source-referenced transient impact.
Evidence: RIOTBOX-1370 carries source-referenced destructive ratios into the readiness summary, updates destructive reconciliation, and adds a mutation smoke that fails validation when stale destructive demotion lacks current source-referenced proof.
Consequences: destructive weak fixtures remain regression controls, not release quality proof. Readiness can advance to the next current product gap only when the report contains current destructive evidence that matches the professional-suite contract.
Status: accepted

---

### RBX-122

Date: 2026-07-02
Topic: Source-selection priority must expose policy family coverage
Phase: P023 Sound Excellence / Production Quality
Question: may P023 readiness treat dense-only source-window policy evidence as covering a `source_selection` priority whose weak-output candidate also names tonal or other source families?
Decision: no. When `source_selection` is the current product priority, readiness must compare candidate source families with policy-covered source-selection families and surface any uncovered families directly in the priority detail.
Why: a strong dense-break source-window policy is useful, but it does not prove tonal-hook or other family source selection is handled. Without explicit coverage, the dashboard can make the next source-selection work look more complete than it is.
Evidence: RIOTBOX-1371 adds source-selection policy family lists to the professional-output suite/readiness path, identifies uncovered candidate families such as `tonal_hook`, and adds a mutation smoke for missing uncovered-family evidence.
Consequences: source-selection family coverage remains diagnostic with `quality_proof: false` and `automated_musical_approval: false`. Covered families show where policy evidence exists; uncovered families drive the next implementation slice.
Status: accepted

---

### RBX-123

Date: 2026-07-02
Topic: Source-selection priority is resolved only after candidate-family policy coverage
Phase: P023 Sound Excellence / Production Quality
Question: when may P023 readiness stop treating `source_selection` as the current product gap?
Decision: readiness may demote `source_selection` only when the professional-output source-selection policy covers all source families named by the current source-selection candidate. Coverage aggregates dense-break product-path policy plus professional source-WAV tonal-hook and sparse-bass-pressure policy cases.
Why: RIOTBOX-1371 exposed the family gap, but the suite already had non-dense policy evidence that was not aggregated. Keeping source-selection current after candidate-family coverage is present would keep engineers on stale controls instead of the next audible gap.
Evidence: RIOTBOX-1372 adds non-dense source-selection policy cases to the professional-output summary, treats one-candidate tonal full-window policy as valid when no expanded search exists, reconciles source-selection as stale once candidate families are covered, and adds a mutation smoke for missing tonal coverage.
Consequences: source-selection policy coverage remains diagnostic and `quality_proof: false`; it resolves prioritization, not human musical approval. The current P023 readiness gap advances to `drum_pressure`.
Status: accepted

---

### RBX-124

Date: 2026-07-02
Topic: Drum-pressure priority must reconcile against current rendered TR-909 proof
Phase: P023 Sound Excellence / Production Quality
Question: should stale weak drum-pressure fixtures stay the current product gap when current dense snare pressure and rendered TR-909 drum-pressure gates pass?
Decision: no. Weak drum-pressure fixtures remain negative controls, but readiness may demote `drum_pressure` only when the current professional suite proves dense snare/break pressure plus rendered TR-909 support contribution, low-band body, and bounded generated/source masking.
Why: old weak drum artifacts are useful regression examples, but they should not keep the implementation loop on drum pressure after current output-path evidence already proves the drum lane lands with enough physical support and without burying the source.
Evidence: RIOTBOX-1373 adds drum-pressure current-evidence reconciliation to the sound-quality readiness report and a mutation smoke that fails validation when stale drum-pressure demotion lacks current rendered TR-909 support proof.
Consequences: drum-pressure reconciliation remains diagnostic and `quality_proof: false`; it advances prioritization only when the report carries current dense and rendered drum-pressure evidence that matches the professional-suite contract.
Status: accepted

---

### RBX-125

Date: 2026-07-02
Topic: Current UI-cue priority must be concrete and musician-facing
Phase: P023 Sound Excellence / Production Quality
Question: what should readiness expose when stale sound-output categories are reconciled and `ui_cue` becomes the current product priority?
Decision: expose a dedicated `ui_cue_priority` detail. It must include the routed candidate, case ids, source families, artifact refs, cue surface, cue reasons, software next step, musician action, and required player cues. The cue surface is timing/source risk before confident bar-locked or live-trigger moves, and it must use unavailable/degraded language rather than a generic weak-output bucket.
Why: once current sound-output evidence demotes old chop, bass, drum, destructive, mix, and source-selection controls, the next risk is musician trust. A vague `ui_cue` category does not tell the team what the player should see before acting on risky timing or source material.
Evidence: RIOTBOX-1374 adds `ui_cue_priority` to the sound-quality readiness report, validates it when `ui_cue` is current top, emits it in Markdown, and adds a mutation smoke for missing current UI-cue detail.
Consequences: UI-cue priority remains diagnostic with `quality_proof: false` and `automated_musical_approval: false`; it tells implementation what cue to build next, not that the sound is release-ready.
Status: accepted

---

### RBX-126

Date: 2026-07-02
Topic: Jam perform-risk must name degraded or unavailable bar/live trust
Phase: P023 Sound Excellence / Production Quality
Question: how should the current UI-cue priority become visible on the instrument surface without adding a new action system?
Decision: update the existing Jam Trust perform-risk line. Degraded timing keeps the `degraded` state label and adds the compact `bar/live?` trust cue; unavailable timing keeps the `unavailable` state label and the same bar/live cue. Trusted or user-confirmed timing remains playable/trusted.
Why: `confirm grid` alone is a useful action, but it does not say why the musician should avoid confident bar-locked or live-trigger moves. The product needs the Trust line to carry unavailable/degraded meaning before risky source/timing material is promoted, without wrapping and hiding the adjacent timing/actionability line.
Evidence: RIOTBOX-1375 updates `source_timing_perform_risk_line`, adjusts Jam snapshot tests for degraded and unavailable timing, and documents the TUI language.
Consequences: this remains a visible trust cue on the existing Jam surface, not a second timing state or action path. Future UI-cue work should keep using the shared source-timing summary and observer evidence.
Status: accepted

---

### RBX-127

Date: 2026-07-02
Topic: Shipped perform-risk cue must be reconciled by app-emitted evidence
Phase: P023 Sound Excellence / Production Quality
Question: how should readiness know that `ui_cue` is no longer the current product gap after the Jam Trust cue ships?
Decision: the app must emit `riotbox.jam_perform_risk_cue_contract.v1` with the Trust cue surface, degraded and unavailable state labels, `bar/live?` actions, required player cues, and explicit non-quality-proof flags. P023 readiness consumes that contract and demotes `ui_cue` to a stale regression control only when the contract passes.
Why: a manually updated plan would let readiness drift away from the instrument. The report should stop asking for the same UI cue only when the product surface proves the musician sees degraded/unavailable bar/live risk.
Evidence: RIOTBOX-1376 adds the app-side contract emitter, centralizes the `bar/live?` cue constant, validates the contract in sound-quality readiness, and adds a mutation smoke that fails if the unavailable cue regresses.
Consequences: `ui_cue` remains a regression guard, not release-quality proof. The current non-stale P023 implementation gap advances to `fixture_threshold`, while release readiness remains blocked by human/demo coverage and review evidence.
Status: accepted

---

### RBX-128

Date: 2026-07-02
Topic: Fixture-threshold routes must not hide current output proof
Phase: P023 Sound Excellence / Production Quality
Question: when should `fixture_threshold` stop being the current P023 implementation gap?
Decision: demote `fixture_threshold` only when it is secondary negative-control routing, has no primary case ids, includes the expected `source_report_not_passed` fixture signal, and current destructive/output evidence already passes. Otherwise it remains current.
Why: the final fixture-threshold route came from an expected-fail negative diagnostic, not a new musician-facing sound failure. Treating that as the top product gap would keep implementation stuck on fixture taxonomy instead of source-family review and demo coverage.
Evidence: RIOTBOX-1377 adds fixture-threshold current-evidence reconciliation, validates the negative-control proof shape, and adds a mutation smoke that fails if a primary fixture-threshold case is incorrectly demoted.
Consequences: weak-output categories can all be stale regression controls. P023 readiness now advances to source-selection and structured human/demo coverage while still blocking release and quality claims.
Status: accepted

---

### RBX-129

Date: 2026-07-02
Topic: Post-reconciliation readiness actions must prioritize source-family review
Phase: P023 Sound Excellence / Production Quality
Question: what should the P023 readiness report recommend after every weak-output category is stale?
Decision: when `current_product_top_candidate_category` is `none`, keep stale weak-output categories visible only as regression controls and make the main Next Actions source-family review plus structured human/demo coverage.
Why: after weak-output reconciliation, listing every stale category as a next action makes the implementation loop look stuck on already-covered fixture work. The musician-facing blocker is review and demo coverage, not another stale weak-output fix.
Evidence: RIOTBOX-1378 filters stale weak-output controls out of main Next Actions when no current weak-output product gap remains and validates that source-family actions remain visible.
Consequences: release readiness and quality claims remain blocked until human/demo evidence exists, but the next engineering/review path is no longer obscured by stale regression controls.
Status: accepted

---

### RBX-130

Date: 2026-07-02
Topic: Source-family readiness actions must point at concrete review candidates
Phase: P023 Sound Excellence / Production Quality
Question: how should P023 readiness guide the next loop once source-family review is the remaining blocker?
Decision: when current weak-output reconciliation leaves no active production-fix bucket, source-family Next Actions must link missing demo-ready families to matching release-demo human-review queue candidates when available.
Why: generic "create or promote a candidate" actions hide the fact that Riotbox already has candidate ids, priorities, blocker reasons, and review prompts. The team and musician need an actionable listening/review task, not another abstract source-selection reminder.
Evidence: RIOTBOX-1379 enriches readiness source-selection actions with candidate id, review priority, demo-worthy reason, not-demo-ready reason, and required verdict state, and validates that candidate context cannot be removed while a matching queue entry exists.
Consequences: readiness remains claim-blocked and `quality_proof: false`, but the next implementation/review loop can move directly to structured listening or candidate fix routing for each source family.
Status: accepted

---

### RBX-131

Date: 2026-07-02
Topic: Readiness source-family review actions must carry artifact refs
Phase: P023 Sound Excellence / Production Quality
Question: how should a reviewer find the concrete WAV and prompt for a source-family readiness action?
Decision: readiness human-review queue summaries and matching source-family Next Actions must preserve the release-demo queue's rendered WAV, metrics, and review-prompt artifact refs as `path` plus SHA-256.
Why: a readiness action that names a candidate but drops the WAV/prompt path still forces the reviewer to reverse-engineer the queue report. The product loop needs direct handoff from release blocker to exact listening artifact.
Evidence: RIOTBOX-1380 carries `rendered_wav`, `metrics`, and `review_prompt` through the readiness queue summary into enriched source-selection actions, and validates that refs cannot disappear while the queue has them.
Consequences: readiness still does not claim quality proof, but it now functions as a concrete review worklist: each blocked source family points at the candidate, reason, verdict state, WAV, metrics, and prompt.
Status: accepted

---

### RBX-132

Date: 2026-07-02
Topic: Readiness Markdown must render source-family review artifact refs
Phase: P023 Sound Excellence / Production Quality
Question: should review artifact refs live only in JSON, or also in the human-readable readiness report?
Decision: the Markdown readiness report must render rendered-WAV, metrics, and review-prompt refs for concrete source-family Next Actions and Human Review Queue entries when those refs exist in JSON.
Why: the Markdown report is the artifact a reviewer will read first. If it names the candidate but hides the WAV and prompt paths, the handoff remains incomplete and the reviewer has to reverse-engineer JSON.
Evidence: RIOTBOX-1381 renders artifact refs through `append_artifact_ref_lines` and adds smoke checks for the bad-timing rendered WAV and review prompt paths in `sound-quality-readiness-report.md`.
Consequences: readiness remains unverified and quality-claim blocked, but the human-readable report now acts as a practical listening worklist.
Status: accepted

---

### RBX-109

Date: 2026-07-01
Topic: Tonal hook/chop replacement must preserve source-character floor
Phase: P023 Sound Excellence / Production Quality
Question: how should routed `chop_policy` fixes improve hook/riff quality when the W-30 level floor already passes?
Decision: dense-break and tonal-hook professional diagnostics must require `hook_chop_source_character_score_floor >= 0.64`. Tonal hook/chop replacement may not replace an already selected source-character floor with a weaker grain merely to widen source-character span.
Why: weak-output routing still names `chop_policy`, and the tonal case was technically passing but barely above the old source-character floor (`0.608`). The product goal is not louder generic support; the first two bars need a source-backed hook/riff with enough character to be worth triggering again.
Evidence: RIOTBOX-1358 changes the tonal replacement guard in the dense-break performance generator, raises the hook-forward floor in the generator, professional source-WAV pack, and professional-output suite contract, and passes `just professional-source-wav-pack-smoke`, `just pro-pressure-source-matrix-smoke`, and `just professional-output-suite-smoke`. The tonal source-WAV case reports `hook_chop_source_character_score_floor=0.644205`, `hook_chop_source_character_score_span=0.101409`, and `hook_to_source_transient_ratio=4.981806`; dense remains above the floor at `0.771178`.
Consequences: future hook/chop work should prefer better source-backed grain selection and transformation over simple gain boosts. These diagnostics remain scripted, `quality_proof: false`, and `human_verdict: unverified`.
Status: accepted

---

### RBX-092

Date: 2026-06-30
Topic: MC-202 professional stems must carry source-expression render-plan evidence
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: how should Riotbox strengthen the current MC-202 professional-output stem without prematurely flipping every legacy `primitive_renderer` contract?
Decision: add a bounded source-expression `Mc202SourcePhraseRenderPlan` to the Feral/professional MC-202 render state and expose `source_expression_render_plan_applied` plus `source_expression_role` in listening-pack selected motif metadata. Keep the existing `pattern_origin: "primitive_renderer"` compatibility field until observer/export contracts are migrated in a dedicated follow-up.
Why: the previous professional stem could pass source-contour and low-body diagnostics while the core MC-202 render state had no source phrase plan, leaving too much of the audible result to reinforcement rather than a rendered motif. A full origin-contract migration would be too broad for the sound slice, but review packs need to show that source-expression role evidence actually reaches the stem.
Evidence: RIOTBOX-1341 adds the render-plan application, manifest/listening-pack fields, and gates them through Feral-pack and MC-202 real-source / producer-grade smokes while keeping `human_verdict: unverified` and `quality_proof: false`.
Consequences: future MC-202 producer-grade work should migrate the legacy primitive-origin field only when observer, export, and manifest consumers can all distinguish non-product controls from source-expression product candidates without weakening no-fallback rules.
Status: accepted

---

### RBX-093

Date: 2026-06-30
Topic: Generated-support mix balance follows MC-202 source-expression roles
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: should the Feral/professional generated-support mix use one static gain policy for every source role?
Decision: no. Product generated-support renders now use a bounded source-expression-aware mix policy derived from the same MC-202 source contour that drives the rendered motif role. Drop sources get only a minimal pressure lift to stay under the generated/source ceiling; hold/neutral sources reduce W-30 dominance and make TR-909 / MC-202 support audible enough for producer-grade review.
Why: a static policy let tonal hold sources pass control-path evidence while burying generated support under the source-backed W-30 chop. Raising the global generated bus would break the synthetic support ceiling, so the product render must scale by source role rather than hidden fallback loudness.
Evidence: RIOTBOX-1341 routes the generated-support WAV, all-lane movement proof, and support generated/source ratio through the source-expression policy. The professional-output suite passes with tonal hold support at 0.15897766 and the weakest Drop case at 0.13010876 while source-first generated/source remains below 0.08.
Consequences: future mix-balance changes must keep generated-support WAVs, manifest ratios, and all-lane contribution proof on the same policy path. Static helper policies can remain as regression controls, but product renders should use source-role evidence.
Status: accepted

---

### RBX-094

Date: 2026-06-30
Topic: MC-202 closeout must route weak or unverified candidates into producer fix work
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: what should happen when MC-202 candidates are source-composed and technically reviewable but still lack human verdicts or expose weak margins?
Decision: the MC-202 producer-grade closeout report now emits per-candidate `mc202_producer_fix_route` fields and aggregate `mc202_producer_fix_candidates`. Routes cover `bass_movement`, `answer_bite`, `hook_restraint`, `source_selection`, `mix_bus`, `destructive_articulation`, and `human_listening`, each with exact artifact refs, software next step, musician payoff, and quality-proof boundaries.
Why: a green technical closeout can still leave the product unclear if weak or unverified musical results become prose notes. P023 needs a machine-readable path from review candidates to the next implementation work, without pretending unverified/scripted diagnostics are producer-grade proof.
Evidence: RIOTBOX-1342 extends `scripts/generate_mc202_producer_grade_closeout.py`, its mutation fixtures, and the MC-202 plan. The closeout smoke now rejects missing/stale fix routes, quality-proof claims, invalid categories, and stale summaries while keeping `quality_proof: false` and `automated_musical_approval: false`.
Consequences: future MC-202 review and demo-bank work should consume these fix candidates before creating free-form follow-up tickets. Human listening remains the promotion gate; producer fix candidates are work-selection evidence, not musical approval.
Status: accepted

---

### RBX-095

Date: 2026-06-30
Topic: MC-202 human-verdict promotion consumes producer fix routing by exact artifact
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: how should structured MC-202 human verdicts reuse producer fix candidates without promoting unverified closeout evidence as quality proof?
Decision: demo-bank promotion may consume `mc202_producer_fix_candidates` from the MC-202 producer-grade closeout only after matching the review case and rendered WAV SHA-256. Weak/fail verdicts derive concrete demo-bank `fix_categories` from the matched closeout route, while `human_listening` remains closeout-only and is dropped once a human verdict exists. Human pass verdicts still carry no fix categories.
Why: the closeout report is good at selecting next work, but it is not a musical approval. The promotion path needs the closeout's exact artifact routing to avoid free-form follow-up categories while preserving the rule that only structured listening can decide demo readiness.
Evidence: RIOTBOX-1343 extends `scripts/promote_listening_review_to_demo_bank.py`, `scripts/validate_release_grade_demo_bank.py`, `scripts/listening_review_workflow.py`, and the demo-bank promotion fixtures. The fixtures now prove closeout-derived weak categories, pass-with-routing metadata, manual category mismatch rejection, stale closeout hash rejection, primitive/template-only rejection, and stale artifact hash rejection.
Consequences: future MC-202 demo-bank entries can explain the exact software fix and musician payoff for weak/fail outputs without claiming `quality_proof`. Producers get a concrete category such as `bass_movement`, `answer_bite`, `hook_restraint`, `mix_bus`, or `destructive_articulation`; unverified closeout evidence remains non-promotional.
Status: accepted

---

### RBX-096

Date: 2026-06-30
Topic: Sparse MC-202 bass movement uses a 12 Hz producer floor
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: how should Riotbox respond when sparse MC-202 bass pressure is source-composed but still routed as weak bass movement?
Decision: sparse MC-202 bass movement now targets and validates at least a 12 Hz source-derived frequency span instead of the earlier 10 Hz diagnostic floor. The policy still ranks pressure/restore bars from source low-band energy, timing centroid, and transient evidence, then expands the selected frequencies only when the source-derived span would otherwise be too narrow.
Why: the previous `sparse_kicksnr_120` candidate was technically source-composed and bass-dominant, but the producer-fix router correctly marked it as close to the movement floor. A musician should hear bass pressure push and move, not merely a polite line that barely satisfies old diagnostics.
Evidence: RIOTBOX-1344 updates the dense/professional sparse-bass render policy, MC-202 source-composed gate, pro-pressure source-matrix smoke, producer-grade closeout, professional-output suite validator, sound-readiness validator, and MC-202 plan. The rendered proof now exposes `sparse_bass_movement_span_margin_hz` so the output path shows margin over the producer floor.
Consequences: future sparse-bass MC-202 claims must satisfy the stronger 12 Hz floor across source-WAV, matrix, closeout, and readiness surfaces. If the source-derived policy cannot reach that floor without trusted source evidence, it should remain weak-routed rather than using fallback music.
Status: accepted

---

### RBX-097

Date: 2026-06-30
Topic: Dense MC-202 answer bite is a measured producer floor
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: should dense-break MC-202 candidates route to `answer_bite` just because they are dense break sources?
Decision: no. Dense-break MC-202 output now exposes `dense_answer_bite_*` proof for source-derived answer placement, scripted-role distance, stab score, stab margin, pressure snap, and aggregate bite score. Producer routing emits `answer_bite` only when those metrics are weak; otherwise dense candidates can move on to the next concrete fix category.
Why: a musician needs the MC-202 answer to cut back at the break with a memorable shove or stab, not merely satisfy a generic pressure-answer role. The software also needs to distinguish "answer still too polite/template-like" from "answer is good enough; now fix live gesture articulation or listen".
Evidence: RIOTBOX-1345 changes dense arrangement selection to keep the opening hook stable while placing pressure/answer bars from source-ranked non-scripted evidence, adds Dense Answer Bite proof and mutation coverage, hardens the pro-pressure matrix and professional-output suite contracts, and removes the unconditional dense `answer_bite` producer route.
Consequences: future dense-break MC-202 quality work should route to `answer_bite` only when the measured bite floor fails. Passing automated answer-bite metrics still do not claim human listening approval; `human_verdict` remains `unverified` until structured review is recorded.
Status: accepted

---

### RBX-098

Date: 2026-06-30
Topic: Dense MC-202 destructive articulation routes from a live-gesture floor
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: should dense-break MC-202 candidates keep routing to `destructive_articulation` after the answer-bite floor passes?
Decision: only when the measured dense pressure-lift articulation remains weak. Dense-break MC-202 output now pushes the second source-derived pressure bar clearly above the first and the producer router names that check as a dense destructive-articulation floor instead of a generic pressure-lift comparison.
Why: after RIOTBOX-1345, the dense MC-202 answer was source-derived and passing, but the closeout still routed the case to destructive articulation because the live-gesture lift was just under the producer floor. A musician needs the second pressure/answer hit to feel like a room-changing shove, not merely a technically valid continuation.
Evidence: RIOTBOX-1346 strengthens the dense pressure-lift policy from `0.94 -> 1.110` to `0.92 -> 1.18`, raising `pressure_lift_bar5_to_bar4_rms_ratio` from `1.0885` to `1.1536` while keeping Dense Answer Bite and destructive dropout/stutter gates green. The MC-202 closeout now drops `destructive_articulation` for `dense_beat03_130`; dense routes only to `human_listening`.
Consequences: future dense-break destructive-articulation work should name the producer floor it is testing. Passing automated lift/articulation metrics still do not claim human listening approval; `human_verdict` remains `unverified` until structured review is recorded.
Status: accepted

---

### RBX-102

Date: 2026-06-30
Topic: MC-202 closeout consumes structured listening labels without synthetic approval
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: how can completed listening reviews resolve MC-202 review-queue entries without weakening quality boundaries?
Decision: the MC-202 producer-grade closeout now accepts an optional `riotbox.human_listening_label_corpus.v1` input. A label can resolve a queue entry only when `source_id`, `source_family`, professional review-pack schema, and `audio_sha256.rebuild_only_performance` match the queued candidate. Resolved entries carry the human verdict and reviewer context, but keep `quality_proof: false` and `automated_musical_approval: false`.
Why: the review queue should be executable work, not a permanent blocker. At the same time, the system must not treat a loose or stale label as approval for a different WAV or source family.
Evidence: RIOTBOX-1350 adds `--label-corpus` to the MC-202 closeout, emits `structured_listening_label_corpus` counts, resolves only matching queue entries, leaves unmatched cases blocked, and adds a closeout smoke fixture that imports a dense pass review, resolves only `dense_beat03_130`, and keeps tonal/sparse in the `structured_human_verdict_missing` blocker.
Consequences: future P023 work can record or import real listening verdicts and watch the closeout queue shrink. Weak/fail labels resolve review work but must still block producer-grade promotion; unverified or stale labels do not resolve anything.
Status: accepted

---

### RBX-104

Date: 2026-06-30
Topic: Sparse bass movement must not pass as barely moving low support
Phase: P023 Sound Excellence / Production Quality
Question: how should Riotbox respond when weak-output routing still names `bass_movement` after sparse bass-pressure diagnostics became source-derived?
Decision: sparse-bass-pressure diagnostics now require `sparse_bass_movement_frequency_span_hz >= 15.0` and `sparse_pressure_low_band_share >= 0.32`. The sparse render path widens the source-feature-expanded contour and increases low-band bass pressure so the gate reflects audible output movement, not only a stronger validator.
Why: the previous 12 Hz floor proved the path was not the old fixed contour, but several passing examples still sat close to the movement and low-band-share floors. A musician should hear the sparse section move with physical low-end pressure rather than a thin midrange contour that technically passes.
Evidence: RIOTBOX-1352 updates the dense-break performance generator, professional source-WAV pack, professional-output suite contract, matrix smoke, MC-202 closeout, producer-fix routing, source-composed review gate, sound-readiness reporting, and roadmap/spec docs. The final source matrix keeps sparse cases above the new floor with minimum movement span `15.857 Hz`, low-band share `0.340`, low-band lift `2.991x`, restore/pressure `1.145x`, and bass dominance margin `0.490`. The professional source-WAV sparse case passes with source/performance correlation `0.203`, low-band share `0.462`, movement span `16.142 Hz`, and restore/pressure `1.166x`. The change remains diagnostic with `quality_proof: false` and `human_verdict: unverified`; structured listening review is still required before promotional claims.
Consequences: future routed `bass_movement` fixes should improve source-derived contour selection, low-band rendering, and listening verdicts rather than lowering the sparse bass pressure floors.
Status: accepted

---

### RBX-103

Date: 2026-06-30
Topic: W-30 hook/chop must stay forward enough to answer weak-output routing
Phase: P023 Sound Excellence / Production Quality
Question: how should Riotbox respond when weak-output routing names `chop_policy` as the top production-fix candidate?
Decision: dense/tonal Hook/Chop diagnostics now require `hook_chop_w30_to_source_margin >= 0.10` and the source-derived W-30/riff render path pushes the selected source grains further forward. Tonal output also raises MC-202 support enough to keep the MC-202/W-30 mix-bus floor passing after the stronger W-30 hook lift.
Why: the previous hook/chop path was source-derived and technically valid, but the weak-output router still had good evidence that some first-two-bar results read as generic support or hookless output. Raising only a report threshold would not help a musician; the rendered W-30/riff layer must cut through while preserving source character and MC-202 support.
Evidence: RIOTBOX-1351 raises the dense pro-pressure W-30/source ratio from `0.309` to `0.346`, dense hook/chop margin from `0.089` to `0.126`, tonal hook/chop margin to `0.158`, and keeps tonal MC-202/W-30 support at `0.209`. `just pro-pressure-source-matrix-smoke`, `just professional-source-wav-pack-smoke`, `just professional-output-suite-smoke`, and `just weak-output-fix-routing-fixtures` pass with the stronger contract.
Consequences: these remain scripted diagnostic outputs with `quality_proof: false` and `human_verdict: unverified`. Future W-30 hook/chop work should improve actual source-backed chop selection, transformation, and listening verdicts rather than lowering the hook-forward floor.
Status: accepted

---

### RBX-101

Date: 2026-06-30
Topic: MC-202 human-listening closeout must be a concrete review queue
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: how should the closeout represent remaining `human_listening` work after automated producer routing clears?
Decision: `human_listening` is no longer sufficient as only a fix category. The MC-202 producer-grade closeout now emits a `structured_listening_review_queue` with one entry per unverified human-listening candidate, including exact candidate WAV, review JSON, prompt, metrics, hashes, source family, MC-202 role, route category, and why automated checks stop at human review.
Why: once hook restraint, answer bite, bass movement, destructive articulation, and mix-bus routing clear, the remaining work must be auditable listening work, not a vague blocker. A musician or reviewer should be able to open the closeout and know exactly what to hear and which prompt to answer.
Evidence: RIOTBOX-1349 adds three queue entries for dense, tonal, and sparse MC-202 candidates, validates that the listed WAV/review/prompt/metrics files exist, and mutation-tests stale counts, missing queue entries, quality claims, stale case IDs, and missing prompts. The closeout still keeps `quality_proof: false` and `automated_musical_approval: false`.
Consequences: future closeout work may consume human verdicts from this queue, but must not turn automated reviewability into musical approval. Demo-bank promotion remains blocked until a structured human/calibrated verdict allows it.
Status: accepted

---

### RBX-100

Date: 2026-06-30
Topic: Tonal MC-202 mix-bus balance is a measured support floor
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: when should tonal-hook MC-202 candidates keep routing to `mix_bus` after hook restraint clears?
Decision: tonal-hook `mix_bus` routing remains only when measured MC-202 support is too buried against the W-30 hook. The professional source WAV pack and suite now expose `tonal_mix_bus_mc202_to_w30_rms_ratio`, and the producer router continues to use a 0.20 floor for tonal `mc202_to_w30_rms_ratio`.
Why: tonal material needs the W-30 riff/stab to stay recognizable, but the MC-202 answer must still be audible as a supporting role. A hidden closeout-only ratio made it too easy to remove routing without proving the musician can hear the MC-202 role.
Evidence: RIOTBOX-1348 raises tonal `mc202_to_w30_rms_ratio` for `tonal_rusharp_120` from `0.180` to `0.208`, keeps `w30_to_source_rms_ratio` at `0.334`, keeps the W-30 hook margin at `0.114`, keeps the strongest tonal element as `stab`, and leaves `rebuild_only_source_character_survival_score` at `0.848`. The MC-202 producer closeout now drops `mix_bus` and routes only to `human_listening`.
Consequences: future tonal mix work must not silence the W-30 hook to inflate MC-202 audibility. Passing this floor is still automated producer-routing evidence only; `quality_proof` remains false and `human_verdict` remains `unverified` until structured listening review accepts the candidate.
Status: accepted

---

### RBX-099

Date: 2026-06-30
Topic: Tonal MC-202 hook restraint is a measured pressure-support floor
Phase: P023 Sound Excellence / MC-202 Producer-Grade Track
Question: should tonal-hook MC-202 candidates route to `hook_restraint` just because they are tonal?
Decision: no. Tonal-hook MC-202 output now treats hook restraint as a measured producer floor: the source-derived pressure support must lift low-band pressure enough while the W-30 hook remains forward and source character survives. Producer routing emits `hook_restraint` only when that measured floor fails.
Why: tonal material should keep the recognizable riff in front while the MC-202 adds a sharp support answer underneath. A broad tonal bucket hid whether the answer actually needed hook-restraint work or whether the remaining problem was mix balance.
Evidence: RIOTBOX-1347 strengthens tonal-only pressure support, raises `pressure_low_band_lift_ratio` for `tonal_rusharp_120` from `2.031` to `2.241`, adds `tonal_hook_restraint_pressure_lift_ratio` to the professional-output suite contract, and removes unconditional tonal `hook_restraint` routing. The closeout now leaves tonal routed to `mix_bus`, not `hook_restraint`.
Consequences: future tonal-hook MC-202 work should route to `hook_restraint` only when the measured tonal pressure-support floor fails. Passing this floor still does not claim human listening approval; `human_verdict` remains `unverified` until structured review is recorded.
Status: accepted

---

### RBX-061

Date: 2026-05-31
Topic: P016 export spine broad review after artifact-set and stem-QA slices
Phase: P016 / Pro Workflow / Export
Question: what must be fixed before wider stem, DAW, or live export scopes build on the current product-export spine?
Decision: keep the current bounded product-mix export spine, typed artifact-set contract, and stem QA skeleton, but add follow-ups before widening export scope: recovery preflight must validate artifact-set entries, and `export_qa.rs` tests should be split before another gate expansion pushes the file over the Rust review budget.
Why: product-mix receipts are now compatible with typed artifact evidence, but recovery still validates legacy receipt paths only; the QA gate is cohesive but close to the soft 500-line file budget.
Evidence: `docs/reviews/p016_export_spine_broad_review_2026-05-31.md`, RIOTBOX-1076, and RIOTBOX-1077.
Consequences: future P016 stem/DAW/live work should address artifact-set-aware recovery and export-QA test/module hygiene before claiming wider export readiness.
Status: accepted

---

### RBX-002

Date: 2026-04-12  
Topic: MVP scope framing  
Phase: Phase 0  
Question: what is the MVP proving?  
Decision: the MVP proves the track-to-instrument path, not full generative autonomy and not DAW completeness.  
Why: this aligns engineering effort with the product spine and protects against scope drift.  
Consequences: Ghost `perform`, advanced export polish, and advanced DSP remain off the early critical path.  
Status: accepted

### RBX-003

Date: 2026-04-12  
Topic: feral mode architecture  
Phase: Phase 0  
Question: how should the feral logic live in the system?  
Decision: feral behavior must be implemented as profile / policy / scoring extensions on top of the core system, not as a second architecture.  
Why: this preserves mergeability, replay consistency, and scope discipline.  
Consequences: new feral work should land in existing modules and specs, not in parallel engines or formats.  
Status: accepted

### RBX-004

Date: 2026-04-12  
Topic: stack freeze v1  
Phase: Phase 0  
Question: which stack decisions need to be frozen before the first implementation slice begins?  
Decision: use `Rust` for the core workspace and runtime-facing model layer, keep `Python` reserved for the later analysis sidecar, target `JSON` for early persisted artifacts, and plan around `cpal`, `tokio`, and `ratatui` for the first runtime-capable stack.  
Why: this is the best fit for realtime control, deterministic state, terminal-native UX, and a later MIR sidecar without forcing premature framework commitments.  
Consequences: the first code slice starts as a Rust workspace, while transport and audio choices get validated by bounded spikes rather than more abstract debate.  
Status: accepted

### RBX-005

Date: 2026-04-12  
Topic: deterministic replay model  
Phase: Core Skeleton  
Question: what should Riotbox treat as replay truth, and how should snapshots relate to action replay?  
Decision: replay truth is the combination of frozen source references, frozen Source Graph references, durable committed action history, and optional snapshots that accelerate restore without replacing the action log. `requested_at` is diagnostic, while commit order and musical commit boundary are replay-relevant.  
Why: replay must not depend on rerunning unstable analysis, re-asking Ghost, or reconstructing captured artifacts from ambient state.  
Consequences: future runtime work should add explicit replay-order metadata, make snapshot anchors more concrete, and preserve musical boundary identity for committed actions.  
Status: accepted

### RBX-006

Date: 2026-04-12  
Topic: CPAL audio direction  
Phase: Core Skeleton  
Question: should Riotbox proceed with `cpal` as the low-level audio I/O entry point for the first runtime-capable audio slice?  
Decision: yes. Use `cpal` as the low-level audio I/O layer and isolate it in `crates/riotbox-audio`, with explicit probing and health metrics kept near the stream layer.  
Why: the library matches Riotbox's callback-oriented audio-core direction and exposes the host/device/config/stream concepts needed for a low-level runtime boundary.  
Evidence: official `cpal` documentation confirms default host/device discovery, supported config enumeration, and output stream construction as the core API surface; the local spike prototype compiles against `cpal` and provides a runnable path for host/device/config probing and callback-gap measurement.  
Consequences: later audio work should build a runtime shell above `cpal` rather than replacing it with a higher-level playback abstraction, and health metrics should be captured from the stream layer from the start.  
Status: accepted

### RBX-007

Date: 2026-04-12  
Topic: Rust-Python sidecar transport  
Phase: Analysis Vertical Slice  
Question: what transport should Riotbox use for the first real Rust-to-Python sidecar integration slice?  
Options considered: newline-delimited JSON over `stdio`, Unix domain sockets, localhost TCP, binary message formats such as MessagePack or Protobuf.  
Decision: use newline-delimited JSON over `stdio` for the first sidecar-facing slice, with explicit request IDs and version fields in messages. Keep future socket-based transports open if concurrency or lifecycle needs outgrow `stdio`.  
Why: this is the smallest debuggable process boundary, keeps transport setup simple while request shapes are still settling, and fits the current goal of bounded request/response analysis without dragging realtime code into sidecar concerns.  
Evidence: the `RIOTBOX-9` spike crate successfully spawns a Python sidecar, completes a `ping` roundtrip, and deserializes a Python-produced stub `SourceGraph` into the existing Rust model.  
Consequences: the next analysis-facing slices should build on a narrow synchronous transport contract first, keep progress streaming optional, and move to sockets only when real workload or lifecycle pressure justifies it. This decision does not freeze `Python` or `NDJSON over stdio` as permanent choices; both should be revisited once the sidecar contract carries more message types, stronger versioning pressure, or external lifecycle needs.  
Status: accepted

### RBX-008

Date: 2026-04-12  
Topic: Rust CI baseline  
Phase: Spec Freeze + Core Model  
Question: what minimum automated checks should Riotbox enforce on the Rust workspace at the current project stage?  
Decision: start with one small GitHub Actions workflow that runs `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings` on pushes to `main` and on pull requests.  
Why: these checks cover formatting drift, broken behavior, and obvious lint-level engineering regressions without prematurely building a large automation surface.  
Evidence: the current workspace already uses `cargo` and a matching `just ci` command locally, so the workflow can mirror existing developer expectations instead of inventing a second build path.  
Consequences: contributors should treat those three commands as the local pre-PR baseline, and future CI growth should add replay, benchmark, or screenshot checks only when those contracts become stable enough to enforce.  
Status: accepted

### RBX-009

Date: 2026-04-12  
Topic: audio runtime shell baseline  
Phase: Core Skeleton  
Question: what is the smallest reusable audio runtime boundary Riotbox should introduce after the completed CPAL spike?  
Decision: add a minimal `AudioRuntimeShell` above `cpal` inside `crates/riotbox-audio`, with explicit lifecycle state, typed output metadata, typed health snapshots, and typed startup errors. Keep the shell limited to stream lifecycle and telemetry for now.  
Why: the project needs a real runtime-facing boundary before scheduler, app-level runtime state, or TUI health surfaces can be added. The smallest useful step is a shell that owns the stream and publishes measurable health without overbuilding the engine.  
Evidence: the new runtime shell compiles cleanly, reuses the existing probe path, and passes unit tests for telemetry accounting, faulted health snapshots, and lifecycle transitions.  
Consequences: future runtime work should build transport, scheduler, and app-facing health state on top of this shell rather than creating a second audio runtime abstraction.  
Status: accepted

### RBX-010

Date: 2026-04-12  
Topic: app-layer runtime health state  
Phase: Core Skeleton  
Question: where should Riotbox represent audio and sidecar runtime health before a full TUI exists?  
Decision: keep runtime-facing health state in `riotbox-app`, not in `riotbox-core`. Reuse typed audio health from `riotbox-audio`, model sidecar availability in the app layer, and derive a Jam-facing runtime summary view there.  
Why: runtime health belongs to orchestration and presentation, not to the stable core domain contracts. Keeping it in the app layer avoids pulling service/runtime concerns into `SourceGraph`, `SessionFile`, or core Jam models too early.  
Evidence: the `RIOTBOX-13` slice extends `JamAppState` with audio and sidecar runtime state, derives a separate runtime summary view, and passes tests covering ready, degraded, and faulted states without changing `riotbox-core`.  
Consequences: future TUI work should consume app-layer runtime summaries, while core contracts stay focused on replay-safe domain state.  
Status: accepted

### RBX-011

Date: 2026-04-12  
Topic: scheduler-facing transport boundary model  
Phase: Core Skeleton  
Question: how should Riotbox represent quantized commit timing before the full scheduler exists?  
Decision: add an explicit `TransportClockState` and `CommitBoundaryState` in `riotbox-core`, and let the action queue commit against that boundary state instead of a bare enum alone. Return stable per-boundary commit order from the queue for scheduler-facing use.  
Why: replay and scheduler work need a concrete musical boundary model instead of relying on hidden timing assumptions or incidental queue vector order. The queue should know which musical window it is committing against, not just that it saw a generic `Bar` or `Phrase`.  
Evidence: the `RIOTBOX-14` slice adds transport and boundary types, queue commits against explicit boundary state, and tests stable commit ordering and boundary identity without changing persistence yet.  
Consequences: future scheduler and replay work should build on this explicit boundary model, and persistence can decide later how much of that runtime boundary metadata becomes durable session history.  
Status: accepted

### RBX-012

Date: 2026-04-12  
Topic: app-layer Jam runtime orchestration  
Phase: Core Skeleton  
Question: where should Riotbox first combine transport clock updates, queue commits, and session-facing Jam refresh logic before the full scheduler and TUI exist?  
Decision: add the first runtime orchestration methods in `riotbox-app`, not `riotbox-core`. `JamAppState` should own transport clock updates, commit queued actions against explicit `CommitBoundaryState`, mirror committed actions into the session action log in stable order, and reseed fresh queue IDs after persisted session history.  
Why: this is orchestration work across runtime state, queue semantics, and presentation refresh, not a new core contract. Keeping it in the app layer avoids pulling scheduler/runtime glue into the core model while still making the Jam shell testable.  
Evidence: the `RIOTBOX-17` slice adds app-level transport and commit methods, covers transport refresh and stable commit propagation with tests, and keeps `riotbox-core` limited to reusable queue primitives plus ID reseeding support.  
Consequences: future scheduler/TUI work should build on `JamAppState` orchestration entry points, while persistence and replay continue to rely on the explicit session action log rather than queue internals alone.  
Status: accepted

### RBX-013

Date: 2026-04-12  
Topic: MemPalace as Riotbox dev-memory tooling  
Phase: Core Skeleton  
Question: should Riotbox adopt MemPalace now as a standard internal project-memory and agent-assist retrieval tool?  
Decision: do not make MemPalace a required default workflow dependency yet, but treat it as a validated optional dev-memory tool using rootless Podman with pinned `Python 3.12` and repo-local persistent storage.  
Why: the direct host trial failed on the current machine baseline (`Python 3.14`), but the real rootless Podman evaluation completed successfully against Riotbox data. The remaining uncertainty is not basic operability; it is whether the retrieval value justifies adding another maintained tool beside repo docs and Linear. For Riotbox, an external dev-memory helper is only worth standardizing if setup is boring and it clearly improves real retrieval tasks.  
Evidence: upstream documentation shows active progress and honest correction of earlier overstated claims, but also ongoing backend and stability work; the host trial installed `mempalace 3.1.0` successfully yet failed during runtime import through the `chromadb` / `pydantic.v1` path on Python 3.14. A real rootless Podman trial with pinned `python:3.12-slim` completed `init`, `mine`, `status`, and multiple Riotbox searches successfully, producing a persistent palace under `.mempalace/`.  
Consequences: Riotbox should continue treating repo docs, the decision log, and Linear as the canonical memory layer. MemPalace is now credible enough to keep as an optional retrieval helper, but it should stay outside product core and should not become a second hidden source of truth. Broader workflow adoption should depend on a comparative bakeoff against `rg` plus repo docs plus Linear.  
Status: accepted

### RBX-014

Date: 2026-04-12  
Topic: MemPalace versus `rg` role split  
Phase: Core Skeleton  
Question: after a broader bakeoff, how should Riotbox position MemPalace relative to the existing `rg`-plus-docs workflow?  
Decision: keep MemPalace as an optional semantic project-memory tool, and keep `rg` as the primary exact lookup tool. MemPalace complements `rg`; it does not replace it.  
Why: the broader bakeoff showed that MemPalace is stronger for question-shaped architecture and planning retrieval across documents, while `rg` remains dramatically faster and better for exact code/symbol navigation.  
Evidence: an eight-task comparison against an expanded Riotbox corpus found strong MemPalace results for questions such as Rust-core rationale, replay truth, and feral-profile semantics, but weaker performance on exact implementation lookup like the Jam runtime slice. Query timing also remained much higher for MemPalace than for `rg`.  
Consequences: Riotbox can justify keeping MemPalace available as an optional retrieval layer for long-horizon project memory, but day-to-day code navigation should continue to rely on `rg` first. Any broader default adoption should focus on workflow polish rather than pretending the tools serve the same job.  
Status: accepted

### RBX-015

Date: 2026-04-12  
Topic: MemPalace operational path  
Phase: Core Skeleton  
Question: if Riotbox keeps MemPalace available, what is the supported local operating path?  
Decision: use a repo-local wrapper around rootless Podman Compose with a pinned `python:3.12` image, repo-local state under `.mempalace/`, and automatic re-mining when `docs/`, `plan/`, `crates/`, or `AGENTS.md` changed. Expose it through `scripts/mempalace.sh` and `just` commands.  
Why: this removes the manual container incantation, hides Python-version concerns, keeps state persistent outside ephemeral containers, and makes the optional tool boring enough to use without pretending it is canonical product infrastructure.  
Evidence: the earlier MemPalace evaluation already proved the rootless Podman path works on Riotbox data; the wrapper and compose setup turn that validated path into a repeatable repo-local command surface.  
Consequences: contributors can use MemPalace through stable project commands, while the tool remains optional and subordinate to repo docs, Linear, Git history, and `rg` for exact lookup.  
Status: accepted

### RBX-015a

Date: 2026-05-01
Topic: MemPalace runtime update and index repair
Phase: Pro Hardening
Question: can Riotbox move the MemPalace container from the older Python 3.12 /
MemPalace 3.1.0 runtime to the current Python/MemPalace stack, and should the
existing palace index be repaired for cosine-distance metadata?
Decision: update the repo-local MemPalace container to pinned
`python:3.14.4-slim` and `mempalace==3.3.4`, keep normal access through
`just`/`scripts/mempalace.sh`, and add `just mem-repair` for explicit index
metadata repair.
Why: PyPI reports MemPalace 3.3.4 as the current release, and the updated
package now installs and runs on Python 3.14. The existing index produced a
MemPalace warning that it lacked cosine-distance metadata, which makes semantic
scores less meaningful for text-embedding search.
Evidence: the updated image built successfully, `mempalace --version` reported
`MemPalace 3.3.4`, `mempalace --palace /palace repair --yes` rebuilt 5154
drawers, `just mem-status` re-mined successfully and reported 5246 drawers, and
`just mem-search "replay recovery"` returned results without the previous
cosine-distance warning. The local Podman runtime required container networking
to be disabled for normal run commands because the host lacks `/dev/net/tun` for
the default networking backend; builds still require normal registry/network
access.
Consequences: MemPalace remains optional dev-memory, not canonical project
truth. The supported operational path remains the repo-local `just` wrapper,
with `mem-repair` available for rare index maintenance rather than direct
container invocations.
Status: accepted

### RBX-015b

Date: 2026-06-03
Topic: Remove MemPalace from active dev-memory workflow
Phase: Pro Workflow / Export
Question: should Riotbox keep maintaining MemPalace as an optional semantic
project-memory tool?
Decision: remove MemPalace from active workflow tooling and keep
`just decision-search` as a bounded `rg` helper over
`docs/research_decision_log.md`.
Why: recent implementation and planning work used `rg`, targeted docs, Linear,
Git, and GitHub rather than MemPalace. A semantic-memory tool that agents almost
never use does not earn the maintenance cost of a container wrapper, corpus
sync, index repair, and special workflow rules.
Evidence: local history showed no recent meaningful `mem-search` usage and only
a recent state touch/sync; the original bakeoff still showed MemPalace helped
some semantic architecture questions, but normal Riotbox implementation
continued to favor exact repo search and targeted file reads.
Consequences: canonical truth remains in repo docs/specs, Linear, and Git
history. Future semantic retrieval should be proposed as a new measured spike,
with Qdrant plus a Riotbox-owned indexer as the preferred candidate if exact
search becomes too noisy for recurring planning questions.
Status: accepted

### RBX-016

Date: 2026-04-12  
Topic: first analysis ingest slice shape  
Phase: Analysis Vertical Slice  
Question: what is the smallest real analysis-facing path Riotbox should add after the transport spike and core skeleton groundwork?  
Decision: add one app-facing `analyze_source_file` ingest path that sends a source file path through the existing stdio sidecar boundary, receives a `SourceGraph`, persists both graph and session JSON, and returns a ready `JamAppState`. Keep the current Python sidecar implementation deliberately bounded to file-based stub analysis rather than full decode quality.  
Why: the project needs a real path from source file to persisted graph without waiting for full MIR quality or reopening the transport contract. This proves the integration seam through app layer, sidecar transport, persistence, and Jam state assembly in one bounded slice.  
Evidence: `riotbox-sidecar` now supports a real file-path request in addition to the older transport stub request, `riotbox-app` can ingest a real source file through that sidecar path and persist JSON artifacts, and tests cover both the happy path and a missing-file failure path.  
Consequences: the next analysis work should improve the actual analysis quality behind the same ingest path rather than inventing a second graph-loading flow. Persistence and Jam assembly can now assume a real sidecar-produced graph path exists, even though the analysis content is still intentionally simple.  
Status: accepted

### RBX-017

Date: 2026-04-12  
Topic: decoded-source analysis baseline  
Phase: Analysis Vertical Slice  
Question: what is the smallest useful improvement behind the new ingest seam before Riotbox takes on real MIR complexity?  
Decision: keep the existing `analyze_source_file` request and `SourceGraph` response shape, but replace the previous file-size heuristic with a decoded WAV baseline in the Python sidecar. Derive source duration, sample rate, channel count, simple energy summaries, and a duration-fit timing estimate from the decoded audio itself.  
Why: the ingest seam already exists, so the next valuable step is to make the sidecar return graph content grounded in actual decoded source data without reopening transport or overcommitting to premature analysis sophistication.  
Evidence: the sidecar now decodes real PCM WAV input via Python stdlib `wave`, the Rust sidecar and app tests use real generated WAV fixtures instead of arbitrary bytes, the happy-path graph reflects decoded metadata, and unsupported files surface a stable explicit failure path.  
Consequences: future analysis work should continue to improve timing, sectioning, and candidate quality behind this same ingest path. Riotbox now has a bounded decoded-source baseline rather than a pure transport stub, but it remains intentionally simple and WAV-focused for now.  
Status: accepted

### RBX-018

Date: 2026-04-12  
Topic: first TUI-facing Jam shell boundary  
Phase: Analysis Vertical Slice  
Question: once analyzed session state exists, what is the smallest real UI slice Riotbox should add without leaking presentation concerns into the core contracts?  
Decision: add the first terminal UI shell entirely in `riotbox-app` using `ratatui` and `crossterm`, with one file-driven binary that either ingests a source file or loads an existing session/graph pair and renders the current `JamViewModel` plus runtime health. Keep `riotbox-core` presentation-free and test the render path with a non-interactive `TestBackend`.  
Why: the project now needs a user-facing Jam surface to make the current ingest and runtime work visible, but the core contracts should remain reusable and not turn into widget models or terminal-specific abstractions.  
Evidence: `riotbox-app` now has a real `riotbox-app` binary, a minimal Jam shell renderer, argument parsing for ingest/load flows, a render test against `ratatui::backend::TestBackend`, and a smoke-launched TTY shell that exits cleanly.  
Consequences: later UI work should deepen this same app-side shell instead of inventing a separate presentation path. The next TUI slices can add richer panels, keybindings, and screenshots while reusing the current `JamAppState` seam.  
Status: accepted

### RBX-019

Date: 2026-04-13  
Topic: first navigable Jam shell interaction model  
Phase: Jam-first Playable Slice  
Question: after the first shell exists, what is the smallest next interaction layer that makes it meaningfully usable without pretending Riotbox already has deep live controls?  
Decision: keep the shell file-driven and app-side, but add a tiny safe keybinding set: `q`/`Esc` to quit, `?` or `h` to toggle a help overlay, and `r` to refresh the shell by reloading or re-ingesting from the current launch mode. Add clearer source context and section visibility directly in the main Jam surface, and store a normalized terminal baseline under `docs/screenshots/`.  
Why: the shell now needs to answer “what am I hearing” and “what can I do next” more clearly, but the project still is not ready for full performance editing or transport control. Small real interactions and stronger source context improve usability without destabilizing the core contracts.  
Evidence: `riotbox-app` now has shell state for launch mode, help visibility, and status messages; the renderer shows source metadata, analysis confidence, and section summaries; tests cover richer render content and key handling; a real TTY smoke pass exercised help, refresh, and quit; and a stable terminal baseline is stored in the repo.  
Consequences: later TUI work should build on this same shell-state seam for richer keybindings, source trust surfaces, and screenshot updates. Live musical controls remain intentionally out of scope until the shell and app-side runtime become more mature.  
Status: accepted

### RBX-020

Date: 2026-04-13  
Topic: Jam shell trust and action cue framing  
Phase: Jam-first Playable Slice  
Question: after the first live-safe shell interactions exist, what is the next smallest UI improvement that makes the Jam shell feel more like an instrument surface instead of a status dashboard?  
Decision: keep the shell app-side and non-destructive, but reframe the main surface around trust and action imminence. Replace the generic top-row status framing with `Now`, `Next`, `Trust`, and `Lanes`, strengthen the header around current scene and next queued move, and expose recent committed actions and source trust more explicitly. Keep deep live editing and transport control out of scope.  
Why: Riotbox now has a real Jam shell, but the shell still needs to answer “what is happening now,” “what lands next,” and “how much should I trust this analysis” at a glance. Those cues are more important to musical use than generic runtime inventory panels.  
Evidence: `riotbox-app` now renders the shell around trust and action cues, tests validate the richer snapshot semantics, a real TTY smoke pass exercised the updated shell, and a new normalized baseline artifact captures the reviewable result in `docs/screenshots/jam_shell_trust_action_baseline.txt`.  
Consequences: the next Jam shell slices should keep deepening this same path with safe musical cues, pending/commit clarity, and better source context rather than opening a second editor path. Explicit live mutation controls remain intentionally bounded until the shell/runtime seam is stronger.  
Status: accepted

### RBX-021

Date: 2026-04-13  
Topic: first live-safe Jam action entry should stay inside the existing queue and transport seam  
Phase: Jam-first Playable Slice  
Question: now that the Jam shell is readable and safe, how should Riotbox introduce its first user-triggered musical actions without inventing a second interaction or execution path?  
Decision: add a small set of bounded Jam keybindings that enqueue real Action Lexicon entries into the existing `ActionQueue`, advance them through the current transport-boundary model, and keep undo as an app-side action-log operation. Do not add a parallel shell-local mutation executor or device-specific shortcut path.  
Why: the current gap is not more passive UI, but the absence of first-class user action entry. The smallest slice that moves Riotbox toward “instrument” status is to let the Jam shell queue a few real actions and visibly commit them on beat/bar/phrase boundaries. Using the existing queue and transport model preserves replayability and keeps the product on one interaction spine.  
Evidence: `riotbox-app` now exposes first live-safe Jam actions for scene mutation, TR-909 fill, and phrase capture, advances transport on a small app-side tick, commits queued actions through the existing commit-boundary flow, and supports one recent undo path. Tests cover the new queueing, boundary commit, undo, and shell keybinding behavior.  
Consequences: later Jam interaction work should deepen the same queue/transport seam for capture, device MVPs, and stronger pending/commit visibility. Full device execution semantics still remain out of scope for this slice.  
Status: accepted

### RBX-022

Date: 2026-04-13  
Topic: the first Jam capture workflow should materialize real capture records on commit  
Phase: Jam-first Playable Slice  
Question: after live-safe Jam actions exist, what is the smallest capture-oriented slice that makes capture feel like a real workflow instead of only another queued command label?  
Decision: when a committed capture action lands, create a real `CaptureRef`, update the W-30 lane's `last_capture`, and expose the newest capture summary directly in the Jam shell. Do this within the current session/action/view seam instead of adding a separate capture runtime or a full Capture screen.  
Why: Phase E still requires a first capture path. That requirement is not satisfied by merely being able to queue a capture action. The user needs to see that a committed capture produced reusable session state with a target and provenance. Materializing capture records at commit time gives Riotbox its first real capture loop while keeping the slice narrow.  
Evidence: committed capture actions in `riotbox-app` now append `CaptureRef` records with source-origin refs and W-30 targeting, update `last_capture`, and feed a new capture summary in `JamViewModel`. Tests cover capture materialization and the Jam shell now renders a dedicated capture panel.  
Consequences: later Capture-screen work and W-30 MVP work should build on these same session capture records rather than inventing a second capture inventory. Deep sample editing and resample routing remain out of scope for this slice.  
Status: accepted

### RBX-023

Date: 2026-04-13  
Topic: TR-909 MVP should start with explicit lane-state side effects before audible drum generation  
Phase: TR-909 MVP  
Question: after the Jam shell can queue and commit safe actions, what is the smallest TR-909 slice that moves the first device MVP forward without pretending the audio engine already supports real drum reinforcement?  
Decision: let committed TR-909 actions update explicit lane state for fill arming, last fill bar, pattern reference, and reinforcement mode, and surface those cues in the Jam shell. Keep the slice inside the current session/action/view seam and defer actual audible drum synthesis to a later audio-facing step.  
Why: Riotbox needs a real device seam before it can support believable TR-909 audio work. The first honest increment is to make `TR-909 fill` and `TR-909 reinforce` actions produce replayable device state that the shell can show and later audio work can consume. That preserves continuity from Phase E into Phase F instead of jumping directly from UI hints to untracked audio behavior.  
Evidence: `riotbox-core` and `riotbox-app` now track TR-909 fill and reinforcement state explicitly, committed TR-909 actions mutate that state at transport boundaries, the Jam shell surfaces the resulting cues, and tests cover the new side effects plus keybinding entry.  
Consequences: later TR-909 slices should consume the same lane state for audible pattern generation and drum reinforcement rather than bypassing it. This slice does not yet satisfy full TR-909 MVP exit criteria because audible reinforcement still remains out of scope.  
Status: accepted

### RBX-024

Date: 2026-04-13
Topic: the next TUI slice should add a Log screen instead of inventing a second action-trust surface
Phase: Jam-first Playable Slice
Question: after Jam has real queued actions, capture materialization, and the first TR-909 device cues, what is the smallest next UI slice that improves user trust without opening a parallel editor path?
Decision: add the first `Log` screen directly inside the existing shell, with explicit screen switching between `Jam` and `Log`, and render queued, committed, rejected, and undone actions from the current session and queue state. Keep the slice read-only and avoid introducing a second log model or a separate inspector runtime.
Why: the TUI spec already prioritizes `Log` immediately after `Jam`, and the current product gap is not a lack of more device controls but a lack of visible action trust. Now that Riotbox has real actions and side effects, users need a dedicated trust surface that answers what just changed, what is queued, and why outcomes differed.
Evidence: `riotbox-app` now has shell-level screen switching, a first Log screen using existing action/session/runtime state, tests covering screen switching and log rendering, and a normalized baseline artifact in `docs/screenshots/jam_log_screen_baseline.txt`.
Consequences: later TUI work should keep `Jam` as the performance surface and deepen `Log` as the trust/history surface instead of adding parallel inspector screens for the same information. Filtering, search, and Ghost-specific log detail remain out of scope for this slice.
Status: accepted

### RBX-025

Date: 2026-04-13
Topic: the next TUI slice after Log should add the Source screen inside the same shell spine
Phase: Jam-first Playable Slice
Question: once `Jam` and `Log` both exist, what is the smallest next UI slice that improves analysis trust without opening a separate source-inspector runtime?
Decision: add the first `Source` screen directly inside the existing shell, extend screen switching so `Jam`, `Log`, and `Source` are all reachable in one TUI spine, and render source identity, timing confidence, sections, candidate summaries, provenance, and source-graph warnings from the existing `SourceGraph`.
Why: the TUI spec puts `Source` immediately after `Log`, and Riotbox already has enough decoded-source structure that users should be able to inspect it in one dedicated place. The next honest improvement is better visibility into analysis-derived structure, not a second inspector toolchain.
Evidence: `riotbox-app` now renders a dedicated `Source` screen, tests cover the new screen and screen switching, and a normalized baseline artifact exists at `docs/screenshots/source_screen_baseline.txt`.
Consequences: later source-oriented work should deepen this same screen for richer structure trust and candidate inspection instead of creating a separate source-debug UI. Deep graph editing and Ghost-specific diagnostics remain out of scope for this slice.
Status: accepted

---

Topic: the next TUI slice after Source should add the Capture screen inside the same shell spine  
Phase: Jam-first Playable Slice  
Question: once `Jam`, `Log`, and `Source` all exist, what is the smallest next UI slice that makes capture feel like a first-class musical workflow without inventing a separate capture runtime?  
Decision: add the first `Capture` screen directly inside the existing shell, extend screen switching so `Jam`, `Log`, `Source`, and `Capture` all remain in one TUI spine, and render readiness, recent captures, provenance, pending capture cues, and routing context from the existing session, queue, and `JamViewModel` state.  
Why: the TUI spec calls out `Capture` as a core screen, and Riotbox already materializes real capture records plus queued capture actions. The next honest improvement is to make that workflow legible in the same shell, not to create a second capture inventory or a deeper W-30 editor before the capture path is visible.  
Evidence: `riotbox-app` now renders a dedicated `Capture` screen, tests cover the new screen and screen switching, and a normalized baseline artifact exists at `docs/screenshots/capture_screen_baseline.txt`.  
Consequences: later capture-oriented work should deepen this same screen for promotion, pinning, and reuse cues instead of opening a separate capture browser. Deep sample editing and full W-30 pad workflow remain intentionally out of scope for this slice.  
Status: accepted

---

Topic: the first capture-promotion flow should bind existing captures instead of pretending capture already equals promotion  
Phase: Jam-first Playable Slice  
Question: now that the `Capture` screen exists, what is the smallest next slice that makes captured material reusable in-flow without inventing a separate W-30 subsystem?  
Decision: keep capture and promotion as distinct steps. Committed capture actions create capture records that may remain unassigned, while `promote.capture_to_pad` updates an existing capture's target inside the current action/session/view seam and surfaces the promotion result directly in the `Capture` screen.  
Why: the PRD requires captured material to be reusable without leaving flow, but collapsing capture and promotion into one side effect makes the workflow semantically muddy and hides the real promotion seam. Distinguishing them keeps the architecture honest while still staying bounded inside the current shell and session model.  
Evidence: `riotbox-app` now queues `promote.capture_to_pad` from the shell, applies promotion as a side effect on an existing `CaptureRef`, updates the action result summary, tracks promoted vs unassigned capture counts in `JamViewModel`, and refreshes the `Capture` screen plus baseline artifact to show the new state. Tests cover both unassigned capture materialization and later promotion.  
Consequences: later W-30 work should build on this same `capture -> promote` path for pinning, pad reuse, and promotion history instead of reintroducing implicit auto-routing. Deep resample routing and full pad editing remain intentionally out of scope for now.  
Status: accepted

---

Topic: the next capture slice should add persisted pinned-capture recall instead of a second capture browser  
Phase: Jam-first Playable Slice  
Question: after the first promotion path exists, what is the smallest next slice that makes capture recall more intentional without opening a new capture-management subsystem?  
Decision: add persisted pin metadata directly to `CaptureRef`, expose pinned capture counts and IDs through `JamViewModel`, and let the shell toggle pin state for the latest capture through a small app-layer interaction. Keep this as explicit session metadata rather than creating a separate favorite store or a new action family.  
Why: the TUI spec explicitly calls out favorite or pinned captures, and the MVP needs meaningful capture recall. A persisted pin flag is enough to make the capture workflow feel more deliberate without disturbing the existing capture/promotion seam or inventing a second browser path.  
Evidence: the session model now stores pin state on captures, `riotbox-app` can toggle pin state for the latest capture, the `Capture` screen shows a dedicated pinned section, and tests cover both the pin toggle and the updated screen rendering.  
Consequences: later capture work should deepen this same persisted pinning path for favorites and reuse flows instead of duplicating capture metadata elsewhere. Deep tagging or folder-like capture management remains out of scope for now.  
Status: accepted

---

Topic: pending TR-909 fill intent should be derived from the queue, not persisted as committed lane state  
Phase: Jam-first Playable Slice  
Question: how should Riotbox surface a queued TR-909 fill without creating save/reload drift between pending intent and committed runtime state?  
Decision: keep the pending TR-909 fill as queue-only intent. `queue_tr909_fill` should not mutate persisted lane state ahead of commit, and the `JamViewModel` should derive its “fill armed” visibility from the pending queue instead of from `session.runtime_state`.  
Why: the TUI contract requires users to distinguish queued from committed state, and the prior implementation could save committed-looking lane state even though the fill action still existed only in memory. Deriving the indicator from the queue preserves the visible cue without lying about what has already happened.  
Evidence: `queue_tr909_fill` no longer flips `fill_armed_next_bar` in the session, `JamViewModel` now computes the armed indicator from pending `tr909.fill_next` actions, and app tests now verify that saving with a queued fill does not persist committed lane state across reload.  
Consequences: later work that needs durable pending-action restoration must either persist the queue explicitly or keep all pending-only cues derived from pending action data rather than from runtime state.  
Status: accepted

---

Topic: ingest should default to embedded graph storage and only write external graph files when explicitly requested  
Phase: Jam-first Playable Slice  
Question: how should the source-file ingest seam store its `SourceGraph` during MVP so it stays aligned with the current session-file contract without losing the ability to write explicit external graph files later?  
Decision: make embedded graph storage the default ingest path. When no explicit graph path is requested, the ingest flow writes the `SourceGraph` into the session as an embedded graph reference. External graph files remain supported only when the caller passes an explicit graph path.  
Why: the current session-file spec says MVP should prefer embedded graphs unless graph size becomes a real problem. Defaulting ingest to external files created extra file coupling and diverged from the current contract without any demonstrated need.  
Evidence: the ingest entry point now accepts an optional graph path, CLI parsing defaults `--source` ingest to no external graph file, tests cover both explicit external graph output and the default embedded-graph path, and save/load behavior continues to work in both modes.  
Consequences: later work can still add explicit external-graph workflows without changing the default MVP contract. Multi-source session questions remain separate and are tracked under the follow-up contract-alignment ticket.  
Status: accepted

---

Topic: Riotbox MVP should make the current single-source assumption explicit instead of silently collapsing plural session refs  
Phase: Jam-first Playable Slice  
Question: should the app start supporting multiple source refs now, or should MVP explicitly freeze to one active source and one matching graph ref until multi-source work is intentionally designed?  
Decision: freeze MVP explicitly to a single active source and a single matching source-graph reference. Keep the schema plural for forward compatibility, but make the spec explicit and reject invalid multi-source sessions in the app/runtime instead of silently loading only the first graph.  
Why: the current app behavior is single-source already. Leaving the plural core shape unqualified made the contract look broader than the runtime really was, which would create silent drift and make later multi-source work harder to reason about.  
Evidence: the app load path now validates the single-source MVP constraint, rejects sessions with multiple source refs or mismatched source/graph IDs, and the session-file spec now calls the MVP restriction out explicitly while preserving the plural schema shape for future migration.  
Consequences: later multi-source work will need an explicit active-source selector and updated app/runtime contracts instead of relying on the current single-source assumption.  
Status: accepted

---

Topic: terminal UI should consume runtime pulses instead of owning transport advancement  
Phase: Jam-first Playable Slice  
Question: how should Riotbox remove musical timing authority from the terminal redraw loop before the full audio scheduler exists?  
Decision: introduce a small app-runtime pulse source outside the TUI event loop and make the app own elapsed-time transport advancement from those pulses. The terminal loop should only render snapshots, dispatch key intents, and consume already-timed runtime signals.  
Why: the periodic codebase review identified a real architecture problem: the shell poll tick was both advancing transport and deciding boundary commits. That made musical timing depend on redraw cadence, which conflicts with the audio-core contract and would force a rewrite once audio or a real scheduler becomes authoritative.  
Evidence: `riotbox-app` now has a dedicated runtime pulse source, the terminal binary no longer computes beat deltas, and `JamAppState` advances transport from elapsed pulse timestamps through an explicit driver state. Tests cover runtime-anchor setup and elapsed-time transport progression while preserving the existing queue and commit-boundary behavior.  
Consequences: later scheduler or audio-runtime work can replace the current pulse source with a stronger timing authority without reopening the TUI contract. The shell stays bounded to rendering and intent dispatch, while the app/runtime seam becomes the place where transport time enters the product.  
Status: accepted

---

Topic: the first TR-909 MVP increment should be one bounded slam control on the existing action seam  
Phase: TR-909 MVP  
Question: what is the smallest real TR-909 control Riotbox should add now that the review-driven cleanup queue is closed and the shell already supports queued lane actions?  
Decision: start the TR-909 MVP with a single `tr909.set_slam` control queued through the existing `ActionQueue` and committed on the current transport-boundary seam. Keep it as a bounded toggle-like live control rather than inventing a separate device subsystem or pretending full drum takeover behavior already exists.  
Why: the roadmap says TR-909 comes first among device MVPs, but the next slice still needs to stay small and reviewable. A bounded slam control adds a real device-facing interaction and visible lane change without reopening transport, persistence, or TUI architecture.  
Evidence: the shell now exposes a dedicated `s` keypath for `tr909.set_slam`, the app queues it as a normal action, the committed side effects update both the TR-909 lane state and the macro intensity, and tests cover queueing, duplicate-pending protection, and committed slam state.  
Consequences: later TR-909 work should deepen the same queue-and-commit seam for reinforcement and takeover behavior rather than adding shortcut execution paths. Full audible drum takeover and richer pattern semantics remain out of scope for this first device-facing increment.  
Status: accepted

---

Topic: the next TR-909 MVP increment should add explicit takeover and release actions on the existing seam  
Phase: TR-909 MVP  
Question: how should Riotbox add controlled TR-909 lane takeover without inventing a second execution path or hiding state transitions behind UI-only toggles?  
Decision: add explicit `tr909.takeover` and `tr909.release` actions, queue them through the existing action seam, and commit them on phrase boundaries. Represent the committed state in TR-909 lane state and expose the pending target separately in the Jam view so the shell can show queued-versus-committed takeover clearly.  
Why: the milestone requires controlled 909 takeover, but the current shell only had fill, reinforce, and slam controls. A takeover/release pair is the next smallest real device-facing increment that keeps replay, queueing, and view state aligned.  
Evidence: the core action vocabulary now includes dedicated takeover and release commands, the app queues them with duplicate-pending protection, committed side effects update lane takeover state on phrase boundaries, and the Jam shell shows both committed takeover state and any queued takeover/release change. Tests cover queueing guards plus takeover and release commits.  
Consequences: later TR-909 work should deepen takeover semantics behind the same commands, for example richer pattern adoption or audio-facing render seams, instead of introducing a separate lane-control system.  
Status: accepted

---

Topic: the first audio-facing TR-909 render seam should be a derived runtime contract, not a second lane-control path  
Phase: TR-909 MVP  
Question: after TR-909 takeover and release exist on the normal action seam, what is the smallest next slice that prepares audible reinforcement work without prematurely building a drum engine or duplicating lane logic inside the audio crate?  
Decision: add a small `riotbox-audio` TR-909 render contract and derive it from committed session lane state plus transport and mixer context inside `riotbox-app`. Expose the derived render mode and routing in the shell, but keep actual drum synthesis out of scope for this slice.  
Why: Phase 3 is only really done once reinforcement becomes audible, but the next honest increment is not full drum generation. Riotbox first needs one explicit audio-facing contract that later render code can consume without bypassing the queue, replay, or committed lane state.  
Evidence: `riotbox-audio` now defines a dedicated TR-909 render state, `riotbox-app` derives that state from the committed session/runtime model on refresh, tests cover idle, reinforce, takeover, and release projections, and the Jam shell shows the current render seam summary.  
Consequences: later audible TR-909 work should consume this render contract and extend it if necessary instead of re-deriving drum state from ad-hoc UI cues or introducing a second device-state system.  
Status: accepted

---

Topic: the first audible TR-909 reinforcement slice should stay inside the existing render seam and audio runtime shell  
Phase: TR-909 MVP  
Question: once Riotbox has an explicit TR-909 render contract, what is the smallest next step that makes reinforcement honestly audible without pretending a full drum-machine engine already exists?  
Decision: drive a bounded callback-side TR-909 reinforcement renderer directly from the existing render seam, and start that audio path from the Jam app without introducing a second device-control system or a separate drum runtime. Keep the sound generation intentionally simple and replay-aligned: support, reinforce, and takeover should become audibly distinct, but full pattern adoption and richer device semantics stay out of scope.  
Why: Phase 3 requires audible reinforcement, but jumping from a render contract straight to a full TR-909 engine would be too large and too architecture-risky. The honest next move is to make the existing seam produce audible results while preserving the queue, transport, and committed lane-state path.  
Evidence: `riotbox-audio` now renders bounded TR-909 reinforcement audio from the render seam in the audio callback, `riotbox-app` starts the audio runtime and keeps it updated from the current committed render state, and tests cover silent idle behavior, audible support-mode output, and zero drum-bus silence.  
Consequences: later TR-909 slices should deepen the same audible render path with better profiles, pattern adoption, and regression fixtures rather than replacing it with a parallel drum subsystem.  
Status: accepted

---

Topic: early TR-909 render profiles should be typed and derived from source plus committed lane state  
Phase: TR-909 MVP  
Question: now that TR-909 reinforcement is audibly real, how should Riotbox make support and takeover sound semantically different without reintroducing stringly callback logic or a second device-control path?  
Decision: keep render-profile choice inside the existing TR-909 render contract and make it typed. Derive source-support profiles from the current source section at the app layer, derive takeover profiles from the committed TR-909 lane state, and let the audio callback consume those typed profiles to vary density, gain, pitch, and decay.  
Why: the next honest step after first audible reinforcement is to deepen musical differentiation, not to invent a parallel drum engine. Typed render profiles preserve the app-to-audio seam, keep the callback free of string parsing, and make profile behavior testable and replay-aligned.  
Evidence: `riotbox-audio` now defines explicit support and takeover render-profile enums, `riotbox-app` derives source-support profiles from source-section context and takeover profiles from committed lane state, and tests cover both the app-side derivation and callback-side audible differences between profiles.  
Consequences: later TR-909 work should extend the same typed render-profile seam with richer pattern adoption and fixture coverage instead of pushing profile semantics back into UI strings or bypassing the current render contract.  
Status: accepted

---

Topic: early TR-909 audible regression coverage should be fixture-backed at both render projection and callback levels  
Phase: TR-909 MVP  
Question: now that Riotbox has audible TR-909 reinforcement plus typed render profiles, what is the smallest verification slice that protects replay-safe behavior without adding new musical logic?  
Decision: add two bounded fixture-backed regression layers. Keep one app-side fixture that checks committed session and source state still project into the expected TR-909 render seam, and one audio-side fixture that checks the callback renderer still produces bounded audible metrics for key render cases.  
Why: the new TR-909 path now spans committed lane state, render projection, and callback output. If that chain drifts silently, Phase 3 audio can become audibly wrong while still compiling. Fixture-backed checks preserve the replay-aligned seam and make later refactors safer without pretending to be full golden-audio approval tests.  
Evidence: `riotbox-app` now loads committed render-projection fixtures for source-support and takeover states, `riotbox-audio` now loads callback render fixtures for idle, source-support, and takeover cases, and both fixture suites run inside the normal Rust test path.  
Consequences: later TR-909 audio work should extend the same fixture-backed regression pattern with richer pattern cases and diagnostics instead of relying only on ad-hoc unit assertions.  
Status: accepted

---

Topic: TR-909 audible render diagnostics should stay read-only and ride on the existing Jam and Log shell seams  
Phase: TR-909 MVP  
Question: once TR-909 reinforcement is audible and fixture-backed, what is the smallest next shell slice that helps humans inspect the render contract without opening a new control path or device-state system?  
Decision: surface concise TR-909 render diagnostics directly in the existing Jam and Log screens. Keep the Jam screen focused on at-a-glance lane cues, and add a compact TR-909 render panel in the Log screen for mode, routing, profile, pattern, mix, and alignment summaries.  
Why: the render seam is now musically meaningful enough that users and reviewers need to see what the audio path believes it is rendering. The next honest step is not more control, but better observability on top of the same committed render contract.  
Evidence: `riotbox-app` now derives richer TR-909 render summaries and warnings from the committed render state, the Jam screen shows concise mode/profile/pattern/mix cues, the Log screen exposes a dedicated render diagnostics panel, and app tests cover both the runtime view derivation and the updated shell snapshots.  
Consequences: later TR-909 work should extend the same read-only diagnostic seam with richer audible metrics or trust cues rather than inventing a separate render-debug model or device inspector outside the current shell.  
Status: accepted

---

Topic: the first TR-909 pattern-adoption step should become typed render state before the audio callback deepens further  
Phase: TR-909 MVP  
Question: once the TR-909 audible seam is observable, what is the next bounded slice that makes the render path more musical without turning `pattern_ref` into callback-side string logic or opening a second drum-engine model?  
Decision: add a typed `pattern adoption` layer to the existing TR-909 render contract. Derive it at the app layer from committed `pattern_ref`, render mode, and current profile context, and let the audio callback vary subdivision, trigger density, accenting, gain, and decay from that typed adoption signal.  
Why: the next honest TR-909 step is to make the current audible seam adopt a bounded pattern shape, not to invent a full device sequencer. A typed adoption layer keeps the render path replay-aligned, keeps string parsing out of the callback, and makes the new musical behavior testable and diagnosable in the shell.  
Evidence: `riotbox-audio` now defines an explicit `Tr909PatternAdoption` enum, `riotbox-app` derives it from committed render context, the callback changes its audible behavior from that adoption layer, fixtures cover both the app-side projection and audio-side regression cases, and the Jam/Log shell diagnostics surface the adopted pattern name.  
Consequences: later TR-909 work should extend the same typed pattern-adoption seam with phrase-aware variation and richer fixtures instead of bypassing it with direct callback heuristics or a separate device-state graph.  
Status: accepted

---

Topic: TR-909 phrase-aware variation and release behavior should extend the existing render seam instead of creating a second phrase engine  
Phase: TR-909 MVP  
Question: after typed pattern adoption exists, what is the next bounded slice that makes the TR-909 lane feel more phrase-aware without adding a second timing model, release engine, or device-control path?  
Decision: add a typed `phrase variation` layer to the existing TR-909 render contract. Derive it at the app layer from committed transport phrase context, current mode/profile context, and explicit release-pattern cues, then let the audio callback vary subdivision, trigger activity, pitch, gain, and decay from that typed phrase variation.  
Why: the next honest TR-909 step is to deepen the existing audible seam so it responds to phrase context and release state, not to invent a second sequencer or phrase-specific runtime model. A typed phrase-variation layer keeps the behavior replay-aligned, preserves the current queue/commit seam, and makes phrase behavior fixture-testable at both render-projection and callback levels.  
Evidence: `riotbox-audio` now defines an explicit `Tr909PhraseVariation` enum, `riotbox-app` derives phrase variation from transport phrase state and release-pattern context, the callback changes audible behavior from that variation layer, fixtures now cover release-tail cases, and the Jam/Log shell diagnostics surface the current phrase variation label.  
Consequences: later TR-909 work should continue extending the same typed render seam with richer musical behavior rather than bypassing it with direct callback heuristics, UI-only phrase modes, or a separate device-state graph.  
Status: accepted

---

Topic: the first MC-202 MVP control should be a committed role toggle on the existing queue seam
Phase: MC-202 MVP
Question: after the current TR-909 lane is stabilized, what is the smallest honest MC-202 entry slice that creates real device progress without opening a second control path or pretending the follower generator already exists?
Decision: start MC-202 with a bounded `mc202.set_role` action that toggles between `follower` and `leader` on the existing `ActionQueue` and `NextPhrase` commit seam. Surface the pending target in the Jam shell, update committed lane state plus a simple phrase reference on commit, and keep generation itself out of scope.
Why: Riotbox needs a real first MC-202 control, but the follower/answer generation path is not ready yet. A committed role toggle uses the existing replay-aligned action seam, makes the lane visible and queueable in the shell, and avoids inventing a parallel device-control path just to enter the milestone.
Evidence: `riotbox-app` now queues `mc202.set_role` as a phrase-boundary action, the commit path updates `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, `riotbox-core` exposes pending MC-202 role intent in `JamViewModel`, and shell tests cover both the keybinding and the pending-role cue.
Consequences: later MC-202 work should build follower/answer generation and live parameter control on top of the same committed role seam rather than bypassing it with direct UI-only state or a second lane model.
Status: accepted

---

Topic: the first MC-202 follower generator should stay phrase-quantized on the existing role seam
Phase: MC-202 MVP
Question: after committed role control exists, what is the smallest honest next slice that creates a usable MC-202 follower-line path without pretending a full synth engine, phrase editor, or answer generator already exists?
Decision: add a bounded `mc202.generate_follower` action on the existing `ActionQueue` and `NextPhrase` commit seam. Surface pending follower generation in the Jam shell, commit it into `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, and keep deeper answer logic and live parameter editing out of scope.
Why: Phase 4 needs a real follower-line path, not just another role toggle. Reusing the current phrase-boundary seam keeps generation replay-safe and visible, while committed lane-state updates make the MC-202 lane feel real without inventing a second phrase engine or UI-only device model.
Evidence: `riotbox-app` now queues `mc202.generate_follower` as a phrase-boundary action, the commit path writes a follower-oriented phrase reference plus touch intensity into session state, `riotbox-core` exposes pending follower generation in `JamViewModel`, and shell tests cover both the new keybinding and the pending-generation cue.
Consequences: later MC-202 work should extend the same committed phrase seam with answer generation and parameter control rather than bypassing it with direct shell state, callback-only heuristics, or a separate MC-202 runtime graph.
Status: accepted

---

Topic: the first deeper MC-202 shell slice should improve lane diagnostics instead of opening a synth inspector
Phase: MC-202 MVP
Question: after the follower-generation action exists, what is the next bounded shell slice that makes the MC-202 lane more legible without inventing a dedicated device pane or second diagnostic surface?
Decision: deepen the existing `Jam` and `Log` screens with clearer MC-202 lane diagnostics. Keep the slice read-only on top of committed lane state, pending-action intent, and the existing action log, and avoid any new device-specific editor or inspector route.
Why: once follower generation exists, users need to see what the MC-202 lane believes it is doing. The shell should answer that directly in the normal operator surfaces instead of forcing a second panel or hidden debug path.
Evidence: `riotbox-app` now shows richer MC-202 lane summaries in `Jam`, adds a dedicated `MC-202 Lane` diagnostics panel in `Log`, and keeps the diagnostics grounded in existing `JamViewModel` fields plus the action log. The footer also now advertises the follower-generation key so visible controls match the actual shell behavior.
Consequences: later MC-202 work should keep deepening these same operator surfaces instead of opening a second synth inspector or a parallel device-debug workflow.
Status: accepted

---

Topic: the first MC-202 answer generator should reuse the existing phrase seam instead of opening a second phrase engine
Phase: MC-202 MVP
Question: after committed follower generation exists, what is the next bounded MC-202 slice that deepens phrase interplay without pretending a full answer editor, callback-side sequencer, or hidden phrase graph already exists?
Decision: add a bounded `mc202.generate_answer` action on the existing `ActionQueue` and `NextPhrase` commit seam. Surface pending answer generation in the Jam shell, commit it into `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`, and keep deeper phrase editing and live synth control out of scope.
Why: the current MC-202 lane already has a replay-safe phrase seam and committed lane-state model. Extending that seam with answer generation creates a real next musical response without inventing a second phrase engine or a UI-only device model.
Evidence: `riotbox-app` now queues `mc202.generate_answer` on `NextPhrase`, commits it into answer-oriented lane state plus touch intensity, exposes the pending answer cue in the shell, and covers the path with fixture-backed regression tests alongside the existing role and follower cases.
Consequences: later MC-202 work should keep extending the same committed phrase seam, including richer answer behavior and parameter controls, instead of bypassing it with direct shell state, callback-only heuristics, or a separate MC-202 runtime graph.
Status: accepted

---

Topic: the first W-30 MVP slice should reuse the capture and promotion seam for live recall
Phase: W-30 MVP
Question: after capture, promotion, and pinning exist, what is the smallest honest W-30 entry slice that creates a real live-recall cue without inventing a second sample browser, pad editor, or playback-control surface?
Decision: start W-30 with a bounded live-recall cue on the existing `w30.swap_bank` action seam. Queue recall against the latest pinned promoted capture first, fall back to the latest promoted capture, commit it on `NextBar`, and update lane focus plus the capture reference on commit.
Why: Riotbox already has enough capture and promotion state to expose a truthful first W-30 cue. Reusing that seam keeps the entry slice replay-safe, visible in the Jam shell, and grounded in the current session model instead of opening a parallel W-30 control path too early.
Evidence: `riotbox-app` now queues `w30.swap_bank` as a live-recall cue, prefers pinned promoted captures for recall targeting, updates `w30.active_bank`, `w30.focused_pad`, and `w30.last_capture` on commit, and surfaces the pending recall in the Jam and Capture shell views. `riotbox-core` now carries W-30 focused-pad and pending-recall state in `JamViewModel`, and app/UI tests cover both the queue path and the committed side effects.
Consequences: later W-30 work should build audible audition, recall variations, and deeper pad handling on top of the same capture/promotion and committed recall seam instead of bypassing it with direct shell-only state or a separate W-30 browser model.
Status: accepted

---

Topic: promoted-material audition should stay on the existing W-30 pad cue seam instead of opening a second browser
Phase: W-30 MVP
Question: after live recall exists, what is the next bounded W-30 slice that makes promoted material feel more like an instrument without inventing a second sample browser, pad editor, or callback-only preview path?
Decision: add one explicit `w30.audition_promoted` action on the existing `ActionQueue` and `NextBar` seam. Queue it against the latest promoted W-30 capture, block conflicting pending W-30 pad cues, surface the pending audition target in the shared shell summary, and commit it into the same lane focus seam plus a bounded `w30_grit` bump.
Why: the repo already has a real capture, promotion, and live-recall seam. Extending that seam with promoted-material audition deepens the W-30 lane musically while keeping the slice replay-safe and visible in the current shell instead of inventing a second W-30 model.
Evidence: `riotbox-app` now queues `w30.audition_promoted` on `NextBar`, commits it through the same W-30 side-effect path that updates bank, pad focus, and last-capture state, adds a bounded grit bump, and exposes the pending audition cue in the Jam and Capture shell views. Tests cover cue conflict blocking, committed side effects, and the shell-visible pending cue.
Consequences: later W-30 work should keep building audible preview and deeper pad behavior on top of the same committed cue seam instead of bypassing it with shell-only flags or a separate preview browser.
Status: accepted

---

Topic: live drum-bus control should stay on the existing mixer seam instead of opening a mixer page
Phase: Jam-First Playable Slice
Question: how should Riotbox close the current usability gap where the TR-909 render seam can be technically running but still silent because the drum bus is at zero?
Decision: add one bounded live drum-bus level control directly in the Jam shell and keep it on the existing persisted `mixer_state.drum_level` seam. Update the Jam and Log shell summaries from the same render-mix projection and avoid opening a second mixer page or a parallel callback-side control model.
Why: the repo already has the right seam: persisted mixer state, app-derived render summaries, and a running audio path. A small live control makes the current render seam audibly testable by ear without widening scope into a full mixer surface.
Evidence: `riotbox-app` now adjusts `session.runtime_state.mixer_state.drum_level` live from the shell, refreshes the same `tr909_render_mix_summary`, and keeps the render warning active when the drum bus reaches zero. Tests cover mixer-state adjustment and the new shell keybindings.
Consequences: later mixer work should deepen the same session/runtime seam with clearer controls and diagnostics rather than bypassing it with a second page or callback-only volume state.
Status: accepted

---

Topic: TR-909 scene-lock variation should reuse the existing takeover seam instead of adding a second editor path
Phase: TR-909 MVP
Question: after takeover, release, pattern adoption, and phrase variation exist, what is the next bounded TR-909 control that deepens the lane musically without opening a second TR-909 editor or phrase engine?
Decision: add one explicit `tr909.scene_lock` action on the existing `ActionQueue` and `NextPhrase` commit seam. Commit it into the same takeover lane state already used by the audio-facing render projection by setting `takeover_enabled`, `takeover_profile`, `pattern_ref`, and `reinforcement_mode`, and surface the pending profile in the Jam shell instead of creating a second TR-909 control model.
Why: the codebase already had a typed `scene_lock` render profile and fixture coverage, but no honest committed control could reach it. A bounded scene-lock action deepens the current TR-909 MVP through the same replay-safe seam that already drives takeover and release, while avoiding a hidden render-only mode or a separate device editor path.
Evidence: `riotbox-core` now treats `tr909.scene_lock` as part of the canonical action vocabulary, `riotbox-app` queues it on `NextPhrase` with the same pending-guard used by takeover and release, committed side effects drive `scene_lock_takeover` lane state and render projection, and Jam-shell key handling plus tests cover the new pending-profile and committed scene-lock path.
Consequences: later TR-909 work should keep extending the same committed takeover seam, including richer scene-lock behavior, instead of bypassing it with callback-only toggles or a separate TR-909 variation editor.
Status: accepted

---

Topic: W-30 diagnostics should deepen the existing Capture and Log screens instead of opening a second control surface
Phase: W-30 MVP
Question: after live recall and promoted-material audition exist, what is the smallest next slice that makes the W-30 lane legible for operators without inventing a separate W-30 page, browser, or hidden preview state?
Decision: surface bounded W-30 diagnostics in the existing `Capture` and `Log` screens. Use the current `JamViewModel`, session capture inventory, and committed action log to show pending cue kind, focused bank/pad, latest promoted capture, last lane capture, and the most recent committed W-30 cue outcome.
Why: the repo already has a truthful W-30 seam for recall and audition, but the shell still hides too much of that state inside generic action history. The next honest step is to make the current seam explain itself in-place, not to open a second W-30 control surface or a preview-only browser.
Evidence: `riotbox-app` now adds a dedicated `W-30 Lane` diagnostics panel to `Log`, deepens `Capture -> Routing / Promotion` with explicit pending cue and promoted-target context, and covers the new shell cues with snapshot-style tests for queued and committed W-30 states.
Consequences: later W-30 work should continue extending these same screens and the existing committed cue seam for audible preview and deeper pad behavior instead of bypassing them with separate shell-only panels or hidden preview state.
Status: accepted

---

Topic: W-30 MVP should gain shared replay-safe regression fixtures before deeper audio-facing preview work
Phase: W-30 MVP
Question: after live recall, promoted-material audition, and shell diagnostics exist, what is the smallest next slice that hardens the current W-30 lane before it grows into an audio-facing preview seam?
Decision: add one shared W-30 regression fixture corpus in `riotbox-app` and reuse it in both committed-state and shell-visible tests. Cover the shipped `live recall` and `promoted audition` cues, assert committed lane state plus result summaries at the app layer, and assert Capture/Log shell output from the same fixture data.
Why: TR-909 and MC-202 already use fixture-backed regressions to keep the current seam replay-safe while the device lane grows. W-30 needed the same verification net before deeper preview or pad behavior could be added honestly.
Evidence: `riotbox-app` now has `w30_regression.json`, fixture-backed committed-state regressions in `jam_app`, and fixture-backed shell regressions in `ui` for both recall and promoted-audition paths.
Consequences: later W-30 work should extend the same fixture corpus when preview render state or deeper pad behavior lands instead of relying only on ad hoc unit tests or manual shell checks.
Status: accepted

---

Topic: W-30 audio-facing preview should start as one typed render seam instead of direct sample playback
Phase: W-30 MVP
Question: after live recall, promoted-material audition, diagnostics, and replay-safe regression coverage exist, what is the smallest next slice that prepares the W-30 lane for later audible preview without opening a second device model or pretending full sample playback is already solved?
Decision: add one typed `W30PreviewRenderState` in `riotbox-audio`, derive it only from the existing committed session and action seam in `riotbox-app`, and mirror it into the audio runtime alongside the existing TR-909 render state. Surface the preview mode, profile, target, and mix summary in the current shell/runtime summaries, but stop short of real W-30 sample playback in this slice.
Why: Riotbox needed an honest audio-facing seam for the W-30 lane before pads become audible or internally resampled. The smallest correct move is to make the preview state explicit and callback-reachable using the same replay-safe lane, capture, and action log state that already drives recall and audition, instead of inventing a hidden preview-only model or jumping straight to sample rendering.
Evidence: `riotbox-audio` now has a dedicated `w30` render-state module and shared runtime storage, `AudioRuntimeShell` updates that preview state on the same path as TR-909 render updates, and `riotbox-app` derives typed preview mode/profile/routing from committed W-30 state and exposes it through `JamRuntimeView` plus shell regressions.
Consequences: later W-30 work should attach audible preview rendering, pad playback, and resample taps to this same typed preview seam rather than bypassing it with ad hoc callback state or a separate W-30 playback path.
Status: accepted

---

Topic: W-30 audible preview should deepen the typed preview seam and keep fresh ingest sessions audibly reachable
Phase: W-30 MVP
Question: once the typed W-30 preview seam exists, what is the smallest honest slice that makes it audibly testable without bypassing the committed render model or opening a second playback path?
Decision: keep W-30 preview audio on the same `W30PreviewRenderState` seam and render it inside the existing audio callback alongside TR-909, using one bounded lo-fi preview synth that responds to preview mode, source profile, grit, transport, and music-bus level. Also open the music bus to a modest default level for fresh ingest sessions so the new audible seam is reachable in normal app launches.
Why: Riotbox already had the right preview contract and callback plumbing, but the W-30 lane was still silent even when preview state was active. The smallest correct move is to make that seam audibly real in place, not to add a separate W-30 player, hidden callback-only state, or a one-off shell preview path. Giving fresh ingest sessions a nonzero music bus keeps the new seam practically testable instead of leaving it gated behind an implicit zero-level default.
Evidence: `riotbox-audio` now mixes a dedicated W-30 preview renderer in the existing output callback, covers live recall, promoted audition, zero-music silence, and stopped-preview audibility with runtime tests, and `riotbox-app` initializes fresh ingest sessions with a nonzero `mixer_state.music_level` so committed W-30 preview work can actually be heard.
Consequences: later W-30 work should keep extending this same seam with richer preview profiles, real pad playback, and deeper diagnostics, and should treat music-bus defaults and controls as part of the same mixer path instead of inventing a second W-30-only gain model.
Status: accepted

---

Topic: Playable W-30 pad hits should use a committed trigger action on the existing preview seam
Phase: W-30 MVP
Question: after live recall, promoted audition, and audible preview exist, what is the smallest next slice that makes the W-30 lane feel playable without inventing a second playback path or bypassing replay-safe action state?
Decision: add one explicit `w30.trigger_pad` action quantized to `next_beat`, keep it on the same committed W-30 lane and action log seam as recall and audition, and carry the trigger through the existing `W30PreviewRenderState` using a monotonic trigger revision plus trigger velocity so the audio callback can retrigger the current pad accent in place.
Why: Riotbox needed a first playable W-30 control, but the existing architecture already had the right lane focus, capture selection, and preview callback seam. The smallest honest move is to commit a trigger action and let the current preview renderer retrigger from that committed state, instead of adding a hidden one-shot player, direct shell-to-audio trigger, or a second W-30 engine.
Evidence: `riotbox-core` now exposes `w30.trigger_pad`, `riotbox-app` queues it on `next_beat`, carries pending trigger cues in the Jam view, commits it into lane state plus result summaries, and derives trigger revision and velocity in `W30PreviewRenderState`. `riotbox-audio` now retriggers the current W-30 preview accent when that revision changes, with app, UI, and runtime regressions covering the new path.
Consequences: later W-30 pad features should keep using this committed trigger seam for replay-safe one-shot behavior, and should extend the existing preview render state instead of creating direct callback-only trigger plumbing.
Status: accepted

---

Topic: W-30 audible preview diagnostics should stay inside the existing Jam and Log shell surfaces
Phase: W-30 MVP
Question: once W-30 preview is audible and the first playable trigger exists, what is the smallest next slice that makes that preview state legible without opening a separate W-30 control page?
Decision: keep W-30 audible preview diagnostics inside the existing `Jam` and `Log` shell surfaces, and derive compact mode, target, mix, and trigger cues from `JamRuntimeView` plus the committed lane state instead of creating a dedicated W-30 browser or a second preview-only panel hierarchy.
Why: the runtime seam already exposes the information needed to understand what the audible preview path is doing. The smallest honest move is to summarize that existing seam where operators already look, not to add a second surface that would drift from the committed queue and render model.
Evidence: `JamRuntimeView` now carries an explicit W-30 trigger summary on top of the existing preview mode, target, and mix summaries; the Jam shell now surfaces those cues inline in the main lane overview; the Log screen deepens the `W-30 Lane` panel with the same audible preview state; and the normalized review artifact at `docs/screenshots/w30_audible_preview_baseline.txt` records the result.
Consequences: later W-30 diagnostics should keep extending these same shell summaries and the current typed preview seam instead of splitting preview interpretation across a second W-30-only surface or a callback-only debug overlay.
Status: accepted

---

Topic: W-30 internal resample taps should extend explicit capture lineage instead of inventing a second capture system
Phase: W-30 MVP
Question: after audible preview, pad triggering, and shell diagnostics exist, what is the smallest next slice that prepares internal W-30 resample taps without pretending the full resample lab already exists?
Decision: add one typed `W30ResampleTapState` on top of the current W-30 runtime seam and derive it only from explicit `CaptureRef` lineage metadata plus the committed W-30 lane focus. Extend `CaptureRef` with `lineage_capture_refs` and `resample_generation_depth`, default new captures to generation zero, and keep the first shell proof point inside the existing capture-oriented shell summaries instead of opening a separate resample page.
Why: Riotbox already has real capture records, promotion, audible preview, and a typed W-30 runtime seam. The smallest honest preparation step for internal resample taps is to make capture-to-capture lineage explicit and mirror one tap-ready state through that same seam, not to open a second capture inventory, a hidden preview-only resample model, or a full resample-lab UI.
Evidence: `riotbox-core` now persists explicit capture lineage metadata, `riotbox-audio` now carries a typed `W30ResampleTapState`, `riotbox-app` derives that tap-ready state from the current lane capture and runtime mix context, and the shell baseline at `docs/screenshots/w30_resample_tap_baseline.txt` records the first compact tap cue on the existing Capture surface.
Consequences: later W-30 resample actions and internal bus taps should populate the same lineage fields, deepen the same tap state, and keep using the existing shell surfaces instead of bypassing session capture history or creating a second W-30 resample runtime.
Status: accepted

---

Topic: the first W-30 internal resample action should stay on the committed capture lineage seam
Phase: W-30 MVP
Question: once internal resample lineage metadata and tap-ready state exist, what is the smallest next slice that makes W-30 internal resampling feel real without opening a second capture inventory, pad editor, or callback-only resample trigger?
Decision: add one explicit `promote.resample` action on the existing `ActionQueue` and `NextPhrase` seam for the W-30 lane. Queue it only against the current committed W-30 lane capture, block duplicate pending W-30 resample actions, and materialize the committed result as a new `CaptureRef` with explicit lineage and incremented `resample_generation_depth`.
Why: the repo already has the right ingredients for internal resampling: committed W-30 lane focus, explicit capture lineage metadata, and a typed resample tap summary. The smallest honest move is to create one replay-safe committed resample action on top of that same seam, not to invent a second capture browser, a hidden callback-only resample path, or a full W-30 lab before the lineage model is exercised for real.
Evidence: `riotbox-app` now queues `promote.resample` against the current `w30.last_capture`, commits it on `NextPhrase`, creates a new `CaptureRef` with cloned source-origin refs plus extended lineage and generation depth, updates the W-30 lane to point at the new capture, and surfaces the pending cue in the current shell flow. App and shell tests cover the queue path, duplicate blocking, committed lineage materialization, and the capture-screen cue.
Consequences: later W-30 internal buses, pad-bank behavior, and deeper resample tooling should continue extending this committed lineage seam instead of bypassing it with hidden resample state or direct callback-only mutation.
Status: accepted

---

Topic: W-30 resample lineage diagnostics should stay in the existing Capture and Log shell surfaces
Phase: W-30 MVP
Question: once the first committed W-30 internal resample action exists, what is the smallest next slice that keeps lineage provenance legible without opening a second W-30 diagnostics page or resample-only browser?
Decision: deepen the existing `Capture` and `Log` surfaces with compact lineage diagnostics derived from the committed lane capture and the typed resample-tap seam. Surface pending resample intent explicitly, show compact generation and lineage counts in the W-30 log lane, and keep fuller lineage-chain context in the capture routing panel instead of creating a separate resample screen.
Why: the repo already has truthful resample state, but after `RIOTBOX-65` operators still had to infer too much lineage from generic capture summaries. The smallest honest move is to summarize the existing committed seam where users already look, not to split W-30 provenance across a second diagnostics hierarchy that could drift from the queue and runtime state.
Evidence: `riotbox-app` now shows pending `promote.resample` intent in the W-30 shell cue, renders compact tap and lineage summaries in `Capture -> Routing / Promotion`, compresses generation and lineage counts into the `Log -> W-30 Lane` panel, and covers the new wording with capture and log shell regressions.
Consequences: later W-30 lab work should keep extending these same shell surfaces and the current committed lineage seam instead of moving provenance into a second W-30-only browser or a callback-only debug path.
Status: accepted

---

Topic: W-30 capture resolution should follow committed lane focus before pad-bank stepping lands
Phase: W-30 MVP
Question: after the periodic codebase review flagged capture-driven W-30 helpers, what is the smallest correction that keeps later bank-step and trigger work honest on the current preview seam?
Decision: when explicit `w30.active_bank` and `w30.focused_pad` exist, resolve W-30 recall, audition, trigger, and internal resample actions from the latest committed capture assigned to that focused pad. Only fall back to the older latest-capture or latest-promoted heuristics when no explicit lane focus exists.
Why: bank-step work becomes partly cosmetic if committed focus can move without changing the capture chosen by recall, audition, trigger, or resample actions. The smallest honest fix is to make the existing helpers respect committed lane focus first, not to invent a second W-30 selection model or defer the inconsistency until after more pad-bank controls land.
Evidence: `riotbox-app` now resolves focused W-30 captures before queueing recall, audition, trigger, and resample actions, and the regression tests explicitly cover focused-bank capture selection instead of the older latest-promoted-only behavior.
Consequences: later pad-bank stepping should update committed lane focus and rely on the same resolver rather than choosing captures through separate shell-only or queue-only heuristics.
Status: accepted

---

Topic: W-30 preview mode should be explicit committed lane state rather than action-log reconstruction
Phase: W-30 MVP
Question: after the periodic codebase review flagged preview-mode reconstruction from `action_log`, what is the smallest correction that keeps the audible preview seam deterministic and replay-safe?
Decision: persist explicit W-30 preview intent in `runtime_state.lane_state.w30.preview_mode`, update it from committed W-30 preview-facing actions, and build the runtime preview seam from that explicit state. Keep a one-time legacy backfill from committed W-30 preview actions only when loading older sessions that do not yet carry the explicit field.
Why: the preview seam should depend on one committed source of truth, not on whichever W-30 action happens to be latest in the historical log. Making preview intent explicit keeps later pad-bank stepping and lane-focus work honest, while the one-time backfill preserves compatibility for older saved sessions without leaving the runtime builder tied to history forever.
Evidence: `riotbox-core` now persists explicit W-30 preview intent in lane state, `riotbox-app` updates it during committed W-30 side effects, and regression tests cover both legacy backfill and the rule that explicit lane state overrides stale action history.
Consequences: future W-30 controls should update `preview_mode` through committed lane state instead of teaching the runtime builder about more action-log patterns. Replay and restore semantics stay explicit even as more W-30 controls land.
Status: accepted

---

Topic: Pending W-30 resample cues should enter the shell through the core Jam view model
Phase: W-30 MVP
Question: after the periodic codebase review flagged a shell-side queue scan for pending W-30 resample intent, what is the smallest correction that restores the presentation boundary without creating a second W-30 summary path?
Decision: extend `JamViewModel.lanes` with explicit pending W-30 resample cue data and have the shell derive its cue label from that core presentation contract instead of scanning `ActionQueue` directly.
Why: the shell already receives pending MC-202, TR-909, recall, audition, and trigger summaries through the core Jam view model. Leaving pending W-30 resample intent as a direct queue scan would keep one small but real boundary leak alive, making later shell work easier to drift into ad hoc queue inspection.
Evidence: `riotbox-core` now surfaces `w30_pending_resample_capture_id` in `LaneSummaryView`, the Jam-view regression fixture covers that new field, and `riotbox-app` no longer scans `ActionQueue` directly for the W-30 resample cue label.
Consequences: future shell summaries should keep extending the core Jam view contract rather than introducing new queue reads inside `ui.rs`. Queue-level state remains modeled centrally before it is rendered.
Status: accepted

---

Topic: first bounded W-30 pad-bank stepping should use an explicit committed focus-step action on the preview seam
Phase: W-30 MVP
Question: once committed W-30 lane focus and preview mode are explicit, what is the smallest next slice that lets operators step across promoted pads without inventing a second shell cursor, pad editor, or preview-only state machine?
Decision: add one explicit `w30.step_focus` action on the existing `ActionQueue` and `NextBeat` seam for the W-30 lane. Resolve its target from the next promoted W-30 pad after the current committed lane focus, block it against other pending W-30 pad cues, and update committed W-30 lane focus plus the existing preview seam when it lands.
Why: stepping should be a real committed musical control, not a shell-only cursor move disguised as recall. A dedicated `w30.step_focus` action keeps pending cues, recent action summaries, and replay history honest while still staying bounded to the current W-30 preview model instead of opening a full pad-bank editor early.
Evidence: `riotbox-core` now exposes `w30.step_focus` explicitly, the Jam view surfaces pending focus-step targets, `riotbox-app` queues it on `NextBeat` from actual promoted W-30 pads, the shell binds it directly, and the app/UI regressions cover both pending and committed focus-step behavior.
Consequences: later W-30 bank-grid work should continue extending this explicit committed focus seam rather than smuggling pad stepping through recall semantics or a separate shell-only focus cursor.
Status: accepted

---

Topic: W-30 live recall should stop overloading the bank-swap action name before bank-manager controls land
Phase: W-30 MVP
Question: once committed focus-step behavior exists, what is the smallest honest cleanup that keeps later bank-manager work from inheriting misleading W-30 action semantics?
Decision: split the existing live-recall behavior onto an explicit `w30.live_recall` action command and reserve `w30.swap_bank` for future real bank-manager movement. Keep the queue target resolution, `NextBar` quantization, committed preview behavior, and focused-pad side effects otherwise unchanged.
Why: the repo already moved pad stepping onto its own committed `w30.step_focus` seam. Leaving live recall on `w30.swap_bank` would make action history misleading and would force the first actual bank-manager slice to either reuse a dishonest command name or create another workaround around the same seam.
Evidence: `riotbox-core` now exposes `w30.live_recall`, the Jam view uses it for pending recall summaries, `riotbox-app` queues recall with that explicit command while preserving the existing recall targeting logic and committed side effects, and the shell baselines plus queue/commit tests were updated to show `w30.live_recall` instead of `w30.swap_bank` for recall cues.
Consequences: later W-30 bank-manager work can now use `w30.swap_bank` for real bank changes without rewriting old recall history again. The current live-recall seam stays replay-safe, but its action log and shell labels are now honest about what the slice actually does.
Status: accepted

---

Topic: first bounded W-30 bank-manager control should swap committed focus across promoted banks without opening a second bank editor
Phase: W-30 MVP
Question: now that live recall has its own explicit action seam, what is the smallest next slice that turns `w30.swap_bank` into a real bank-manager control while staying on the existing preview and commit boundaries?
Decision: use `w30.swap_bank` as a `NextBar` control that rotates to the next promoted W-30 bank, preserves the current focused pad when that pad exists in the target bank, falls back to the first promoted pad in that bank otherwise, and commits the same W-30 preview-facing lane updates through the existing focused-pad seam.
Why: the current W-30 MVP already has explicit committed focus, preview mode, and live recall semantics. A first real bank swap should therefore be a bounded movement across existing promoted banks, not a new shell-only bank cursor or a second bank-manager state machine. Reusing the committed focus seam keeps action history honest and lets later bank-grid work refine the same path instead of replacing it.
Evidence: `riotbox-app` now resolves `w30.swap_bank` from actual promoted W-30 targets, queues it on `NextBar`, blocks it against other pending W-30 pad cues, updates lane focus plus the last capture on commit, and distinguishes pending bank cues from recall cues in the shell. `riotbox-core` now carries a dedicated `w30_pending_bank_swap_target`, and queue/commit plus shell regressions cover both the pending and committed bank-swap behavior.
Consequences: later W-30 bank-manager work should keep extending this explicit committed bank-swap seam rather than inventing a separate bank navigation surface. The current slice stays bounded to promoted-bank rotation only; full bank-grid editing, empty-bank travel, and deeper pad-forge controls remain out of scope.
Status: accepted

---

Topic: first bounded W-30 pad-forge control should apply one explicit damage profile on the current preview seam
Phase: W-30 MVP
Question: once committed W-30 trigger, recall, bank-swap, and internal resample behavior exist, what is the smallest honest first step toward pad-forge behavior without introducing per-pad forge state or a second W-30 editor?
Decision: implement `w30.apply_damage_profile` as one bounded `NextBar` control on the current focused W-30 capture seam. It targets the current preview-facing pad capture, reuses the existing committed bank and pad focus path, and raises the existing `w30_grit` macro to one explicit `shred` profile level instead of inventing a full per-pad damage model yet.
Why: the repo already has one replay-safe W-30 preview seam with explicit bank, pad, capture, and grit state flowing into the audio runtime. A first pad-forge move should deepen that seam instead of bypassing it with a hidden forge editor, a callback-only grit toggle, or a prematurely detailed damage-profile schema that the current session model cannot yet persist honestly.
Evidence: `riotbox-app` now queues `w30.apply_damage_profile` on `NextBar`, resolves it from the current W-30 targetable capture, blocks it against other pending W-30 pad cues, preserves the current preview mode while raising committed `w30_grit`, and records an explicit damage-profile result summary on commit. `riotbox-core` now surfaces pending damage-profile targets in `JamViewModel`, and queue, commit, and shell regressions cover both the pending cue and the committed grit update.
Consequences: later W-30 pad-forge work should refine this same committed damage seam instead of introducing a separate forge state machine. The current slice remains intentionally bounded to one `shred` profile and one global grit macro; per-pad forge persistence, multiple named damage profiles, and deeper bank-grid editing remain out of scope.
Status: accepted

---

Topic: W-30 bank-manager and pad-forge follow-ups should deepen the current shell diagnostics instead of opening a second diagnostics surface
Phase: W-30 MVP
Question: once `w30.swap_bank` and `w30.apply_damage_profile` exist on the committed preview seam, how should the shell expose them without regressing the current W-30 capture, lineage, and preview diagnostics?
Decision: keep the slice presentation-only and make the existing `Jam`, `Capture`, and `Log` surfaces carry explicit bank-manager and pad-forge diagnostics. Show the bank-manager state as one compact status next to the current pending cue, show pad-forge state next to the current W-30 mix and capture cues, and compress the Log/Capture wording enough that older lineage and trigger diagnostics stay visible in the same panels.
Why: the repo already has one honest W-30 shell spine. A diagnostics follow-up should deepen that spine instead of adding a separate W-30 debug page or a second forge-specific surface. The main risk in this slice is not missing state, but crowding the fixed terminal layout enough to hide older preview and lineage cues, so the shell wording needs to stay compact and explicit.
Evidence: `riotbox-app` now surfaces explicit bank-manager and pad-forge diagnostics in the `Jam` lane summary, the `Capture -> Routing / Promotion` panel, and the `Log -> W-30 Lane` panel. New shell regressions cover committed bank-swap plus damage-profile state, while existing W-30 shell regressions still pass after the wording compaction. The review artifact at `docs/screenshots/w30_bank_forge_diagnostics_baseline.txt` records the updated shell cues.
Consequences: later W-30 shell work should keep extending the same Jam/Capture/Log surfaces unless the roadmap explicitly calls for a new operator surface. Deeper W-30 forge behavior remains out of scope here; the slice only makes the current committed bank-manager and pad-forge moves legible.
Status: accepted

---

Topic: W-30 bank-manager and pad-forge hardening should extend the shared W-30 regression corpus instead of creating a second fixture path
Phase: W-30 MVP
Question: once `w30.swap_bank` and `w30.apply_damage_profile` have shipped on the committed preview seam, how should the repo harden them without fragmenting the current W-30 regression story?
Decision: extend the existing `w30_regression.json` corpus so the new bank-manager and pad-forge controls use the same fixture-backed committed-state and shell regression path as live recall and promoted audition. Add only the extra fixture metadata needed to express multi-bank setup and initial W-30 preview state, and keep the slice verification-only.
Why: the W-30 MVP already has one honest replay-safe regression seam for committed app state and shell output. Bank swap and damage profile are the same class of committed preview-lane actions, so giving them a separate fixture file or a second one-off test harness would create drift in the repo’s verification model rather than just widening the existing safety net.
Evidence: `riotbox-app` now covers `live_recall`, `promoted_audition`, `swap_bank`, and `apply_damage_profile` from the shared `w30_regression.json` corpus. The same corpus now drives committed-state assertions in `jam_app` and shell-visible assertions in `ui`, including the new bank-manager and pad-forge diagnostics shipped in `RIOTBOX-75`.
Consequences: later W-30 controls on the same committed preview seam should keep extending the shared fixture corpus unless they require genuinely new runtime dimensions. The slice remains intentionally verification-only and does not change shipped W-30 behavior.
Status: accepted

---

Topic: W-30 internal resample taps should become audibly real on the existing audio callback instead of staying diagnostics-only
Phase: W-30 MVP
Question: once the app/runtime seam already derives a typed `w30_resample_tap` state, what is the smallest honest next step that makes it audible without introducing a second W-30 render path?
Decision: route the existing `W30ResampleTapState` through the same realtime callback that already mixes TR-909 support and W-30 preview. Keep the slice bounded to one synthetic internal-capture tap voice that reacts to source profile, lineage depth, generation depth, grit, transport-running state, and music-bus level, and verify it with direct audio callback tests instead of opening a second fixture harness yet.
Why: the repo already had explicit app/runtime diagnostics for internal resample lineage, but the audio runtime still ignored that state. Making the current seam audibly real is the smallest step that turns the typed W-30 resample path into product-visible behavior while preserving the one-callback audio architecture and avoiding a hidden W-30-only render loop.
Evidence: `riotbox-audio` now threads shared resample-tap state into `build_silent_output_stream`, snapshots it in the callback, and mixes a bounded `render_w30_resample_tap_buffer(...)` voice next to the existing TR-909 and W-30 preview paths. Direct runtime tests now cover idle silence, audible lineage-ready taps, and zero-music-bus silence for the new seam, and the full repo verification loop stays green.
Consequences: later W-30 resample-lab work should keep extending this same callback seam instead of creating a parallel internal-resample renderer. Richer profile fixtures, shell diagnostics, and loop-freezer reuse cues remain follow-up work and should build on the same typed runtime state.
Status: accepted

---

Topic: W-30 resample-lab diagnostics should stay in the existing Jam, Capture, and Log shell spine after the audible seam lands
Phase: W-30 MVP
Question: once internal resample taps are audibly real on the callback seam, how should the shell expose that state without opening a second W-30 lab page or regressing older preview, bank-manager, and pad-forge cues?
Decision: keep the slice presentation-only and deepen the existing shell spine with compact resample-lab diagnostics. Jam now shows one `tap` summary next to the current W-30 mix line, Capture now shows source, route, mix, and lineage for the current resample tap, and Log now uses a compressed resample-lab line pair that fits the existing W-30 lane panel width.
Why: the repo already has one honest W-30 operator surface. After `RIOTBOX-77`, the risk is not missing state but leaving the audible resample seam hard to read unless operators inspect generic action history or source lineage by hand. Extending the current Jam/Capture/Log path keeps the shell aligned with the one existing W-30 runtime seam instead of adding another diagnostics surface.
Evidence: `riotbox-app` now renders explicit resample-tap summaries across Jam, Capture, and Log, the shell regressions cover both committed lineage diagnostics and a cross-surface resample-lab snapshot, and the normalized artifact at `docs/screenshots/w30_resample_lab_diagnostics_baseline.txt` records the expected cues.
Consequences: later W-30 resample work should keep extending these same shell surfaces unless the roadmap explicitly calls for a separate operator page. The current slice remains presentation-only and does not change the shipped audio behavior from `RIOTBOX-77`.
Status: accepted

---

Topic: audible W-30 internal resample taps should use the same fixture-backed callback regression pattern as the other shipped audio seams
Phase: W-30 MVP
Question: once the resample-tap callback path is audibly real, how should the repo harden it without inventing a one-off verification style?
Decision: add a dedicated `w30_resample_audio_regression.json` fixture corpus and a callback-level test that evaluates the same active-sample and peak bounds already used for TR-909 and W-30 preview. Keep the slice verification-only and leave the shipped render behavior unchanged.
Why: the repo already has one honest audio-regression pattern for callback-visible behavior. Leaving W-30 internal resample taps on direct one-off tests alone would make the newest audible seam easier to drift than the older TR-909 and W-30 preview paths. Extending the existing fixture shape is the smallest consistent hardening move.
Evidence: `riotbox-audio` now parses `w30_resample_audio_regression.json`, maps fixture rows into `RealtimeW30ResampleTapState`, and verifies idle silence, transport-running lineage-ready taps, stopped-tap audibility, and zero-music-bus silence through the same active-sample and peak assertions used elsewhere.
Consequences: later W-30 audio callback work should keep widening the shared fixture-backed regression net instead of adding seam-specific harnesses. This slice changes no runtime behavior; it only makes future drift on the audible resample seam easier to catch.
Status: accepted

---

Topic: first W-30 loop-freezer reuse should stay on the existing capture seam instead of opening a second reuse editor path
Phase: W-30 MVP
Question: once W-30 preview, bank-manager, damage-profile, and internal-resample seams already exist, what is the smallest honest way to let operators freeze and reuse a loop without inventing a parallel W-30 editor flow?
Decision: add one bounded `w30.loop_freeze` action on the current W-30 capture seam. Queue it on `NextPhrase`, reuse the currently committed W-30 capture target, materialize exactly one new pinned capture on commit, preserve capture lineage through explicit `lineage_capture_refs`, and keep the same W-30 preview/runtime path after commit.
Why: the repo already has one replay-safe W-30 capture and preview seam. A first freezer cue should deepen that seam instead of creating a second “reuse lab” path with separate persistence, routing, or preview rules. The main risk in this slice is lineage drift, not missing UI surface, so the action needs to leave reuse explicit in the same capture model and shell surfaces.
Evidence: `riotbox-core` now exposes `w30.loop_freeze` in the action lexicon and Jam view, while `riotbox-app` queues it on the existing W-30 lane, commits it into a new pinned capture with preserved lineage, and surfaces the pending/committed freeze cues in `Jam`, `Capture`, and `Log`. The shared `w30_regression.json` corpus now covers the committed freeze case for both app-state and shell regressions.
Consequences: later W-30 freezer and reuse work should keep extending the same capture lineage and preview seam unless the roadmap explicitly calls for a fuller editor workflow. Richer reuse browsing, loop editing, and multi-slot freeze management remain follow-up work and stay out of scope here.
Status: accepted

---

Topic: first W-30 slice-pool browse should stay on the current pad-lineage seam instead of opening a second browser model
Phase: W-30 MVP
Question: once loop-freeze reuse already leaves multiple captures on one W-30 pad target, what is the smallest honest next slice that lets operators step through that pool without inventing a separate slice browser, inventory model, or preview-only state?
Decision: add one bounded `w30.browse_slice_pool` action on the existing W-30 lane. Queue it on `NextBeat`, cycle through the captures already assigned to the currently focused W-30 bank/pad, commit it through the same preview-side-effect seam as live recall, and surface only a minimal pending cue in the existing shell.
Why: the repo already has one replay-safe W-30 capture and preview path. After loop-freeze, the immediate need is not a richer browser but a small way to move across the current pad’s committed reuse pool. Reusing the existing lane focus, `last_capture`, and preview-mode seam keeps the slice deterministic and visible without opening a shadow W-30 inventory architecture.
Evidence: `riotbox-core` now exposes `w30.browse_slice_pool` in the action lexicon and Jam view, while `riotbox-app` queues it against the current W-30 target, commits the next capture in that target’s assigned pool into `last_capture`, keeps preview mode on the existing live-recall path, and surfaces the pending browse cue in the shell. Queue and committed-state tests cover the bounded browse behavior.
Consequences: later W-30 slice-pool work should keep extending this same committed pad-lineage seam unless the roadmap explicitly calls for a fuller browse/editor workflow. Richer cross-pad slice browsing, preview profiling, and dedicated diagnostics remain follow-up slices.
Status: accepted

---

Topic: first W-30 slice-pool browse should project a distinct preview profile on the existing live-recall seam
Phase: W-30 MVP
Question: once slice-pool browse is committed on the current pad-lineage seam, what is the smallest honest next step that makes that consequence audible without inventing a second W-30 preview/editor mode?
Decision: keep browse on the existing `W30PreviewRenderMode::LiveRecall` seam and add one typed `W30PreviewSourceProfile::SlicePoolBrowse`. Derive it only from the last committed `w30.browse_slice_pool` action, surface it in the Jam shell as `recall/browse`, and give the audio callback one bounded browse-specific envelope/frequency pattern behind the same preview state.
Why: the current W-30 MVP already has one replay-safe preview seam and one committed pad-lineage model. A first browse consequence should deepen that seam instead of opening a parallel browser-preview architecture with separate persistence or routing rules. The real need is a distinct committed preview consequence, not a richer editor.
Evidence: `riotbox-app` now derives `slice_pool_browse` from committed browse history while keeping preview mode on `live_recall`, fixture-backed app and shell regressions cover the browse case, and `riotbox-audio` now encodes the new profile in shared state plus callback-level audio regressions to keep browse audibility distinct from normal promoted recall.
Consequences: later slice-pool work should continue extending the same committed preview seam unless the roadmap explicitly introduces a fuller browse/editor workflow. Richer pool visualization, cross-pad navigation, and deeper preview shaping remain bounded follow-up slices.
Status: accepted

---

## 4. Mandatory Research Topics

Topic: first Scene Brain slice should project deterministic scene candidates from source sections into existing session state
Phase: Scene Brain
Question: what is the smallest honest first Scene Brain step after W-30 MVP that creates multiple usable scene candidates without opening a second arrangement or scene-graph architecture?
Decision: derive deterministic scene candidate IDs from ordered `SourceGraph.sections`, store them in the existing session `scene_state.scenes`, and normalize `active_scene` plus transport `current_scene` onto that same committed state when no scene is already set. Keep the first slice bounded to candidate projection and the shell visibility that already exists.
Why: the current repo already has one explicit session scene state, one transport scene pointer, and Jam shell visibility for current scene and scene count. A first Scene Brain slice should deepen that spine instead of inventing a second scene graph or separate arrangement inventory before selection and transition semantics even exist.
Evidence: `riotbox-app` now derives scene candidates from analyzed section order during ingest and app-state normalization, and targeted tests prove both empty-session projection and persisted ingest state without changing the current queue or launch architecture.
Consequences: later Scene Brain work should build scene-select, launch, and restore behavior on the same session and transport state. Richer scene graphs, energy management, and transition logic remain follow-up slices.
Status: accepted

---

Topic: first Scene Brain selection should queue one committed `scene.launch` on the existing transport seam
Phase: Scene Brain
Question: once Riotbox already derives deterministic scene candidates, what is the smallest honest next step that lets the operator move to another scene without inventing a second arrangement path, transition model, or editor workflow?
Decision: add one bounded `scene.launch` action that cycles to the next committed scene candidate, queues on `NextBar`, commits through the existing action queue and transport boundary seam, and updates only the current session `active_scene` plus transport `current_scene`. Keep richer scene launch, restore, and transition logic out of scope.
Why: the repo already has one explicit scene list, one transport scene pointer, and one replay-safe queue and commit model. The next honest move is not a richer scene editor but a single committed scene-select control that proves scene changes can stay explicit, logged, and replay-safe on the existing seam.
Evidence: `riotbox-app` now queues `scene.launch` for the next candidate, blocks duplicate pending scene launches, commits it on the current bar boundary, and updates both session scene state and transport scene state with targeted regression coverage plus a minimal shell key.
Consequences: later Scene Brain work should continue from this same queueable scene-launch seam when adding restore, recovery, or richer transition semantics. Selection UIs, scene diagnostics, and transition policies remain follow-up slices.
Status: accepted

---

Topic: early Scene Brain diagnostics should stay in the existing Jam and Log shell spine
Phase: Scene Brain
Question: once Riotbox has deterministic scene candidates and one committed `scene.launch` seam, what is the smallest honest next step that makes that state legible to the operator without opening a second scene page or a shell-only scene model?
Decision: surface active scene, next scene candidate, pending scene launch, and committed transport-scene context directly in the existing `Jam` overview and `Log` summary panels. Keep the slice presentation-only on top of the shipped app and runtime seam.
Why: the repo already keeps TR-909, MC-202, and W-30 seams visible inside the current shell spine. Scene Brain should become legible the same way before introducing richer scene launch, restore, or transition controls. A first diagnostic slice should deepen the current shell, not create a separate scene browser or debug page.
Evidence: `riotbox-app` now shows active scene plus next-candidate context in the Jam overview, folds scene state into the existing Log summary without adding a new page or panel family, and covers the new shell state with focused scene-diagnostic snapshot tests.
Consequences: later Scene Brain work should keep extending the same shell surfaces unless the roadmap explicitly calls for a fuller scene page. Replay-safe scene fixtures and richer launch or restore behavior remain follow-up slices.
Status: accepted

---

Topic: first Scene Brain recovery should reuse the committed restore pointer on the existing transport seam
Phase: Scene Brain
Question: once Riotbox already has deterministic scene candidates, one committed `scene.launch` seam, and visible scene diagnostics, what is the smallest honest next step that makes scene recovery real without opening a second transition model or scene browser?
Decision: add one bounded `scene.restore` action that targets the existing session `restore_scene` pointer, queues on `NextBar`, commits through the current action queue and transport boundary seam, and swaps the restore pointer back to the previously active scene when the restore lands. Keep richer transition shaping, scene recovery policy, and deeper restore diagnostics out of scope.
Why: the current contracts already name scene restore as part of the TUI and action lexicon, and `scene.launch` by itself still leaves Scene Brain without a real recovery path. The smallest honest move is to reuse the explicit restore pointer already present in session state instead of inventing a second scene stack, transition graph, or shell-only recovery model.
Evidence: `riotbox-app` now queues `scene.restore` on the same `NextBar` seam as `scene.launch`, blocks overlapping scene transitions, updates `active_scene`, transport `current_scene`, and `restore_scene` together on commit, exposes a minimal pending restore cue in the Jam shell, and covers both committed state and shell visibility with focused regressions.
Consequences: later Scene Brain work should continue from the same committed restore pointer when adding richer launch/restore cues, scene transition policy, or more musical recovery behavior. Replay-safe restore fixtures and more detailed shell diagnostics remain separate follow-up slices.
Status: accepted

---

Topic: first-run onramp should stay inside the existing Jam shell spine
Phase: Playable shell UX
Question: once Riotbox already has real lanes and actions, what is the smallest honest first-run improvement that helps a new user find one meaningful play moment without inventing a second onboarding shell or wizard?
Decision: add a bounded first-run onramp directly inside the existing Jam screen and help overlay. Show a reduced `Start Here` guidance block only while the session is still in an early state, and let it evolve from `start transport` to `first move queued` to `first change landed` instead of opening a separate first-run mode.
Why: the current user problem is not missing engine state, but missing orientation in the first 30 to 60 seconds. A small guidance layer on top of the shipped Jam shell preserves the current runtime and screen architecture, reduces equal-rank noise at first contact, and gives Riotbox one obvious first move without pretending the shell is already fully simplified.
Evidence: `riotbox-app` now swaps the dense source row for a dedicated `Start Here` block only during early first-run states, extends the help overlay with stage-aware onboarding hints, and covers both the untouched mature shell and the new first-run guidance with focused UI regressions.
Consequences: deeper perform-first Jam simplification and richer onboarding remain separate work. `RIOTBOX-94` can still reduce the long-term Jam surface, and later product work can still add a fuller first-run flow if the small inline onramp proves insufficient.
Status: accepted

---

Topic: Jam should become perform-first before Riotbox adds a separate inspect mode
Phase: Playable shell UX
Question: once Riotbox already has strong lane state, trust cues, and multiple support screens, what is the smallest honest next step that makes the Jam surface feel more like an instrument and less like an engine dashboard?
Decision: keep one Jam screen for now, but reduce it to a perform-first hierarchy: `Now`, `Next`, and `Trust` on the top row; three compact lane cards for `MC-202`, `W-30`, and `TR-909` in the middle; and a lower row containing only `Pending / landed`, `Suggested gestures`, and `Warnings / trust`. Move source detail, section lists, macro dumps, and deeper diagnostics off the main Jam surface and keep them on `Source`, `Capture`, `Log`, or the help overlay.
Why: the strongest UX feedback was not that Riotbox lacked state, but that too much equal-priority information was competing on the primary Jam surface. The next honest improvement is to reorder and reduce the surface, not to open a second inspect architecture before the simpler Jam hierarchy has been tested in use.
Evidence: `riotbox-app` now removes the old Source / Sections / Macros row from the main Jam surface, replaces it with three lane cards plus suggested-gesture and warning blocks, shortens the footer/help language toward primary versus secondary actions, and keeps fixture-backed Jam regressions green after the wording and layout shift. The review artifact at `docs/screenshots/jam_perform_first_baseline.txt` records the new hierarchy.
Consequences: a future inspect-mode split remains possible, but it is no longer the default next move. Follow-up UX slices should first test whether the reduced Jam surface plus better help text is enough. If further reduction still fails, a later ticket can add a deeper inspect layer without reopening the current screen contract.
Status: accepted

---

Topic: the first 30 seconds after ingest should bias toward one obvious success path
Phase: Playable shell UX
Question: after adding the bounded first-run onramp and reducing the Jam surface, what is the smallest next step that makes the first playable moment easier to discover without creating a second onboarding system?
Decision: keep the existing inline `Start Here` guidance, but sharpen it into one explicit first-success flow: `[Space]` to start transport, `[f]` to queue one first fill, and `[2]` to confirm the landed result in `Log`. Once the first action is armed or committed, keep the guidance focused on only the next decision: let it land, then either capture the keeper or undo it.
Why: the next user problem was no longer “what screen is this?” but “what do I do first, and how do I know it worked?” A bounded single-path flow is the smallest honest improvement. It preserves the current Jam/help architecture and gives new users one clearer first success before widening back out to the rest of the shell.
Evidence: `riotbox-app` now rewrites the first-run `Start Here` and help-overlay copy around a single first fill and confirmation loop, removes the earlier first-step ambiguity between multiple actions, and keeps the focused first-run UI regressions green.
Consequences: later first-run work can still introduce richer post-success guidance or broader onboarding, but the inline path should remain singular and easy to scan. If this still proves too open, the next follow-up should improve the moment-after-success guidance before inventing a separate onboarding surface.
Status: accepted

---

Topic: Jam should speak in gesture language on the perform-first surface
Phase: Playable shell UX
Question: once the Jam surface is reduced and the first-run path is tighter, what is the smallest next change that makes Riotbox feel less like an internal action model and more like an instrument?
Decision: keep the deep `Log`/diagnostic surfaces technically precise, but shift the perform-first Jam surface, footer, help overlay, and status line toward clearer gesture vocabulary. Use words such as `voice`, `jump`, `follow`, `hit`, and `push` where those improve immediacy, while leaving the internal action model and command ids unchanged.
Why: the remaining UX friction was not capability but wording. The Jam shell was still presenting several actions in engine terms (`role`, `scene select`, `trigger`, `reinforce`) even after the hierarchy was improved. Translating the outward-facing layer is the smallest honest next move because it changes how the shell reads without inventing a new behavior model.
Evidence: `riotbox-app` now updates status messages, footer/help guidance, Jam MC-202 card wording, and perform-facing pending/landed labels to use the curated gesture vocabulary while leaving the deeper `Log` diagnostics and action ids intact. The fixture-backed Jam shell regressions stay green after the wording change.
Consequences: future UX work should preserve the split between perform language and diagnostic language. If later tickets add an inspect surface, it can stay more technical; the default Jam surface should continue optimizing for musical intent first.

---

Topic: Jam inspect mode should deepen confidence without restoring the old dashboard
Phase: Playable shell UX
Question: after the perform-first surface and clearer gesture language both land, what is the smallest next step that adds confidence depth without re-bloating the default Jam screen?
Decision: keep the current perform-first Jam layout as the default, but add one explicit `perform / inspect` toggle inside `Jam`. The inspect view should preserve the top-level `Now / Next / Trust` frame while swapping the lower half for lane detail, source structure, material flow, and compact diagnostics. Do not create a second dashboard screen or hide this behind `Log`, `Source`, or `Capture`.
Why: the current UX gap is no longer missing information; it is missing confidence depth at the moment a user wants to look slightly further without leaving the Jam context entirely. A bounded inspect mode is the smallest honest follow-up because it preserves the reduced default surface and reuses existing app/runtime/source seams instead of reviving the older all-at-once Jam dump.
Evidence: `riotbox-app` now adds an explicit Jam inspect toggle, blocks that toggle during the first-run guided path, reuses the existing MC-202/W-30/TR-909 diagnostic lines plus source-graph and capture/runtime summaries, and keeps focused snapshot and key-handling tests green. The review artifact at `docs/screenshots/jam_inspect_mode_baseline.txt` records the bounded inspect hierarchy.
Consequences: later Jam UX work should keep the split clear: perform mode is the default instrument surface, inspect mode is a read-only confidence layer, and deeper technical truth should still live on `Log`, `Source`, and `Capture`. If later feedback asks for even more detail, the next move should be to refine inspect density, not to reopen a second hidden dashboard path.
Status: accepted

---

The following topics require explicit entries before related implementation scales:

- audio backend and latency baseline
- sidecar transport choice
- deterministic replay model
- analysis provider baseline
- benchmark threshold policy
- Ghost budget and safety policy

---

## 5. Decision Hygiene

Every major decision should record:

- what problem it solved
- what alternative was rejected
- what evidence supported it
- what follow-up work it created

If that is not written down, the decision is not stable enough to rely on.

---

Topic: first workflow benchmarks should start as explicit interaction budgets, not a second measurement system
Phase: Playable shell UX
Question: after recent Jam UX work, what is the smallest honest way to start recording the roadmap's workflow benchmarks without inventing a new analytics or stopwatch architecture?
Decision: record the first workflow baseline as an explicit operator-path budget derived from the shipped example-source flow and the current quantization seam. For now, use the README/example-source path plus the first-run gesture path to document `time to first playable Jam state` and `time to first successful capture` in repo markdown under `docs/benchmarks/`.
Why: Riotbox needs benchmark visibility now, but the project does not yet need a second runtime measurement subsystem just to start tracking user-path budgets. The smallest honest move is to tie the benchmark to one shipped fixture, one shipped gesture path, and the current `NextBar` / `NextPhrase` commit model.
Evidence: the repo now has an explicit benchmark artifact at `docs/benchmarks/jam_workflow_baseline_2026-04-17.md`, grounded in the shipped `Beat08_128BPM(Full).wav` example path and the current first-run `Space -> f -> c` loop.
Consequences: later benchmark work should keep the same workflow names and fixture references, but can replace the derived timing budget with semi-automated or fully automated stopwatch data once that path exists. Until then, this benchmark family should stay small, readable, and tied to the shipped shell semantics.
Status: accepted

---

Topic: audio callback timing should be the live transport authority before deeper audio QA hardening
Phase: Pro Hardening
Question: once the app shell, queue, and audio render seams already exist, what is the smallest honest next move that makes live transport timing and quantized commit boundaries depend on the audio runtime instead of an app-side wall-clock pulse thread?
Decision: remove the app-local 20ms `RuntimePulseSource` thread and let `riotbox-audio` publish a typed timing snapshot derived from callback-owned beat progression. The app should consume that timing snapshot, reconstruct bar/phrase context from the current source graph, and commit queued actions from crossed boundaries observed in the audio-owned timing stream.
Why: the repo review finding was not about missing features; it was about musical honesty. As long as the app advanced transport and committed actions from its own wall-clock pulse, control-plane jitter could become the de facto timing spine while the callback merely rendered a lagging copy. Exposing callback-owned timing first is the smallest bounded fix because it moves authority toward the audio runtime without redesigning the whole scheduler in one slice.
Evidence: `riotbox-audio` now owns a small shared transport control/state seam and publishes `AudioRuntimeTimingSnapshot` values from callback progress; `riotbox-app` consumes those snapshots in the event loop, removes the old `runtime.rs` pulse thread module, and keeps focused transport/commit tests green under the new path.
Consequences: this does not yet finish the transport redesign. The app still reconstructs bar/phrase indices and still mirrors the current audio-owned beat position back into lane render state. Later hardening can push more of that contract into shared/core or audio-owned surfaces, but deeper audio QA and replay work now has one truthful live timing spine to build on.
Status: accepted

---

Topic: Capture target routing should use typed Jam view intent, not display strings
Phase: W-30 MVP
Question: once Capture guidance started using different next-step wording for W-30 pad targets and Scene targets, should the TUI branch on formatted labels such as `pad bank-a/pad-01`?
Decision: keep `CaptureSummaryView.last_capture_target` as the display label, but add a typed `last_capture_target_kind` projection for routing decisions. Capture `Do Next` and `hear ...` labels should branch on that kind and only use the display label for rendering.
Why: display wording is allowed to change as the TUI becomes more musical. If behavior-level guidance depends on string prefixes, a wording cleanup can silently change whether Riotbox offers W-30 audition/hit guidance or Scene confirmation guidance.
Evidence: `JamViewModel` now exposes `CaptureTargetKindView`; Capture `Do Next` and `capture_heard_path_label` branch on the typed kind while preserving the existing visible W-30 and Scene wording. Tests cover W-30 pad, Scene, and unassigned target projections.
Consequences: this is still a view projection over the existing `CaptureTarget` model, not a persistence change. Future Capture routing surfaces should consume typed view intent first and render display labels second.
Status: accepted

---

Topic: W-30 pending audition intent belongs in the Jam view model
Phase: W-30 MVP
Question: after Capture started explaining raw and promoted auditions in musical next-step language, where should raw-vs-promoted pending audition intent live?
Decision: project pending W-30 audition intent from the existing `ActionQueue` into `LaneSummaryView` as a typed view object containing kind, target, and quantization. The TUI may still render action ids in diagnostic surfaces, but Capture `Do Next` and compact lane cues should not reconstruct raw-vs-promoted audition state by scanning generic pending action command strings.
Why: raw and promoted auditions share the same `[o]` gesture but need different user-facing guidance. Keeping that distinction only as command strings in the generic pending list made the Capture surface fragile and duplicated classification logic in `riotbox-app`.
Evidence: `JamViewModel` now exposes `W30PendingAuditionView`; Capture `Do Next` renders queued raw/promoted audition guidance from that projection; focused tests cover raw and promoted pending audition kind, target, and quantization.
Consequences: this remains a presentation-model projection over the existing action system, not a second queue or W-30 action path. Future W-30 pending cue details should prefer typed Jam view projections when the perform-facing UI needs semantic intent beyond a generic command label.
Status: accepted

---

Topic: Capture handoff source readiness belongs in the Jam view model
Phase: W-30 MVP
Question: after Capture started showing compact `src` / `fallback` handoff cues, should the TUI derive that readiness by inspecting the latest session capture directly?
Decision: project Capture handoff readiness into `CaptureSummaryView` as typed view state. The TUI may still show detailed provenance in Capture inspect areas, but perform-facing handoff copy should consume the Jam view projection instead of scanning `session.captures`.
Why: `src` / `fallback` is not only raw provenance; it changes the user's confidence in `[w] hit` and `[p]->[w]` next steps. Keeping that decision in the view model keeps Capture guidance aligned with the typed projection pattern and prevents the TUI from accumulating more session-model branching.
Evidence: `JamViewModel` now exposes `CaptureHandoffReadinessView`; Capture `Do Next` and heard-path copy render the existing `src` / `fallback` wording from that projection; focused core and TUI tests cover fallback and source-backed readiness.
Consequences: this remains presentation-model state over the existing Capture model, not a persistence change or source-cache redesign. Future Capture confidence cues should prefer typed Jam view projections before adding more UI-side session inspection.
Status: accepted

---

Topic: MC-202 audio proof should start as an offline render seam before live mixer integration
Phase: MC-202 MVP / Audio QA
Question: after the routine audio audit found MC-202 state-proven but not musician-audible, should the next slice jump straight into live callback integration?
Decision: add a bounded offline MC-202 render seam and include one follower-vs-answer case in the lane recipe listening pack. Keep live TUI mixer integration out of this slice.
Why: the current MC-202 lane already has replay-safe role, follower, and answer state, but no audio contract. A small offline render seam gives the QA layer a real WAV and metric target without pretending the live instrument mix is finished or adding a hidden callback path too early.
Evidence: `riotbox-audio` now exposes `Mc202RenderState`, an offline render helper, unit coverage for distinct follower/answer output, and the lane recipe listening pack renders `mc202-follower-to-answer` with a minimum RMS delta.
Consequences: later MC-202 work should wire this typed render state into the app/audio runtime deliberately, with live mixer controls and TUI cues, rather than growing the offline proof into a shadow synth architecture.
Status: accepted

---

Topic: MC-202 live audio should consume the typed render seam instead of direct callback heuristics
Phase: MC-202 MVP / Audio QA
Question: once MC-202 has an offline render proof, what is the smallest honest step that lets a musician hear committed follower/answer state in the live Jam path?
Decision: thread `Mc202RenderState` through `riotbox-app` runtime projection, `AudioRuntimeShell` shared callback state, and the existing mixbuffer. Derive mode, routing, phrase shape, touch, transport, and music-bus level from committed session/runtime state, then render the bounded MC-202 bass voice beside TR-909 and W-30 without adding a second audio subsystem.
Why: the user-facing gap was that MC-202 gestures could be committed and logged while still not being part of the live sound. Reusing the typed render seam preserves queue/commit determinism and keeps the callback free of stringly role parsing or UI-only heuristics.
Evidence: `riotbox-app` now builds and exposes MC-202 render diagnostics from committed role/follower/answer state, `riotbox-audio` mirrors MC-202 render state through atomic shared runtime storage, and runtime tests prove the mixed buffer contains active MC-202 bass output.
Consequences: this is still a bounded first bass seam, not a finished MC-202 engine. Later MC-202 work should improve sound design, phrase continuity, live controls, and source-aware bass behavior on this same render path.
Status: accepted

---

Topic: MC-202 touch control should adjust the committed render seam directly
Phase: MC-202 MVP / Audio QA
Question: after MC-202 follower and answer state can be heard in the live mix, what is the smallest useful live control that does not create a second synth-control path?
Decision: expose `<` and `>` as bounded Jam controls for `runtime_state.macro_state.mc202_touch`. The controls refresh the existing typed `Mc202RenderState`, surface the current touch value in MC-202 diagnostics, and keep phrase generation plus role selection on the existing queue / commit seam.
Why: the musician needs one immediate performance parameter after a bass phrase lands, but Riotbox should not invent an ad hoc callback-only synth model. Touch is already persisted in session state and consumed by the renderer, so it is the safest first live control.
Evidence: `riotbox-app` now updates MC-202 touch through `JamAppState`, the shell maps `<` / `>` to that state refresh, app tests verify session/runtime-view projection, and `riotbox-audio` proves low-vs-high touch changes the same MC-202 phrase buffer metrics.
Consequences: future MC-202 live controls should follow this pattern: persisted macro or lane state first, typed render projection second, callback consumption third, and an output-path regression proving the audible seam changed.
Status: accepted

---

Topic: MC-202 phrase mutation should be quantized and render-state backed
Phase: MC-202 MVP / Audio QA
Question: after live touch control exists, how should Riotbox add the first MC-202 phrase mutation without opening a hidden sequencer or callback-only phrase path?
Decision: add `mc202.mutate_phrase` as a bounded `NextPhrase` action on the existing queue / commit seam. Commit writes an explicit MC-202 phrase variant into session lane state, projects that variant into the typed `Mc202RenderState`, and keeps direct live touch as the only immediate MC-202 macro control for now.
Why: Phase 4 requires quantized phrase mutation, but the safe first step is one replayable phrase variant, not a full phrase editor. Persisting the variant keeps replay/restore honest and lets audio QA prove the same lane state reaches the renderer.
Evidence: `riotbox-core` exposes pending MC-202 mutation and persists `phrase_variant`, `riotbox-app` queues and commits `mc202.mutate_phrase` from `G`, Jam/Log diagnostics show the variant, and `riotbox-audio` verifies `mutated_drive` differs from follower-drive output via delta-RMS and max-sample thresholds.
Consequences: future MC-202 phrase work should add richer variants or source-aware generation through the same committed lane-state seam before adding any editor or MIDI-style sequencer surface.
Status: accepted

---

Topic: MC-202 recipe proof should use signal-delta listening cases, not only loudness deltas
Phase: MC-202 MVP / Audio QA
Question: now that MC-202 has live touch and phrase mutation, how should the recipe-level QA pack prove those gestures are audible and not just visible in state/logs?
Decision: extend the lane recipe listening pack with explicit `mc202-touch-low-to-high` and `mc202-follower-to-mutated-drive` cases, and require sample-by-sample signal delta RMS alongside normal RMS delta.
Why: touch changes are partly loudness/energy changes, but phrase mutations can remain similarly loud while still being musically different. A plain RMS comparison can miss identical-output or fallback-collapse bugs when two phrases have similar energy. Signal-delta RMS catches actual waveform difference and writes the evidence beside the WAVs.
Evidence: `lane_recipe_pack` renders seven cases, writes signal delta metrics into comparisons and pack summaries, and the local packs passed with MC-202 touch signal delta RMS `0.006608`, pressure signal delta RMS `0.009436`, and mutated-drive signal delta RMS `0.010100`.
Consequences: future listening-pack cases should prefer paired signal-delta checks when the musical claim is "different phrase or gesture", and use plain RMS only as an additional energy sanity check.
Status: accepted

---

Topic: MC-202 pressure should be a quantized role phrase, not a free-running bass layer
Phase: MC-202 MVP / Audio QA
Question: how should Riotbox add the first explicit pressure behavior without turning the MC-202 lane into an unbounded sequencer?
Decision: add `mc202.generate_pressure` on the existing `NextPhrase` queue / commit seam. The commit stores role `pressure`, clears phrase variants, raises MC-202 touch to a bounded pressure value, and projects to a typed `Pressure` render mode with a sparse `pressure_cell` phrase shape.
Why: Phase 4 asks for pressure and identity without overplaying. A quantized pressure role gives the performer one clear gesture for offbeat pressure while preserving replayability, undo/log visibility, and the same render-state path as follower, answer, touch, and phrase mutation.
Evidence: `riotbox-core` exposes the new action and pending cue, `riotbox-app` queues/commits `P` pressure through the existing MC-202 phrase-control path, and `riotbox-audio` plus the lane recipe listening pack prove `pressure_cell` differs from follower drive with signal delta RMS `0.009436`.
Consequences: future pressure work should add note-budget/source-aware policy on top of this committed role seam rather than adding callback-only heuristics or a separate MC-202 phrase editor.
Status: accepted

---

Topic: MC-202 instigator should be a quantized role phrase on the existing render seam
Phase: MC-202 MVP / Audio QA
Question: how should Riotbox add the missing `instigate` behavior from the feral addendum without opening a second MC-202 engine or free-running sequencer?
Decision: add `mc202.generate_instigator` on the existing `NextPhrase` queue / commit seam. The commit stores role `instigator`, clears phrase variants, raises MC-202 touch to a bounded instigator value, and projects to a typed `Instigator` render mode with an `instigator_spike` phrase shape.
Why: the MC-202 lane needs one sharper push gesture in addition to follower, answer, and pressure. Keeping it as a committed phrase role preserves replayability, pending/commit visibility, undo/log behavior, and the existing app-to-audio typed render path.
Evidence: `riotbox-core` exposes the new action and pending cue, `riotbox-app` queues/commits `I` instigator through the existing MC-202 phrase-control path, app/UI fixtures cover committed state and shell output, and `riotbox-audio` plus the lane recipe listening pack prove `instigator_spike` differs from follower drive with signal-delta thresholds.
Consequences: future instigator work should add note-budget and source-aware contour policy on top of this committed role seam, not as callback-only heuristics or a separate MC-202 phrase editor.
Status: accepted

---

Topic: MC-202 anti-overplay should be a typed note budget on the render seam
Phase: MC-202 MVP / Audio QA
Question: what is the smallest note-budget policy that reduces MC-202 clutter without inventing a phrase editor, source-aware scorer, or callback-only heuristic?
Decision: add `Mc202NoteBudget` to the typed MC-202 render state and derive it from the committed phrase shape. `pressure_cell` uses a sparse budget, `instigator_spike` uses a push budget, `mutated_drive` keeps a wider budget, and follower/answer/root phrases use a balanced budget.
Why: the lane needs anti-overplay behavior, but the first policy should stay deterministic and replay-aligned. Putting the budget in render state makes it visible to app projection, shared audio runtime state, tests, and listening-pack metrics without creating a second phrase system.
Evidence: `riotbox-audio` now caps active steps per 16-step cycle from `Mc202NoteBudget`, tests prove the balanced budget reduces density without silencing follower drive, `riotbox-app` projects the budget from committed phrase shape, and the lane recipe pack passes after recalibrating touch/instigator thresholds for the less-dense output.
Consequences: future source-aware contour and phrase scoring work should choose or adjust this typed budget from analysis/scene context rather than bypassing it with ad hoc callback gating.
Status: accepted

---

Topic: MC-202 contour hints should be section-derived render state, not extracted melody
Phase: MC-202 MVP / Audio QA
Question: what is the smallest source-aware contour step that moves MC-202 beyond static phrase shapes without inventing pitch tracking or a phrase editor?
Decision: add `Mc202ContourHint` to the typed render state and derive it from the current projected source/scene section. Build sections lift, high-energy drop/chorus sections drop, break/intro/outro or low-energy sections hold, and unknown sections stay neutral.
Why: Phase 4 asks for contour following with feral simplification. A coarse section-derived hint gives the lane source awareness while preserving replayable committed roles, deterministic app projection, and callback-safe rendering.
Evidence: `riotbox-app` projects the hint from source sections into runtime diagnostics, `riotbox-audio` mirrors it through shared runtime state and changes phrase intervals in the renderer, and the lane recipe pack proves `mc202-neutral-to-lift-contour` differs from neutral follower drive with signal delta RMS `0.007847`.
Consequences: later hook-response and source-aware phrase scoring should refine this typed hint instead of bypassing it with callback-only heuristics or full melody extraction.
Status: accepted

---

Topic: MC-202 hook response should be explicit answer-space restraint
Phase: MC-202 MVP / Audio QA
Question: how should MC-202 avoid doubling hook-like sections without adding a phrase editor or Ghost-driven composition?
Decision: add `Mc202HookResponse` to the typed render state and derive `answer_space` for follower/leader roles when the current source/scene section is hook-like, currently chorus labels or hook/chorus tags. `answer_space` uses a sparse note budget and offbeat response gating in the renderer.
Why: the feral addendum asks for hook-response rules instead of hook doubling. Making the rule explicit in render state keeps it visible to TUI diagnostics, shared audio state, tests, and listening-pack metrics while avoiding callback-only heuristics.
Evidence: `riotbox-app` projects chorus/hook context to `Mc202HookResponse::AnswerSpace`, `riotbox-audio` renders answer-space with lower density and offbeat offsets, and the lane recipe pack proves `mc202-direct-to-hook-response` differs from direct follower drive with signal delta RMS `0.008681` and RMS delta `0.004777`.
Consequences: future source-aware phrase scoring can refine which sections are hook-like, but it should keep using the typed hook-response seam rather than hiding hook restraint in ad hoc phrase substitution.
Status: accepted

---

Topic: MC-202 recipe QA should replay the musician flow, not only isolated pairs
Phase: MC-202 MVP / Audio QA
Question: after individual MC-202 gestures have output proofs, what is the smallest test that proves the documented recipe path works as a sequence?
Decision: add an app-level recipe replay regression that queues and commits follower, answer, pressure, instigator, phrase mutation, and touch adjustment, then renders each landed `Mc202RenderState` through the audio seam and compares successive buffers.
Why: isolated pair tests can pass while the musician-facing flow still breaks through ordering, queue/commit state, or render projection drift. A bounded replay test catches sequence-level regressions without requiring full TUI automation or realtime device capture.
Evidence: `mc202_recipe_replay_proves_control_and_audio_path` verifies queue/commit state, landed render modes/shapes, non-silent MC-202 buffers, and signal-delta thresholds across the current Recipe 2 gesture chain.
Consequences: broader recipe replay and observer correlation should build from this pattern: drive the same actions a musician performs, assert the control path, then prove the nearest audio seam changed.
Status: accepted

---

Topic: MC-202 MVP exit requires state and audio undo, not log-only undo
Phase: MC-202 MVP / Replay QA
Question: after follower, answer, pressure, instigator, touch, mutation, contour, note budget, hook response, and recipe replay proof exist, what still blocks calling the first MC-202 MVP exit-clean?
Decision: require a bounded MC-202 undo rollback slice before closing the MVP phase. Undo must restore the previous MC-202 lane state, refresh the typed render state, and prove the rendered output returns to the previous audible seam; marking the action log entry as `undone` is not enough.
Why: the phase definition explicitly requires replay and undo to remain intact. Riotbox is an instrument for trying moves and backing out; if a musician hears an MC-202 move and presses undo, the sounded lane state must roll back, not only the diagnostic history.
Evidence: the current `undo_last_action` path marks the latest undoable action as undone and appends an undo marker, while MC-202 side effects are already committed into session lane state, macro touch, and typed render projection.
Consequences: complete the rollback on the existing action/log/session/render seam. Do not introduce a second MC-202 history stack or callback-only undo path.
Status: accepted

---

Topic: MC-202 undo should restore session-backed lane snapshots and audio output
Phase: MC-202 MVP / Replay QA
Question: what is the smallest undo implementation that makes MC-202 experimentation musically reversible without introducing a second phrase system?
Decision: store bounded MC-202 undo snapshots in `runtime_state.undo_state` at commit time, keyed by action id. When undo targets an MC-202 phrase action, restore the previous role, phrase reference, phrase variant, and touch from that snapshot, then refresh the typed render projection.
Why: undo has to change the sounded lane state, not just mark a history row as undone. Keeping the snapshot in session runtime state makes the rollback explicit and serializable while avoiding callback-local memory or a separate MC-202 history stack.
Evidence: `undo_mc202_phrase_move_restores_lane_state_and_audio_path` commits follower then answer, undoes answer, verifies the action log and Jam lane return to follower state, and proves the post-undo render buffer matches the previous follower buffer while differing from the undone answer buffer. `session_file_roundtrips_via_json` also covers persisted MC-202 undo snapshots.
Consequences: future lane-specific undo should use the same pattern: capture bounded pre-state before applying committed side effects, restore it through session state, and prove the nearest audible seam changed back.
Status: accepted

---

Topic: MC-202 MVP can close after undo rollback proof
Phase: MC-202 MVP / Project Closeout
Question: is `P006 | MC-202 MVP` exit-clean after the MC-202 undo rollback slice merged?
Decision: close `P006 | MC-202 MVP` as the first honest MC-202 MVP. The previous blocker is resolved by session-backed undo snapshots that restore lane state, typed render state, Jam view state, and the nearest audible render seam.
Why: the phase definition requires usable follower basslines, live sound control, quantized mutation, pressure without clutter, and intact replay/undo. Those conditions now have explicit implementation and regression evidence.
Evidence: `RIOTBOX-314` merged into `main` as `27fd2e5ebe7906ea2408c8532a571acdbe2f7464`; `undo_mc202_phrase_move_restores_lane_state_and_audio_path` proves the rollback path and rendered output behavior. The MC-202 exit review was updated on 2026-04-26 to mark the required follow-up closed.
Consequences: future MC-202 work can continue as refinement, not MVP exit cleanup. The active product spine should move back to W-30 MVP and end-to-end playable audio QA unless a regression reopens MC-202.
Status: accepted

---

Topic: W-30 MVP exit requires real capture artifacts before pad-bank expansion
Phase: W-30 MVP / Audio QA
Question: after source-backed W-30 preview, focus-aware pad targets, loop-freeze lineage, and resample tap diagnostics exist, what is the next smallest blocker toward a musician-usable W-30 MVP?
Decision: make committed W-30 capture paths write real source-backed capture WAV artifacts before widening the pad-bank or internal-bus resample engine. The current `CaptureRef.storage_path` contract should point to an actual artifact for normal source-window captures, while avoiding any file I/O in the realtime audio callback.
Why: Phase 5 requires useful loops that can be captured and reused. Today W-30 has strong provenance and a proven preview seam, but capture storage is still metadata-only and pad playback is still one focused preview state. Real capture artifacts are the smallest foundation that improves musician trust and unlocks later pad playback and bus-print resampling.
Evidence: `docs/reviews/w30_mvp_gap_review_2026-04-26.md` reviewed the current W-30 app/audio seams and found that `CaptureRef` materialization, source-window preview, and resample lineage are real, while `captures/*.wav` files are not written by the normal app capture path.
Consequences: the next W-30 implementation slice should write source-window-backed PCM capture files for committed captures, prove those files are non-silent and source-derived, and keep realtime boundaries clean. Full pad-bank voices and internal-bus print resampling should build on that artifact seam instead of bypassing it.
Status: accepted

---

Topic: Source-backed W-30 captures should materialize session-relative WAV artifacts
Phase: W-30 MVP / Audio QA
Question: what is the smallest implementation that makes committed W-30 captures more than metadata while preserving realtime safety?
Decision: when the app commits a capture with `CaptureRef.source_window`, a loaded `SourceAudioCache`, and a session file path, write a PCM16 WAV artifact to `CaptureRef.storage_path` relative to the session directory. Keep the write on the app commit path and never in the realtime audio callback.
Why: musicians need captured loops to be concrete artifacts they can trust, inspect, reload, and later assign to pads. The first safe version can print the decoded source window without attempting internal-bus recording or full sampler playback yet.
Evidence: `committed_source_backed_capture_writes_wav_artifact` loads a real PCM source, commits a W-30 capture, verifies `captures/cap-01.wav` exists, reloads it through `SourceAudioCache`, checks duration/sample-rate/channel-count and non-silence, then saves and reloads the session. `writes_source_window_as_pcm16_wav_artifact` covers the lower-level source-window writer.
Consequences: future W-30 pad playback and internal bus print resampling should consume this artifact seam where appropriate instead of treating `storage_path` as decorative metadata.
Status: accepted

---

Topic: focused W-30 pad playback should prefer committed capture artifacts
Phase: W-30 MVP / Audio QA
Question: after committed W-30 captures write real WAV artifacts, what is the smallest playback change that proves captured material is reusable without building a full sampler engine?
Decision: cache committed capture artifacts in the app/control plane and let focused W-30 preview / trigger projection prefer that artifact audio over source-window projection. Source-window projection remains a fallback for legacy sessions or missing artifacts, and the realtime callback still receives only the bounded `W30PreviewSampleWindow`.
Why: Phase 5 needs captured material to be reusable without leaving flow. If pad trigger continues to depend only on original source-window metadata, `captures/*.wav` becomes a trust artifact but not yet the playback source. Preferring the artifact proves the storage seam is musically active while staying smaller than independent multi-pad sample voices.
Evidence: `focused_w30_pad_trigger_uses_capture_artifact_preview_when_source_cache_unavailable` commits a source-backed capture, removes the original source, disables the source cache, promotes/triggers the focused W-30 pad, then renders non-silent artifact-backed audio that differs from the synthetic fallback preview.
Consequences: this is still one focused preview seam, not full pad-bank polyphony. Future W-30 work can widen from artifact-backed focused playback into richer pad voices and internal bus prints without changing the capture artifact contract.
Status: accepted

---

Topic: W-30 internal bus print should start as a bounded offline artifact seam
Phase: W-30 MVP / Audio QA
Question: after source-backed capture artifacts exist, what is the smallest honest internal resample print that records reusable processed audio without adding a DAW-style recorder?
Decision: define the first W-30 internal bus print as an offline app/control-plane operation for one committed W-30 capture. It should render a bounded processed result from the focused capture artifact through the existing W-30 preview / resample policy, write a new PCM16 WAV artifact, and materialize a `CaptureType::Resample` with explicit lineage and incremented generation depth.
Why: Phase 5 requires internal bus resampling to create reusable material, not only lineage metadata or a diagnostic tap. Keeping the first version offline and single-capture preserves realtime safety while giving musicians a concrete "print the chaos" artifact that later pad playback can consume.
Evidence: the W-30 MVP gap review identifies internal resampling as the remaining blocker after capture artifacts and focused artifact-backed playback. Existing `promote.resample`, `W30ResampleTapState`, and capture lineage fields already provide the control and provenance spine; the missing piece is a printed audio artifact and output comparison.
Consequences: the implementation follow-up should prove queue -> commit -> printed capture -> artifact reload, and compare the printed result against raw capture/source and synthetic fallback controls. Full multitrack recording, live callback recording, and export workflows remain later slices.
Status: accepted

---

Topic: W-30 promote.resample should print a reusable bus artifact
Phase: W-30 MVP / Audio QA
Question: what is the smallest implementation that turns `promote.resample` from lineage metadata into a reusable audio result?
Decision: when `promote.resample` commits, render a bounded offline bus print from the focused capture artifact plus the existing W-30 resample policy, write it to the new resample capture's `storage_path`, and cache that artifact for later focused-pad playback. Resample captures should omit `source_window` unless they are literal source copies.
Why: musicians need resampling to "print the chaos" into material they can hear, reload, and reuse. Keeping the first print offline and single-capture preserves realtime safety while turning the existing action, lineage, and tap state into a real artifact.
Evidence: `committed_w30_internal_resample_prints_reusable_bus_artifact` commits source capture -> promotion -> resample, verifies `captures/cap-02.wav` exists and reloads, asserts lineage/generation metadata, checks non-silent metrics, and compares the printed artifact against both raw capture audio and the synthetic resample-tap control.
Consequences: the printed artifact is still a bounded MVP bus print, not full multitrack recording or export. Future W-30 work can improve the render policy and pad-bank playback while keeping the artifact/provenance contract stable.
Status: accepted

---

Topic: W-30 MVP exit remains blocked on duration-aware focused pad playback
Phase: W-30 MVP / Audio QA
Question: after capture artifacts, artifact-backed focused playback, and bounded bus-print resampling exist, is `P007 | W-30 MVP` exit-clean?
Decision: do not close W-30 MVP yet. The storage and resample blockers are resolved, but the pad playback criterion still needs one bounded implementation slice: focused W-30 pad playback should render from the committed artifact over a duration/loop policy instead of only through the fixed `W30_PREVIEW_SAMPLE_WINDOW_LEN` preview window.
Why: musicians can now hear artifact-backed W-30 material, but the current seam is still a diagnostic preview voice, not a convincing loop-length pad playback path. Closing the phase here would overstate the playable sampler behavior.
Evidence: `docs/reviews/w30_mvp_exit_review_2026-04-26.md` compares the Phase 5 criteria with current app/audio code and tests. It finds source-backed capture artifacts and bus-print artifacts satisfied for MVP, while focused pad playback still caps material through the fixed preview sample window.
Consequences: the next W-30 implementation should stay narrow: one focused duration-aware pad voice, existing queue/commit actions, preloaded artifact data, no realtime file I/O, and output-path tests proving longer artifact playback differs from both fallback and the fixed-window preview.
Status: accepted

---

Topic: Focused W-30 pad playback should use a bounded artifact-duration playback window
Phase: W-30 MVP / Audio QA
Question: what is the smallest implementation that makes focused W-30 pad hits more than a fixed preview-window replay?
Decision: add a separate preloaded `W30PadPlaybackSampleWindow` to the W-30 preview render state. Build it from the committed capture artifact, cap it to a bounded `16_384` mono samples for the first MVP pass, loop it in the callback, and prefer it over the existing fixed `W30PreviewSampleWindow` when rendering focused pad audio.
Why: Phase 5 needs pads to feel playable without jumping straight to full pad-bank polyphony. A bounded focused-pad playback window keeps callback input preloaded and deterministic, avoids realtime file I/O, and gives musicians audible material beyond the old `2048`-sample diagnostic preview.
Evidence: `w30_pad_playback_uses_duration_window_beyond_fixed_preview_len` proves the callback reaches samples beyond the fixed preview window and differs from the fixed-preview control. `focused_w30_pad_trigger_uses_capture_artifact_preview_when_source_cache_unavailable` proves the app path loads committed capture artifacts into the focused pad playback window, renders non-silent audio beyond the preview boundary, and differs from both fixed-preview and fallback controls.
Consequences: this is still one focused W-30 pad voice, not full multi-pad sampler polyphony. Later work can widen the same artifact playback contract into richer pad-bank behavior without reopening capture storage or resample provenance.
Status: accepted

---

Topic: Close P007 W-30 MVP after focused artifact playback lands
Phase: W-30 MVP
Question: after `RIOTBOX-322`, is the W-30 MVP phase exit-clean?
Decision: close `P007 | W-30 MVP` as the first honest W-30 MVP. Useful source-window captures write real artifacts, focused W-30 pads can play committed artifacts beyond the old fixed preview window, internal bus resampling prints reusable artifacts, reuse stays inside the queue/commit Jam flow, and provenance remains explicit.
Why: the phase definition is now satisfied without pretending Riotbox has a full sampler engine. The remaining W-30 work is expansion and polish, not an MVP blocker.
Evidence: `RIOTBOX-317` writes source-backed capture artifacts, `RIOTBOX-318` makes focused playback prefer committed artifacts, `RIOTBOX-320` prints bounded resample artifacts, and `RIOTBOX-322` adds duration-aware focused pad playback with output-path proof. `docs/reviews/w30_mvp_exit_review_2026-04-26.md` records the blocker and its closure.
Consequences: move the roadmap spine to the next phase after W-30 MVP. Later W-30 work should be scoped as sampler expansion, sound-design refinement, richer pad-bank behavior, or export/listening-pack work, not as a prerequisite for closing P007.
Status: accepted

---

Topic: Scene Brain MVP needs sequence-level output proof before phase closeout
Phase: Scene Brain / Audio QA
Question: after W-30 MVP closes, what is the next smallest honest P008 blocker to address?
Decision: require a bounded Scene Brain recipe replay output regression before treating P008 as exit-clean. The proof should drive the existing `scene.launch -> scene.restore` flow through queue/commit state, keep TR-909 source support and MC-202 follower active, render before/launched/restored mixed-lane buffers, and compare signal-delta metrics alongside scene state and diagnostics.
Why: Scene Brain already has deterministic scene candidates, contrast target selection, restore pointers, Jam/Log readability, TR-909 scene-target support, and MC-202 scene/source contour. The remaining MVP risk is musical honesty: isolated lane hints are not enough to prove that a musician hears a meaningful scene transition.
Evidence: `docs/reviews/scene_brain_mvp_gap_review_2026-04-26.md` re-audits P008 and records passing scene-filtered core, app, and audio tests. It identifies sequence-level output proof and explicit transition intent as the remaining blockers.
Consequences: the next implementation should stay inside the existing Source Graph, Session, ActionQueue, Jam view, TR-909, and MC-202 render-state seams. If current lane rules do not produce a strong enough before/launched/restored contrast, add the smallest deterministic scene-transition policy needed instead of introducing a second arranger or shadow audio path.
Status: accepted

---

Topic: Scene Brain jump-restore proof should compare mixed lane output, not isolated diagnostics
Phase: Scene Brain / Audio QA
Question: what proof is enough to close the first sequence-level output gap for the current Scene Brain flow?
Decision: add an app-level regression that drives `scene.launch -> scene.restore`, renders mixed TR-909 + MC-202 output before launch, after launch, and after restore, and checks both control-path state and signal-delta audio behavior.
Why: Scene Brain is a musician-facing transition flow. Proving only TR-909 support, only MC-202 contour, or only Jam/Log text would miss failures where the sequence lands in state but does not produce a meaningful audible result.
Evidence: `scene_jump_restore_replay_proves_state_and_mixed_audio_path` keeps TR-909 source support and MC-202 follower active, verifies scene state, restore pointer, Jam scene view, TR-909 scene-target support context, MC-202 contour hints, non-silent mixed renders, launch/restore signal-delta thresholds, and baseline return after restore.
Consequences: this closes the first output-proof gap from the Scene Brain MVP review. It does not close P008 by itself; a bounded explicit scene-transition policy is still needed before claiming the default arrangement no longer feels static or that scene transitions are musically intentional enough for MVP closeout.
Status: accepted

---

Topic: Scene transition intent should be a typed Jam-view policy before becoming an arranger
Phase: Scene Brain
Question: what is the smallest explicit Scene Brain transition policy after the mixed jump-restore output proof?
Decision: add `SceneTransitionPolicyView` to the core Jam view for launch and restore targets. Derive action kind, direction (`rise`, `drop`, `hold`), TR-909 intent, MC-202 intent, and a bounded intensity from current and target scene energy. Surface that policy on Jam pending/footer cues.
Why: Scene Brain needs explicit transition intent, but a full Scene Graph or arranger would be too early. The existing Source Graph, session scene state, launch/restore actions, TR-909 scene-target support, and MC-202 contour seams already provide enough information for a deterministic first policy projection.
Evidence: core scene tests assert `drop` and contrast-target policy derivation; app scene tests assert `SceneTransitionPolicyView` during the mixed `jump -> restore` replay and UI tests show `policy rise/drop` plus `909`/`202` intents on pending scene cues.
Consequences: the policy is read-only for now and intentionally does not create a new audio path. Later arrangement movement should consume this policy or widen it, not replace it with a shadow arranger.
Status: accepted

---

Topic: Scene movement should be persisted as landed state before any full arranger exists
Phase: Scene Brain / Audio QA
Question: what is the smallest honest arrangement movement after typed Scene transition intent exists?
Decision: store the last landed Scene movement in session scene state and let the existing render projections consume it. The movement records launch/restore kind, source scene, target scene, `rise/drop/hold` direction, bounded intensity, and TR-909 / MC-202 lane intent. TR-909 maps it to phrase variation plus a bounded slam floor; MC-202 maps it to contour/touch shaping.
Why: P008 needs default Scene progression to stop feeling like static state labels, but a full arranger, source-playback repositioner, or Ghost/Feral scene planner would be too large and would risk a shadow architecture. A persisted movement record keeps replay deterministic and makes the audible seam inspectable.
Evidence: `scene_jump_restore_replay_proves_state_and_mixed_audio_path` now asserts landed `SceneMovementState`, TR-909 phrase variation, MC-202 contour, non-silent mixed output, launch/restore signal deltas, and that restore keeps movement energy instead of collapsing back to the pre-launch baseline.
Consequences: this closes P008 for the bounded MVP. It is not a finished arranger: automatic scene chains, source-position jumps, richer transition envelopes, and W-30 scene movement remain follow-up work on the same persisted movement seam.
Status: accepted

---

Topic: Start Feral policy with a visible scorecard projection, not a new engine
Phase: Feral Policy Layer
Question: what is the smallest safe first P009 slice after Scene Brain MVP closes?
Decision: start with a bounded `FeralScorecardView` projection over existing Source Graph assets, candidates, relationships, and analysis summary. Surface it through an existing TUI consumer before any feral score becomes behavior.
Why: the feral addendum says new scores need consumers, and the current Source Graph already has enough break, hook, quote-risk, and candidate vocabulary to produce a useful first scorecard without adding a second graph, sampler, arranger, or Ghost path.
Evidence: `docs/reviews/feral_policy_entry_audit_2026-04-26.md` maps current Source Graph, W-30, TR-909, MC-202, and Scene Brain seams against the feral addendum and recommends scorecard-first implementation.
Consequences: the next implementation should add visible scorecard projection and tests. Audio-producing feral behavior should wait until one existing action path consumes that scorecard with output-path proof.
Status: accepted

---

Topic: MVP crash recovery should be explicit manual recovery before automatic repair
Phase: Pro Hardening
Question: after safe temp-write/rename saves and truncated JSON failure tests exist, what crash recovery behavior is honest enough for MVP without hiding replay truth?
Decision: keep normal session load deterministic and side-effect free. Treat orphan hidden temp files as interrupted-write clues, not authoritative recovery inputs. Do not add automatic autosave fallback yet. Future autosave files must have explicit sibling names and must never overwrite the canonical session without user action.
Why: deterministic replay depends on knowing exactly which session file was loaded. Silent fallback to a nearby file could make a corrupted or stale replay truth look valid and would make debugging user reports harder. Manual recovery is less magical but safer until recovery-candidate scanning and UI prompts are covered.
Evidence: `crates/riotbox-core/src/persistence.rs` serializes before writing, writes a hidden sibling temp file, then renames into place. `truncated_session_json_load_fails_without_replacing_adjacent_valid_session` proves partial JSON fails explicitly while adjacent valid files remain manually loadable. `docs/specs/session_file_spec.md` now records the orphan-temp, autosave, and manual fallback boundary.
Consequences: the next recovery implementation should add a non-mutating recovery-candidate scanner before any guided TUI prompt. `load_session_json` should not learn automatic fallback behavior.
Status: accepted

---

### RBX-026

Date: 2026-05-03
Topic: Rust-first all-lane Source Timing Intelligence
Phase: Analysis Foundation / Pro Hardening
Question: how should Riotbox add professional beat-grid, timing, and phrase intelligence without turning MC-202, TR-909/TR-202-style rhythm support, W-30 slicing, or future lanes into isolated timing systems?
Decision: implement Source Timing Intelligence as a Rust-first product contract across all timing-aware lanes. Represent tempo, meter, beat grid, swing, phrase, confidence, drift, degradation, and competing timing hypotheses explicitly in Source Graph, session/replay state, and QA outputs. External MIR/Python tooling may be used for research comparison and offline validation, but runtime lane behavior, replay truth, and user-visible timing contracts must not depend on a Python-only implementation.
Why: Riotbox needs timing intelligence that can make MC-style questions, TR-style answers, W-30 slices, source captures, scene movement, and future lanes land musically together. A lane-specific or tool-specific timing shortcut would create architecture drift and make output quality hard to prove.
Evidence: `docs/plans/source_timing_intelligence_plan.md` defines the all-lane plan, quality gates, evaluation corpus, and phased implementation path. `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, and `docs/specs/technology_stack_spec.md` now anchor the plan against the roadmap, phase exits, and stack boundary.
Consequences: near-term implementation should start with typed timing models, fixtures, deterministic offline analysis, and lane consumer seams before changing audible behavior broadly. Audio-producing timing slices must include source-vs-control output proof and documented confidence/degradation behavior.
Status: accepted

---

### RBX-027

Date: 2026-05-03
Topic: P011 remains MVP-spine hardening; P012+ becomes Post-MVP roadmap
Phase: Roadmap / Pro Hardening
Question: how should Riotbox interpret the project codes after P011 without confusing bounded MVP exits with 1.0 release readiness?
Decision: keep `P011 | Pro Hardening` as the active final MVP-spine closeout project. Treat `P012` and later project codes as Post-MVP / product-to-1.0-release phases, beginning with Source Timing Intelligence and ending with `P020 | Riotbox 1.0 Release Cut` as a release milestone rather than the end of product development. Maintain only a coarse project / phase overview for P012-P020 until P011 is exit-clean; do not decompose distant Post-MVP work into a broad ticket list yet.
Why: Riotbox has bounded MVP exits for MC-202, W-30, Scene Brain, Ghost Watch / Assist, and Feral policy, but P011 still needs replay, recovery, export reproducibility, and stage-style reliability to become trustworthy. Starting detailed Post-MVP ticketing before that closeout would create backlog noise and blur the active hardening gate.
Evidence: `docs/phase_definition_of_done.md` now names P011 as the final MVP-spine hardening project, and `docs/execution_roadmap.md` records the P011-P020 project / phase map without creating a ticket list.
Consequences: Linear should keep P011 active and high-priority until its exit checklist is clean. P012 may be prepared as near-next orientation, while P013-P020 should remain coarse phase placeholders until the active gate moves.
Status: accepted

---

### RBX-028

Date: 2026-05-10
Topic: MC-202 Session v1 should keep compatibility labels behind typed helpers
Phase: Repo Ops / MC-202 typed contract follow-up
Question: after MC-202 queue, side effects, replay, and render projection consume typed helpers, should Session v1 migrate persisted MC-202 lane state from compatibility labels to typed enum fields?
Decision: do not change the Session v1 JSON shape for MC-202 role, phrase-intent, or undo snapshot fields now. Keep stable compatibility labels in persisted JSON, and require behavior consumers to parse those labels through typed core helpers before queue, replay, render, observer, or QA decisions.
Why: the real drift risk was behavior branching on arbitrary raw strings, not the persisted label shape itself. Existing sessions, fixtures, archive evidence, TUI labels, and observer/audio QA already depend on the stable labels. Changing the JSON shape would create migration cost without a musician-facing or architecture benefit while Session v1 is still sufficient.
Evidence: Stage 1-3 of `docs/reviews/mc202_typed_contract_migration_plan_2026-05-10.md` have moved current behavior to typed helper boundaries, and `docs/specs/session_file_spec.md` now records the Session v1 compatibility-label contract.
Consequences: future MC-202 roles or phrase intents must extend the typed helpers first. A typed-field JSON migration is allowed only as part of a documented session-version migration with legacy fixture load, roundtrip, restore, deterministic replay, undo snapshot compatibility, TUI/observer label, and audio-output proof where applicable.
Status: accepted

### RBX-029

Date: 2026-05-10
Topic: P012 Feral-grid output evidence must require lane-specific Source Grid alignment
Phase: Source Timing Intelligence / Audio QA
Question: when a Feral-grid manifest reports `pass`, is pack-level audio activity enough, or must the strict observer/audio gate prove lane-specific Source Grid alignment?
Decision: the original P012 strict observer/audio correlation gate requires Feral-grid manifests to include pack-level `source_grid_output_drift` plus lane-specific `tr909_source_grid_alignment` and `w30_source_grid_alignment`. Missing, malformed, below-ratio, or over-offset metrics fail the output path.
Why: P012 is about shared timing authority across lanes. A generic non-silent mix can hide the actual failure mode musicians hear: one lane walking away from the source grid while logs and pack status still claim success.
Evidence: `strict_evidence_rejects_missing_required_source_grid_alignment` now fails a synthetic `pass` manifest with no Source Grid alignment evidence, `cargo test -p riotbox-app --bin observer_audio_correlate -- --nocapture` covers the strict gate, and `just p012-all-lane-source-grid-output-proof` runs the current Feral-grid TR-909/W-30 proof plus MC-202 phrase-grid proof together.
Consequences: this is still a bounded P012 QA gate, not production arbitrary-audio beat/downbeat detection. At the original P012 decision point, MC-202 remained proven through phrase-grid recipe evidence until the source-derived question/answer placement engine landed.
Update 2026-05-20: RBX-032 extends the Feral-grid strict gate to require `mc202_source_grid_alignment` after the P013 representative showcase gained dedicated MC-202 bass-pressure source-grid proof.
Status: accepted

---

### RBX-030

Date: 2026-05-10
Topic: MC-202 P012 proof should bridge internal phrase timing to a Source Graph phrase slot
Phase: Source Timing Intelligence / Audio QA
Question: what is the smallest honest P012 MC-202 phrase-slot proof before a full source-derived question/answer arranger exists?
Decision: extend the lane recipe listening pack with `metrics.mc202_source_phrase_slot` for required MC-202 cases. The metric proves the generated candidate consumes a selected Source Graph phrase-grid slot and starts on that source phrase boundary, while the existing `mc202_phrase_grid` metric continues to prove internal sixteenth-grid note alignment.
Why: MC-202 output can be internally quantized but still unrelated to source-derived phrase timing. P012 needs a bridge proof that MC-202 is attached to Source Graph timing before the full source-aware question/answer engine lands.
Evidence: `mc202_lane_recipe_cases_consume_source_phrase_slots` and `mc202_source_phrase_slot_gate_rejects_non_source_phrase_boundary_candidate` cover the audio-side metric, observer/audio strict evidence rejects missing `mc202_source_phrase_slot`, and `just recipe2-observer-audio-gate` validates the generated manifest plus observer correlation path.
Consequences: this is a bounded synthetic-source proof. It does not claim production arbitrary-audio phrase arrangement, but future MC-202 source-aware placement should consume the same Source Graph phrase-slot concept rather than introduce a lane-local timing model.
Status: accepted

---

### RBX-031

Date: 2026-05-10
Topic: W-30 P012 proof should require bounded source-loop closure evidence
Phase: Source Timing Intelligence / Audio QA
Question: is W-30 source-grid alignment enough proof that the current source chop is a usable loop/chop unit?
Decision: require Feral-grid manifests to include `metrics.w30_source_loop_closure` beside `w30_source_grid_alignment`. The first metric proves the selected source-backed preview is non-silent, maps back to its selected source window, and has faded edges inside edge-delta / edge-absolute budgets before strict observer/audio QA treats the W-30 output path as passing.
Why: A W-30 stem can land on the grid but still be an unsafe or fallback-like chop if the selected micro-loop has loud unclosed edges or no real source-backed preview evidence. P012 needs timing and loop-closure proof to move toward musician-trustworthy source-derived sampling.
Evidence: `w30_source_loop_closure_proves_repeat_safe_faded_chop_window` covers the audio-side metric, `strict_evidence_rejects_w30_source_loop_closure_failures` covers observer/audio strict rejection, and Feral-grid manifest assertions require the metric to pass.
Consequences: this is a bounded micro-loop/chop-window QA proof, not the final automatic W-30 loop detector. Future loop detection should widen the same Source Graph timing/closure evidence instead of adding a lane-local timing model.
Status: accepted

---

### RBX-032

Date: 2026-05-20
Topic: P013 Feral-grid proof should include MC-202 lane-specific Source Grid alignment
Phase: All-Lane Musical Depth / Audio QA
Question: after the representative showcase gained a dedicated MC-202 bass-pressure stem, is phrase-slot proof enough for Feral-grid strict QA, or should the bass stem also carry source-grid alignment evidence?
Decision: require Feral-grid manifests and strict observer/audio correlation to treat `metrics.mc202_source_grid_alignment` as a lane-specific output proof beside `tr909_source_grid_alignment`, `w30_source_grid_alignment`, pack-level `source_grid_output_drift`, and W-30 loop-closure evidence.
Why: the representative showcase can otherwise hide a drifting or weak bass-pressure stem behind stronger grid-locked TR-909, W-30, or full-mix peaks. P013 is explicitly all-lane musical depth, so the MC-202 bass lane needs its own source-grid proof in the same generated-support context.
Evidence: RIOTBOX-810 added MC-202 source-grid alignment metrics to `feral_grid_pack`, RIOTBOX-811 surfaced them through observer/audio correlation and strict validators, and RIOTBOX-812 moved manifest ownership into a real module without changing the JSON/output contract. The local verification path included `cargo test -p riotbox-audio --bin feral_grid_pack`, `just syncopated-source-showcase-smoke`, representative showcase generation, `just audio-qa-ci`, `just ci`, and GitHub Actions Rust CI #1960.
Consequences: the older P012 MC-202 phrase-slot proof remains the lane-recipe bridge for question/answer placement, but Feral-grid showcase packs must also prove that audible MC-202 support lands near the chosen source grid. This is still bounded showcase QA, not a full production source-derived arranger.
Status: accepted

---

### RBX-033

Date: 2026-05-21
Topic: P012 Jam / Source Timing actionability should stay summary-owned
Phase: Source Timing Intelligence / Musician UX
Question: after Jam, Source, Help, observer snapshots, and P012 proof summaries all use Source Timing readiness language, where should the musician-facing action phrase live?
Decision: keep the actionability phrase on the shared Jam Source Timing summary, not in screen-local policy matching. Jam / Source surfaces should consume the same summary-owned phrase when they tell the musician whether the grid can steer moves, needs confirmation, should be listened to first, or is using fallback.
Why: P012 timing trust is only useful if musicians can see the same meaning everywhere. Local remapping of degraded-policy strings across panels makes it easy for Jam, Source, Help, observer snapshots, and QA readouts to drift apart.
Evidence: RIOTBOX-874 added `SourceTimingSummaryView.actionability`, used it in Help / Source / observer surfaces, and the cadence review in `docs/reviews/p012_jam_source_timing_surface_review_2026-05-21.md` found the remaining Jam compact-readiness gap as a follow-up rather than a new timing architecture problem.
Consequences: future Jam / Source timing wording should reuse the shared summary phrase first. If a surface needs shorter wording, add a summary-owned compact variant or document why that surface intentionally omits actionability.
Status: accepted

---

### RBX-034

Date: 2026-05-21
Topic: P012 readiness actionability labels should use shared producer helpers
Phase: Source Timing Intelligence / QA Surface
Question: after Jam, observer/audio, Feral-grid manifests, P012 proof summaries, and the probe CLI all expose actionability, should each Rust producer keep its own readiness/manual-confirm-to-language mapping?
Decision: do not add more Rust producer-local mappings for Source Timing readiness cue or actionability. Keep downstream validators as independent compatibility checks, but move runtime producer vocabulary behind a shared helper before extending the actionability contract further.
Why: P012 actionability only helps musicians if the same state means the same thing everywhere. Repeated local match tables are easy to keep aligned today but become brittle when new readiness states, manual-confirm policies, or shorter surface variants arrive.
Evidence: `docs/reviews/p012_source_timing_actionability_surface_review_2026-05-21.md` found the current labels coherent but duplicated in the Feral-grid manifest builder, standalone probe CLI, and observer/audio summary fallback.
Consequences: the next implementation work should centralize the Rust readiness label helper, then tighten manifest validators so generated Feral-grid manifests cannot silently drop `cue` or `actionability`.
Status: accepted

---

### RBX-035

Date: 2026-05-22
Topic: P015 TUI module ownership should continue through semantic helper modules
Phase: Productization Alpha / TUI Maintainability
Question: after the first P015 TUI split batch, should remaining work continue converting mixed include shards into child modules or pause until a larger UI rewrite?
Decision: continue with small semantic extraction slices. Do not mechanically remove every `include!`; extract only coherent responsibilities such as W-30 preview labels, first-run capture routing, W-30 resample labels, footer style-token tests, and W-30 preview/source-readiness tests.
Why: the broad TUI review found no immediate product-spine or audio/replay blocker, but `ui.rs` still acts as a mixed registry/include root and the largest test shards still combine multiple surfaces. Small semantic modules reduce review and agent context cost without changing behavior.
Evidence: `docs/reviews/p015_tui_module_ownership_review_2026-05-22.md` reviewed `crates/riotbox-app/src/ui` after RIOTBOX-920 through RIOTBOX-922 and identified the next bounded slices.
Consequences: P015 TUI cleanup should keep using normal Linear/branch/PR/CI/archive slices. Future splits should name real ownership boundaries and avoid mechanical file churn.
Status: accepted

---

### RBX-036

Date: 2026-05-22
Topic: P012 compact proof surfaces should keep generated phrase evidence visible
Phase: Source Timing Intelligence / Audio QA
Question: after generated Feral-grid proof surfaces gained cue/action, downbeat ambiguity, anchor alignment, and groove alignment, what remaining timing evidence should be surfaced next?
Decision: add generated-path phrase count and phrase-bar evidence to the compact P012 Markdown summary after the equivalent generated TSV index surface lands.
Why: the Source Timing spec requires observer/audio summaries to preserve phrase count and phrase-bar evidence so QA can distinguish no phrase grid, short-loop material, and stable preliminary phrase evidence without reopening manifests. The Markdown phase proof now exposes most generated-path timing evidence but still hides phrase counts in JSON.
Evidence: `docs/reviews/p012_proof_surface_review_2026-05-22.md` found no blocker after `RIOTBOX-946` through `RIOTBOX-950`, but identified the remaining Markdown phrase-evidence gap as the next bounded implementation slice.
Consequences: this is display/validator work only. It must not change analyzer behavior or treat bounded generated phrase evidence as production-grade arbitrary-audio phrase detection.
Status: accepted

---

### RBX-037

Date: 2026-05-23
Topic: user-confirmed source timing grid is session truth
Phase: Source Timing Intelligence
Question: where should musician acceptance of a source timing grid live?
Decision: user grid acceptance is recorded through `source_timing.confirm_grid` and persisted in `runtime_state.source_timing.confirmed_grid` with source id, hypothesis id, confirming action id, and timestamp.
Why: confirmation changes how much trust the musician and later workflows can place in the selected timing hypothesis after audition, so it must survive save/load and replay instead of living as TUI-local state.
Evidence: RIOTBOX-972 added typed action params, queue / commit side effects, replay support, observer fields, session serialization coverage, and app queue/commit tests.
Consequences: Source Graph evidence remains unchanged; analyzed confidence and user-accepted trust stay separate. Revert / undo semantics still need a dedicated follow-up before confirmation can be removed explicitly.
Status: accepted

---

### RBX-038

Date: 2026-05-23
Topic: confirmed grid wording is musician trust, not analysis mutation
Phase: Source Timing Intelligence / Musician UX
Question: how should Jam and Source show a user-confirmed timing grid after the action commits?
Decision: display confirmed grid state from `runtime_state.source_timing.confirmed_grid` as `grid confirmed` / `user confirmed` on Jam and Source timing surfaces, while preserving Source Graph confidence, warning, and degraded-policy evidence as separate analysis facts.
Why: the musician needs a clear “I accepted this after listening” cue, but replay and QA still need to distinguish that acceptance from analyzer certainty.
Evidence: RIOTBOX-973 wires the confirmation key outcome into the shell event loop, commits the immediate action, and renders the confirmation state in Jam/Source timing text.
Consequences: future Source Map trust-row and lane-consumer slices should read the same session truth instead of inventing TUI-local confirmation flags or mutating Source Graph analysis.
Status: accepted

---

### RBX-039

Date: 2026-05-23
Topic: source timing grid confirmation has an explicit revert action
Phase: Source Timing Intelligence
Question: how should user-accepted timing trust be removed without corrupting analysis evidence?
Decision: add `source_timing.revert_grid` as the explicit counterpart to `source_timing.confirm_grid`. It is a session-scope immediate action carrying the confirmed source id and optional hypothesis id, and it clears matching `runtime_state.source_timing.confirmed_grid` through queue, commit side effect, and replay.
Why: confirmation is musician trust, not analyzer truth. Removing that trust must be auditable and replayable, and must not delete Source Graph timing evidence or rely on UI-local state.
Evidence: RIOTBOX-974 wires the revert action through core command vocabulary, app queue/commit side effects, replay executor support, shell status handling, and focused tests.
Consequences: downstream lane consumers can treat confirmation as reversible session truth. Full undo semantics remain separate; this action is the explicit trust-state removal path.
Status: accepted

---

### RBX-040

Date: 2026-05-23
Topic: Source Map navigation uses transport seek, not an editor cursor
Phase: Source Timing Intelligence
Question: how should musicians move through the analyzed source map before capture without turning Riotbox into a destructive sample editor?
Decision: expose typed shell intents for previous / next bar and previous / next phrase, but commit the result as `transport.seek` with structured `position_beats`. The Source Map renders the current region from Session transport state and Source Graph evidence.
Why: the musician needs fast, predictable source audition navigation, while replay / restore must have one transport truth. A separate Source Map cursor would create a hidden second selection model and weaken capture provenance.
Evidence: RIOTBOX-975 adds Source Map navigation intents, arrow-key outcomes, immediate seek commit handling, transport side effects, observer labels, Source Map current-region rendering, and focused control/render tests.
Consequences: capture-length and source-audition slices should consume the same transport position instead of inventing arbitrary sample-range selection. Future waveform/canvas work may visualize the position more densely but must not replace the transport seek contract.
Status: accepted

---

### RBX-041

Date: 2026-05-23
Topic: capture length is session intent that drives source windows
Phase: Source Transport / Capture Workflow
Question: how should a musician choose “one beat”, “one bar”, “four bars”, or “phrase” without turning Source Map into a sample editor?
Decision: add `capture.set_length` as an immediate session-scope action and persist the selected `CaptureLengthIntent` in `runtime_state.capture`. Subsequent source-window capture actions may omit explicit bars and resolve their window from the selected intent at commit time.
Why: the musician needs to know what `c` will capture before pressing it, and replay / restore need the same length choice without a hidden TUI selector or arbitrary sample-range model.
Evidence: RIOTBOX-976 wires typed capture-length params, queue / commit side effects, replay support, shell controls, observer state, Capture screen rendering, and source-window tests for one-beat and phrase-grid durations.
Consequences: capture remains a musical gesture on the transport grid. Phrase length depends on Source Timing phrase evidence and falls back to the established four-bar span when phrase evidence is missing; future waveform/canvas work should visualize this range without replacing the typed intent contract.
Status: accepted

---

### RBX-042

Date: 2026-05-23
Topic: Source Map trust row reads user-confirmed grid state
Phase: Source Timing Intelligence / Musician UX
Question: how should Source Map show a grid that the musician confirmed after listening when analyzer confidence still required manual confirmation?
Decision: derive Source Map trust from matching `runtime_state.source_timing.confirmed_grid` before falling back to analyzer cue text. A matching source/hypothesis confirmation renders as `grid confirmed` and allows bar-grid Source Map mode without mutating Source Graph timing evidence.
Why: the musician needs the Source Map to reflect the accepted trust decision directly, while replay and QA still need the distinction between analyzer confidence and user acceptance.
Evidence: RIOTBOX-977 adds Source Map projection tests for confirmed, unconfirmed, and mismatched confirmation state plus Source screen render coverage.
Consequences: lane consumers should continue to read typed Session trust state directly in later slices; Source Graph confidence and warnings remain analysis evidence, not user-trust state.
Status: accepted

---

### RBX-043

Date: 2026-05-23
Topic: source timing confirmation needs observer-probe proof
Phase: Source Timing Intelligence / QA
Question: how should the manual grid confirmation path be proven beyond unit tests and static render assertions?
Decision: add `just source-timing-confirmation-probe` as a CI-safe headless observer probe. The probe presses the real `C` shell control, validates the normal user-session observer stream, records the immediate `source_timing.confirm_grid` commit, and asserts `grid_confirmed` runtime state without rewriting Source Graph cue or warning evidence.
Why: confirmation is musician trust, not analyzer truth. A repeatable observer proof makes the user-visible control path auditable across key outcome, queue history, commit event, Session runtime state, and observer fields.
Evidence: RIOTBOX-978 adds the `source-timing-confirmation` probe scenario, normal observer validation, focused probe tests, and an `audio-qa-ci` target.
Consequences: future source timing trust changes should extend this probe or add a sibling probe instead of relying only on UI snapshots.
Status: accepted

---

### RBX-044

Date: 2026-05-23
Topic: lane consumers gate source windows on typed timing readiness
Phase: Source Timing Intelligence / Capture Workflow
Question: how should source-window consumers distinguish analyzer-locked timing, manual-confirm-required timing, and user-confirmed timing?
Decision: add shared typed `SourceTimingConsumerReadiness` in the Jam view/core contract. Source-window capture may use analyzer-locked timing or a matching user-confirmed grid, but unconfirmed manual-confirm timing does not silently materialize bar-accurate source-window reuse.
Why: manual confirmation is musician trust, not analyzer truth. Downstream lanes need a stable typed contract instead of branching on raw `grid_use` strings or app-local UI state.
Evidence: RIOTBOX-979 wires capture source-window materialization through the shared readiness helper and tests unconfirmed vs user-confirmed manual timing.
Consequences: later TR-909 / MC-202 / W-30 source consumers should reuse the same helper when they need to decide whether timing is only analyzed, requires user confirmation, or has been accepted by the musician.
Status: accepted

---

### RBX-045

Date: 2026-05-23
Topic: Source Map waveform rendering defaults to block rows, with Canvas optional
Phase: Source Timing Intelligence / TUI UX
Question: should the Source Map waveform move to Ratatui Canvas now, or keep the 1-2 row block renderer as the default?
Decision: keep the default Source Map waveform as one or two plain-text block rows, with separate marker/text rows for peaks, bars, playhead, capture range, timing mode, and trust. Ratatui `Canvas` remains an optional expanded Source/Lab renderer only after the block-map contract is stable and enough terminal height exists to make dense waveform rendering readable.
Why: musicians need fast orientation more than sample-editor density. Block rows are readable in narrow panels, stable in monochrome snapshots, and easy to test from the shared Source Map projection. Canvas/Braille can be useful for diagnostics, but it is harder to parse at a glance and more sensitive to terminal/font and Ratatui marker behavior.
Evidence: RIOTBOX-980 records the comparison in `docs/reviews/source_map_waveform_canvas_spike_2026-05-23.md` and updates the TUI/source-map plan contracts.
Consequences: future Source Map implementation should improve the block renderer first. Any Canvas path must consume the same `SourceMapView` / source-window projection data and must not introduce a second Source Map truth or hide playhead/grid trust/capture range behind dense glyphs or color.
Status: accepted

---

### RBX-046

Date: 2026-05-23
Topic: Source Map capture range is read-only capture-intent projection
Phase: Source Timing Intelligence / Capture Workflow
Question: how should the Source Map show what `c` is likely to capture without becoming a sample editor?
Decision: add a compact capture-range marker row to the Source Map view model. The row is derived from Session transport position, Source Timing readiness, and `runtime_state.capture.length_intent`; it appears only when timing readiness permits bar-accurate source-window reuse.
Why: musicians need to see the practical effect of `1 beat`, `1 bar`, `4 bars`, or `phrase` before pressing capture, but arbitrary start/end selection would create a second editor cursor and weaken replayable transport truth.
Evidence: RIOTBOX-981 projects the range in `SourceMapView`, renders it on the Source screen, and tests locked/confirmed timing versus fallback/manual-confirm timing.
Consequences: later capture UI may make the marker richer, but it must remain projection-only unless a future action/session contract explicitly adds manual selection.
Status: accepted

---

### RBX-047

Date: 2026-05-23
Topic: observer snapshots expose Source Map projection state
Phase: Source Timing Intelligence / QA
Question: how should QA verify Source Map capture-range visibility without relying only on terminal snapshots?
Decision: add a top-level `source_map` observer snapshot block sourced from the shared `SourceMapView`. It exposes mode, trust label, playhead column, capture range row / availability, navigation hint, and capture hint.
Why: the capture-range marker is musician-facing projection state. Observer QA needs to prove the same visible truth as the TUI while avoiding a second observer-only Source Map derivation.
Evidence: RIOTBOX-982 adds observer fields and focused tests for bar-grid timing versus untrusted fallback timing.
Consequences: future Source Map UI changes should keep observer output aligned with `SourceMapView`; observer code must not independently recompute timing readiness, capture range, or navigation state.
Status: accepted

---

### RBX-048

Date: 2026-05-23
Topic: user-session probes assert Source Map capture-range availability
Phase: Source Timing Intelligence / QA
Question: how should the new `source_map` observer projection be covered beyond unit-level observer snapshots?
Decision: extend existing user-session observer probe tests to assert `source_map` mode and capture-range availability for locked/bar-grid timing and unavailable capture ranges for fallback or untrusted timing.
Why: this keeps the visible Source Map capture target in the same headless observer stream used for broader workflow QA, and catches drift between unit snapshots and probe-generated events.
Evidence: RIOTBOX-983 adds assertions to the Feral-grid observer probe tests for locked, cautious/manual-review, and fallback timing paths.
Consequences: future probe scenarios that rely on capture intent should check `source_map` fields rather than scraping rendered terminal text.
Status: accepted

---

### RBX-049

Date: 2026-05-23
Topic: Source capture preview and queue target the next bar boundary
Phase: Source Timing Intelligence / Capture Workflow
Question: should the Source Map `cap` preview follow the current playhead beat or the boundary where pressing `c` will actually land?
Decision: make `capture.bar_group` queue at `next_bar` and project the Source Map capture range from the next bar boundary after the current Session transport position. The range remains read-only and is derived from Session transport, Source Timing readiness, and `runtime_state.capture.length_intent`.
Why: musicians read `cap next bar` as a promise that pressing `c` will land on the next bar. Starting the marker at the floored current beat made the map look like an editor selection and could disagree with the queued action boundary.
Evidence: RIOTBOX-984 changes the capture queue target, SourceMapView range projection, observer snapshot expectation, Capture screen pending cue, and focused tests.
Consequences: follow-up capture/source-window slices must preserve this next-bar start contract when tightening committed source-window provenance and end-to-end audio QA.
Status: accepted

---

### RBX-050

Date: 2026-05-23
Topic: observer snapshots expose committed capture source-window provenance
Phase: Source Timing Intelligence / Capture Workflow QA
Question: how should QA prove that a visible Source Map capture range landed as the committed source window?
Decision: add a top-level `capture` observer snapshot block for the latest capture. When the capture has source-window provenance, the snapshot exposes source id, start/end seconds, duration, frame bounds, source-origin count, and the creating action id.
Why: Source Map preview tests prove what the musician should expect before pressing `c`, but source-window QA also needs a stable observer surface for what actually landed after commit. This avoids relying only on rendered text or internal Session inspection.
Evidence: RIOTBOX-985 adds observer fields and a committed `capture.bar_group` snapshot test at a `Bar` boundary.
Consequences: later end-to-end workflow probes should correlate `source_map.capture_range_row`, `transport_commit` boundary, and `capture.source_window` instead of deriving capture provenance separately.
Status: accepted

---

### RBX-051

Date: 2026-05-23
Topic: Capture screen states the capture target boundary before queueing
Phase: Source Timing Intelligence / Capture Workflow UX
Question: how should the Capture screen explain what `c` will queue before the musician commits a capture?
Decision: show a compact target label in the Capture screen derived from runtime capture length intent and the shared Source Map readiness/range projection. Trusted or user-confirmed timing renders labels such as `4 bars @ next bar`; untrusted timing renders `4 bars @ listen first`; phrase capture without phrase evidence renders compact fallback as `phrase->4bar @ next bar`.
Why: musicians need to know whether `c` is going to land cleanly before they press it. The label must not invent a second selector; it should describe the same projection used by Source Map and committed source-window QA.
Evidence: RIOTBOX-986 adds Capture screen target labels and render coverage for listen-first, next-bar, and phrase-fallback cases.
Consequences: future Capture screen controls should keep this label aligned with `SourceMapView` and the committed capture observer snapshot rather than recomputing unrelated UI-only state.
Status: accepted

---

### RBX-052

Date: 2026-05-23
Topic: Source monitor seek proof uses the callback-equivalent offline render seam
Phase: Source Timing Intelligence / Source Transport QA
Question: how should QA prove that source-map bar or phrase navigation changes what the musician hears?
Decision: prove source seeking at the source monitor mix seam by rendering a decoded source cache before and after a transport-position seek with transport still running. The fixture uses distinct bar-level markers and asserts non-silent output plus a material RMS delta between the pre-seek and post-seek excerpts.
Why: source-map navigation already commits a `transport.seek` without pausing. The missing proof was not another UI/log assertion; it was an output-path assertion that `position_beats` selects a different decoded source window in the monitor path consumed by the realtime callback.
Evidence: RIOTBOX-987 adds `source_monitor_seeked_running_transport_changes_audible_source_excerpt`, which renders through `render_source_monitor_mix_offline` and the same source monitor policy used by the audio callback.
Consequences: later end-to-end probes can build on this seam when adding full user-session source transport QA, but they should keep source cache loading outside the realtime callback and compare audible output rather than only observer text.
Status: accepted

---

### RBX-053

Date: 2026-05-23
Topic: Source transport restore compares projection output, not app-local flags
Phase: Source Timing Intelligence / Restore QA
Question: how should restore QA prove that confirmed-grid source transport state still drives capture projection after reload?
Decision: save/restore tests should compare the full `SourceMapView` projection before and after reload while also asserting the underlying Source Graph timing evidence is unchanged. Replay coverage should apply the same durable source transport actions into `RuntimeState`: play/seek, monitor mode, source grid confirmation, and capture length intent.
Why: the musician-visible contract is the Source Map/capture projection, not an internal boolean. Confirmed-grid state lives in Session runtime truth and must unlock bar-accurate projection without mutating analyzer evidence; unconfirmed manual-confirm timing must remain a listen-first fallback after reload.
Evidence: RIOTBOX-988 adds confirmed and unconfirmed restore tests for Source Map capture projection plus a core replay test for the same runtime action family.
Consequences: future end-to-end source transport QA should compare observer/TUI projection surfaces derived from Session and Source Graph state instead of adding hidden app-local restore flags.
Status: accepted

---

### RBX-054

Date: 2026-05-23
Topic: Source transport map/capture workflow gets one observer-plus-output gate
Phase: Source Timing Intelligence / Workflow QA
Question: how should the complete Source Transport / Map / Capture recipe be proven without relying on a manual terminal session?
Decision: add `just source-transport-map-capture-probe`, a CI-safe workflow proof that starts with manual-confirm timing in listen-first mode, confirms the grid, seeks the Source Map, captures a source-window-backed bar group, raw-auditions, promotes, triggers W-30, and correlates the observer path with a W-30 source-vs-fallback output comparison.
Why: the parent workflow needs one musician-path proof that joins visible Source Map trust/range evidence, committed Session/action evidence, and actual output metrics. Separate unit tests prove pieces; this gate proves they compose without inventing a second QA path.
Evidence: RIOTBOX-989 adds a `source-transport-map-capture` user-session observer probe, a validation script, Just target, and audio QA docs. The gate asserts unconfirmed capture range remains unavailable, confirmed range becomes available, `transport.seek` lands, capture source-window provenance appears, and W-30 candidate output differs from fallback.
Consequences: future polish can broaden this into richer listening packs or Canvas waveform review, but the source transport recipe should keep this observer-plus-output gate as the minimum regression proof.
Status: accepted

---

### RBX-055

Date: 2026-05-23
Topic: Source Map block rows read typed bucket evidence first
Phase: Source Timing Intelligence / Source Map
Question: how should the default block Source Map become more waveform-like without adding a second timing or editor model?
Decision: add defaultable `source_map.buckets` evidence to Source Graph and have `SourceMapView` prefer those buckets for energy and peak rows, while preserving section-energy and anchor/asset fallbacks when bucket evidence is absent.
Why: the musician-facing block rows need a durable analysis contract before later visual polish. Buckets let the compact map reflect decoded source energy/transient shape without turning the Source Map into sample selection state or Ratatui Canvas-specific rendering.
Evidence: RIOTBOX-990 adds Source Graph bucket types, legacy JSON default handling, bucket-backed Source Map row tests, and spec updates.
Consequences: future sidecar extraction can populate these buckets from decoded audio, and future Canvas/Braille views must consume the same bucket/projection data rather than introducing separate waveform truth.
Status: accepted

---

### RBX-056

Date: 2026-05-23
Topic: Decoded WAV sidecar emits Source Map bucket evidence
Phase: Source Timing Intelligence / Source Map
Question: when should `source_map.buckets` become real analysis output instead of only fixture data?
Decision: make the existing decoded-WAV stdio sidecar baseline emit a bounded deterministic bucket set. Each bucket carries time span, RMS-derived energy class, local peak / positive-flux peak class, confidence, and `provider:decoded.wav_baseline` provenance.
Why: the compact Source Map should reflect source audio shape during normal ingest, not only tests. Keeping the algorithm bounded and simple preserves the current baseline sidecar role while making the product contract observable through Rust Serde and user-session views.
Evidence: RIOTBOX-991 extends `json_stdio_sidecar.py` bucket extraction and the Rust sidecar client test to assert parsed bucket evidence on generated PCM WAV input.
Consequences: future MIR work can improve bucket quality behind the same contract; bucket evidence still does not replace timing, seek, or capture-window authority.
Status: accepted

---

### RBX-057

Date: 2026-05-23
Topic: Source Map bucket evidence is proven through app ingest observer projection
Phase: Source Timing Intelligence / Source Map QA
Question: how should QA prove decoded Source Map buckets reach the musician-facing surface after normal ingest?
Decision: add an app-bin ingest test that analyzes a generated WAV through the stdio sidecar, builds the normal Jam shell state, and compares observer `source_map` rows with the shared `SourceMapView` projection.
Why: sidecar-client parsing proves the JSON contract, but the musician sees the app projection. The proof must cover the bridge from decoded analysis output into Jam/observer state without scraping terminal text or adding a second Source Map derivation.
Evidence: RIOTBOX-992 adds `source_map_bucket_ingest`, which asserts parsed bucket evidence, a multi-level bucket-backed energy contour, peak evidence, and observer snapshot parity with `SourceMapView`.
Consequences: future Source Map UI polish should keep observer snapshots aligned with `SourceMapView` so decoded bucket evidence remains testable without terminal-only baselines.
Status: accepted

---

### RBX-058

Date: 2026-05-27
Topic: Beat20 real-source downbeat remains ambiguous without stronger anchor evidence
Phase: Source Timing Intelligence / Real-source timing confidence
Question: should the current Beat20 local example be promoted by simple low-band downbeat scoring?
Decision: keep Beat20-like rows as manual-confirm-only / ambiguous-downbeat unless a future detector adds stronger musical evidence such as trusted kick/backbeat anchors, a clearly larger downbeat phase margin, or another documented downbeat cue.
Why: the current local report has stable BPM/beat evidence but downbeat margin `0.005`, three alternate downbeat phases, and transient-only anchors. A bounded 165 Hz low-band RMS/flux probe raised the best observed phase margin only to about `0.027`, still below the existing `0.05` ambiguity margin and not enough to justify locked or short-loop-manual-confirm behavior.
Evidence: RIOTBOX-1014 records the Beat20 feasibility guardrail in `docs/reviews/p012_beat20_downbeat_feasibility_2026-05-27.md`.
Consequences: future P012 detector work can still target Beat20, but it must add new evidence and preserve the current fallback for sources without defensible bar-one support.
Status: accepted

---

### RBX-059

Date: 2026-05-29
Topic: P013 closes as bounded representative all-lane musical-depth baseline
Phase: P013 / All-Lane Musical Depth
Question: when is P013 complete enough to move the active roadmap to P014?
Decision: close P013 after the representative showcase carries explicit output proof for W-30 source accent/slice/chop behavior, TR-909 source accent/kick-pressure behavior, MC-202 pressure/grid/source-contour behavior, and all-lane generated-support mix movement, while the P012 all-lane source-grid proof still passes.
Why: P013's roadmap goal was deeper TR-909, MC-202, W-30, and mix behavior on the P012 timing spine, not a finished arranger, source-derived MC-202 phrase planner, final W-30 loop detector, or taste oracle. Keeping P013 bounded prevents the musical-depth phase from swallowing P014 arrangement / scene work.
Evidence: RIOTBOX-1029 records the P013 exit review in `docs/reviews/p013_exit_review_2026-05-29.md`; `just p012-all-lane-source-grid-output-proof` and the representative showcase musical-quality gate passed on 2026-05-29 with a selected `tonal_hook_chop/head` candidate.
Consequences: P014 becomes the active next execution track. Future arrangement / scene work must preserve the P012 timing and P013 representative musical-depth regression baselines instead of creating a parallel timing, arrangement, or mix truth.
Status: accepted

---

### RBX-060

Date: 2026-05-30
Topic: P014 starts with a read-only Arrangement / Scene contract on the product spine
Phase: P014 / Arrangement / Scene System
Question: how should Arrangement / Scene work begin without creating a second arranger or scene truth?
Decision: add a shared `ArrangementSceneContractView` to the Jam/Core view layer and document P014 in `docs/specs/arrangement_scene_system_spec.md`. The view is read-only: it derives readiness from Source Graph presence, Source Timing consumer readiness, Session scene state, queued scene transitions, and landed Scene movement state.
Why: P014 needs an explicit contract before adding source playback repositioning, scene chains, or richer transition policy. Keeping the first slice as a derived contract makes later P014 behavior reviewable against Source Graph, Session, Action Lexicon, queue / commit, replay, observer/audio, and P012/P013 regression baselines instead of letting a shadow arranger appear in app-local state.
Evidence: RIOTBOX-1030 adds the contract view, targeted Jam view tests for ready/manual-confirm/fallback boundaries, `docs/specs/arrangement_scene_system_spec.md`, and `docs/architecture_phase_map.md`.
Consequences: future P014 slices must extend this contract and prove output path for audible changes. Contract-only changes may use targeted core/app tests plus docs/spec updates; audible arrangement behavior must add non-collapsed audio evidence.
Status: accepted

---

### RBX-062

Date: 2026-05-31
Topic: P016 export QA evidence policies need identity validation before real stem claims
Phase: P016 / Pro Workflow / Export
Question: after adding typed artifact-set lineage and fallback comparison evidence slots, is presence-only structural QA enough for future stem export scopes?
Decision: keep the current lineage and fallback comparison policies as opt-in structural gates, but require a follow-up validation slice before any real stem export scope depends on them. The follow-up should reject blank source graph hashes, blank capture ids, blank fallback reference identities, and comparison evidence with no metric fields.
Why: the P016 export spine now has the right typed places for evidence, but a `Some` evidence object or non-empty list is not enough to prove useful provenance. Tightening the identity shape before stem writing prevents future PRs from claiming stronger QA with placeholder data.
Evidence: `docs/reviews/p016_export_qa_evidence_broad_review_2026-05-31.md` records the finding after RIOTBOX-1076, RIOTBOX-1075, RIOTBOX-1077, RIOTBOX-1078, and RIOTBOX-1079.
Consequences: the next P016 slice should strengthen structural validation only. It should not implement stem writing, DAW export, live recording export, real source-vs-fallback rendering, or threshold interpretation.
Status: accepted

---

### RBX-063

Date: 2026-06-03
Topic: Automated musical fitness reports reject known bad output modes without claiming taste approval
Phase: P000 / Audio QA Workflow
Question: how should Riotbox add a code-driven anti-bad-output gate without pretending metrics can certify musical taste?
Decision: add `riotbox.automated_musical_fitness.v1` as a deterministic manifest-metrics validator. The report separates `technical_status`, `automated_musical_fitness_status`, `result`, selected candidate, failure codes, compact score breakdown, and `human_verdict: unverified`.
Why: existing representative showcase checks were too showcase-specific and could encourage agents to treat one known-good example shape as musical quality. A stable automated fitness contract can reject silence, fallback collapse, source-fake/masked output, static loops, lane imbalance, and weak low-end/transient evidence while leaving human taste approval explicit and unverified.
Evidence: RIOTBOX-1107 adds `scripts/validate_automated_musical_fitness.py`, deterministic positive/negative fixtures, and `docs/benchmarks/automated_musical_fitness_v1_2026-06-03.md`.
Consequences: future fixture and CI work should broaden source families and cross-source examples instead of tuning around one representative showcase. Passing automated fitness means "no known bad-output mode caught", not "this sounds good".
Status: accepted

---

### RBX-064

Date: 2026-06-03
Topic: First stem-package writer boundary stays local, explicit, and gated
Phase: P016 / Pro Workflow / Export
Question: what is the first allowed stem-package writer boundary before `export.stem_package` can become runnable?
Decision: reserve `stem_package.local_ci_package_v1` as the first writer boundary: a future local app-side side-effect path for an explicit Session export request, starting with deterministic offline stem render providers for proven roles such as drums/bass, a fixed local package layout, and commit only after written stems, manifest, proof, receipt evidence, and all required stem-package QA gates pass.
Why: P016 now has receipt, manifest, proof, readiness, observer, and QA gate contracts. The next risk is a writer accidentally inferring stems from product-mix proof, filenames, observer state, or fallback placeholders. A named boundary keeps the first writer narrow and reviewable without surfacing it prematurely to musicians.
Evidence: RIOTBOX-1126 updates the Action Lexicon, Session, and Audio QA specs with source-of-stems, destination layout, reusable/new writer pieces, commit/receipt order, and minimal output proof.
Consequences: future implementation tickets may build only this local boundary first. UI, Ghost, or CLI surfacing still requires CI-safe writer proof with per-stem non-silence, repeated hash stability, lineage, fallback comparison, manifest/proof files, and explicit listening-review status when audible behavior changes.
Status: accepted

---

### RBX-065

Date: 2026-06-04
Topic: Audio Judge / Musical Fitness becomes a planned calibrated quality track
Phase: P021 / Audio Judge / Musical Fitness
Question: should Riotbox treat agent-side musical-pass judgment as part of the current sound-quality work or as its own roadmap track?
Decision: create `P021 | Audio Judge / Musical Fitness` as a planned follow-on project while keeping the current dense-break 8-bar source-backed performance Golden Path as the immediate sound-quality implementation target.
Why: Riotbox needs stronger audible output before TUI/export polish has product value, but an agent cannot honestly claim `musical_pass` from logs or simple metrics alone. A calibrated judge needs review packs, deterministic audio evidence, human pass/weak/fail labels, and a music-understanding spike such as CLAP/MERT-style embeddings combined with Riotbox-owned metrics.
Evidence: Linear P021 was created with initial issues RIOTBOX-1185 through RIOTBOX-1188 covering the agent musical review pack, human label corpus, CLAP/MERT-style judge spike, and musical-pass gate policy.
Consequences: future agents may use P021 to block weak output and eventually assign calibrated agent musical-pass verdicts, but P021 must not become a hidden taste oracle, second arranger, runtime dependency, or replacement for human listening before labeled Riotbox examples validate it.
Status: accepted

---

### RBX-066

Date: 2026-06-04
Topic: Audio judge spike starts as offline calibration evidence, not a taste oracle
Phase: P021 / Audio Judge / Musical Fitness
Question: how should Riotbox prototype CLAP/MERT-style musical judging without letting model output become product truth?
Decision: add `riotbox.audio_judge_spike.v1` as an offline QA spike that compares a deterministic Riotbox metrics baseline against human listening label coverage while reporting optional CLAP/MERT-style provider availability. The recommendation must stay `not_ready` until enough generated review packs are matched to pass/weak/fail labels and any embedding provider proves value beyond metrics.
Why: Riotbox needs a path for agents to reason about audible quality, but musical pass cannot be inferred from logs, deterministic metrics, or uncalibrated embeddings. Keeping the spike optional and report-driven prevents hidden runtime dependencies, hidden taste memory, and premature product claims.
Evidence: RIOTBOX-1187 adds `scripts/prototype_audio_judge_spike.py`, committed fixture coverage, generated dense-break smoke wiring, and the `audio_judge_spike_v1_2026-06-04` benchmark note.
Consequences: future P021 work should add real labeled packs, weak/fail examples, and provider comparison before changing musical-pass policy. CLAP/MERT-style providers remain offline QA experiments unless calibration evidence shows they separate weak hooks and strong hooks better than Riotbox metrics alone.
Status: accepted

---

### RBX-067

Date: 2026-06-04
Topic: Musical-pass language is gated by explicit agent and human verdict states
Phase: P021 / Audio Judge / Musical Fitness
Question: when may Riotbox automation, humans, or a future calibrated judge claim that audio has musically passed?
Decision: add `riotbox.musical_pass_gate_policy.v1` with eight explicit states: `technical_fail`, `technical_pass`, `agent_fail`, `agent_weak`, `agent_promising`, `human_musical_pass`, `human_musical_fail`, and `calibrated_agent_musical_pass`. Only the human pass and future calibrated-agent pass states may claim musical pass.
Why: P021 needs agents to block weak output autonomously without turning logs, simple metrics, or uncalibrated embeddings into taste approval. The policy makes `agent_promising` useful but limited, keeps `human_verdict: unverified` honest, and defines the evidence floor for any future calibrated agent pass.
Evidence: RIOTBOX-1188 adds `scripts/validate_musical_pass_gate_policy.py`, committed policy fixtures, `just musical-pass-gate-policy-fixtures`, and audio-QA/benchmark documentation.
Consequences: future PRs must use these verdict terms precisely. A calibrated agent musical pass needs offline judge validation, matched pass/weak/fail labels, and source-family boundaries before it can be claimed.
Status: accepted

---

### RBX-068

Date: 2026-06-04
Topic: Dense-break Golden Path prioritizes pro-pressure sound output before export/TUI polish
Phase: P022 / Professional Sound Output
Question: how should Riotbox prevent the dense-break Golden Path from passing while still sounding underpowered or merely fixture-valid?
Decision: make the dense-break performance render itself more aggressive and add pro-pressure guards on top of the existing anti-collapse checks. The render now uses a source-present break hook, explicit W-30 hook-riff layer, harder break snaps, stronger MC-202/sub pressure, destructive dropout/stutter impact, a controlled slam bus, and peak-normalized restore. The pack must prove full-performance assertiveness against the source, hook transient, pressure-over-hook lift, and restore-over-pressure impact while keeping `human_verdict: unverified`.
Why: Riotbox needs the thing it exports or surfaces in the UI to sound useful first. Export polish or TUI affordances are not valuable if the rendered musical result is still polite, weak, or only technically valid.
Evidence: RIOTBOX-1192 updates `scripts/generate_dense_break_performance_pack.py`, the dense-break smoke gate, and the dense-break/agent-review benchmark docs. The local comparison from the old baseline to the new render raises full-performance RMS from about -20.6 dBFS to about -14.9 dBFS, chop-hook transient from about 0.104 to about 0.148, dropout/stutter transient from about 0.045 to about 0.112, and restore RMS from about -17.3 dBFS to about -11.2 dBFS without claiming human musical pass.
Consequences: future sound-quality work should improve audible render behavior first and then tighten objective proof. `agent_promising` remains a bounded anti-weak-output verdict, not a musical-pass claim.
Status: accepted

---

### RBX-069

Date: 2026-06-05
Topic: Scripted audio render packs are diagnostics, not product-quality proof
Phase: P021 / P022 / P023 Audio QA and Sound Excellence
Question: may hardcoded or scripted generated audio count as technical or musical quality proof for Riotbox?
Decision: no. Hardcoded or scripted audio generation may stay in professional-output workflows only as smoke, regression, or diagnostic evidence. It can prove that a harness runs, artifacts are fresh/hash-bound, known weak-output shapes are rejected, and listening-review plumbing works. It must not prove technical production quality, musical pass, source-aware production behavior, bass pressure quality, or product demo readiness. Reports that use scripted generation should expose machine-readable evidence boundaries such as `evidence_role`, `quality_proof: false`, `scripted_generation: true`, and `human_verdict: unverified`.
Why: the current dense-break and professional-output packs use real Riotbox stems but still include scripted arrangement behavior such as fixed section roles, fixed mix weights, and a `pressure_lift` that is not yet a source-aware production decision. Treating those outputs as quality proof would let Riotbox pass by rehearsing a known recipe instead of proving musician-useful source transformation. The MC-202 `bass_pressure` stem also shows why naming matters: it can read more like a melodic contour phrase than true low-band pressure.
Evidence: P023 roadmap text now states that `pressure_lift` and rise/restore gestures must become source-aware production decisions, and that hardcoded/scripted audio may be used only as smoke/regression/diagnostic evidence. RIOTBOX-1202, RIOTBOX-1206, and RIOTBOX-1207 were updated with the same distinction.
Consequences: next implementation work should add machine-readable evidence-boundary fields to professional-output reports and CI should fail if scripted diagnostics claim `quality_proof: true` or are counted as product quality. RIOTBOX-1201 remains valid as negative rendered diagnostic fixtures, not quality proof. Product demos and quality proof require source-aware policy, non-hardcoded fixture evidence, and appropriate human or calibrated verdict states.
Status: accepted

---

### RBX-070

Date: 2026-06-05
Topic: Pad/noise edge sources use an explicit cautious pressure policy
Phase: P022 / Professional Sound Output
Question: should thin low-band, high-noise pad material be allowed to pass through the dense-break pressure policy when transient-like motion is present?
Decision: no. Pad/noise-like sources that have very low low-band energy but strong high-band/noise energy and enough transient-like motion take a bounded `pad_noise` pressure-policy path. That path treats the source as a gated texture candidate, not as dense-break proof, and stays diagnostic with `quality_proof: false` and `human_verdict: unverified`.
Why: the edge-source diagnostic showed `DH_Fadapad_120_A.wav` could be classified as `dense_break` because noisy high-band motion tripped the dense-break transient rule. That made the warning useful, but the policy itself still pretended pad/noise was break material. Riotbox should either transform pad/noise as texture or cue/reject it, not promote it to a breakbeat contract.
Evidence: RIOTBOX-1219 adds the `pad_noise` pressure policy, updates edge-source diagnostics and the professional-output suite to require the new policy path, and keeps the case weak-routed with concrete source-selection/UI/chop/destructive follow-ups.
Consequences: future pad/noise work should improve the gated texture/stab behavior or user cue from this explicit family. It must not reintroduce dense-break promotion as a passing edge-source path unless a later spec defines a stronger source-family classifier and corresponding proof.
Status: accepted

---

### RBX-071

Date: 2026-06-05
Topic: Ambiguous downbeat sources use a cautious confirmation path, not confident bar-locked arrangement
Phase: P022 / Professional Sound Output
Question: should bad-timing edge sources be allowed to pass through the normal dense-break/destructive arrangement policy while source timing is ambiguous?
Decision: no. Sources whose timing report is `candidate_ambiguous` or `manual_confirm_only` take an explicit `bad_timing` pressure-policy path and a `manual_confirm_cautious_arrangement` timing policy. That policy keeps the render audible, emits a confirmation cue, requires user confirmation, and rejects confident bar-locked policy until timing is confirmed.
Why: a wrong downbeat is musically worse than an honest caution path. Riotbox should not arrange a source as if the grid were proven when the timing layer says the performer must confirm it first.
Evidence: RIOTBOX-1220 updates the dense-break performance generator to accept source-timing confidence, routes the bad-timing edge case into `bad_timing`, adds edge-source diagnostics and negative fixtures for missing cautious policy / bad bar-lock allowance, and surfaces the family in the professional-output suite key metrics.
Consequences: future bad-timing work should improve source-aware confirmation, user cueing, and safe destructive/cut behavior behind this policy. Passing the edge diagnostic remains smoke/regression evidence only; it must keep `quality_proof: false` and `human_verdict: unverified`.
Status: accepted

---

### RBX-072

Date: 2026-06-05
Topic: Sparse bass-pressure movement must be source-derived before it can count as stronger P022 evidence
Phase: P022 / Professional Sound Output
Question: may sparse-bass pressure keep using only fixed frequency contours while claiming source-aware production movement?
Decision: no. Sparse-bass-pressure diagnostics now derive bass-pressure movement from the source low-band envelope and timing centroid, then report whether the movement is source-derived, how far it moved away from the old fixed contour, and how much frequency span the pressure gesture covers. Sparse reports must fail when the movement collapses back to the fixed contour or lacks variation.
Why: the prior sparse path could sound more like a hardcoded melodic contour than bass pressure that reacts to the source groove. Riotbox needs bass pressure that shoves with the input material, not just a label on a fixed MC-202 gesture.
Evidence: RIOTBOX-1221 adds source-derived sparse bass movement to the dense-break performance generator, surfaces the proof in the pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds a negative fixture that mutates the sparse report back to fixed movement.
Consequences: this is a stronger diagnostic step toward source-aware production behavior, but it remains scripted render evidence with `quality_proof: false` and `human_verdict: unverified`. Future work should replace more hook/chop and pad/noise behavior with source-derived decisions and collect listening verdicts before claiming product quality.
Status: accepted

---

### RBX-073

Date: 2026-06-06
Topic: Dense and tonal hook/chop selection must be source-derived before it can count as stronger P022 evidence
Phase: P022 / Professional Sound Output
Question: may dense-break and tonal-hook diagnostics keep using only a fixed first-bar W-30 grain while claiming source-aware hook/chop behavior?
Decision: no. Dense-break and tonal-hook diagnostics now use a bounded `hook_chop_policy` that scans multiple source/W-30 candidates, selects separate hook and chop offsets, and reports whether selection is source-derived, how far it moved away from the old first-bar static choice, and how much offset contrast exists between hook and chop.
Why: the previous W-30 hook/riff layer could make different sources feel more organic, but it still began from a fixed first-bar grain selection. Riotbox needs hook/chop choices that follow the source material before those diagnostics can be treated as stronger P022 evidence.
Evidence: RIOTBOX-1222 adds source-derived hook/chop selection to the dense-break performance generator, surfaces the proof through the pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds negative fixtures that reject non-source-derived or static tonal hook/chop reports.
Consequences: this remains scripted render evidence with `quality_proof: false` and `human_verdict: unverified`. Future work should continue replacing scripted arrangement behavior with source-derived decisions and collect listening verdicts before claiming release-grade musical quality.
Status: accepted

---

### RBX-074

Date: 2026-06-06
Topic: Dense and tonal destructive gesture cues must be source-derived before they count as stronger P022 evidence
Phase: P022 / Professional Sound Output
Question: may dense-break and tonal-hook diagnostics keep using fixed destructive stutter/restore cue choices while claiming stronger source-aware destructive behavior?
Decision: no. Dense-break and tonal-hook diagnostics now use a bounded `destructive_gesture_policy` that scans source/W-30 candidates, selects separate stutter and restore cue offsets, and reports whether selection is source-derived, how far it moved away from the old fixed choices, and how much offset contrast exists between stutter and restore.
Why: the dropout/stutter/restore gesture is one of the places where Riotbox should feel physically playable. If it keeps using fixed cue choices, the render can sound like the same trick around different source material instead of abusing the actual source.
Evidence: RIOTBOX-1225 adds source-derived destructive gesture selection to the dense-break performance generator, surfaces the proof through the destructive-variation validator, pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds negative mutations that reject non-source-derived or static dense/tonal destructive reports.
Consequences: this remains scripted render evidence with `quality_proof: false` and `human_verdict: unverified`. Future work should keep replacing remaining scripted production choices with source-derived decisions and collect listening verdicts before claiming release-grade musical quality.
Status: accepted

---

### RBX-075

Date: 2026-06-06
Topic: Eligible P022 arrangement role order must be source-derived before it counts as stronger arrangement evidence
Phase: P022 / Professional Sound Output
Question: may dense-break, tonal-hook, and sparse-bass-pressure diagnostics keep using fixed source-family hook/chop/pressure role recipes while other production choices become source-derived?
Decision: no. Eligible diagnostics now derive the first six arrangement roles from bounded source/W-30 bar candidates, preserve the bounded dropout/restore tail, and report candidate count plus distance from the old source-family scripted role order.
Why: source-derived hook/chop, destructive gesture, and bass movement proof can still sound rehearsed if the eight-bar performance always follows the same role recipe. Riotbox needs the arrangement to react to where the source has hook, chop, and pressure evidence before it can claim stronger source-aware production behavior.
Evidence: RIOTBOX-1226 adds source-derived arrangement-role selection to the dense-break performance generator, surfaces the proof through the pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds negative mutation coverage that rejects non-source-derived dense arrangement reports.
Consequences: this remains diagnostic scripted evidence with `quality_proof: false` and `human_verdict: unverified` because the role vocabulary, destructive/restore tail, and mix recipe remain bounded. Future work should keep replacing remaining scripted mix and production choices with source-derived decisions and collect listening verdicts before claiming release-grade musical quality.
Status: accepted

---

### RBX-076

Date: 2026-06-06
Topic: Eligible P022 mix-bus treatment must be source-derived before it counts as stronger render evidence
Phase: P022 / Professional Sound Output
Question: may dense-break, tonal-hook, and sparse-bass-pressure diagnostics keep using one fixed mix-bus recipe while other production choices become source-derived?
Decision: no. Eligible diagnostics now use a bounded `mix_treatment_policy` that scans source/W-30 bar candidates, derives hook/chop/pressure/restore bus drive, slam, gain, pressure peak, and restore bass treatment from source energy, low-band, high-band, transient, and W-30 response, and reports candidate count, distance from the old fixed mix recipe, source-energy span, and output contrast.
Why: source-derived hook/chop, destructive gestures, bass movement, and arrangement order can still sound rehearsed if the final bus treatment always pushes every source through one fixed recipe. Riotbox needs mix pressure, snap, and restore weight to react to the source before the rendered diagnostic can be treated as stronger source-aware production evidence.
Evidence: RIOTBOX-1227 adds source-derived mix-treatment selection to the dense-break performance generator, applies it to the rendered hook/chop/pressure/restore/final buses, surfaces proof through the pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds positive and negative gates that reject non-source-derived or fixed-collapsed mix treatment.
Consequences: this remains diagnostic scripted evidence with `quality_proof: false` and `human_verdict: unverified` because the role vocabulary and destructive/restore tail remain bounded. Future work should keep replacing remaining scripted production choices with source-derived decisions and collect listening verdicts before claiming release-grade musical quality.
Status: accepted

---

### RBX-077

Date: 2026-06-06
Topic: Pad/noise edge sources need source-derived texture proof, not only weak routing
Phase: P022 / Professional Sound Output
Question: is it enough for pad/noise material to avoid dense-break promotion and route to a fix category?
Decision: no. The pad/noise edge diagnostic now uses a bounded `pad_noise_texture_policy` that scans source/W-30 candidates, chooses separate gate and stab offsets, derives gate duty plus texture/stab gain, applies that texture layer to the rendered performance, and reports candidate count, fixed-choice distance, gate/stab offset distance, and transient shape.
Why: avoiding dense-break misclassification is necessary but musically incomplete. Noisy pad material should become a useful gated texture, stab, or cut candidate when appropriate, while still staying honest that it is not proven breakbeat output and not product-quality proof.
Evidence: RIOTBOX-1223 adds source-derived pad/noise texture selection to the dense-break performance generator, surfaces the proof through edge-source diagnostics and the professional-output suite, and adds positive and negative gates that reject non-source-derived or collapsed pad/noise texture reports.
Consequences: pad/noise remains weak-routed diagnostic evidence with `quality_proof: false` and `human_verdict: unverified`. Future work should improve source selection, UI cues, and listening-review labels before treating pad/noise output as demo-ready musical material.
Status: accepted

---

### RBX-078

Date: 2026-06-06
Topic: Eligible dropout/restore tails must prove source-derived shape instead of one fixed recipe
Phase: P022 / Professional Sound Output
Question: can dense-break, tonal-hook, and sparse-bass-pressure diagnostics keep one fixed dropout/stutter/restore tail while the rest of the render becomes source-derived?
Decision: no. Eligible diagnostics now use a bounded `tail_shape_policy` that scans source/W-30 bar candidates and derives dropout silence fraction/gain, stutter density/gain, and restore source/snap/drum/bass/drive treatment from source energy, low-band, high-band, transient, and W-30 response.
Why: a fixed final cut makes otherwise source-aware renders feel rehearsed. The last silence cut, stutter, and restore need to react to the actual material so the performer hears a source-specific choke and return rather than the same trick on every source.
Evidence: RIOTBOX-1228 adds source-derived tail-shape selection to the dense-break performance generator, applies it to the rendered dropout/stutter and restore path, surfaces proof through the pro-pressure source matrix, professional-source WAV pack, and professional-output suite, and adds positive and negative gates that reject non-source-derived or fixed-collapsed tail shape.
Consequences: this remains diagnostic scripted evidence with `quality_proof: false` and `human_verdict: unverified` because the role vocabulary is still bounded and no structured human listening verdict has approved the output as product-quality proof.
Status: accepted

---

### RBX-079

Date: 2026-06-06
Topic: Professional-output diagnostics must expose what hits hardest
Phase: P022 / Professional Sound Output
Question: is pass/RMS/non-collapse evidence enough for P022 professional outputs when the musician still needs to know what carries the room?
Decision: no. P022 professional-output diagnostics now expose a bounded `strongest_audible_element` proof with allowed labels for kick, snare/break, bass pressure, stab/texture, silence, and restore impact, plus score, margin, candidate count, and ambiguity gates.
Why: a render can be technically non-silent, source-backed, and non-collapsed while still being unclear about whether the break, bass, stab, cut, or restore is the musical center. Riotbox needs machine-readable impact identity so QA and PR review can discuss the same audible target instead of only aggregate loudness.
Evidence: RIOTBOX-1229 adds strongest-element proof to dense-break performance reports, surfaces it through the pro-pressure source matrix, professional-source WAV pack, edge-source diagnostics, and professional-output suite, and adds positive/negative smoke gates that reject missing or ambiguous strongest-element evidence.
Consequences: this remains diagnostic scripted evidence with `quality_proof: false` and `human_verdict: unverified`. Strongest-element labels can guide review and follow-up work, but they do not prove musical quality until structured human listening or a calibrated agent gate approves the artifact.
Status: accepted

---

### RBX-080

Date: 2026-06-06
Topic: Rebuild-only P022 output must prove transformed source character, not only non-silence
Phase: P022 / Professional Sound Output
Question: are rebuild-only RMS, low source correlation, and source-on/source-off contrast enough to prove that source character survives without raw-source masking?
Decision: no. P022 rebuild-only diagnostics now expose a bounded source-character survival proof: source/rebuild-only spectral similarity, transient retention, RMS retention, and a combined survival score.
Why: a source-layer-off render can be non-silent and not waveform-correlated with the raw source while still sounding like generic fallback material. Riotbox needs proof that the rebuilt output keeps enough transformed source signature to be worth listening to, while separately rejecting raw-source copy/masking.
Evidence: RIOTBOX-1230 adds rebuild-only source-character proof to dense performance reports, surfaces it through the pro-pressure source matrix, professional-source WAV pack, edge-source diagnostics, and professional-output suite, and adds positive and negative smoke gates for missing/lost source-character evidence.
Consequences: this is stronger diagnostic evidence only. It keeps `quality_proof: false` and `human_verdict: unverified`; human or calibrated listening approval is still required before treating the artifact as product-quality sound.
Status: accepted

---

### RBX-081

Date: 2026-06-06
Topic: Source-character survival gates need real weak-WAV evidence, not only report mutations
Phase: P022 / Professional Sound Output
Question: is a JSON/report mutation enough to prove that rebuild-only source-character survival gates catch generic fallback-like audio?
Decision: no. P022 now keeps a dense-break weak source-character fixture that renders a real `06_rebuild_only_performance.wav`, intentionally strips useful source character from the rebuild-only path, and validates that the report fails with `rebuild_only_source_character_not_surviving`.
Why: a report mutation proves the validator sees a bad field, but it does not prove the audio artifact path can produce and reject the actual weak case musicians care about: active output that no longer sounds like the loaded source being transformed.
Evidence: RIOTBOX-1231 adds `--weak-source-character-fixture`, `--validate-weak-source-character-report`, and `just dense-break-weak-source-character-fixture-smoke`. The validator reloads the rendered source/rebuild-only WAVs and recomputes source-character survival before accepting the expected failure.
Consequences: this is negative diagnostic evidence only. It keeps `quality_proof: false` and `human_verdict: unverified`; it proves the gate rejects one weak rendered artifact, not that passing outputs are musically release-grade.
Status: accepted

---

### RBX-082

Date: 2026-06-06
Topic: Weak professional-output failures must route to concrete sound-fix categories
Phase: P022 / Professional Sound Output
Question: can weak/fail professional-output reports pass routing if they only expose raw failure codes or an unknown generic bucket?
Decision: no. Weak-output routing now requires a known routing signal, a concrete proposed fix category, and a short musician-facing `musician_fix_reason` for every routed weak/fail case. Unknown weak/fail failure codes fail the routing report until they are mapped.
Why: engineers and musicians need to know what to improve next: source selection, chop policy, drum pressure, bass movement, mix-bus treatment, destructive gesture, fixture threshold, or UI cue. A pass/fail metric without an actionable production category slows down the sound-quality loop.
Evidence: RIOTBOX-1232 maps rebuild-only source-character loss to `source_selection`, adds `matched_known_routing_signal` and `musician_fix_reason`, routes a live generated weak source-character WAV report, and adds an invalid unknown-code fixture that must fail.
Consequences: routing is still diagnostic and does not claim musical approval. It preserves `quality_proof: false` and `human_verdict: unverified`; it only turns known weak evidence into the next concrete production fix.
Status: accepted

---

### RBX-083

Date: 2026-06-06
Topic: Professional listening packs need demo-readiness reasons before human review
Phase: P022 / Professional Sound Output
Question: should generated professional listening packs only expose metrics and review artifacts, or also explain why a candidate is worth hearing and why it is not demo-ready yet?
Decision: include both reasons. Each professional-output review case now carries `demo_readiness`, `demo_worthy_reason`, and `not_demo_worthy_reason` in the pack report and `review.json`, and appends the same information to the prompt.
Why: raw metrics do not tell a musician what to listen for. A candidate can be useful for review because the strongest element, source-character survival, pressure, restore, bass, or chop target is promising, while still not being demo-ready because no structured human verdict exists.
Evidence: RIOTBOX-1233 adds reason generation to the professional output listening pack, suite identity checks for reason presence, and Just gates that reject missing review/prompt reason fields.
Consequences: these reasons guide review and demo-bank promotion work only. They preserve `human_verdict: unverified` and `quality_proof: false`; they do not turn scripted diagnostic renders into demo-ready product audio.
Status: accepted

---

### RBX-084

Date: 2026-06-06
Topic: Oversized professional-output suite contracts belong in validators, not inline Justfile blocks
Phase: P022 / Professional Sound Output
Question: should P022 keep expanding large inline `jq` expressions inside `Justfile` as the professional-output suite adds more proof surfaces?
Decision: no. Cross-report professional-output suite checks now live in `scripts/validate_professional_output_suite_contract.py`, while `just professional-output-suite-smoke` stays responsible for running the generator, invoking the validator, and keeping compact negative mutations.
Why: the suite contract is too important to hide in unreadable shell. Moving it to a named validator makes failure codes, thresholds, artifact checks, and evidence-boundary rules reviewable without weakening the audio-QA gate.
Evidence: RIOTBOX-1224 replaces the long inline professional-output suite `jq` block with the validator and keeps negative mutations for non-source-derived hook/chop evidence and scripted demo-readiness promotion.
Consequences: public `just` recipe names stay stable. The validator is the durable place for future professional-output suite thresholds; `Justfile` should remain a command catalog, not the primary contract implementation.
Status: accepted

---

### RBX-085

Date: 2026-06-12
Topic: P023 needs an aggregate sound-quality readiness report, not another taste oracle
Phase: P023 / Sound Excellence / Production Quality
Question: how should Riotbox show current 10/10 sound-product progress without pretending scripted diagnostics or unverified agent reports prove musical quality?
Decision: add a P023 sound-quality readiness report that aggregates the existing rubric, source corpus, release-grade demo bank, weak-output routing, and professional-output suite context into release blockers and next production fix categories.
Why: contributors and musicians need one place to see what is blocking release-grade sound, which artifacts are worth hearing, and whether the next fix should target source selection, chop policy, drums, bass movement, mix bus, destructive gestures, fixture thresholds, or UI cues. A new score would hide those decisions and risk becoming a fake taste oracle.
Evidence: RIOTBOX-1234 adds `scripts/generate_sound_quality_readiness_report.py`, `just sound-quality-readiness-report-smoke`, JSON/Markdown output, negative validation for blocked quality claims and premature release-ready reports, and docs under `docs/benchmarks/sound_quality_readiness_report_v1_2026-06-12.md`.
Consequences: the report may block quality claims and steer implementation, but it does not approve audio. Scripted or unverified diagnostics remain non-quality proof until source-family coverage, structured human verdicts, and release-grade demo evidence exist.
Status: accepted

---

### RBX-086

Date: 2026-06-12
Topic: Weak-output routing should emit P023 production-fix candidates
Phase: P023 / Sound Excellence / Production Quality
Question: should weak-output routing stop at per-case categories, or should it produce bounded implementation candidates for the next P023 slices?
Decision: weak-output routing now emits `production_fix_candidates` grouped by fix category. Each candidate carries case ids, primary case ids, source families, artifact refs, routing reasons, software next step, and musician payoff.
Why: per-case categories are useful but still leave engineers to manually translate reports into implementation work. P023 needs a tighter loop from weak sound to the next production fix without pretending the router approves audio.
Evidence: RIOTBOX-1235 adds candidate construction to `scripts/route_weak_output_fixes.py`, extends the weak-output and sound-quality readiness smokes, and surfaces the candidates through the P023 readiness report.
Consequences: candidates remain diagnostic work-selection inputs only. They keep `quality_proof: false` and `automated_musical_approval: false`; unknown weak/fail reasons still fail routing instead of falling into a generic bucket.
Status: accepted

---

### RBX-087

Date: 2026-06-12
Topic: P023 release-demo coverage needs a source-family gate
Phase: P023 / Sound Excellence / Production Quality
Question: is dense-break plus one non-dense demo-bank coverage enough for P023 release-ready claims?
Decision: no. P023 now has a source-family release-demo coverage gate that compares the real-source corpus with the release-grade demo bank and reports candidate, human-verdict, and demo-ready human-pass coverage per required source family.
Why: a strong dense-break or tonal example can hide missing sparse, pad/noise, weak-source, or bad-timing evidence. Release-grade sound needs breadth across source families, and missing coverage should be visible without requiring ignored local WAV files in CI.
Evidence: RIOTBOX-1236 adds `scripts/validate_source_family_release_demo_coverage.py`, `just source-family-release-demo-coverage-fixtures`, JSON/Markdown output, negative blocked-quality validation, and tighter source-family blockers in the P023 sound-quality readiness report.
Consequences: candidate-only and weak/fail human-verdict families remain useful production evidence, but they block release-ready claims until a demo-ready human-pass entry exists. The gate is coverage evidence only, not a taste oracle or quality proof.
Status: accepted

---

### RBX-088

Date: 2026-06-14
Topic: MC-202 source phrase planning must be an explicit Session/Core plan, not a hidden renderer trick
Phase: P013+ / P023 Sound Excellence
Question: how should Riotbox promote primitive MC-202 pressure / answer support into source-derived bass and answer phrases?
Decision: accept `docs/plans/mc202_source_phrase_planning_plan.md` as the RIOTBOX-1035 implementation plan. The plan adds a typed `Mc202SourcePhrasePlan`, derives bounded phrase candidates from trusted Source Graph timing and source evidence, commits the selected plan through the existing MC-202 queue / Session / replay seam, and renders it on the existing MC-202 audio path while keeping primitive shapes as fallback and A/B controls.
Why: the current MC-202 lane has pressure, contour, hook-response, and source-grid proof, but it can still sound like a static question/answer pattern. Source-derived MC-202 behavior must be replayable product state with output-path proof, not a callback-local heuristic or scripted diagnostic render.
Evidence: RIOTBOX-1035 now carries the implementation plan, and the plan is anchored from the docs README, execution roadmap, phase definition of done, and this decision log.
Consequences: future MC-202 bass / answer quality claims must distinguish source-derived plans from primitive fallback. Scripted or diagnostic artifacts remain `quality_proof: false` and `human_verdict: unverified` until structured listening review or calibrated quality gates approve them.
Status: accepted

---

### RBX-089

Date: 2026-06-14
Topic: Source-derived intelligence requires source evidence, not template mutation
Phase: Global / P023 Sound Excellence
Question: may Riotbox treat hardcoded phrases, scripted demos, fixed templates, fingerprint-only variation, or source-aware template mutation as product-quality intelligence?
Decision: no. Across the whole product, source-derived or intelligent behavior requires source evidence to change a musical decision and audible output. The required surfaces are source evidence, musical decision, product-spine representation, audible consequence, and quality proof with same-source reproducibility plus cross-source diversity.
Why: MC-202 source phrase planning exposed a general risk: a slice can correctly add replayable state and source-dependent variation while still falling short of the product target. That infrastructure is valuable, but it must be labeled as scaffold/control until the system actually listens to source features and composes or chooses from them.
Evidence: RIOTBOX-1262 upgrades the MC-202 source phrase plan, adds the global Product Intelligence Rule to `AGENTS.md` and the `riotbox-development` / `riotbox-rave-punk-production` skills, and tightens the roadmap and audio-QA specs.
Consequences: future PRs, specs, manifests, demos, and review notes must label hardcoded/scripted/template-mutated artifacts as scaffold, control, diagnostic, or `quality_proof: false`. Product-quality claims require feature-derived decisions and output-path proof; otherwise the work remains partial even if the architecture path is correct.
Status: accepted

---

### RBX-090

Date: 2026-06-15
Topic: Sound features stay open until musician-facing target quality is met
Phase: P023 Sound Excellence / Production Quality
Question: may Riotbox close a sound-producing feature as complete when only a prerequisite, architecture foundation, or diagnostic scaffold has landed?
Decision: no. For sound-producing product features, a foundation PR may merge as an implementation step, but the feature track remains open until the audible musician-facing target quality is met. For MC-202 source-composed bass / answer phrases, RIOTBOX-1264 is the open producer-grade parent track and RIOTBOX-1265 through RIOTBOX-1270 are implementation steps toward measured source evidence, candidate generation, scoring, rendering, automated QA, and structured listening review.
Why: closing small technical slices as "done, but not final quality" created a misleading product picture. Riotbox needs high implementation quality and high sound quality; the workflow must not optimize for ticket throughput at the expense of the instrument.
Evidence: RIOTBOX-1264 now owns the Level 4 MC-202 producer-grade target in `docs/plans/mc202_source_phrase_planning_plan.md`, with child tickets RIOTBOX-1265 through RIOTBOX-1270 covering the complete path from measured source evidence to listening-reviewed demo readiness.
Consequences: future sound-feature PRs must distinguish merged implementation steps from complete product features. A ticket may be closed only for its scoped step, but parent sound-quality tracks stay open until source evidence, musical decision, product-spine representation, audible consequence, automated diversity / collapse gates, and structured listening review support the product claim.
Status: accepted

---

### RBX-091

Date: 2026-06-29
Topic: Improvement README becomes split quality tracks, not a parallel roadmap
Phase: Global / P023 Sound Excellence / Engineering Quality
Question: how should Riotbox incorporate the temporary improvement README that proposed module refactors, audio-runtime hardening, musical algorithm work, sidecar hardening, QA layering, and UX work?
Decision: accept the content as split work tracks in `docs/plans/riotbox_improvement_tracks_plan.md` and `docs/engineering/module_policy.md`, while keeping audible instrument quality as the primary product goal. Code quality is not optional: semantic module ownership, include-shell migration, realtime audio safety, and QA contracts must proceed in bounded slices that protect musical work rather than displacing it.
Why: the README identified real weaknesses, especially textual `include!` splits and runtime/audio-QA hardening needs, but its broad PR sequence would over-prioritize refactoring and group too much musical work together. Riotbox needs both stronger sound and stronger code quality, with each slice scoped tightly enough to review and prove.
Evidence: RIOTBOX-1320 incorporates the README into canonical docs, splits the broad musical quick-win proposal into separate TR-909, MC-202, W-30, Source Timing, Scene Brain, sidecar, QA, and UX tracks, and removes the temporary planning file after incorporation.
Consequences: future agents should use the improvement tracks as backlog and guardrails, not as a reason to pause sound-product progress. Musical fallback output remains forbidden on product paths; unavailable or degraded source-backed material must be surfaced honestly instead of filled with synthetic replacement music.
Status: accepted

---

### RBX-133

Date: 2026-07-11
Topic: P023 becomes the single active product priority until one live human-passed Usable Musical Alpha exists
Phase: P023 Sound Excellence / Production Quality
Question: how should Riotbox convert its large diagnostic, QA, and offline-render investment into a genuinely usable instrument without weakening the product spine or audio-QA honesty?
Options considered: keep P016 active while treating quality as a parallel ladder; create a new P024 project for the musical alpha; keep producing P023 readiness infrastructure; or make the existing P023 project own one bounded live human-passed exit.
Decision: make `RIOTBOX-1396` the bounded active exit and defer P016, P021, and P022 unless a ticket names the exact Golden Path blocker it removes. Classify work as `audible_vertical_slice`, `contract_enabler`, or `maintenance/regression`; require exact live runtime / mixer proof for product-facing instrument progress; stop after at most two review-ready but still unverified candidate generations and perform or explicitly hand off structured human listening. Require positive demo families to earn human passes, while weak and bad-timing sources may satisfy their family contract through reviewed degraded / unavailable / reject behavior.
Why: repository evidence showed high delivery velocity but low conversion into live musician value: strong sound policy remained concentrated in offline diagnostics, all current listening-review artifacts remained unverified, and conflicting active-phase language let automatic ticket selection prefer the next machine-verifiable report gap. Reusing P023 avoids another project split while preserving replay, realtime, provenance, and negative-fixture strengths.
Evidence: Linear project P023 is urgent and active with `RIOTBOX-1396` as parent outcome; `RIOTBOX-1397` through `RIOTBOX-1405` encode the ordered reset, real listening, live timing / W-30 / performance path, capture / replay, human pass, and later controlled expansion. The roadmap, phase DoD, architecture map, workflow, demo-bank spec, AGENTS guardrails, and project skills carry the same bounded direction.
Consequences: offline packs, fixtures, reports, and validators remain valuable diagnostic evidence but cannot close an instrument-progress claim. A contract enabler names exactly one audible follow-up. A waiting human review outranks another report layer. Broader source-family and export work resumes only after the dense-break live alpha earns a structured human pass, except for a directly named blocker.
Follow-up: complete `RIOTBOX-1397`, then record real candidate verdicts in `RIOTBOX-1398` before selecting another sound-policy fix.
Status: accepted

---

### RBX-134

Date: 2026-07-11
Topic: live source ingest and all live lane timing consumers share the Rust Source Timing trust boundary
Phase: P023 / Live Musical Alpha
Question: how should Riotbox move the stronger offline Rust timing evidence into the live instrument without creating a second timing truth or trusting weak estimates silently?
Decision: live `riotbox-app --source` ingest replaces only the sidecar timing payload with the deterministic Rust source-timing model before graph hashing and persistence. TR-909, MC-202, W-30 preview, Source Monitor timing, and transport meter/phrase projection read one readiness-gated helper backed by Source Graph plus Session confirmation. `--source-bpm` is a confirmation of a matching Rust primary hypothesis within 1 BPM, not an arbitrary override; it commits the existing `source_timing.confirm_grid` action and fails before persistence when no matching grid exists.
Why: the offline detector, Session confirmation action, and replay identity already existed, but the live ingest still persisted weaker sidecar timing and audio projections read raw compatibility BPM fields. That allowed untrusted timing to reach the instrument and split the effective timing authority.
Evidence: RIOTBOX-1330 adds live Rust-probe enrichment and provenance, same-source stability and cross-source identity tests, unavailable/manual-confirm gating, shared lane-consumer assertions, explicit confirmation mismatch rejection, and ingest/save/restore confirmation coverage.
Consequences: timing analysis remains outside the realtime callback; Source Graph remains analysis truth and Session remains musician trust truth. This enables the real-source W-30 sampler slice but does not claim production-grade arbitrary-audio detection or audible quality by itself.
Status: accepted

---

### RBX-135

Date: 2026-07-12
Topic: focused W-30 pads use duration-aware capture playback with a source-derived chop plan
Phase: P023 / Live Musical Alpha
Question: how should a committed real-source capture become a playable hook without streaming or analysis work in the realtime callback, inventing fallback music, or losing replay identity?
Decision: project the complete capture outside the callback into a bounded 16384-sample mono representation carrying original duration and sample-rate identity. Derive eight deterministic chop starts from quantized short-time source-energy rises, then let transport and pad triggers select that prepared plan. Normal recall forms an eighth-note source chop; the committed damage action selects a denser policy with reordered slices, retrigger omissions, and moderate pitch-down. Preserve click-safe trigger attack and loop crossfade. Artifact replay preserves committed macro state and must not invent a default grit mutation.
Why: straight full-capture playback was technically correct but earned repeated human `weak` verdicts because it behaved like a quiet loop rather than an instrument. Single-sample onset selection was also too sensitive to persisted PCM rounding. The bounded energy-rise plan is source-dependent, callback-safe, stable through PCM16 artifact hydration, and immediately playable.
Evidence: RIOTBOX-1333 proves same-source stability, cross-source diversity, full-duration cursor behavior, rate / reverse capability, click-safe loop and trigger edges, coherent control-to-callback state, exact callback-block mixer parity, committed action / artifact replay, and no silent or clipping collapse. Exact-path candidate `local-riotbox-1333-live-w30-v8` received separate human `pass` verdicts for the normal chop hook and destructive variation on 2026-07-12.
Consequences: this is one mono focused-pad sampler seam, not a full streaming pad-bank engine or general source-intelligence claim. The fixed riff vocabulary remains a bounded sampler policy whose slice material is source-derived; multi-pad polyphony, stereo playback, broader codecs, and richer musician-editable chop sequencing remain later slices. P023 remains open until the all-lane Golden Path passes its complete exit contract.
Status: accepted

---

### RBX-136

Date: 2026-07-14
Topic: dense live policy uses selected candidate ownership and may leave bass unassigned
Phase: P023 / Live Musical Alpha
Question: when an MC-202 action requested pressure but source evidence selects an answer or pickup candidate, should Riotbox keep treating MC-202 as the bass lane and continue polishing its timbre until it sounds larger?
Decision: no. Derive one typed shared `LivePerformancePolicy` from matching confirmed timing, the dense Source Graph, and the committed MC-202 source phrase plan. The selected candidate family determines the audible MC-202 intent. Only `sub_pressure_shove` assigns `bass_owner=mc202`; answer, callback, pickup, stay-out, and fallback candidates assign `bass_owner=unassigned`. The requested compatibility role remains action/replay context but cannot force bass behavior. The derived policy may apply bounded W-30, TR-909, and MC-202 level/touch/slam floors, but it must never generate replacement bass.
Why: repeated Beat08 iterations made the sound louder, harder, and technically cleaner, yet human listening still rejected it as boring and not loopable. The source evidence was marginal for bass pressure, so further oscillator polishing would have hidden a wrong musical ownership decision. An instrument must first decide whether bass is actually needed and which lane has evidence to own it before evaluating bass pressure.
Evidence: RIOTBOX-1400 moves the dense policy into core/shared live projection, fixes callback-block transport continuity, strengthens physical TR-909 projection, and renders exact live-path lane stems. Structured listening found the Beat03 instigator direction materially improved, while Beat08 bass v1-v8 remained weak/rejected. Tightened `sub_pressure_shove` eligibility makes Beat08 select `fill_pickup_instigator` with `bass_owner=unassigned`; exact-path and MC-202 tests prove the non-silent selected role without claiming a bass pass.
Consequences: future reviews must state intended lane role and bass owner before asking a listener to judge pressure. `unassigned` is an honest product decision, not an automatic failure and not permission for fallback sound. A human-passed MC-202 bass candidate requires a separate trusted, legally usable source with confirmed timing and strong low-band movement evidence; Beat08 must not be recycled as that proof.
Status: accepted

---

### RBX-137

Date: 2026-07-15
Topic: Golden Path review requires macro-development and absolute pressure evidence
Phase: P023 / Live Musical Alpha
Question: what did the rejected RIOTBOX-1400 all-lane candidate reveal about judging musical progress, bass pressure, and reference comparison?
Decision: treat RIOTBOX-1400 as a contract enabler and make RIOTBOX-1401 own the next audible slice. Golden Path candidates must develop across an eight-bar review window through a source-derived hook, pressure lift, destructive role swap or drop, and materially changed return. A near-identical short loop fails unless an explicit held-loop mode and an already human-passed hook justify it. Bass review names the typed owner first and uses absolute low-band energy or lift in addition to relative spectral share. Raw-level and loudness-matched A/B renders serve different verdicts and must not conceal weak product gain. Commercial reference recordings remain local, ignored comparison material only.
Why: Beat03 v2 was audibly stronger and punchier than v1, but the listener still preferred the source and rejected the candidate as unusable. Measurement showed that a high relative low-band share hid essentially no absolute low lift, high-band energy collapsed relative to the source, and repeated two-bar units were nearly identical. Short gaps created activity but no larger dramatic arc or recognizable Riotbox character.
Evidence: RIOTBOX-1400 exact live-path renders, structured human verdicts, same-tool source/candidate spectral and loudness measurements, and repeated-block similarity analysis.
Consequences: louder or cleaner processing alone is not musical-alpha progress. The next slice must change source-backed arrangement ownership and return behavior rather than continue polishing the same short loop. Commercial references cannot enter product sources, fixtures, generated artifacts, commits, or redistribution paths.
Follow-up: RIOTBOX-1401 implements and reviews the bounded source-derived eight-bar Golden Path arc.
Status: accepted

---

### RBX-138

Date: 2026-07-15
Topic: first-playable monitor and gesture proof follows the exact live mixer path
Phase: P023 / Live Musical Alpha
Question: how should Riotbox make Source, Blend, Riotbox, and the first performance gestures safely reachable without mistaking observer state or a separate W-30 renderer for audible product progress?
Decision: commit `source_monitor.set_mode` synchronously at its Immediate boundary even when no audio device is running, while preserving the normal Session/replay and observer commit record. Treat valid prepared PCM as monitor-ready across source/output sample-rate differences using bounded allocation-free interpolation in the callback. If source material is genuinely absent, Source stays explicitly silent and Blend keeps only its real Riotbox component while reporting degraded source availability. Prove the first-playable path with one stateful callback-block RuntimeMix sequence plus same-position counterfactual branches for `w`, `f`, `s`, `y`, and `Y`; every branch continues through Source Monitor and the master limiter.
Why: the previous UI could call `M` immediate while leaving it pending forever after audio startup failure; 44.1 kHz Golden Path sources became unavailable on a 48 kHz PipeWire output and could silence Blend; and the first-playable probe paired multi-gesture observer evidence with a separate W-30-only renderer that could not prove Blend, fill, slam, or scene movement in the product mix. Fresh static renders also reset W-30 callback state and could misrepresent retrigger behavior.
Evidence: RIOTBOX-1335 adds modal and readiness-gated TUI guidance, typed monitor cycling, immediate commit coverage without an audio runtime, 44.1-to-48 kHz Source/Blend regression coverage, persistent RuntimeMix sequence tests, monotone observer commits for `M/w/f/s/y/Y`, and a generated exact-mixer manifest with non-silent Source/Blend/Riotbox routes plus measurable per-gesture deltas. The scripted pack uses the canonical `riotbox.audio_qa_evidence_boundary.v1` diagnostic contract and remains `quality_proof: false` and `human_verdict: unverified` until a structured human listen.
Consequences: the supported first-run path can now expose audible product behavior without requiring Log navigation or hiding degraded audio. The observer and exact-mixer pack are correlated only at `action_contract_only` scope because they use different deterministic source fixtures, Sessions, and transport timelines; they do not establish sample-exact trace-to-audio causality. Metric success establishes reachability and non-collapse, not a musical-alpha pass; RIOTBOX-1401/1402 still own stronger musical development and human acceptance. Separate diagnostic renderers remain useful for narrow component tests but cannot substitute for exact mixer evidence when a ticket claims live instrument progress.
Status: accepted

---

### RBX-139

Date: 2026-07-16
Topic: live TR-909 fills earn contrast through typed arrangement focus, not global gain
Phase: P023 / Live Musical Alpha
Question: how should the first-playable `f` gesture remain replay-safe and callback-safe while becoming clearly audible under a dense Source/W-30/MC-202 Blend bed?
Decision: derive one callback-local `FillFocus` articulation only from an audible, running typed `Tr909RenderMode::Fill` on `DrumBusSupport` and the current transport position. During the final beat, apply one smooth sample-position envelope to Source in Blend and to the non-TR-909 Riotbox bed, leave TR-909 itself unchanged, and return fully at the next bar. Source-only remains sample-identical, inactive or silent/wrong-route states do not duck, and no Session, replay, action, or app-local state is added. Revert the rejected Candidate-5 unslammed global Fill gain floor; preserve its predecessor's source/policy pressure floor for BreakReinforce and preserve the accepted `1 -> 2 -> 4 -> 8` Fill rhythm.
Why: Candidate 4's ratchet was creative in isolation but masked in the continuous Blend sequence. Candidate 5 raised the isolated TR-909 Fill by `4.83 dB`, yet the exact full-mix Fill moved only `0.23 dB`, retained `0.9932` waveform correlation, left the low band effectively unchanged, and still kept TR-909 about `11 dB` below the static bed. Human review correctly reported that the difference remained too small. The missing behavior was arrangement/performance impact, not another global level increase.
Evidence: RIOTBOX-1335 FillFocus tests prove typed activation, a smooth last-beat/bar-reset envelope, sample-exact Source-only invariance, decisive non-TR-909/Blend-bed removal, exact 127-frame envelope partitioning, canonical 128-frame RuntimeMix parity, unchanged TR-909 gain, and clean headroom. The complete `riotbox-audio` library suite passes `145/145`; the fresh exact Blend candidate still requires mandatory technical pre-playback analysis and a structured human verdict before any musical pass claim.
Consequences: future explicit live gestures must first create a time-local musical role change under the real mixer before seeking more lane gain. Global pressure floors cannot stand in for fill, slam, trigger, launch, or restore articulation. FillFocus is a bounded current 4/4 Golden Path policy, not a second arrangement model or permission to add hidden callback state.
Status: accepted

---

### RBX-140

Date: 2026-07-16
Topic: exposed live fills use independent drum voices and real dramatic rests
Phase: P023 / Live Musical Alpha
Question: after FillFocus made the `f` gesture clearly audible, how should Riotbox replace the exposed but thin composite ratchet without hiding it under more gain or adding a second musical truth?
Decision: keep the typed `FillFocus` arrangement articulation and the existing unslammed Fill gain, but render the supported dense-break `MainlineDrive + PhraseDrive` Fill through fixed callback-local kick, snare-body/noise, and metallic-hat voices with independent phases and envelopes. Replace RBX-139's preserved `1 -> 2 -> 4 -> 8` composite contour with `1 / 2 / 4 / 6` sounding events whose final eight-slot grid is `kick / rest / snare / hat / kick+snare / snare / hat / rest`. Model both rests as false trigger decisions. Pre-ramp the bed before the final-beat kick, release the Fill voice sum at the bar edge, clear hidden tails before a new bar, and realign callback state when returning to the legacy non-Fill renderer. This state remains derived/private realtime state, not Session, replay, action, or app truth.
Why: Candidate 6 earned a clear arrangement cut but the human verdict remained `technically_ok_but_musically_weak`: too thin, too uninteresting, and not useful live. Analysis found one shared oscillator/envelope repeatedly replacing kick, snare, and hat, a final beat dominated by nearly identical midrange ticks, internal composite clamping, too little 120-500 Hz body, and no silence before the downbeat. More global Fill gain would repeat the rejected Candidate-5 failure.
Evidence: the voice-separated control raises isolated final-beat 120-500 Hz RMS from `0.006982` to `0.024608` and 2-10 kHz attack from `0.010363` to `0.011636`, with `0.2551` peak, no clipping, and no master-limiter dependency. Tests prove the exact owner/rest map, overlapping voice tails, distinct body/attack regions, full-block/127/128-frame sample identity, a fresh continuous second bar, clean Fill-to-Break state, exact one-subdivision seek reset, and sample-exact SourceSupport/BreakReinforce/Takeover legacy output. The `riotbox-audio` library suite passes `153/153`. Exact RuntimeMix Candidate-7 preflight and structured human listening remain required before any musical pass.
Consequences: arrangement contrast and drum-phrase quality are separate gates. A clear dropout cannot compensate for a weak exposed phrase, and a body-rich isolated Fill cannot compensate for a buried live mix. Fill low-end evidence describes drum thump while bass ownership remains unassigned. This decision does not claim distinct orchestration when `PhraseDrive` is combined with another adoption profile; TakeoverGrid-specific Fill ownership is a later audible slice. Broader voice/pattern expansion must follow a human-useful Golden Path result rather than multiplying procedural variations.
Status: accepted

---

### RBX-141

Date: 2026-07-16
Topic: fixed live Fill vocabulary is versioned product instrumentation with explicit source pressure modulation
Phase: P023 / Live Musical Alpha
Question: after the voice-separated Fill remained programmed and its gain-tuned payoff remained difficult to distinguish, how should Riotbox make one structurally stronger close without continuing scattered float tuning or falsely calling fixed vocabulary source-derived behavior?
Decision: replace RBX-140's `1 / 2 / 4 / 6` close with one typed, versioned Fill-recipe authority shared by trigger policy, callback dispatch, and `FillFocus`. Preserve `PhraseDriveAccentGhostV1` as the reproducible Candidate-8 control, and select `PhraseDriveChokeDiveStompV1` only for the supported `MainlineDrive + PhraseDrive` Golden Path. Its `1 / 2 / 4 / 5` sounding-event arc ends `kick / rest / snare / short hat / choke / pitch-dive kick+snare flam / ghost hat / rest`; choke is articulation, not a sounding event. Keep oscillator, decay, and retrigger values in named callback-local voice profiles rather than the musical recipe. Expose the stable recipe ID through the typed TR-909 render contract so exact-path manifests record the actual selection inputs instead of duplicating string logic. Keep source-derived `transient_backbeat` pressure separate from recipe selection: it modulates the resolved TR-909 drum level and slam intensity but does not select steps or recipe identity. Record the recipe-owned focus effect on the non-TR-909 bed and Blend source as affected RuntimeMix paths.
Why: Candidate 7 improved body but the human verdict remained weak and too programmed, with no memorable peak. Candidate 8 established measurable accent hierarchy but repeated listening found no meaningful audible improvement. Another inline gain pass would repeat the same failure while leaving step plans, trigger predicates, focus timing, voice articulation, and QA as separate numeric authorities. Candidate 9 therefore changes the gesture grammar through air, choke, pitch movement, flam, and exit rather than only level.
Evidence: the recipe extraction preserved the pre-phase-review Candidate-9 RuntimeMix Fill WAV byte-for-byte (`sha256 cd3bf7d435105ad1f6ffa766d9200f73f639fa01cbfb58d912644b685c992450`). The later shared downbeat-phase correction intentionally rescheduled the exact-path Fill from cursor 20 to the selected source downbeat at cursor 23; the fresh phase-corrected exact Blend Fill is `sha256 f13839facd85e0c29cbd0691d71b588d4b7bdd4d3f6109d9f130c6dd143a9f9f`. Tests prove the shared Candidate-8/9 prefix, distinct versioned closes, `1 / 2 / 4 / 5` sounding density, recipe-owned focus landmarks, generic Fill behavior, callback partitions, and Fill transitions. The dense-break manifest records `phrase_drive_choke_dive_stomp_v1`, its typed `mainline_drive + phrase_drive` selection inputs, source `transient_backbeat` plus derived and resolved pressure values, exact drum/slam/focus RuntimeMix paths and artifacts, the source bar-phase identity, and a role-aware exact Blend Fill-to-break/slam boundary proof. Candidate 9 still requires structured human listening before any musical pass.
Consequences: a fixed, typed, versioned primitive reached by an explicit committed performer gesture is valid product instrument vocabulary, but it is not a missing-source fallback and cannot claim source-derived recipe selection, composition, demo readiness, musical quality, or Riotbox intelligence. Source evidence may truthfully modulate its pressure without changing that recipe provenance; source-modulated product primitives use `riotbox.primitive_renderer_boundary.v2` and must disclose the exact modulation path. Listening manifests distinguish `product_primitive_vocabulary` from `non_product_diagnostic_control`, keep `quality_proof: false`, and block only promotion to source-derived recipe/pattern intelligence. RIOTBOX-1401 remains responsible for moving recipe selection and longer-form development under real source evidence.
Status: accepted

---

### RBX-142

Date: 2026-07-16
Topic: typed Undo markers close the Action/Session/Replay relation
Phase: P023 / Live Musical Alpha
Question: how can a live Fill or monitor action be undone without colliding with pending action ids, losing commit provenance, or making every later snapshot unusable?
Decision: allocate queued and structural action ids through the single `ActionQueue` authority. Commands advertise undoability only when a typed commit-time pre-state snapshot and restoration path exist. Persist each successful `undo.last` as a non-undoable Immediate action with `ActionParams::Undo { target_action_id }` and its own `ActionCommitRecord`. Replay validates and omits that structural marker and its undone target. Treat pre-marker snapshots as stale, but accept snapshots at or after the matching marker as replay anchors. A TR-909 Fill undo restores only the prior committed `last_fill_bar`; it never re-arms queue-only pending state. Restore rejects duplicate persisted action ids. Live target selection requires an accepted result, exactly one target commit record, and the typed snapshot. Newer overlapping typed-domain mutations are conflict barriers; `source_timing.revert_grid` therefore prevents an older MC-202 snapshot from restoring revoked source-plan trust.
Why: the prior direct marker could reuse an id already held by a pending Fill, had no durable target/boundary/sequence relation, and made post-undo snapshots indistinguishable from stale pre-undo payloads. The default `Undoable` label also overstated commands without an implemented inverse.
Evidence: focused regressions cover pending Fill plus monitor undo plus later Fill commit with unique ids through JSON reload and replay; Fill commit/undo restoration; strict marker result/policy/Immediate/latest-target/one-target validation; shared `1,2,3` boundary sequencing across normal, Undo, batched, and direct side-effect commits; rejection of duplicate ids and malformed marker relations; rejection of pre-undo snapshots; acceptance of a post-undo snapshot followed by a replayed tail action; safe normalization/roundtrip of legacy committed actions without new snapshot fields; rejection of a typed live target without its unique commit record; and the MC-202 action → matching source-grid revert → Undo regression that proves the cleared source plan is not resurrected.
Consequences: `undo.last` remains a structural action rather than an executable replay side effect, but it now participates honestly in the product spine. Non-undoable actions remain explicit debt instead of UI/session promises. Legacy untyped markers cannot make a payload into a trusted post-undo anchor. Successful committed typed actions without a persisted pre-state snapshot degrade to `NotUndoable`; legacy undone Source Monitor or TR-909 Fill histories without a trusted typed marker are rejected because their former rollback did not establish replay convergence.
Status: accepted

---

### RBX-143

Date: 2026-07-16
Topic: selected source downbeat phase is shared product timing truth
Phase: P023 / Live Musical Alpha
Question: when the selected primary hypothesis places bar 1 on a non-zero transport beat cursor, which phase owns Jam transport, capture preview, phrase selection, and exact live-path QA?
Decision: resolve the earliest evidenced primary `BarSpan` through its matching one-based `BeatPoint`, convert that identity once into a zero-based `TransportBarGridAnchor`, and use it for Jam bar/phrase projection, Source Map next-bar capture rounding, and exact RuntimeMix gesture boundaries. A populated but inconsistent bar/beat grid is unavailable rather than permission to invent cursor-zero phase. Phrase selection reads the selected primary hypothesis's non-empty phrase grid before the top-level compatibility grid. Exact-path manifests persist the source beat identity, transport cursor, source bar identity, meter, and committed gesture boundaries.
Why: the dense-break source selected beat 4 as bar 1, which is transport cursor 3, but several live consumers restarted bar arithmetic at cursor 0. That placed the prior Fill at cursor 20, one beat after the actual source downbeat at cursor 19/23, while logs and zero-phase bar labels still looked internally plausible.
Evidence: focused regressions cover anchor-aware transport mapping, next-bar rounding, Source Map capture range, divergent primary/top-level phrase grids, normal Jam transport at cursor 19, and the exact dense-break timing fixture. The fresh C9 manifest records anchor beat 4 / cursor 3 / bar 1 and commits W/F/S/y/Y at cursors 20/23/27/31/35 with bars 5/6/7/8/9; the smoke validator asserts this relation and passes without manifest failures.
Consequences: any future source-aware quantizer or arrangement boundary must consume the shared anchor or explicitly report unavailable phase. Zero-phase arithmetic remains only the compatibility fallback when no source bar grid exists. The phase correction changes the full exact-path audio schedule and therefore invalidates pre-correction C9 listening hashes and comparisons even though the typed Fill recipe itself is unchanged.
Status: accepted

---

### RBX-144

Date: 2026-07-17
Topic: reviewed weak Fill micro-cut becomes a versioned long-choke/stomp close
Phase: P023 / Live Musical Alpha
Question: after phase-corrected Candidate 9 remained technically valid but sounded effectively unchanged from Candidate 8 in the full Blend, how should Riotbox strengthen the pause without returning to scattered gain tuning?
Decision: preserve `PhraseDriveChokeDiveStompV1` as the historical Candidate-9 control and select a new `PhraseDriveLongChokeDiveStompV2` for `MainlineDrive + PhraseDrive`. Keep the first three beats stable, then use a three-hit final-beat setup, choke at step 27, two explicit rests, and a late DiveStomp at step 30. Pair it with a recipe-owned `FillFocus` pocket that reaches zero before the choke and stays absent until beat `3.75`. Judge the change as arrangement/performance impact plus drum/transient payoff, not generic pressure or whole-render loudness.
Why: structured listening marked C9 `technically_ok_but_musically_weak`: its approximately 44 ms audible hole was too short and masked to change the room. C8/C9 whole-render correlation had also been confounded by an earlier source-phase correction. A new versioned recipe keeps the reviewed control reproducible and makes the musical grammar, rather than isolated floats, own the intended contrast.
Evidence: the phase-corrected C9 and V2 exact-path `F` counterfactuals are sample-identical before the gesture (`sha256 7081a3c1ded08fefb5cecdd9fca86065aa3bce24839c71585e32c09c6e88f0ce`). C9's exact Blend hole is 44.4 ms; V2's is 166.1 ms. In that matched window V2 RMS is 0.34% of C9, while the late payoff is 17% higher in RMS, approximately 87% higher in 120-500 Hz drum body, and approximately four times higher in 2-10 kHz attack energy. Everything after the changed closing beat is sample-identical. The exact RuntimeMix smoke and the complete `riotbox-audio` suite pass without clipping, click-boundary, callback-partition, or non-Fill regressions.
Consequences: Candidate 10 was technically review-ready and later received the structured human verdict `technically_ok_but_musically_weak`: the pause worked, but the return stomp was not yet characteristic or physical enough. The stronger deterministic primitive remains performer-owned vocabulary, not source-selected composition or demo-readiness proof. Future recipe changes must preserve prior reviewed IDs and compare against a phase-identical exact-path control.
Status: accepted

---

### RBX-145

Date: 2026-07-17
Topic: preserve the reviewed Fill pause and version only its weak return articulation
Phase: P023 / Live Musical Alpha
Question: after Candidate 10 made the arrangement pause clearly audible but its payoff remained musically weak, how should Riotbox strengthen the return without lengthening the pause or resuming float-by-float gain tuning?
Decision: preserve Candidate 10 as `PhraseDriveLongChokeDiveStompV2` and test an experimental `PhraseDriveLongChokeImpactStompV3` for `MainlineDrive + PhraseDrive`. Freeze the four-beat grid, trigger owners and levels, `FillFocus`, 166 ms pocket, opening pitch dive, and delayed snare crack. Change only the typed step-30 articulation by scheduling one fixed 24 ms delayed low-drum aftershock inside the same callback-local payoff event. Treat it as drum/transient body, not an assigned bass lane or second grid event.
Why: structured listening found C10's silence successful but its return insufficiently characteristic and physical. Another pause change, global level lift, or small trigger-weight edit would obscure which perceptual dimension improved. A delayed body event changes articulation and time structure while keeping the reviewed arrangement control reproducible.
Evidence: C10 and C11 exact-path `F` counterfactuals remain sample-identical (`sha256 7081a3c1ded08fefb5cecdd9fca86065aa3bce24839c71585e32c09c6e88f0ce`). Their candidate Blends are sample-identical through `0.365270833 s`, including the opening return attack, and sample-identical again after `0.454770833 s`; only the declared aftershock tail differs. In the isolated voice regression, late 40-120 Hz drum body rises from `0.205354` to `0.246411` (about 20%) with `0.178498` delta RMS and `0.046255` maximum adjacent-frame delta. In the exact RuntimeMix changed window, RMS rises 4.34%, low-band spectral magnitude rises about 9.5%, correlation is `0.983024`, peak remains `0.429641`, and the whole render rises only 0.28% in RMS. The full `riotbox-audio` suite, manifest fixtures, clippy, and exact dense-break live-path smoke pass.
Consequences: Structured listening rejected Candidate 11: the change remained musically irrelevant and the Fill was not something the musician would use. V3 is therefore removed from active product selection rather than promoted. Candidate 10/V2 remains the reproducible scaffold while Riotbox fixes the discovered source-bar phase mismatch and re-evaluates the complete build-up/payoff order. Do not add another aftershock or micro-parameter revision.
Status: rejected

---

### RBX-146

Date: 2026-07-17
Topic: align the complete live Fill arc to the confirmed source-bar phase
Phase: P023 / Live Musical Alpha
Question: why did repeated payoff changes remain musically ineffective even after the Fill pause became technically clear?
Decision: project the confirmed primary timing hypothesis's zero-based bar anchor from the existing `LivePerformancePolicy` into `Tr909RenderState`. The callback-safe Fill recipe grid, voice reset boundaries, tail release, and paired `FillFocus` envelope subtract that same anchor before evaluating bar-local position. Preserve zero-phase behavior only when no finite confirmed anchor is available. Remove the rejected V3 aftershock and re-evaluate the existing V2 recipe after phase correction.
Why: the dense-break source bar begins at transport cursor 23 on an anchor phase of cursor 3, while the audio recipe previously used `position_beats mod 4`. The committed source-aware Fill window was therefore correct, but the audible four-beat recipe was rotated to `payoff/pause -> beat 1 -> beat 2 -> beat 3` instead of `beat 1 -> beat 2 -> beat 3 -> payoff/pause`. Parameter changes touched the opening event while the gesture then decayed through three ordinary beats, explaining why technically measurable versions did not become musically usable.
Evidence: a unit regression proves that absolute position 23 with confirmed anchor 3 renders sample-exactly like the established zero-phase recipe at position 20, while the unaligned position-23 control differs. `FillFocus` has a matching phase-equivalence regression. In the exact RuntimeMix smoke, the same 164.625 ms digital silence moves from `0.176646-0.341271 s` to `1.541542-1.706167 s`, placing it in the final beat. The corrected V2 candidate differs from the old C10 artifact across 68.1% of 10 ms windows, with whole-render correlation `0.708122` and delta RMS `0.065290`; this is a structural reordering, not a loudness adjustment. The manifest records anchor cursor `3`, resolved render phase `3.0`, and `runtime_mix.tr909.source_bar_grid_phase`.
Consequences: pre-fix C9-C11 listening verdicts remain valid for their exact hashes but cannot judge the corrected four-beat narrative. One fresh structured listening review is required for the phase-corrected V2 artifact. If that correctly ordered gesture remains unusable, route to a new musical vocabulary rather than further phase or micro-parameter work.
Status: accepted

---

### RBX-147

Date: 2026-07-17
Topic: replace the rejected closing-accent Fill lineage with a half-bar live takeover
Phase: P023 / Live Musical Alpha
Question: after the correctly phased V2 Fill remained indistinguishable and unusable to the musician, what bounded gesture can complete RIOTBOX-1335 reachability without another closing-accent or float-tuning iteration?
Decision: preserve the reviewed historical recipe IDs, but select `PhraseDriveBreakCutStompV1` for the supported `MainlineDrive + PhraseDrive` Golden Path. Preserve only beats one and two as context. Before beat three, use the recipe-owned click-safe `FillFocus` to remove the Blend source and non-TR-909 Riotbox bed for the complete final half bar. Give beat three a six-event syncopated drum call and beat four a five-event rush, callback-local choke/rest, and late DiveStomp. Treat this as fixed performer-triggered `primitive_renderer` vocabulary with bass ownership unassigned, not source-derived composition or a bass-pressure claim.
Why: the phase correction proved a real timing defect but structured human review still rejected its output as having no musically relevant difference. Every prior candidate concentrated its change in the last fraction of one beat while the continuous source and melodic bed owned nearly the whole gesture. A half-bar arrangement takeover changes the performer function and event grammar rather than increasing another trigger, tail, or gain float.
Evidence: the exact RuntimeMix candidate selects `phrase_drive_break_cut_stomp_v1`, commits at source-aligned cursor `23`, clips no samples, and does not engage the limiter. Against the rejected correctly phased V2 candidate, the second-half waveform correlation is `0.535613` with `0.062067` delta RMS; against the same-position BreakReinforce counterfactual the whole-render correlation is `0.659626` with `0.072324` manifest delta RMS and `0.939227` relevant 10 ms window activity. The new four-beat density is `1 / 2 / 6 / 5`; Audio, App, Core policy, manifest fixtures, and the exact dense-break live-path smoke pass. The exact three-repeat review artifact is `sha256 0c9d5f411862854c0f04255ac17736757262743a49a88f1ff0b636755e9ec4b4`.
Consequences: this change satisfies the bounded RIOTBOX-1335 reachability requirement that `f` cause a plainly large exact-product-path delta. It does not certify hook memorability, Riotbox character, demo readiness, or musician preference. The listener explicitly declined to issue a verdict for this example, so `human_verdict` remains `unverified`; no pass is inferred from metrics or agent judgment. RIOTBOX-1401 owns the next source-backed curated preset and its human musical-quality gate. Do not reopen RIOTBOX-1335 for another Fill micro-variant unless the exact live gesture collapses technically.
Status: accepted

---

### RBX-148

Date: 2026-07-17
Topic: make Feral Break Alpha a versioned product preset and an eight-bar live arc
Phase: P023 / Live Musical Alpha
Question: how should Riotbox expose one curated Golden Path without returning to scattered float tuning, scripted arrangement truth, or false bass claims?
Decision: add the typed `feral_break_alpha_v1` preset under `feral_rebuild` and activate it through the new immediate, session-scoped `preset.activate` action. Persist named profile/preset identity in `RuntimeState.style`, materialize one centralized set of monitor/macro/mixer defaults, and replay the same versioned definition. Keep W-30 `source_hook_lead`, TR-909 `break_pressure`, MC-202 `source_evidence_selected`, and bass ownership through the existing `LivePerformancePolicy`; the preset must not create source material, captures, phrase plans, patterns, scenes, or fallback audio. Build its exact-path eight-bar candidate from committed live gestures: W-30 hook for two bars, TR-909 pressure lift for two, one-bar destructive Fill, one-bar scene role swap, and a two-bar restored return with W-30 damage.
Why: the prior exact path proved individual gesture reachability but produced only a five-bar diagnostic sequence and required manual control reconstruction. Earlier float-by-float Fill iterations also showed that technically different values can remain musically irrelevant. A named versioned recipe makes the defaults reviewable and replay-stable, while the longer source-backed arc gives hook, lift, removal, contrast, and changed return distinct performer-owned jobs.
Evidence: Core and App tests prove typed ID persistence, legacy Session V1 defaults, Queue/Commit, observer/TUI exposure, and deterministic replay. The exact callback-block RuntimeMix run against the real `Beat03_130BPM(Full).wav` source records `preset_id=feral_break_alpha_v1`, `typed_bass_owner=unassigned`, 32 beats / eight bars, no clipping or limiter activity, hook-to-pressure delta RMS `0.049884`, and hook-to-return correlation `0.019427` with delta RMS `0.124567`. It also records capture -> raw audition -> promotion -> save -> restart -> live recall -> trigger on the same product spine; the restarted trigger routes to `music_bus_preview` with RMS `0.094251`. Raw and RMS-matched source/candidate A/B artifacts are generated locally. The exact eight-bar candidate is `sha256 c105b171d592f84a137b49e5142d47d5e47db2b3ee92ead0cd482aef25061820`.
Consequences: RIOTBOX-1401 may claim one curated, replayable, exact-product-path candidate and documented five-minute workflow, but not a human musical pass or recognizable Riotbox character. The current real-source candidate assigns bass owner `unassigned`; it is judged for hook, drum/transient impact, destructive contrast, and return, not failed for absent bass. RIOTBOX-1402 owns the exact structured human verdict. Any later change to these preset defaults requires a new versioned ID or an explicit compatibility decision rather than silently rewriting `feral_break_alpha_v1`.
Status: accepted

---

### RBX-149

Date: 2026-07-17
Topic: project later performance boundaries into trusted short-source Scene sections
Phase: P023 / Live Musical Alpha
Question: how can a two-bar analyzed source retain source-backed MC-202 phrase ownership when a live arrangement commits at bar 5 or later?
Decision: when the commit boundary is outside the source's own bar/phrase range, first resolve its typed projected Scene to the corresponding Source Graph section. Prefer the section-bounded overlap of a primary source phrase; otherwise derive a phrase slot only from the intersection of that section and the confirmed primary bar grid. Use the same projected section for feature-ownership checks. Without a recognized projected Scene, trusted primary grid, or non-empty section/grid intersection, remain unavailable.
Why: Session transport bars describe the developing performance, not extra bars inside a short source file. Looking up performance bar 5 directly in a two-bar source falsely discarded real source evidence and prevented the live policy from loading. Mapping through the existing Scene projection preserves source identity without wrapping source-monitor audio, inventing a phrase, or adding a second timing truth.
Evidence: MC-202 source-phrase unit tests cover primary phrase preference, short-source projected phrase-grid reuse, bounded section/grid derivation, and fail-closed unknown Scenes. The real Beat03 exact-path run now resolves `scene-01-intro`, derives a source-backed Fill-pickup instigator with bass owner `unassigned`, and passes the strengthened eight-bar, A/B, limiter, capture, restart, recall, and trigger validator. Source-only reference playback still starts at source time zero and remains bounded by the existing clamp-at-EOF monitor contract.
Consequences: short loops may drive longer live arrangements through projected source sections, but transport advancement alone never proves source phrase ownership. Future explicit source-loop/wrap behavior remains a separate monitor-mode contract; this decision does not silently loop decoded source audio.
Status: accepted

---

### RBX-150

Date: 2026-07-17
Topic: make the Feral Break Alpha destructive Fill product-owned and prove its negative space locally
Phase: P023 / Live Musical Alpha
Question: after the exact `feral_break_alpha_v1` candidate remained repetitive and musically weak, why did selecting a stronger typed Fill recipe initially leave its eight-bar artifact byte-identical, and what is the smallest honest correction?
Decision: preserve `feral_break_alpha_v1` as the reviewed historical identity and introduce `feral_break_alpha_v2`. V2 retains the same typed lane roles and mixer/macro defaults, but its committed TR-909 Fill projects the distinct typed `PhraseDriveHardCut` variation after any explicit Scene-movement variation; non-Fill projection and explicit Scene movement keep their existing ownership. Select the versioned `PhraseDriveBreakCutStompV2` primitive only for `MainlineDrive + PhraseDriveHardCut`, while the historical `MainlineDrive + PhraseDrive` pair remains on V1. Beat four chokes at step 24, rests through steps 25-29, returns with one late DiveStomp at step 30, and ends on a rest. The exact Alpha renderer must read the recipe from the actual destructive-stage `RuntimeMixRenderPlan`, not from the separate gesture proof. Gate the product artifact on complete silence during steps 26-29 and a non-silent hard return at step 30.
Why: the first revised manifest advertised V2 by borrowing recipe identity from a separate main gesture, while the Alpha Fill occurred on an odd ambient phrase cycle and actually selected `PhraseLift + generic_fill_v1`. Consequently the new full Alpha artifact was byte-identical to the human-rejected v1 candidate. Aggregate stage deltas had hidden both the false attribution and the absence of a macro arrangement event.
Evidence: an invariant first made the renderer fail honestly with `generic_fill_v1` instead of publishing the mismatched manifest. App projection tests prove that the same odd-phrase Fill remains `PhraseLift + GenericFillV1` without the preset but resolves to `PhraseDriveHardCut + PhraseDriveBreakCutStompV2` under v2; a separate regression proves that v1's even-phrase Fill remains `PhraseDrive + PhraseDriveBreakCutStompV1`. Against the rejected v1 candidate (`sha256 c105b171d592f84a137b49e5142d47d5e47db2b3ee92ead0cd482aef25061820`), the corrected eight-bar artifact is byte-distinct (`sha256 f4c74703c157d66ede579b9a9200e9532f12dd04ba58b5326d81f7e5f46564d3`). The exact Fill stage records 18.50% silence instead of 0.12%; its declared half-beat negative-space window is digitally silent (`rms 0`, `silence_ratio 1.0`) and the immediate step-30 return reaches `rms 0.207901`, `peak 0.690721`, with no clipping or limiter dependency. The exact dense-break validator passes.
Consequences: manifest recipe provenance must come from the artifact-owning plan; a different proof path cannot stand in for it. Whole-stage metrics remain diagnostic and cannot prove a dramatic pause, so Golden-Path arrangement gestures require role-specific time-local evidence. V2 is technically review-ready but retains `human_verdict: unverified`; measurements cannot promote it to a musical pass. V1 remains replayable and unchanged.
Status: accepted

---

### RBX-151

Date: 2026-07-18
Topic: keep one Golden Path taste target while gating shared audible tuning across real sources
Phase: P023 / Live Musical Alpha
Question: should an audible DSP, mix, pattern, or performance-policy change be tuned and reviewed only against the current Golden Path source?
Decision: keep one supported Golden Path as the structured human taste target, but require every shared audible tuning change to pass a nearest-exact product-path matrix of at least three contrasting real sources before another human review. The matrix rejects exact-path failure, silence, clipping or hidden limiter dependency, timing regressions, and near-identical source-backed hook envelopes. It remains diagnostic and cannot replace the Golden Path human verdict.
Why: a single-source pass can overfit one break, hide source-dependent hot paths, or make a fixed transformation look like general instrument behavior. Requiring broad human listening on every tuning iteration would be slow and dilute the Golden Path; a bounded automated safety/diversity matrix catches structural collapse while preserving one focused taste decision.
Evidence: the first Beat03-only W-30 bite candidate passed its exact diagnostic, while the new four-source matrix exposed hot Beat20 monitor/restart mixes and a false click classification on a legitimate sustained DH BeatC downbeat. The corrected policy respects an active preset's W-30 mixer ceiling, gives the non-bass MC-202 instigator less bus allocation, and distinguishes a supported transient from an isolated discontinuity. Beat03, Beat08, Beat20, and DH BeatC then pass the exact RuntimeMix renderer; after normalizing every eight-bar W-30 hook envelope to the same 512-point time axis, their maximum pair correlation is `0.492000`, below the diagnostic collapse threshold `0.95`.
Consequences: `just dense-break-live-source-matrix` is required for shared audible dense-break tuning before a new Golden Path listening request. Its generated `source-matrix-report.json` stays `quality_proof: false` and `human_verdict: unverified`. Commercial references remain local comparison material and never enter this source matrix.
Status: accepted

---

### RBX-152

Date: 2026-07-19
Topic: require explicit resample lineage before routing the synthetic W-30 internal tap
Phase: P023 / Live Musical Alpha
Question: why did an isolated promoted W-30 hook contain a persistent background hum even though the musician had not requested or committed an internal resample?
Decision: keep `W30ResampleTapState` on the existing callback seam, but project it as audible only when the focused capture is typed `CaptureType::Resample`, has non-empty capture lineage, and has positive resample generation depth. Ordinary loop/pad capture, raw audition, pad promotion, live recall, and pad trigger keep the tap idle and silent. `promote.resample` remains the explicit action that creates the first eligible resample capture; capture presence or W-30 focus alone is not activation evidence.
Why: the older preparation seam automatically marked every focused capture `CaptureLineageReady` and routed a synthetic oscillator beside the source-backed pad. In the exact promoted Pad session, `cap-01` had no lineage and generation depth zero, yet the tap produced a fixed promoted-profile oscillator near 177 Hz. The musician correctly heard it as background hum during a claimed isolated hook review. That automatic scaffold polluted the Golden Path, lacked explicit performer ownership, and made the playback assignment false.
Evidence: a new App regression proves that even a malformed ordinary Pad carrying fake lineage/depth remains on the default idle/silent tap state, while committed explicit resample tests retain their audible lineage-ready route. Capture-shell diagnostics now report `tap idle/silent` for ordinary capture. The exact callback-block Beat03 live-path run passes after removal; the eight-bar candidate remains non-silent and unclipped with peak `0.626358`, RMS `0.076353`, and no limiter dependency. Its hash changes to `e885a0641d3210a38b099c5133f0f7e801c00ab5cce0a3e27b79e0e8b41e83dc`, so the earlier accepted-for-iteration hash remains historical and the corrected hook requires fresh listening. The required Beat03, Beat08, Beat20, and DH BeatC exact source matrix also passes; its maximum time-normalized W-30 hook-envelope correlation is `0.494912`, below the `0.95` collapse threshold.
Consequences: internal routing does not make a voice inaudible. Future isolated listening briefs must inventory every callback/mixer contributor and either silence unclaimed taps/support voices or label the artifact as a composite. Explicit resample workflows retain the synthetic primitive for their bounded existing contract; this decision does not claim that primitive is finished resample sound design.
Status: accepted

---

### RBX-153

Date: 2026-07-21
Topic: make the Feral Break Alpha v2 QA path match the documented instrument path
Phase: P023 / Live Musical Alpha
Question: why did the corrected hook still sound like unrelated material was pasted together, and why did the exact gesture pack fail after moving the preset to Riotbox-only monitoring?
Decision: materialize `BreakReinforce` and `Riotbox` as typed `feral_break_alpha_v2` preset defaults; keep Source and Blend only as explicit monitor A/B evidence. Execute the exact performance path in documented order `w -> s -> f -> y -> Y+D`. The V2 Fill owns a whole-bar non-TR-909 focus handoff and a recipe-local output trim, while historical V1 retains its original half-bar focus and unity gain. Prove the changed return as two same-boundary performer actions: a restore-only control must recover the pre-jump Scene projection, then committed W-30 damage must account for the additional audible change without altering non-W-30 lanes.
Why: the prior QA renderer secretly queued TR-909 preparation that the musician recipe did not name, rendered `w -> f -> s`, and left the raw source doubled beneath a promoted W-30 capture from a different source phase. Moving to the honest Riotbox-only path exposed that Scene restore alone was too subtle because raw-source repositioning was no longer leaking into the candidate. Lowering the gesture threshold would have certified a status change rather than the documented changed return.
Evidence: the generated 132 BPM exact RuntimeMix smoke and the real Beat03 path pass with no clipping or limiter activity. Across Beat03, Beat08, Beat20, and DH BeatC, Fill pre-limiter peak is `0.910073` with zero limited samples. Changed-return relative RMS deltas range from `1.0858` to `1.2567`, with waveform correlations from `0.0364` to `0.2156`. The four-source W-30 hook-envelope matrix remains diverse at maximum correlation `0.494912`, below the `0.95` collapse threshold. Targeted tests preserve V1 recipe focus/gain independently from V2.
Consequences: scripted exact-path evidence may not hide prerequisite actions absent from the musician flow. Current-version audio tuning must not mutate historical recipe controls. The diagnostic pack remains `quality_proof: false` and `human_verdict: unverified`; a fresh structured human verdict is still required for RIOTBOX-1402.
Status: accepted

---

### RBX-154

Date: 2026-07-21
Topic: close the first P023 Usable Musical Alpha exit on exact-live evidence
Phase: P023 / Live Musical Alpha
Question: when may RIOTBOX-1396 close without mistaking diagnostics, a scripted render, or an attractive fixed demo for a usable live instrument?
Decision: close the first bounded P023 exit only after the exact `riotbox-app` TUI/audio-callback path lands the documented `w -> s -> f -> y -> Y+D` performance arc, passes technical preflight, and receives a fresh human musical pass. Treat the accepted eight-bar action sequence as QA choreography around reusable elements, not as Riotbox's preferred fixed composition. Continue P023 through `RIOTBOX-1404` by expanding the passed live policy to tonal-hook and sparse-pressure material.
Why: RIOTBOX-1401 already proved capture, raw audition, promotion, intentional save/quit, restart, recall, trigger, and deterministic replay, but the phase required exact-live musical judgment. RIOTBOX-1402 then corrected hidden QA preparation, doubled-source monitoring, gesture order, and quantized timing before asking the listener again. The resulting pass establishes one usable dense-break baseline while preserving the musician's direction that the elements should be loopable in another order.
Evidence: PR #1369 merges the product-path corrections and strict observer validator. The final observer lands the five stages at transport positions `50.067233 -> 58.058831 -> 66.100849 -> 70.084042 -> 74.142866` and stops at `81.907573`, validating as `8 -> 8 -> 4 -> 4 -> 7.90757`. The isolated 14.787-second callback capture is SHA-256 `327f9d4d00bd18c294bcf26f86c8b8a3b23f8e4f85474572735139d627d5ce61`, measures `-18.6 LUFS` and `-0.3 dBTP` without clipping, and received Markus's direct verdict: `pass, ich wuerde es bestimmt anders loopen aber die elemente sind schon gut`. Recipe 17 and restart/recall callback proof preserve the same source-backed product spine.
Consequences: RIOTBOX-1396 may close as one human-passed dense-break Usable Musical Alpha. This does not claim broad source-family quality, a finished arranger, export readiness, release readiness, or MC-202 bass pressure; Beat03's typed bass owner remains `unassigned`. The strongest musical review element is the break/pause, the TR-909 is the hardest active transient layer, and the source-backed hook remains clear within two bars. Future UX must expose the accepted elements for performer-selected confirmation and looping rather than freezing the review choreography.
Status: accepted

---

### RBX-155

Date: 2026-07-21
Topic: represent musician-declared BPM and downbeat phase as a typed manual grid
Phase: P023 / Controlled Expansion
Question: how may a trusted tonal source with unavailable analyzer timing enter the exact live capture path without weakening timing detection or silently assuming source-start phase?
Decision: keep `--source-bpm` alone as confirmation of a matching Rust-probe hypothesis. Add the explicit paired form `--source-bpm <bpm> --source-downbeat-seconds <seconds>` to install one typed `Manual` Source Graph hypothesis whose beat/bar compatibility projection is derived deterministically from both musician declarations. Require at least one complete bar, retain analyzer hypotheses and warnings, expose `musician_manual` to observers, and confirm the selected hypothesis through the existing `source_timing.confirm_grid` queue/commit/Session/replay/revert path. Do not add a second timing action or persistence model.
Why: `DH_RushArp_120_A.wav` is a musically useful tonal-hook source but the Rust probe correctly finds no kick/downbeat evidence. BPM alone cannot determine phase. Rejecting the file forever would block tonal live use; inventing phase zero or loosening onset thresholds would convert a musician fact into false analyzer confidence.
Evidence: Core tests prove deterministic grid construction, BPM/phase-bound hypothesis identity, declared phase, rejection of invalid BPM/phase/short windows, analyzer-warning retention, and no phrase claim from a partial fourth bar. App tests prove BPM-only rejection for a timing-poor tonal WAV, paired manual ingest, committed confirmation, observer origin, save/restart identity, and trusted consumer readiness. The real local `DH_RushArp_120_A.wav` retains `LowTimingConfidence`, `SparseOnsets`, and `WeakKickAnchor`, stores a `manual-source-grid-v1-*` identity bound to 120 BPM / 0 seconds, and completes the exact callback-path diagnostic with no clipping or limiter dependency.
Consequences: manual timing is explicit musician-owned truth, not automatic source intelligence. UI/QA may call it confirmed only after the existing action commits and must keep its manual origin visible. RIOTBOX-1421 is a bounded enabler and cannot claim a tonal musical pass; it immediately returns control to RIOTBOX-1404 for source-aware hierarchy, restraint, arrangement, and structured listening.
Status: accepted

---

### RBX-156

Date: 2026-07-21
Topic: derive tonal and sparse held-state character without copying the dense Alpha choreography
Phase: P023 / Controlled Expansion
Question: how should the human-passed dense live policy expand to a tonal hook and sparse drum source when all three currently collapse into the same fill-pickup family, TR-909 lead threshold, and scripted eight-bar QA arc?
Decision: retain one shared `LivePerformancePolicy`, but add the typed held-state characters `dense_break`, `tonal_hook`, and `sparse_pressure`. Derive them only from trusted persisted phrase-audio evidence using a named `0.10` two-axis contrast margin; missing or neutral evidence preserves the dense default. Tonal promotes W-30, restrains TR-909 to support/anchor vocabulary, and converts only a generic MC-202 fill-pickup to `stay_out`. Sparse assigns the hardest transient to TR-909, keeps the source rhythm on W-30, and converts only that generic pickup to bounded punctuation. Explicit bass, answer, and stay-out candidates retain ownership. Explicit Fill and Scene vocabulary overrides held-state defaults. Review each non-dense character as a four-bar exact-callback held loop, not by forcing the dense `8 -> 8 -> 4 -> 4 -> 8` choreography to pass. Destructive intent is character-aware: dense and tonal retain source-backed pitch drag, while sparse uses a grid-locked `1.0x` source chop with a bounded per-trigger choke instead of changing playback rate and allowing source kicks to drift against TR-909.
Why: RushArp, BeatC Kick/Snare, and accepted Beat03 all produced transient evidence near `0.74` and selected `fill_pickup_instigator`, so a single `0.72` threshold made their product projections almost identical. Dense gesture gates then incorrectly failed the intentional tonal MC-202 silence and restrained Slam/Scene deltas. Lowering those gates or copying the medley would mis-state the musician role and loopability question.
Evidence: Core tests separate measured tonal, sparse, neutral dense, explicit bass, and stale-section behavior. Exact callback-block controlled renders pass for Beat03, RushArp, and BeatC without clipping or limiter activity. Two independent renders per positive expansion source are byte-identical. Pairwise dense/tonal/sparse 20 ms RMS-envelope comparisons stay below the diagnostic collapse correlation limit `0.95` and above mean absolute delta `0.01`; tonal versus sparse measures correlation `0.411383` and delta `0.061910`. Tonal resolves W-30 lead / MC-202 stay-out; sparse resolves TR-909 transient lead / MC-202 punctuation. All three truthfully keep bass owner `unassigned`. Structured listening records `keep` for the tonal and sparse held loops and for both destructive variants. The first sparse destructive candidate exposed confusing source kicks at `1.1476x`; the accepted replacement is deterministic at `1.0x`, resolves a `0.3608`-step gate for the committed `0.82` damage intensity, preserves the rhythm, and received a fresh human pass specifically confirming that the intermediate kicks are gone.
Consequences: `just controlled-source-live-matrix` remains the deterministic overfitting gate and its generated report remains `quality_proof: false` / `human_verdict: unverified`; the separate structured reviews carry the human taste verdict. Numeric tuning remains owned by the typed policy and numeric-values guide; filenames, raw loudness, or a commercial reference recording may not control classification. With the controlled expansion accepted, P023 returns to the next bounded roadmap slice rather than multiplying more variants of these sources.
Status: accepted

---

### RBX-157

Date: 2026-07-21
Topic: separate fixture calibration from live demo-bank readiness
Phase: P023 / Controlled Expansion
Question: why did P023 readiness appear to have human passes and edge-family priorities even when no real musician demo bank had been supplied?
Decision: make `live_readiness` the default evidence mode for source-family coverage, the release-demo human-review queue, and the aggregate sound-quality readiness report. Do not resolve an omitted live demo-bank path to the checked-in fixture. Keep deterministic CI through an explicit `fixture_calibration` mode labeled `fixture_only`. In live mode, count pass/weak/fail only when an entry carries a non-fixture reviewer plus the path and matching SHA-256 of its structured review. Require queues and derived packs to match the readiness report's evidence mode, fixture flag, bank state, normalized bank path, and bank SHA-256. Model positive-family success as a demo-ready human pass, but model `weak_source` and `bad_timing` success as a reviewed degraded, unavailable, or reject product-path outcome that confirms no fallback music.
Why: the fixture bank intentionally contains synthetic artifact hashes and calibration verdicts. Treating it as the live default hid missing dense-break evidence and made weak/bad-timing sources look as though they needed forced demo-ready music. That confused deterministic schema coverage with musician-facing truth.
Evidence: default live coverage and queue generation now report a missing real bank, zero eligible human verdicts, zero demo-ready coverage, and dense-break review/import as the first action. Explicitly passing the fixture bank in live mode still yields zero eligible human verdicts. Fixture-mode coverage, queue, listening-pack, readiness, and demo-bank promotion smokes remain deterministic. A live-mode mutation proves a non-fixture hashed human review plus typed reject evidence satisfies the bad-timing family contract without a demo-ready pass.
Consequences: CI fixtures remain calibration evidence only. Real demo-bank promotion preserves reviewer and review-hash provenance. RIOTBOX-1403 makes no sound-quality claim; it directly unblocks RIOTBOX-1405 to prove the actual weak/bad-timing product path and structured degraded/reject UX.
Status: accepted

---

### RBX-158

Date: 2026-07-21
Topic: bind negative source-family success to exact degraded product evidence
Phase: P023 / Controlled Expansion
Question: when may weak-source or bad-timing handling count as successful without forcing the source into demo music?
Decision: derive the compact Jam performance state from the shared typed Source Timing consumer readiness and expose the exact reason, bar-locked permission, live-policy state, per-lane generated-output configuration, and fallback state in the observer. Require live negative-family coverage to reference a hash-bound `riotbox.degraded_product_review.v1` over the exact source, Source Graph, Session, and observer stream. The review must prove an exercised real callback path, stopped transport, empty queue/commit history, source-preview-only monitoring, no configured TR-909/MC-202/W-30 output, no confident bar-locked policy, no fallback music, and a human pass for visible risk, useful reason, and understandable next action. Use the explicit `weak_source` demo-bank family instead of aliasing it to `other`. Permit a completed negative review to omit `rendered_wav`.
Why: the earlier live-readiness schema could accept a generic hashed review plus a hand-written `product_path_reviewed: true` boolean. It did not bind the claim to the product artifacts and still required a rendered WAV, which encouraged exactly the synthetic replacement output the negative-family contract forbids. Beat20 also appeared as source score `0.84` at the top of the compact Trust panel even though its timing required confirmation, obscuring the musician-facing performance risk.
Evidence: real PipeWire/TUI ingest classifies Beat20 as `degraded / needs_user_confirmation` with ambiguous downbeat evidence and Fadapad as `unavailable / unavailable` with sparse-onset timing. Both runs exercise the audio callback while transport remains stopped, expose source-only monitoring, leave the Session action/commit log empty, configure no generated lanes, activate no live source policy, and report `fallback_music_present: false`. The compact 80x24 Trust panel now leads with `degraded | bar/live?` or `unavailable | bar/live?`, places the concrete warning directly below it, and leaves the generic source score to wider/source-detail surfaces. Analyzer-locked timing remains `trusted` even when the unrelated overall source score is low, keeping TUI and observer state aligned. Rust exact-RuntimeMix regression proves the unconfirmed Riotbox-only path renders digital silence instead of replacement music. Fixture mutations reject stale artifact hashes, forged stored proofs, unsafe state in any observer snapshot, hidden fallback state, contradictory readiness, and fixture reviews presented as live human passes.
Consequences: honest degradation or refusal is a product success for negative families but never a sound-quality or demo-ready claim. Raw source preview remains distinct from generated Riotbox output. CI calibration remains structurally useful but cannot satisfy live readiness. Markus accepted the exact compact degraded/unavailable state, reason, and next action as a bounded product `pass` with the explicit qualification that the TUI will still be revised generally. Hash-bound live reviews now satisfy `weak_source` and `bad_timing`; the qualification prevents this verdict from becoming a general TUI or sound-quality approval.
Status: accepted

---

### RBX-159

Date: 2026-07-21
Topic: W-30 internal resample taps must render hydrated capture audio instead of a synthetic proxy voice
Phase: P023 / Controlled Expansion
Question: how should the existing lineage-ready W-30 resample callback become genuinely source-backed without creating a second persistence, capture, or realtime I/O path?
Decision: keep `CaptureRef`, its artifact path, and explicit lineage as replay truth. Hydrate the focused resample artifact through the existing non-realtime capture cache, deterministically select its strongest energy/transient region, and project that original PCM into a fixed `4096`-sample mono grain with source start-frame, sample-rate, and frame-count identity. Mirror that bounded grain through the existing coherent callback state. The callback may chop, rate-shift, saturate, and edge-emphasize only those source samples. It receives tempo and position from the shared transport snapshot. If hydration fails, expose typed `source_audio_unavailable`, route silent, and emit no fallback sound. Retire the fixed 130.81/164.81/196 Hz oscillator, shimmer, and hardcoded 124/92 BPM behavior from the product path.
Why: the old voice proved callback wiring but sounded like an unrelated hum and could remain audible without any captured PCM. Metadata-reactive oscillation is not source-backed resampling, and preserving it would violate the no-placeholder product contract. Loading audio inside the callback or persisting the bounded projection would instead violate realtime isolation or create shadow replay truth.
Evidence: queue/commit still creates a lineage-safe `CaptureType::Resample` artifact and the runtime now reports `source_audio_ready` only when that artifact is hydrated. Missing material stays digitally silent and produces an observer/runtime warning. Snapshot restore reproduces the same capture identity and byte-equivalent tap render. Exact RuntimeMix renders for Beat03, Beat08, and Beat20 are non-silent, unclipped, deterministic for the same source, and pairwise distinct: correlations range from `-0.168341` to `-0.011637` with RMS deltas from `0.027515` to `0.035376`. Each corresponding missing-source control is digital silence. A direct raw-level A/B used the complete `3.692313`-second Beat03 source at `-14.8 LUFS`, `250 ms` silence, then the first `3.692313` seconds of the exact tap at `-29.8 LUFS`. Markus described the candidate as a very timid, gentle tap and could no longer recognize the source. The structured verdict is `technically_ok_but_musically_weak`, strongest element `chop`, source `source_lost`, and hook `weak`.
Consequences: the bounded PCM window is heap-owned runtime cache state derived from Session/artifact truth, not a new persistence model; keeping it out of inline `RuntimeMixRenderPlan` copies preserves the existing large-render stack boundary. Commercial references remain measurement/listening references only and never enter the source window. The implementation contract is accepted, but the human weak verdict blocks demo-ready promotion and is not a P023 sound-quality pass. RIOTBOX-1422 owns the concrete level, source-character, and performer-triggered hard-variation follow-up. It must tune source-derived transforms and triggerability on this seam and must not restore a free-running tone as fallback.
Status: accepted

---

### RBX-160

Date: 2026-07-24
Topic: separate legal multi-family development sources from rotating fresh holdouts
Phase: P023 / Controlled Expansion
Question: how can RIOTBOX-1422 improve the source-backed W-30 resample without continuing to tune hardcoded values against one narrow loop family or consuming its acceptance evidence?
Decision: add the versioned `riotbox.source_holdout_rotation.v1` contract over local ignored CC0 source derivatives. Require at least twelve eligible real sources and source packs across the six core families, a candidate matrix of at least five eligible sources across four families, and two disjoint multi-family holdout sets. Keep holdout classification provisional and audio unheard until acceptance. When a holdout informs later implementation, remove it from unseen/reserve evidence, record the consuming ticket and date, and add a fresh replacement. Treat `dense_full_mix` as a masking stress family that never counts as positive `dense_break` coverage. Keep commercial recordings reference-only and reject their paths or flags from the corpus.
Why: the earlier sound-excellence corpus was a useful coverage map but repeated a narrow local example family and did not enforce development/holdout separation. Human review correctly reclassified the first new dense candidate as a complete mix, revealing that filenames, loudness, and spectral density cannot establish a drum-led break. A separately acquired CC0 sampled drum loop then received human suitability approval as the development `dense_break`, while both original dense candidates remained untouched holdouts.
Evidence: the tracked contract records sixteen sources from OpenGameArt contributions: fifteen core-family-eligible sources across fourteen independent source packs and six development families, plus one confirmed `dense_full_mix` stress source. The local qualification inventory binds author, page/download provenance, CC0 license, excerpt offset, and SHA-256. `just source-holdout-rotation-fixtures` rejects candidate family collapse, insufficient or reused holdouts, consumed holdouts left marked unseen, unsafe paths, commercial-reference leakage, narrow source-pack collapse, and missing files. `just source-holdout-rotation-local-files` verifies every local derivative by SHA-256, 48 kHz stereo PCM16 format, duration, and clipping boundary.
Consequences: RIOTBOX-1423 is a `contract_enabler`, not musical quality proof. RIOTBOX-1422 must use the development matrix for implementation choices and reserve fresh holdouts for cross-family acceptance. Any holdout-driven follow-up invalidates that source's unseen status until rotation is recorded and replacement evidence exists. The ignored CC0 audio remains local and unredistributed.
Status: accepted

---

### RBX-250

Date: 2026-08-02
Topic: define percussive hardness as force without global pitch/rate substitution
Phase: P023 / Sound Excellence
Question: what must `Hard` mean after RIOTBOX-1428 H31 Stage A passed timing, cross-source difference, attack/body, crest, and spectrum gates but sounded only lower-pitched and not more forceful?
Decision: `percussive_hard` is a typed musician-facing role, not a generic damage preset. It preserves one aligned onset, recognizable source identity, and `1.0x` playback while increasing immediate attack force, retaining physical post-attack body, preserving bite, and controlling the tail. Global resampling/transposition is prohibited; a local source-consistent spectral or resonant change is not an automatic reject but cannot itself prove force. Pitch dive, destructive damage, hook hardness, and bass hardness are separate roles. Lower, louder, darker, dirtier, doubled, or merely different output cannot satisfy this role. A human `different but not harder` verdict rejects and freezes the recipe.
Why: H31 Stage A v1 reconstructed the body near `0.78x`. The listener heard the first half as higher and the prepared half as lower while timing remained correct; absolutely nothing sounded more forceful. Automated crest, attack-RMS, body-retention, body-band, edge-band, and difference gates all passed, so they screened signal collapse but failed the semantic role.
Evidence: the exact `4.926848 s` loudness-matched Beat03 A/B contained eight sample-exact source events followed by eight sample-exact prepared events, integrated at `-16.2 LUFS` with `-1.0 dBFS` true peak. The structured RIOTBOX-1428-H31-STAGE-A verdict is `reject`, strongest element `none`, with source transformed but present. Three-source development evidence remained technically passing and therefore demonstrates why cross-source metrics cannot substitute for the force verdict.
Consequences: retire all global playback-rate transformation from the next percussive-Hard recipe. Fold the narrow pitch-stable draft into the broader contract accepted by RBX-251, and align the audio-QA, AGENTS, and rave-punk production guardrails. A second source-synchronous attack/body attempt was mechanically rejected across three sources for near-identity and/or body loss before human playback. Product integration remains blocked until a dedicated research prerequisite defines the perceptual model and falsifiable hypotheses, after which RIOTBOX-1428 must implement materially different candidates and earn the isolated human pass.
Status: accepted

---

### RBX-251

Date: 2026-08-02
Topic: model percussive force and beat impact as typed, multi-scale constructs before further DSP implementation
Phase: P023 / Sound Excellence
Question: after several technically different `Hard` candidates remained perceptually unchanged, duller, lower, dirtier, doubled, or weaker, what must Riotbox understand and freeze before it changes the renderer again?
Decision: treat a beat as an interaction across event articulation, pulse/meter, pattern topology, performance timing and dynamics, role interlock, phrase/arrangement, mix/playback, and listener/use. Keep strike force, timbral hardness, punch, body/weight, bass pressure, aggression, groove, heaviness, softness, and arrangement impact as separate typed constructs. Do not encode one universal hardness score, genre timing offset, random-jitter humanizer, or literature-derived product constants. RIOTBOX-1429 is research-only: it owns the evidence-labeled vocabulary, analysis rubric, historical failure explanation, and unexecuted preregistration. The already-existing RIOTBOX-1428 owns Stage-A implementation and falsification across contrasting legal sources; its Stage-B product promotion remains blocked until the exact mechanism passes mechanical controls and a directional human listening verdict.
Why: RIOTBOX-1422 H27-H30 and RIOTBOX-1428 H31 demonstrated that time/frequency deltas and scalar revisions can prove change while missing the intended musical property. Primary research shows that event-local force, timbral hardness, and punch use multiple source- and listener-dependent cues; groove and heaviness additionally depend on role, tempo, meter, phrase, mix, and context. Random timing deviation and generic added density are not reliable groove mechanisms, and distortion or low-end energy can increase aggression or weight without producing a harder strike.
Evidence: `docs/engineering/percussive_force_and_beat_impact.md` combines the committed RIOTBOX-1428 H31 negative evidence and the landed RIOTBOX-1422 H27-H30 hash-bound closeout with primary psychoacoustic, performance, groove, metal-production, funk/breakbeat, masking, silence, and live-playback research. H27 and H30 carry structured-review records; H28 and H29 carry artifact-bound human observations without standard review packs. Their audio remains local, so H27-H31 are registered only as historical observations, not artifact-complete executable controls. `docs/benchmarks/percussive_force_development_matrix_v1.json` records only a future Stage-A design and explicitly states that RIOTBOX-1429 produces no analyzer, candidate render, human playback, or quality proof.
Consequences: implementation ideas discovered during research remain falsifiable hypotheses rather than code or agent law. The narrow hardness draft is folded into `docs/engineering/percussive_force_and_beat_impact.md`. RIOTBOX-1428 must preserve the research conclusions while testing structurally distinct source-adaptive mechanisms; it may reject every family. A standalone skill is unnecessary unless later validated work establishes a genuinely independent recurring workflow; only validated durable rules may enter the existing project-owned Riotbox skills.
Status: accepted

---

### RBX-252

Date: 2026-08-10
Topic: freeze a source-blind Stage-A falsification boundary before source qualification
Phase: P023 / Sound Excellence
Question: how can RIOTBOX-1428 test structurally different percussive-force mechanisms without tuning another recipe to familiar audio, consuming holdouts, or mistaking numerical change for perceived force?
Decision: freeze `riotbox.percussive_force_stage_a_protocol.v1`, `riotbox.percussive_force_development_matrix.v2`, and `riotbox.source_holdout_rotation.v2` before any fresh source feature, event qualification, candidate render, or holdout access. Treat the current state as a `StageAPrequalificationSession`: it may own only the frozen event/source-contrast catalog contract and source-independent synthetic reject evidence. Rendering and reject-only screening require a fresh development-only `StageAQualificationSession` with a new bounded access log; prequalification state cannot be reused as proof that source files were qualified. Preserve F1 `f1_ab_energy_redistribution_v1`, F2 `f2_exact_complementary_three_band_v1`, and structurally distinct F3 `f3_causal_envelope_contrast_dynamic_residual_v2` as the three active hypotheses. Keep `f3_os4_onset_residual_v1` immutable and rejected after its source-independent 4x-versus-8x residual test failed at `-43.38236728008191 dB` against the required `<= -60 dB`; do not retune it or inherit its topology into F3 v2. Keep F3's full controller hashes as provenance only. Its actionable cross-source identity is the separate anatomy-independent `riotbox.f3_source_response_diversity.v1` quantized response, with within-rate invariance and bounded cross-rate drift; event indices, anatomy lengths, sample rate, channel count, filenames, and source metadata cannot earn diversity. Numeric passports are reject-only experimental, numerical-stability, safety, and review-cost values, never perceptual hardness thresholds.
Why: the earlier H27-H31 work proved that louder, darker, dirtier, pitch-shifted, or metrically different output can pass scalar diagnostics while sounding unchanged or weaker. A source-blind freeze prevents the next detector, topology, thresholds, and controls from adapting to the same examples after results are known. Separating provenance from actionable diversity also prevents timing/anatomy metadata from masquerading as musical source response.
Evidence: the raw SHA-256 pins are protocol `35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840`, matrix `aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6`, and registry `af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812`; their canonical semantic hashes are respectively `7681ab68a9fe2261c97b7499e298e9e6dcb7cbe60df64355dceff577d7fc0848`, `57edb217b8dd17166826274d96ef091bd6fd2a88a9688e37b3ef0b7a6d27e94b`, and `6cfe11cd10a5947427a09335fbd4795706c71530b6f6a7e5b9883259bcca8ce1`. Canonical validation, 55 fail-closed Stage-A mutation fixtures, strict duplicate-key/non-finite JSON rejection, source-holdout registry fixtures, F1/F2 tests, and the 17-case F3-v2 synthetic preflight pass. F3-v2 repeats are deterministic at 44.1, 48, and 96 kHz; its source-response goldens are `[2,9,3,8,4,8,2,4]` at 44.1 kHz and `[2,9,3,7,4,8,2,3]` at 48/96 kHz, with the declared rate-local hashes. No holdout audio, fresh development qualification, event catalog, candidate render, or human force verdict exists yet.
Consequences: the next execution must start a fresh development-only access log, verify exact registered identities before opening selected files, compute the frozen detector/anatomy/source-contrast catalog, and either stop fail-closed or render the complete 3-family by 4-source by 2-event matrix. Mechanical survival can request only a bounded blinded human comparison; it cannot award `percussive_hard`. A unique human directional pass is required before any Stage-B RuntimeMix or TUI promotion. Any protocol equation, threshold, topology, partition, or stop-rule change now requires a version bump, a new decision, and recomputation; source- or filename-specific retuning remains forbidden.
Status: accepted

---

### RBX-253

Date: 2026-08-10
Topic: accept the frozen Stage-A development qualification rejection before candidate rendering
Phase: P023 / Sound Excellence
Question: what follows when the first RBX-252 `StageAQualificationSession` admits only two of the four frozen development sources and therefore cannot construct the required source-contrast catalog?
Decision: accept the typed `positive_source_failed` result and stop the current Stage-A execution. Do not run the 3-family by 4-source by 2-event matrix, render candidate audio, request human playback, impute missing source features, substitute fallback events, or retune the frozen detector/anatomy to the observed files. Preserve the RBX-252 protocol, matrix, and registry byte-for-byte as the record of what was tested. Treat the outcome as a mechanical source/event-admission rejection only, never as a perceptual judgment about hardness or source quality.
Why: `oga_marwan_cinematic_percussion` produced no frozen detector peak, while `oga_william_hector_horde_war_drums` produced one peak for which v1 emitted `edge_only_impulse`. That code proves only that no physical-onset candidate passed the frozen baseline, peak, signal-floor, and persistence gates; it does not prove an acoustically edge-only or bodyless impulse. Both sources consequently had zero event-level onsets, resolved bodies, and frozen events and failed `insufficient_eligible_events` plus `source_feature_requirements_unmet`. `oga_cinameng_can_be_so_beautiful` and `oga_frosty_ham_osdrums` each qualified three frozen events, but two admitted sources cannot satisfy the frozen four-source contrast and partition contract without forbidden imputation.
Evidence: session `2f1e5ba2-ca1e-42b4-b0e1-3faa7591dde9` used implementation snapshot `8285107a8c396c7fcbfd52cbe24e3dc8c3a108c56a57487cdb178977a5b2de94` and the unchanged RBX-252 raw pins. Git commit `c60cbb392491950fdbb2edaf15a9f8926db51c71` permanently preserves every implementation path/byte pair covered by that aggregate; later hardening is not part of the executed snapshot. The exact-case v3 access log completed four in-process deliveries in canonical order with no source-directory discovery, holdout-audio access, or commercial-reference access. The committed local evidence hashes are session `0e5cf6e764330360b22752271f6b3b1e0623c280a230d07b479925346dbc84c8`, access log `edb3ff494335221fb586890d2094dda38c6f53306b6a9aa4d9a01998332f53e1`, qualification rejection `67fff3c8b50dd17f783992d21bacde909555a99d7d90f18830c5c857eb7daa85`, and local artifact commit marker `6ba958ba742666513352335c32039e761797498fc0002feff344c01ebc5ad6d1`. The Python runner, not the separate Rust binding scaffold, owned this source path. The detailed result and temporary local-evidence boundary are recorded in `docs/reviews/riotbox_1428_stage_a_development_qualification_rejection_2026-08-10.md`.
Consequences: Stage A has no event catalog, candidate matrix, survivor, listening request, human verdict, hardness proof, or product progress. F1, F2, and F3 were never rendered on a development event. Any retry must start from a new versioned preregistration and a new decision under RBX-252 change control before source recomputation; this rejected session cannot be reclassified or reused as qualification. The next decision must explicitly choose and pre-freeze any legal corpus expansion or prequalification revision instead of adapting the current versions to these results.
Status: accepted

---

### RBX-254

Date: 2026-08-10
Topic: constrain the historical Stage-A v1 evidence and require an honest v2 retry boundary
Phase: P023 / Sound Excellence
Question: what must change after the post-execution Ultra review found an over-specific v1 refusal code, a shallow in-memory protocol object, an unexecuted Rust binding scaffold, and an ambiguous F3 causality claim?
Decision: preserve Protocol v1, Matrix v2, Registry v2, session `2f1e5ba2-ca1e-42b4-b0e1-3faa7591dde9`, and their rejection byte-for-byte as historical evidence only. Harden the Python protocol object recursively, close the v1 runner with typed refusal before every validation/preflight/access boundary, and withdraw the unreachable Rust binding surface without recomputing any source result. Do not change v1's serialized `edge_only_impulse` code; constrain its interpretation to the measured gate failure. Require the next source execution to freeze Protocol v2, Prequalification v3, Impact Role v2, Event Anatomy v2, v2 source-analysis/unbound-qualification/bound-catalog-or-rejection schemas, Matrix v3, and any Registry v3 corpus expansion before access. Keep Detector v1 unchanged unless a source-blind new decision changes its equations or thresholds.
Why: the v1 analyzer assigned `edge_only_impulse` whenever no candidate jointly passed baseline, peak, signal-floor, and persistence checks, so the name claims anatomy the algorithm did not establish. The executed `FrozenStageAProtocol` validated raw bytes but exposed a mutable nested document and forgeable constructor. The Rust Gate/Bind types were synthetic scaffolding and were not called by the Python qualification. F3's envelopes and controllers are causal only conditional on offline source-frozen whole-source means, anatomy, and masks; the old `strict_causality_pass` checked activation timing, not end-to-end streaming causality.
Evidence: Git commit `c60cbb392491950fdbb2edaf15a9f8926db51c71` reconstructs the executed implementation aggregate `8285107a8c396c7fcbfd52cbe24e3dc8c3a108c56a57487cdb178977a5b2de94`. The subsequent source-blind tests forge the expected protocol SHA with altered bytes, attempt nested mutation and method shadowing, prove the closed v1 runner reaches no validation/preflight/access callback, distinguish fixed-mean prefix causality from whole-source reconditioning, and keep the Rust PCM scaffold private and immutable. No source audio, holdout audio, candidate render, or playback is used by those corrections.
Consequences: RBX-253's overall `positive_source_failed` stop remains valid, but its specific v1 reason code is not acoustic classification evidence. Later hardening has a different implementation snapshot and cannot inherit the old session. Protocol v2 must name F3's scope `conditional_on_source_frozen_offline_state` and bind the analyzer's exact frozen DC means before any future source-backed candidate F3 render. F3-v2 may remain only if that binding is bit-identical and no equation, sample, controller hash, diversity identity, or output changes; otherwise bump F3 and its affected identities. The new corpus must be frozen as one complete batch before recomputation, with no sequential survivor selection or detector retuning.
Status: accepted

---

### RBX-255

Date: 2026-08-10
Topic: replace the consumed Stage-A-v2 acquisition Batch v1 with one complete metadata-only Batch v2
Phase: P023 / Sound Excellence
Question: how may RIOTBOX-1430 perform one further development-only acquisition after Batch v1 rejected during the second registered file's frozen header gate?
Decision: preserve Protocol P2 byte-for-byte with raw/semantic SHA-256 `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` / `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`; no detector, anatomy, source-contrast, event ordinal, threshold, numeric passport, or source-processing algorithm changes. Preserve acquisition Batch v1 and its consumed attempt as immutable rejection evidence. Authorize a new `riotbox.percussive_force_stage_a_v2_acquisition_batch.v2` only after it freezes this complete ordered metadata-only replacement: (1) provisional `dense_break`, farfadet46, OpenGameArt page `https://opengameart.org/content/loopable-beat-for-ludumdare-game`, attachment `test.wav`, direct URL text `https://opengameart.org/sites/default/files/test.wav`, `1695934` bytes, attachment ID `45609`; (2) provisional `sparse_drums`, celestialghost8, page `https://opengameart.org/content/cc0-scraps`, attachment `slowdrum - Track 02 (New song).wav`, direct URL text `https://opengameart.org/sites/default/files/slowdrum%20-%20Track%2002%20%28New%20song%29.wav`, `580400` bytes, attachment ID `93984`; (3) provisional `electronic_drums`, cosmac, page `https://opengameart.org/content/8-bit-disco-loop`, attachment `title.wav`, direct URL text `https://opengameart.org/sites/default/files/title_1.wav`, `676908` bytes, attachment ID `239763`. All three named content pages declare CC0, identify distinct new OGA authors, expose WAV metadata, and do not disclose a third-party source or sample pack. These family assignments are metadata hypotheses only and grant no source, event, hardness, or musical fitness. Batch v2 gets exactly one ordered three-GET attempt, no HEAD/probe/retry/redirect/substitution/survivor/fallback, separate v2 log/quarantine/final/manifest paths, fresh raw/semantic and implementation pins, complete no-network fixtures, independent review, and an immutable pre-GET commit. Any failure rejects the entire v2 batch and requires another new versioned decision; there is no Batch-v3 authorization here.
Why: Batch v1 was consumed fail-closed at `request_2_header` after two requests and one successful header gate. Keeping its first source would be result-driven sequential survivor selection. The complete replacement was selected from named HTML content pages and author links before any Batch-v2 attachment request. Candidates with disclosed third-party soundfonts, ambiguous anonymous/external author identity, conflicting licensing, non-WAV payloads, or insufficient attachment identity were excluded rather than weakening policy. The selected byte counts reduce duration risk but do not predict RIFF/WAVE format or acoustic fitness; only the unchanged frozen gates may decide those facts.
Evidence: `docs/reviews/riotbox_1430_stage_a_v2_acquisition_batch_v1_rejection_2026-08-10.md` is pinned at raw SHA-256 `2ab9d34888d5ba0a442a408f2e10e9f201fdbfa6291ffdd09d003908c93da619`. It binds attempt `1e6a1070-5df4-4813-8e07-dc30fae7c70a`, v1 access-log SHA-256 `703806c0f6548f1af2e2f51408553e51f060ea51f4f47202079d63145540c174`, request counts `2/1`, and observed v1 payload hashes `00d1ec0b442db60ade056fe24a72c18cc0f8deed23301f5ec961029f3eb810f9` and `2212c182906ae1b7449e26c31b4c96f132c348a33fdd82c0b00f785f7a677e5f`. Batch v2 must bind and forbid every Batch-v1 identity and both observed payload hashes. Metadata access was limited to the named OpenGameArt HTML pages and author links; no Batch-v2 WAV, source directory, holdout audio, commercial reference, candidate render, or playback was opened.
Consequences: Batch v1 and its existing recipes remain historical and must never be retried or redirected. Batch v2 must use a parallel versioned contract/runner/validator/artifact stack and disjoint exact paths. Registry v3, Matrix v3, source qualification, the three-family candidate matrix, rendering, and listening remain blocked until the complete v2 acquisition validates and is atomically published. The algorithms and thresholds may not be tuned after source results; any needed change requires a new version and Decision-Log entry.
Status: accepted

---

### RBX-256

Date: 2026-08-10
Topic: accept the consumed Stage-A-v2 acquisition Batch-v2 format rejection
Phase: P023 / Sound Excellence
Question: what follows when the single RBX-255 Batch-v2 attempt completes all three registered GETs but the third body fails the unchanged header-only PCM sample-rate gate?
Decision: accept Batch v2 as `rejected_fail_closed_no_publication` and preserve Protocol P2, Batch v1, Batch v2, both attempts, and every observed payload identity as immutable rejection evidence. Do not publish either header-valid Batch-v2 entry, reuse it as a survivor, substitute another URL, retry any request, weaken or retune the sample-rate gate, create Registry v3 or Matrix v3, compute source/event features, render a candidate, or request playback. RBX-256 records the terminal rejection only and does not authorize acquisition Batch v3.
Why: the runner consumed exactly three preregistered GETs in ordinal order. Entries one and two passed the frozen RIFF/WAVE PCM header contract at 44.1 kHz; entry three was exact-length but its PCM sample rate lay outside the frozen inclusive range, so the entire non-sequential batch correctly rejected at `request_3_header`. Retaining the first two would turn a complete-batch preregistration into forbidden result-driven survivor selection. A format failure conveys no family, event, hardness, force, musical, or human-listening verdict.
Evidence: `docs/reviews/riotbox_1430_stage_a_v2_acquisition_batch_v2_rejection_2026-08-10.md` is pinned at raw SHA-256 `e842b1f303120887423e168322dd2dcd9e64925dc679a624c0a63cda9ab8ac9c`. It binds attempt `58913eec-8ded-4187-b1e8-e6de07d04f6f`, access-log raw SHA-256 `ac4faa5cdb51f03d6e41ec5e41caa57643b67370d94177b0ef4a1e84e55fe83c`, pre-GET commit `5134399e6dbe310d32d976c780dd03d8f0b30cf8`, implementation aggregate `690318c53fc5bb43254ce8cfdc99fec00d85a3613317d242f49b1cb6806992f1`, request counts `3/2`, and observed Batch-v2 payload hashes `fd5c5277301069fb9d9fe53706e8eed3deb3ce90cd554c405b26c3e223adc2da`, `a507d3d8c55e2cf8bb93d9adcc49caa3e34c122f9484e636cd60271f9d1f7e6c`, and `494877eb8e6677acc7c9bf7e2dc0c8e857e5a60291f17d9f6ca972bb8bb823d3`. The deterministic rejected access log validates against the frozen v2 contract and current implementation; `.next`, quarantine, final batch, and publication-probe paths are absent. No audio decode, PCM sample iteration, source feature/event computation, candidate render, playback, holdout access, or commercial-reference access occurred.
Consequences: RIOTBOX-1428 development qualification and its 3-family by 4-source by 2-event matrix remain blocked. Any further acquisition must start with a separate targeted decision that freezes a complete three-entry metadata-only Batch v3, forbids every Batch-v1 and Batch-v2 identity and observed payload hash, uses disjoint versioned paths and a parallel fail-closed stack, and preserves P2 byte-for-byte. Until then there is no legal GET, Registry v3, Matrix v3, candidate, or human listening artifact.
Status: accepted

---

### RBX-257

Date: 2026-08-11
Topic: authorize one bounded Freesound metadata resolution before freezing Stage-A-v2 acquisition Batch v3
Phase: P023 / Sound Excellence
Question: how may RIOTBOX-1430 resolve exact original-file metadata for a complete professional-promise Freesound replacement batch without accessing audio, adapting P2 to source results, or treating a disclosed sample-pack origin as an automatic rights conflict?
Decision: preserve Protocol P2 byte-for-byte at raw/semantic SHA-256 `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` / `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`; preserve Registry v2, acquisition Batches v1/v2, both consumed attempts, both rejection reports, and all five observed predecessor payload hashes as immutable rejection evidence. Before any source-file request, authorize exactly one ordered metadata-only Freesound API pass over these three already selected public sound identities: (1) provisional `dense_break`, uploader `djericmark`, sound `724939`, page `https://freesound.org/people/djericmark/sounds/724939/`, internal pack `40482` / `DRUM LOOPS`; (2) provisional `sparse_drums`, uploader `Cyclez`, sound `493560`, page `https://freesound.org/people/Cyclez/sounds/493560/`, disclosed external pack page `https://www.junodownload.com/products/schranz-samples-from-hard-techno-to-schranz/4346197-02/`; (3) provisional `electronic_drums`, uploader `justabeat`, sound `458897`, page `https://freesound.org/people/justabeat/sounds/458897/`, internal pack `25877` / `PO32 Drum Machine Loop`. Request only the exact Sound Instance endpoints `/apiv2/sounds/724939/`, `/apiv2/sounds/493560/`, and `/apiv2/sounds/458897/` in that order with one identical fixed `fields` projection containing only `id,url,name,created,license,type,channels,filesize,bitdepth,duration,samplerate,username,md5,is_remix,pack,download`. Permit at most one status-200 JSON response of at most 65536 bytes per identity, no redirect, retry, search, preview field or endpoint, descriptor, similarity result, download endpoint, attachment body, range request, audio byte, fallback, substitution, or survivor. The dedicated account password and long-lived API client credentials may reside only in process memory or the operating-system credential store; OAuth authorization codes, access tokens, refresh tokens, and web cookies remain process-memory-only. No credential or authorization value may enter argv, environment, repository files, ignored files, logs, exceptions, stdout, or stderr. The resulting exact metadata snapshot may bind Batch v3 but grants no source, family, event, hardness, force, musical, or human-review fitness and does not itself authorize an original-file GET.

Replace RBX-255's provider-specific blanket exclusion of any disclosed sample pack with this v3-wide provenance rule before source access: an exact provider sound page that explicitly declares CC0 remains admissible for local ignored development use when uploader, page, sound identity, pack identity or external pack URL, and absence of a conflicting license or an explicit unauthorized third-party-rights statement are all recorded. A disclosed internal or external sample-pack origin is evidence to preserve, not an automatic pass or rejection. This rule applies uniformly to every Freesound candidate and is not a Cyclez-only exception. It does not prove chain of title, author ownership, acoustic quality, or product fitness. Raw sources and derived QA/listening artifacts remain local and uncommitted; neither source audio nor the Freesound API becomes Riotbox product content or a product runtime dependency.
Why: the complete three-family replacement was chosen from public HTML metadata before any original-file request, decode, header result, feature, event, render, or playback. Freesound publicly exposes CC0, uploader, page, family-relevant description, nominal WAV format, and an authenticated original-download route, but rounds file size and requires API metadata to resolve the exact byte count and MD5 needed by the unchanged pre-GET boundary. Treating every disclosed sample pack as equivalent to conflicting or unauthorized third-party material is too broad for a non-redistributed local development corpus; commercial availability does not by itself conflict with an explicit CC0 grant for the named file. Keeping conflict and explicit unauthorized-source checks preserves the actual rights-risk boundary without weakening the acoustic experiment. The change is recorded after public provenance metadata but before all source bytes and applies generally, so it cannot tune detector, anatomy, source-contrast, renderer, threshold, or numeric-passport behavior to an acoustic result.
Evidence: the three named public Freesound pages declare Creative Commons 0 and expose distinct uploader, page, nominal WAV, duration, rate, width, channel, and login-gated download-link metadata. `724939` identifies a 133-BPM breakbeat played on a Spaun kit; `493560` identifies a hard-techno Schranz kick loop and discloses the external pack link; `458897` identifies a PO-32 drum-machine loop. Their uploader, profile, sound/page, pack, and download-link identities are case-insensitively disjoint from Registry v2 and acquisition Batches v1/v2. The user explicitly authorized acceptance of the Freesound API terms for the dedicated development account. One API credential named `Riotbox Stage A Development Acquisition` was created with scope described as internal development-only acquisition, no product integration, and no source redistribution. The account password and API client values were transferred without terminal echo into three Riotbox-scoped operating-system credential-store entries; none was printed, written to the repository, or placed in an environment variable. No OAuth code or token was created. No API sound metadata, original file, source directory, source/holdout audio, commercial reference audio, candidate render, or playback was accessed while making this decision.
Consequences: RBX-257 authorizes only the three exact metadata responses needed to freeze a complete Batch v3. After a successful metadata snapshot, a separate targeted decision must pin every exact API identity, original filename, byte count, MD5, request order, OAuth download boundary, destination, raw/semantic contract hash, parallel validator/artifact/runner stack, no-network fixtures, independent review, and immutable pre-download commit before any original-file request. Any metadata-pass failure stops without audio access and requires a new versioned metadata decision. P2 algorithms, detector/anatomy/source-contrast catalogs, event ordinals, thresholds, numeric passports, and source-processing equations remain unchanged.
Status: accepted

---

### RBX-258

Date: 2026-08-11
Topic: accept the consumed Freesound metadata-v1 rejection and authorize one durable metadata-v2 pass
Phase: P023 / Sound Excellence
Question: how may RIOTBOX-1430 resolve the same preregistered Freesound metadata after RBX-257 stopped on a provider license-representation mismatch without retrying the consumed pass, adapting any acoustic contract, or losing another request's audit evidence?
Decision: accept the RBX-257 metadata-v1 pass as `rejected_fail_closed_no_audio_access` at repository HEAD `ffdbb676f74d4e52ad6eccb702b2768585ac512c`: exactly request 1 for sound `724939` started, one bounded status-200 JSON object was observed, zero entries were accepted, and the pass stopped at `request_1_license_representation` because `license` was not byte-equal to `Creative Commons 0`. The actual returned scalar, raw body, byte count, SHA-256, attempt UUID, and access-log SHA-256 were not durably captured and must not be reconstructed. Requests for `493560` and `458897` did not occur. Preserve Protocol P2 byte-for-byte at raw/semantic SHA-256 `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` / `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`, Registry v2, Matrix v2, Batches v1/v2, both source-acquisition rejections, all five predecessor payload hashes, RBX-257, and its rejected metadata pass as immutable history.

Authorize one new versioned metadata-v2 attempt over the same complete ordered identities `724939`, `493560`, and `458897`, with the exact RBX-257 Sound Instance endpoints and fixed projection `id,url,name,created,license,type,channels,filesize,bitdepth,duration,samplerate,username,md5,is_remix,pack,download`. This is a new attempt, not a retry, resume, survivor pass, or substitution. Before network access, freeze and validate a versioned JSON contract, parallel validator/runner/fixture stack, exact output paths, implementation aggregate, and immutable Gate commit. Before DNS or socket activity the runner must create a unique access log using fsynced write plus no-replace atomic publication and fsync the parent. It must pin the decision, version, exact HEAD, contract raw/semantic hashes, implementation aggregate, attempt UUID, ordered requests, fixed fields, limits, `prepared` state, and zero counts. Before every possible request it must revalidate those bindings and durably transition that entry to `request_intent` with outcome unknown; a crash after intent consumes and rejects the attempt and may never be resumed.

Freeze this provider-wide CC0 representation set before the new response: exact JSON strings `Creative Commons 0`, `http://creativecommons.org/publicdomain/zero/1.0/`, and `https://creativecommons.org/publicdomain/zero/1.0/`. Map each to canonical `CC0-1.0` while retaining the exact accepted raw string. No trimming, case folding, redirect following, substring recognition, URI rewriting, or general Creative Commons matching is permitted. Reject null, non-string, whitespace/case variants, other licenses, `/legalcode`, missing or extra suffixes, and any other value. The inspected official Freesound source commit returns `obj.license.deed_url`; its CC0 fixture uses the HTTP URI, while the official API documentation still names `Creative Commons 0`. The official Creative Commons HTTPS deed identifies CC0 1.0 Universal. The two exact URI strings and the exact prose label therefore map to the same canonical `CC0-1.0` license identity; this does not claim which value the consumed response contained or which Freesound source commit was deployed. This is metadata-schema normalization after a representation mismatch, disclosed before metadata-v2 access; it uses no audio, header, feature, event, render, or listening result.

Permit at most three ordered GETs, one per identity, to origin `https://freesound.org` on port `443`, with exact paths `/apiv2/sounds/724939/`, `/apiv2/sounds/493560/`, and `/apiv2/sounds/458897/`. Each exact request target appends only `?fields=` plus the frozen field projection in order, with every comma encoded as uppercase `%2C` and no other query parameter. Pin a `30.0`-second timeout and these request headers only: `Host: freesound.org`, `Accept: application/json`, `Accept-Encoding: identity`, `Connection: close`, a fixed non-secret Riotbox metadata-v2 user agent, and `Authorization: Token <credential>`. Obtain that credential only through `/usr/bin/secret-tool` from service `riotbox-freesound`, credential `api-client-secret`, application `stage-a-development-acquisition`; keep its value in process memory and never put it in the query, argv, environment, files, logs, exceptions, stdout, or stderr. Forbid proxies, sent cookies, HEAD/probes, redirects, retries, search, preview fields or endpoints, descriptors, similarity, download endpoints, range requests, audio bytes, fallback, substitution, and survivors.

Accept one status-200 JSON body of at most `65536` bytes per request. Require response media type `application/json`, allowing only an optional UTF-8 charset parameter; require `Content-Encoding` absent or exact identity after HTTP token normalization; allow `Transfer-Encoding` only when absent or exact chunked and never together with `Content-Length`; when `Content-Length` is present require decimal bytes at or below the cap and exact agreement with the received body. Persist only normalized values for `Content-Type`, `Content-Length`, `Content-Encoding`, and `Transfer-Encoding`, plus booleans for `Location` and `Set-Cookie` presence; reject either presence and never persist their values. After receiving a complete bounded body, durably record the ordinal, sound ID, status, those allowlisted facts, exact byte count, and raw SHA-256 before parsing or semantic validation. On a license mismatch, record the JSON type and, only for a string of at most `256` UTF-8 bytes whose code points are all printable and contain no CR/LF, its canonical JSON-escaped value, UTF-8 byte count, and SHA-256; otherwise record type, byte count when available, and SHA-256 only. Never persist arbitrary response dumps or request/authorization headers. The metadata-v2 JSON contract must freeze the exact request-target strings, user-agent value, header rules, limits, evidence allowlist, and credential-store identity before the immutable Gate commit.

Keep accepted bodies quarantined until all three exact entries validate. Only an all-pass attempt may atomically publish one metadata snapshot through a no-replace rename. Any failure seals the access log `rejected`, stops before later requests, forbids partial survivors/publication, and requires another targeted versioned decision. A successful snapshot authorizes only a later decision that freezes Batch v3's exact metadata, OAuth original-file download boundary, local ignored destination, validators, no-network fixtures, independent review, and immutable pre-download commit. RBX-258 does not authorize an original-file request, source decode, feature/event calculation, candidate render, playback, holdout access, or commercial-reference access.
Why: RBX-257 correctly stopped under its literal contract, but its helper compared the provider response only with prose documentation and did not durably log intent or safe response evidence before validation. Official source-blind provider evidence at Freesound commit `4318dcdce8dbf5663658e0d9287401bb0ff5e140` provides a plausible schema explanation: the inspected Sound Instance serializer returns the stored license deed URL, and the inspected official CC0 fixture binds `Creative Commons 0` to `http://creativecommons.org/publicdomain/zero/1.0/`. It does not prove the deployed commit or reconstruct the lost response. Correcting this closed provider representation boundary does not change source choice, format acceptance, P2 numerics, detector/anatomy/source-contrast behavior, event ordinals, or musical acceptance. Durable intent-before-request and evidence-before-validation prevent another ambiguous consumed pass.
Evidence: `docs/reviews/riotbox_1430_stage_a_v2_freesound_metadata_v1_rejection_2026-08-11.md` records the exact durable facts and explicit evidence gaps. The official Sound Instance documentation is `https://freesound.org/docs/api/resources_apiv2.html#sound-instance`. At the inspected official Freesound commit, `apiv2/serializers.py` raw SHA-256 `760280657c7fbe0958d6e51328ca78bb8b545727ea6988fa5b8c54fb73d43360` implements `return obj.license.deed_url`; `sounds/fixtures/licenses.json` raw SHA-256 `f2e47f403c3a87b9222c834b4ea2e6968fa8b95e63286cbfc71b9901eec2502c` records the CC0 HTTP deed URL. The official Creative Commons deed is `https://creativecommons.org/publicdomain/zero/1.0/`. No second sound-metadata request, original-file request, preview, audio byte, local source directory, decode, PCM iteration, feature/event computation, render, playback, holdout, or commercial reference was accessed while making this decision.
Consequences: metadata-v1 remains terminal rejection history. Metadata-v2 must use disjoint exact paths and a parallel fail-closed implementation with no-network fixtures and independent review. Success may provide metadata for Batch v3 but cannot itself admit a source or make any family, format-header, event, hardness, force, musical, or human-review claim. All P2 algorithms, catalogs, thresholds, and numeric passports remain frozen.
Status: accepted

---

### RBX-259

Date: 2026-08-11
Topic: replace the unexecuted metadata-v2 framework with one pragmatic Stage-A development acquisition
Phase: P023 / Sound Excellence
Question: how does RIOTBOX-1430 reach real source qualification without turning three fixed Freesound files into a permanent acquisition subsystem?
Decision: RBX-258 remains historical authorization but its proposed validator/runner/fixture stack is abandoned before its Gate and before any metadata-v2 network, credential, or audio access. Authorize exactly three ordered metadata GETs for the already frozen sound IDs from `docs/benchmarks/percussive_force_stage_a_v2_freesound_metadata_allowlist_v1.json`. Use the existing OS-keyring API credential, accept only the frozen exact CC0 representations, retain the full preregistered pack/conflict provenance, and persist only the bounded exact metadata responses and hashes. Do not search, preview, substitute, retry, request an original file, discover source directories, touch holdouts or commercial-reference audio, change P2, or add permanent downloader code. If and only if all three metadata records match, mechanically create and commit one complete Batch-v3 document using P2's existing `riotbox.percussive_force_stage_a_v2_acquisition_batch.v1` and `riotbox.percussive_force_stage_a_v2_acquisition_access_log.v1` contracts, with exactly one authoritative HTTPS download URL and exact attachment byte count per entry. Only that later immutable Batch-v3 commit may authorize the three original-file GETs; no redirect is allowed.
Why: the earlier implementation direction had grown to roughly 3,400 uncommitted Python lines while producing no source, qualification, matrix, or audible product evidence. Exact allowlisting, keyring authentication, bounded transfer, hash capture, and reuse of the existing strict WAV-header inspector provide the actual safety boundary. Crash-transaction frameworks, parallel validators, and adversarial same-user file-race machinery do not improve Riotbox's sound or the validity of this one local development acquisition.
Consequences: finish the acquisition, Registry-v3/Matrix-v3 freeze, fresh Protocol-v2 qualification, and—only after four-source admission—the frozen 3-family × 4-source × 2-event matrix within RIOTBOX-1430. Operational source acquisition is not a product subsystem and must not displace its directly enabled audible follow-up.
Status: accepted

---

### RBX-260

Date: 2026-08-11
Topic: replace the consumed Freesound OAuth acquisition attempt with one session-authenticated Batch v4
Phase: P023 / Sound Excellence
Question: what follows when the first Batch-v3 API download GET returns a non-200 response before any body byte is read?
Decision: accept Batch v3 as `rejected_fail_closed_no_publication`. Attempt `35987198-8410-4d08-92f6-3f605e6a210c` performed five OAuth control-plane requests, then exactly one original-file GET for sound `724939`; the response failed the status-200 gate before body streaming, requests 2 and 3 did not occur, and no quarantine or final directory survived. The temporary runner failed to persist the actual non-200 scalar, so it must not be reconstructed. Authorize one complete Batch-v4 attempt over the same three ordered IDs, API download URLs, byte counts, metadata MD5 bindings, Development destinations, and frozen P2 header contract. Change only authentication from an in-memory OAuth2 bearer to an in-memory Freesound web session cookie. Before Batch-v4 source access, require a source-free session proof: login redirects once and the exact `/apiv2/me/` endpoint returns 200 without redirect; do not read or persist its body. Then permit at most the same three ordered no-redirect original GETs. Persist each response status before applying the status gate. Do not add a permanent downloader, change P2, substitute a source or URL, touch holdout/commercial-reference audio, decode or play audio, compute source features, or render a candidate during acquisition.
Why: the OAuth attempt produced no source/audio evidence and therefore gives no basis for changing source selection, format rules, algorithms, or thresholds. A separate source-free check proved that the dedicated account's in-memory web session is accepted by Freesound's session-authenticated API path: login returned 302 and `/apiv2/me/` returned 200 with no redirect while its body remained unread. Official Freesound code also lists SessionAuthentication for the API download view. Reusing the exact already-pinned API URLs with working session authentication is the narrow correction; another framework or ticket would not improve product evidence.
Evidence: the ignored v3 access log raw SHA-256 is `2e7068d86cf0391f6f316b92830ab88e0fb4420a35f705513625de6aaf1fa7f9`; it records counts `1/0`, OAuth control count `5`, no publication, and every forbidden scope assertion false at repository HEAD `f5cb3843e911b1ed3080fcb7989a8095b7ead980`. `docs/reviews/riotbox_1430_stage_a_v2_acquisition_batch_v3_rejection_2026-08-11.md` preserves the bounded rejection facts. No response body, source directory, decode, PCM iteration, feature/event computation, render, playback, holdout audio, or commercial reference was accessed.
Consequences: Batch v3 is consumed and must not be retried. Batch v4 remains the same RIOTBOX-1430 operational acquisition, not a new Linear ticket or product subsystem. Success still grants only exact identity/hash/header evidence and must proceed directly to Registry-v3/Matrix-v3 and the fresh Protocol-v2 qualification.
Status: accepted

---

### RBX-261

Date: 2026-08-11
Topic: remove the incorrect media-type Accept constraint from the consumed Batch-v4 download attempt
Phase: P023 / Sound Excellence
Question: what follows when the first session-authenticated API download GET returns HTTP 406 before any body byte is read?
Decision: accept Batch v4 as `rejected_fail_closed_no_publication`. Attempt `e2aaf002-466a-4a9f-a7be-400226743528` passed its source-free session proof, then performed exactly one original-file GET for sound `724939`; Freesound returned status `406`, `Content-Type: application/json`, `Content-Length: 57`, and no redirect. The response body remained unread, requests 2 and 3 did not occur, and no quarantine or final directory survived. Authorize one complete Batch-v5 attempt with the same session authentication, three ordered IDs, API URLs, exact byte counts, metadata MD5 bindings, Development destinations, and frozen P2 header contract. The only request change is to send `Accept: */*` instead of the runner-invented `Accept: application/octet-stream`; persist that exact value in Batch v5 and retain all response/body/no-redirect gates.
Why: Freesound's API download view still passes through Django REST Framework content negotiation. The inspected official configuration registers JSON, browsable API, YAML, JSONP, and XML renderers, so the request-only `application/octet-stream` Accept value has no negotiable renderer and causes 406 before the file-returning view runs. Freesound's own download example supplies Authorization only and leaves curl's default `*/*` Accept behavior intact. This is a source-independent HTTP correction, not a source, format, algorithm, threshold, or musical change.
Evidence: the ignored v4 access log raw SHA-256 is `588ad3a3bc775536d37682e471b85f5415595ee1e3ee62864e7166b6a269c30c`; it records control-plane count `3`, original request counts `1/0`, status `406`, JSON media type, 57 declared bytes, no redirect, no publication, and every forbidden scope assertion false. `docs/reviews/riotbox_1430_stage_a_v2_acquisition_batch_v4_rejection_2026-08-11.md` preserves the rejection. No response body, source directory, decode, PCM iteration, feature/event computation, render, playback, holdout audio, or commercial reference was accessed.
Consequences: Batch v4 is consumed and must not be retried. Batch v5 remains the same RIOTBOX-1430 operational acquisition and must proceed directly to source qualification if its exact identity/hash/header gate passes. No new Linear ticket or permanent acquisition code is authorized.
Status: accepted

---

### RBX-262

Date: 2026-08-11
Topic: freeze the Protocol-v2 development registry and 3 x 4 x 2 matrix before the single qualification session
Phase: P023 / Sound Excellence
Question: which exact source-blind contracts authorize the first and only fresh Stage-A-v2 Development qualification after Batch v5 completed?
Decision: freeze `docs/benchmarks/source_holdout_rotation_v3.json` at raw SHA-256 `9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502` and semantic SHA-256 `6896a7a8a90e8534ae0c1390c73c22c2c717a89cedbde5a333e9eda767cc1e6a`; freeze `docs/benchmarks/percussive_force_development_matrix_v3.json` at raw SHA-256 `0dff59b8d871f75eccd75a5df1ff8080c777f4b76b3559957ce415762b16aa5e` and semantic SHA-256 `dedf34c0f61d61d875c700fec155dbcda367f7e89fff86a167367119dacd5bb8`. The exact positive set is `oga_cinameng_can_be_so_beautiful`, `freesound_djericmark_724939`, `freesound_cyclez_493560`, and `freesound_justabeat_458897` in that order. Authorize one fresh `StageAQualificationSession` to open only those exact registered Development paths through no-follow bounded access and compute the unchanged Protocol-v2 detector, anatomy, source features, source contrast, and partition gate. Matrix execution remains forbidden unless that qualification passes.
Why: Batch v5 completed all three original-file downloads and strict RIFF/PCM header checks, so the source-blind v3 contracts can now bind the four-source, four-author, three-family hypothesis without changing any P2 algorithm, equation, threshold, topology, role, ordinal, or stop rule. Registry v3 preserves all 18 Registry-v2 entries and all nine active holdout tuples byte-semantically while adding only the three consumed Batch-v5 Development identities. Matrix v3 preserves the frozen F1/F2/F3 versions, two event ordinals, 24 conditions, controls, refusal semantics, and holdout prohibitions.
Evidence: the Batch-v5 sealed manifest raw SHA-256 is `4c7fd063ac84609b48f6c99eb24f1f4b3910cee2f644b4e5c8d7ff6d5eb88d55`; its ignored access log raw SHA-256 is `d830789d394fa30ec78040a15061763009914b5cc5d5bb5a2696ad65572c96c2`. Registry-v3 and Matrix-v3 duplicate-key, predecessor-pin, identity, format, cardinality, cross-product, and holdout-tuple checks passed source-blind; an independent Matrix-v3 review reported no findings. Protocol-v2 validation, all 51 mutation fixtures, shared v1/v2 analysis fixtures, qualification-owner fixtures, and development-access fixtures passed before any qualification WAV open.
Consequences: Registry v3 and Matrix v3 are immutable inputs to this execution. Source results may only pass or reject them; they may not tune or rewrite P2. Any contract change requires Protocol v3, the relevant component-version bump, and a new decision before recomputation. On qualification rejection, stop without matrix render or listening. On pass, execute exactly the frozen 3-family x 4-source x 2-event matrix before any human playback. Holdout audio, commercial references, source-directory discovery, candidate rendering during qualification, and fallback audio remain forbidden.
Status: accepted

---

### RBX-263

Date: 2026-08-11
Topic: accept the single Stage-A-v2 qualification rejection and close the frozen Matrix-v3 execution path
Phase: P023 / Sound Excellence
Question: what is the terminal consequence of the first Protocol-v2 Development qualification result?
Decision: accept qualification session `7ecb935c-de82-40c9-8045-05f82293014f` as `rejected_fail_closed`. `oga_cinameng_can_be_so_beautiful` and `freesound_djericmark_724939` qualified with three frozen events each. `freesound_cyclez_493560` produced only one eligible event and `freesound_justabeat_458897` produced none; both failed `insufficient_eligible_events` and `source_feature_requirements_unmet`. Freeze the rejection artifact at raw SHA-256 `f023736a0c1c7b19668a4497015adffe096fc5cea948135320908789704c9f3d` and close Matrix v3 without execution, candidate rendering, or human playback.
Why: Protocol v2 requires all four positive sources to provide at least two eligible events and complete source-feature vectors before source-contrast partitioning. Missing vectors cannot be imputed and passing sources cannot rescue failed sources. The result is therefore a source-set admission failure, not a tunable detector observation and not a perceptual hardness verdict.
Evidence: all four exact Development files passed raw SHA-256 and strict signed RIFF/PCM verification. The access log raw SHA-256 is `f892cd4e300c7aec63f345386cc04a8a220b8b1239d4b770ae4dbdb27dcaa0f8`; it records four completed deliveries, no directory discovery, and no holdout audio access. The final session raw SHA-256 is `5ca4bcb032e82399932c1e275b1885362a0bef7e0b1952da41affde5e71de326`; the qualification commit marker raw SHA-256 is `96cee5f546e52f7eeee4aeacce7ba84d822616c84ea929c2b64a7e8b7fac3639`. `docs/reviews/riotbox_1430_stage_a_v2_development_qualification_rejection_2026-08-11.md` records the bounded result.
Consequences: no Matrix-v3 condition may be rendered or heard, and this session may not be rerun. Protocol v2, Prequalification v3, Detector v1, Anatomy v2, Registry v3, Matrix v3, thresholds, roles, ordinals, and algorithms remain immutable historical inputs. A later attempt requires Protocol v3, every applicable component-version bump, a newly frozen legal Development snapshot, and a new decision before source access. `quality_proof=false`, `hardness_proof=false`, `human_verdict=unverified`, and no Riotbox product-path claim exists.
Status: accepted

---

### RBX-264

Date: 2026-08-11
Topic: remove consumed Stage-A acquisition executables after the one-off Development download
Phase: P023 / Sound Excellence
Question: which acquisition machinery should remain in the product repository after Batch v5 and qualification are terminal?
Decision: retain only the final Batch-v5 identity contract, its metadata allowlist, Registry-v3 evidence bindings, decision-log hashes, and the exact Git history. Remove the two consumed downloader generations, their artifact/transaction modules, their validators and mutation fixtures, and every runnable acquisition/reconciliation command. Remove the superseded Batch-v1 through Batch-v4 JSON and review scaffolds from the live tree. Keep the Protocol-v2 validator and the actual Development qualification path.
Why: acquisition was a one-off operational step, not Riotbox product behavior. The permanent implementation had grown to more than fifteen thousand lines without advancing detection, rendering, listening, or the instrument. Its concurrency, reconciliation, and validator layers no longer protected a callable workflow once Batch v5 completed and would create ongoing review and maintenance cost.
Evidence: commits `a937e5e8` and `5134399e` preserve the removed acquisition implementations exactly. Final acquisition identity is frozen by `docs/benchmarks/percussive_force_stage_a_v2_acquisition_batch_v5.json`, the allowlist, the sealed-manifest hash `4c7fd063ac84609b48f6c99eb24f1f4b3910cee2f644b4e5c8d7ff6d5eb88d55`, and Registry v3. The executed qualification is independently reproducible at commit `7e32036e`; its implementation snapshot never included or called the removed acquisition modules.
Consequences: Riotbox has no supported downloader or acquisition-retry subsystem. Future one-off Development acquisition must use an exact allowlist, standard client, OS keyring, bounded local access, and existing hash/WAV primitives; it must not recreate transaction, race, reconciliation, credential, or validator frameworks unless reusable product behavior demonstrably requires them. Deleted files remain recoverable from Git history, but must not be restored as active commands for this consumed batch.
Status: accepted

---

### RBX-265

Date: 2026-08-11
Topic: freeze the compact Stage-A-v3 Development source pool and deterministic reserve selection before new audio access
Phase: P023 / Sound Excellence
Question: how may RIOTBOX-1430 continue after the terminal Protocol-v2 source-set rejection without retuning the frozen force hypotheses or rebuilding an acquisition subsystem?
Decision: freeze `docs/benchmarks/percussive_force_stage_a_protocol_v3.json`, `docs/benchmarks/percussive_force_stage_a_source_pool_v1.json`, and `docs/benchmarks/percussive_force_development_matrix_template_v4.json` before any new original-file request. Protocol v3 inherits Protocol v2's `/numeric_passports`, `/prequalification`, and `/precandidate` sections exactly at canonical SHA-256 `0b486bf697d92c48bf6bd42544132c9e974a0f29d747e3ac787c14e684875c4a`, `9a1e5683e2e60585578781c84983e7bb3803619570f117a4391bc7958dd34555`, and `4965700d4bb424f3873ecd0a20dfeb906a2b982eaa9c678b0af0429a5dbddb16`. Detector v1, Anatomy v2, source contrast v1, rhythmic-location and event-ordinal policies, F1 v1, F2 v1, and F3 v2 remain unchanged. Freeze the complete ordered CC0 Development pool as Freesound IDs `183441`, `268110`, `217345`, `211478`, `266735`, `385676`, `431873`, `423272`, `353853`, `591426`, `365847`, `213512`, `19059`, `369570`, and `51219`, with fifteen distinct authors and five metadata candidates in each of `dense_break`, `sparse_drums`, and `electronic_drums`. Authorize at most those fifteen ordered original-file GETs through a standard one-off client using the Riotbox-scoped OS keyring, with no preview, redirect, automatic retry, substitution, directory discovery, holdout-audio access, commercial-reference access, repository credential, or repository downloader. Reject an unsupported header before PCM iteration for that candidate only. Qualify all successfully admitted pool entries in ordinal order without early stop. Enumerate every four-source combination of qualified entries in lexicographic pool-ordinal order and select the first containing four authors, all three required families, and an unchanged valid source-contrast partition with at least three clusters. If no combination passes, reject the complete pool without matrix, render, or playback and require a new version before further source access. If one passes, bind Matrix v4 mechanically as the unchanged three-family by four-source by two-event cross-product before candidate rendering.
Why: Protocol v2 showed that an all-or-nothing four-file batch can reject after two sources qualify even though the force mechanisms themselves were never exercised. A preregistered reserve pool separates source admission from algorithm choice while preventing result-driven source substitution, detector retuning, author reuse, and early stopping. The compact overlay references the immutable v2 algorithm sections instead of copying the 142-kilobyte predecessor, and the one-off standard-client boundary follows RBX-264 rather than recreating transaction, reconciliation, or downloader infrastructure.
Evidence: the raw SHA-256 pins are Protocol v3 `21f716e3aeb6c9198671e34be21e225585a41135f875bccedb3c57df625e7eb4`, source pool v1 `9c729eaac4156cb556d9f2538635e04829b3ffec3ad012d742fb765ad1e2a8ba`, and Matrix template v4 `0bdff640827e92fba9675ab9ede69dea5789bf0a6f3702bcf84254a9eec85df3`. Protocol v2 remains pinned at raw SHA-256 `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878`; the protected Registry v3 remains pinned at raw SHA-256 `9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502`, with nine active holdout metadata entries and no holdout-audio access. The compact validator and seven fail-closed mutations pass. Metadata search used three exact bounded Freesound API queries and opened no preview or audio. The four historical v2 files remain only two known-positive Development examples, one one-event stress example, and one detector-negative stress example; none is fresh acceptance evidence.
Consequences: the next action is the bounded one-off acquisition followed immediately by strict hash/header admission and one fresh Development-only qualification session. Source results may reject candidates and choose only through the frozen combination rule; they cannot change family labels, pool order, format gates, algorithms, equations, thresholds, ordinals, or selection logic. Mechanical success grants only Matrix-v4 execution and then a bounded structured human request. Automation cannot award `percussive_hard`; `different but not harder` rejects and freezes the recipe. Until that review, `quality_proof=false`, `hardness_proof=false`, `human_verdict=unverified`, and there is no product-path progress claim.
Status: accepted

---

### RBX-266

Date: 2026-08-11
Topic: accept the Stage-A-v3 acquisition rejection and freeze a standard PCM/WAVE v4 resume
Phase: P023 / Sound Excellence
Question: what follows when the seventh ordered Protocol-v3 original file is core PCM24/48 kHz but the inherited strict parser rejects its non-16-byte `fmt` chunk before qualification?
Decision: accept the Protocol-v3 access run as `rejected_fail_closed` after seven attempted and six completed requests. Preserve Protocol v3, source pool v1, Matrix template v4, and the local v3 access log without reinterpretation or retry under v3. Candidate ordinal 3 / Freesound `217345` remains terminally rejected as IEEE Float32 and may not be retried. Preserve the five published header-only admissions at ordinals `1`, `2`, `4`, `5`, and `6` with their exact v3 access-log SHA-256 identities; no PCM sample iteration, source feature, event computation, render, or playback occurred. Freeze `riotbox.percussive_force_stage_a_protocol.v4`, `riotbox.percussive_force_stage_a_source_pool.v2`, `riotbox.percussive_force_stage_a_pcm_wave_admission.v2`, and `riotbox.percussive_force_development_matrix_template.v5` before any further original-file request. PCM admission v2 retains RIFF/WAVE, format tag `1`, signed little-endian integer PCM, 16/24-bit samples, 44.1/48 kHz, one/two channels, and the 16-second maximum. It accepts either the base 16-byte PCM `fmt` payload or a bounded 18–64-byte WAVEFORMATEX payload whose `uint16 cbSize` equals `fmt_chunk_size - 18`; malformed or incoherent chunks still reject the complete v4 access. Authorize exactly ordinals `7` through `15` from the unchanged v1 metadata pool, once each in ascending order under v4, with no redirects, automatic retry within v4, preview, search, substitution, directory discovery, holdout audio, commercial reference, or credential persistence. After header admission, qualify every admitted ordinal from the combined frozen set in original order without early stop and apply the unchanged lexicographic four-source selection and 3 x 4 x 2 matrix rules.
Why: the v3 failure exposed a container-serialization assumption, not a source-feature, detector, anatomy, contrast, force, or musical result. Standard PCM may be carried as the 16-byte base structure or a coherent WAVEFORMATEX structure with an explicit extension length. Versioning that general container rule before another request preserves fail-closed history while avoiding a source-specific exception. Retaining the five exact unanalysed files avoids waste; completing the still-frozen pool advances qualification without another search or downloader framework.
Evidence: the terminal ignored v3 access log raw SHA-256 is `417e6b41f8ddb9c03650dc42f7c900d4ff56aebf929caaf38fa3dfe175cb15eb`. It records five admitted header-only files, one unsupported Float32 rejection, the seventh response's matching byte count/MD5/SHA and core PCM24/48 kHz fields, then failure `PCM fmt chunk must be exactly 16 bytes`; requests 8–15 did not occur. The new raw pins are Protocol v4 `0f7bff0744a0b229136f192b93ea1e537849c30f2906ab9247e279baf567e724`, source pool v2 `e82f21c965678ba8fe1937ee0652ad8456ac73ddb66c2956a871fc26d0404505`, and Matrix template v5 `2fcdcab61b72858419ae53aa9f74fb7a4fb6fd61c3889c6511d0c385b887201e`. The metadata-only v4 validator and six fail-closed mutations pass. Protocol v2 and its three inherited canonical algorithm-section hashes remain unchanged.
Consequences: v3 cannot resume. V4 may make at most nine exact original-file GETs, starting again at ordinal 7 under the new version and then continuing through ordinal 15. Any unsupported core audio format rejects only that candidate before PCM iteration; any identity, transfer, hash, RIFF structure, or WAVEFORMATEX-coherence violation rejects the v4 run. Only a completed v4 access may begin the single fresh Development qualification. Automation still cannot award hardness or musical quality, and no candidate may be played before Matrix-v5 mechanical gates pass.
Status: accepted

---

### RBX-267

Date: 2026-08-11
Topic: reject the incompatible seventh source and freeze the final untouched-source continuation
Phase: P023 / Sound Excellence
Question: what follows when Protocol v4 confirms that source-pool ordinal 7 has an internally incoherent extended PCM `fmt` chunk?
Decision: accept the Protocol-v4 access as `rejected_fail_closed` after its only request. Preserve its response identity and terminal log, do not retry ordinal 7, and do not broaden PCM/WAVE admission again. Freeze `riotbox.percussive_force_stage_a_protocol.v5` as the minimal final continuation. It preserves the five header-only admissions at ordinals `1`, `2`, `4`, `5`, and `6`; treats Float32 ordinal `3` and incoherent-header ordinal `7` as terminal format rejections; and authorizes exactly one ordered GET each for the eight still-untouched metadata identities at ordinals `8` through `15`. All metadata identities, authors, family labels, Detector v1, Anatomy v2, source contrast v1, ordinals, F1, F2, F3, PCM/WAVE admission v2, qualification gates, deterministic four-source selection, and 3 x 4 x 2 matrix semantics remain unchanged. No redirect, retry, preview, search, substitution, directory discovery, holdout audio, commercial reference, or credential persistence is permitted.
Why: another parser expansion for one malformed or nonstandard file would be result-driven format overfitting and would spend effort without improving Riotbox. The pool already contains five unanalysed admitted files and eight untouched reserves. Permanently rejecting the incompatible file and proceeding through the previously frozen metadata order is the smallest path to actual source qualification.
Evidence: the terminal v4 access-log raw SHA-256 is `fa15bb32c1b9285f878ffbec676ffae627e8aac1d2ea1d4d810241919abcce9e`. Its single response for Freesound `431873` matched the frozen byte count and MD5 and raw SHA-256 `94ad4daca2686c7c448eb88a63af0bc74abee0482125060b2acf7a7dc5c4b539`, then failed `incoherent_fmt_extension_size` before publication, PCM iteration, analysis, or playback. Protocol v5 raw SHA-256 is `455440aabc1a433bbc7fbcc2093b85f6d1c66e1bba081526e082c50ed8248519`; its metadata-only validator and four fail-closed mutations pass.
Consequences: ordinals 3 and 7 are format-stress negatives, not musical-quality verdicts. V5 gets at most eight original requests and then proceeds directly to one fresh Development qualification if access completes. A v5 access failure stops the source lane and requires explicit new versioning; success may not alter algorithms or selection and must move immediately to qualification and, only on selection pass, Matrix v6 rendering.
Status: accepted

---

### RBX-268

Date: 2026-08-11
Topic: freeze the thirteen-source Stage-A-v5 Development qualification set before PCM iteration
Phase: P023 / Sound Excellence
Question: which exact source identities may the fresh Protocol-v5 qualification analyze after the final continuation access completed?
Decision: freeze `docs/benchmarks/percussive_force_stage_a_bound_source_set_v1.json` before opening any selected file for PCM iteration or source-feature computation. The exact qualification order is pool ordinals `1`, `2`, `4`, `5`, `6`, `8`, `9`, `10`, `11`, `12`, `13`, `14`, and `15`. Each of the thirteen entries binds its unchanged metadata identity, author, family, exact Development path, CC0 declaration, raw SHA-256, sample rate, channel count, integer PCM width, and 16-second maximum. Begin one fresh Development-only `riotbox.percussive_force_stage_a_qualification_session.v5`, open only those exact paths through bounded no-follow access, analyze every source once with the unchanged Protocol-v2 Detector/Anatomy/source-feature contract, and never stop early. Among individually qualified sources, enumerate four-source combinations in lexicographic original-ordinal order; require four authors, all three families, exactly one valid unchanged source-contrast partition, and at least three clusters. Select the first passing combination. No valid combination rejects qualification without render or playback. A selected combination mechanically binds `riotbox.percussive_force_development_matrix.v6` for the unchanged F1/F2/F3 by four-source by two-event cross-product.
Why: acquisition identity and qualification evidence have different roles. Freezing the exact thirteen-file set after header admission but before PCM analysis prevents missing-file substitution, result-driven family relabeling, early survivor selection, and post-analysis hash changes while allowing the previously frozen reserve rule to do its intended job.
Evidence: source-set raw SHA-256 is `7ec185a51233d83c49d8227b0e81acb2ca83c24bc31783a9343dc71d090e47a6`. It binds terminal v3 access log `417e6b41f8ddb9c03650dc42f7c900d4ff56aebf929caaf38fa3dfe175cb15eb` for ordinals 1/2/4/5/6 and completed v5 access log `dd9be23138651ae226bce7af6f8765e6014f19847a20d342f62fceaedd7cdcbd` for ordinals 8–15. A metadata-only cross-check confirms thirteen distinct case IDs/authors, all three families, exact pool identity/path/license agreement, and exact access-log SHA/header agreement. Source audio, holdout audio, and commercial references were not opened by that validation.
Consequences: the next operation is source analysis, not further acquisition or contract design. The qualification may only pass or reject the frozen sources and select through the frozen combination order; it cannot alter algorithms, thresholds, formats, families, hashes, ordinals, or source contrast. Automation still grants no human hardness or quality verdict.
Status: accepted

---

### RBX-269

Date: 2026-08-11
Topic: accept the thirteen-source qualification pass and freeze the exact Stage-A Matrix v6 render
Phase: P023 / Sound Excellence
Question: which exact four sources, events, renderer implementation, and condition order may execute after the Protocol-v5 Development qualification passes?
Decision: accept qualification session `e300a627-caa4-4f08-8d22-d7e849c3030e` as a mechanical Development pass. Nine of thirteen bound sources qualified individually. Freeze the first lexicographic four-source combination satisfying all gates: pool ordinal 5 `freesound_dabromusic_266735` / `dense_break`, ordinal 9 `freesound_dr_skitz_353853` / `sparse_drums`, ordinal 12 `freesound_garzul_213512` / `electronic_drums`, and ordinal 13 `freesound_aikighost_19059` / `electronic_drums`. Its unique valid partition has three clusters: `{5}`, `{9,12}`, and `{13}`. Freeze `docs/benchmarks/percussive_force_development_matrix_v6.json` and renderer `crates/riotbox-audio/src/bin/percussive_force_stage_a_matrix.rs` before candidate output. Execute exactly the 24 explicitly listed conditions in selected-source, event-ordinal 1/2, then F1/F2/F3 order. The Rust runner must invoke the existing public F1-v1, F2-v1, and F3-v2 renderers without a strength parameter, preserve full source length/rate/channels and exact samples outside the frozen event support, record resolved policy evidence, and reject basic safety/identity/body failures. Any rendered survivor remains blocked from playback until full-source Python detector/anatomy and remaining frozen mechanical screens complete.
Why: the reserve pool has now done its job without detector or mechanism retuning. Binding the exact selected events and renderer bytes before output prevents event substitution, family reordering, strength tweaking, or post-render policy changes. The Rust runner is a direct execution seam for existing hypotheses, not another DSP implementation or product path.
Evidence: qualification artifact raw SHA-256 is `f35f9412f8e07e6ced0922e6433d12cb9133e49003b257ef5850f2d72337f679`; session, access log, and qualification commit are respectively `048467631a8284d411a8e825612648b281383455836be03eefdb0a514be7926f`, `2d0ac93c525b624c25bf17b55fd73d21d7be9963ef1e52ad73f09b77cbbce6ab`, and `6c8379245e3df1f2c6559a4a4a34147a6632c3b600e60552ebdf0cc93ea2a3a4`. Matrix v6 raw SHA-256 is `cd29b23fd3d39ac5184f73585b825aabf987b865e6f37253260ce2287ac95c00`; renderer raw SHA-256 is `534f960b5b0afe98ece921d260f4fd30b8c616da8d3b7ab21cfe841f32bd4362`. Mechanical binding confirms the exact qualification hash, four source/path/hash identities, eight event boundaries, 24 unique condition IDs, and renderer pin. The renderer compiles and all 46 percussive-force Rust tests pass before rendering.
Consequences: candidate rendering is now authorized only through Matrix v6. No candidate or family may be heard merely because the Rust renderer succeeds; the frozen advanced mechanical screens still gate listening. Automation may reject survivors but cannot award `percussive_hard`, quality, or product-path progress.
Status: accepted

---

### RBX-270

Date: 2026-08-11
Topic: accept Matrix-v6 basic rejections and freeze full-source advanced screening before candidate access
Phase: P023 / Sound Excellence
Question: which rendered Matrix-v6 conditions may enter the frozen full-source detector/anatomy and confound screens?
Decision: accept Matrix v6 as completely executed across all 24 conditions. Nineteen conditions are terminal renderer/basic-screen rejections. Freeze exactly five technical pre-survivors for advanced analysis: `f2_freesound_dabromusic_266735_event1`, `f1_freesound_aikighost_19059_event1`, `f3_freesound_aikighost_19059_event1`, `f1_freesound_aikighost_19059_event2`, and `f3_freesound_aikighost_19059_event2`. Freeze `scripts/validate_percussive_force_stage_a_matrix_v6.py` before it opens any candidate WAV. For raw and attenuation-matched views it must use the unchanged source-frozen means and event identity, rerun the frozen detector/anatomy, enforce event-count/onset/proxy integrity, near-identity, zero-lag correlation, 24-band attack spectral cosine, body-energy range, boundary discontinuity, gain-only fit, exact complementary-three-band static-EQ fit, static odd-distortion fit, unchanged timebase, and the already-recorded renderer family-direction evidence. An unavailable or singular confound basis rejects; no screen may award hardness.
Why: renderer success is only the first reject boundary. Reanalysis must be implementation-pinned before inspecting candidate details so that a surviving mechanism cannot influence metric code or exception handling. Screening all five through one general implementation avoids candidate-specific analysis.
Evidence: Matrix result raw SHA-256 is `26ade073e0aa993904e8ff304fdae171afcf64147ec28cd4873dc350a69f7e76`. The five listed conditions passed finite/full-length/timebase, strict peak, untouched-region, raw PCM inequality, near-identity, zero-lag correlation, and raw body-energy gates. Advanced-screen runner raw SHA-256 is `925cf271bca91b01646a22a310cc68aad2d77d264d98a65b1c73ee2822a9aa32`; its source-blind contract preflight and synthetic identity/correlation fit goldens pass before candidate access.
Consequences: only an advanced survivor may be packaged for bounded structured human review. Zero survivors stop without playback. Advanced automation still cannot call a candidate harder, good, or product-ready.
Status: accepted

---

### RBX-271

Date: 2026-08-11
Topic: preserve raw event identity across Matrix-v6 attenuation-matched screening
Phase: P023 / Sound Excellence
Question: what follows when advanced-screen v1 rejects every pre-survivor and its attenuation-matched view changes the already-qualified event classification solely because uniform view gain was applied before rerunning the amplitude-sensitive detector?
Decision: accept advanced-screen v1 as immutable terminal evidence with raw result SHA-256 `d78461412f98e145e9767a111c3c5654cdaf8f0b54f84e31d2e56dcf7c9cd406`. It retains zero survivors and authorizes no playback. Freeze `scripts/validate_percussive_force_stage_a_matrix_v6_v2.py` as a versioned correction before reopening any candidate WAV. V2 changes only the event-integrity basis: both raw and attenuation-matched views use the already-qualified raw source event identity and one raw candidate detector/anatomy classification, while every signal-domain metric continues to use the view's declared source/candidate gains. All thresholds, F1/F2/F3 renders, exact candidate hashes, Matrix-v6 conditions, fit bases, body/identity/boundary screens, and the rule that an unavailable or singular confound basis rejects remain unchanged. The four Aikighost `confound_screen_undefined` rejections therefore remain terminal; no candidate-specific exception is permitted.
Why: uniform gain matching is a comparison view, not a new source qualification. Letting an absolute-level detector erase both sides' raw events makes temporal identity depend on the diagnostic display gain and contradicts RBX-270's source-frozen event identity. Binding event classification before view gain restores that invariant without relaxing a threshold or using a musical result to select behavior.
Evidence: v1's DABRO F2 raw view passed every advanced screen with four source and four candidate events; its matched view alone reported zero and zero after gains `0.9126318163647862` and `0.9131697229268734`, while the unchanged signal-domain fit values still passed. Advanced-screen v2 raw SHA-256 is `fd36358946e2faf20be91cff55bc044b8c11f0b42f1251a97751bb0de34e8584`; its source-blind `--validate-only` preflight passes with `candidate_audio_accessed=false`.
Consequences: after this decision and implementation are committed, v2 may reanalyze only the exact five Matrix-v6 pre-survivors and must write a new exclusive v2 result. V1 is never overwritten or reclassified. Only a v2 survivor may proceed to bounded structured human review; automation still cannot award hardness or quality.
Status: accepted

---

### RBX-272

Date: 2026-08-11
Topic: accept one Matrix-v6 advanced survivor for bounded human review
Phase: P023 / Sound Excellence
Question: which exact Matrix-v6 artifact may proceed after committed advanced-screen v2 completes?
Decision: accept advanced-screen-v2 result SHA-256 `a0339027cf0b194dfa95a5e6baa9b1833ba2102e55fee7524f11aa4d4a11231b`. Exactly `f2_freesound_dabromusic_266735_event1` survives. Bind source `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/05_dense_266735.wav` at raw SHA-256 `b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73` and candidate `artifacts/audio_qa/riotbox-1430/stage-a-v6-matrix/f2_freesound_dabromusic_266735_event1.wav` at raw SHA-256 `7a9e17bae841cebef05d990efcebd571578c73db49e047869afcc47368df0fe2`. Permit one local, ignored, hash-bound A/B review artifact containing the same bounded window and channel/rate mapping in the order raw source, declared silence, raw candidate, declared endpoint silence. The listener question is only whether B preserves the recognizable hit while adding immediate attack, physical body, and bite at `1.0x`; louder, darker, dirtier, or merely different is a reject.
Why: v2 removes only the diagnostic gain/event-identity contradiction and leaves every numerical threshold and confound screen intact. Its sole survivor is now eligible for the human judgment automation is forbidden to make. A compact A/B comparison minimizes listener fatigue and prevents unrelated unchanged sections of the source from dominating the judgment.
Evidence: both v2 views retain four source and four candidate events with one corresponding cluster. Raw and matched near-identity deltas are `0.061087517365865644` and `0.061102590446940566`; zero-lag correlations are `0.9981332418439249` and `0.9981332418439247`; attack spectral cosine is `0.9995513290159923`; raw/matched body-energy ratios are `0.9632024733178203` and `0.9643382338950708`; gain-only, static-EQ, static-distortion, and boundary screens pass. Automation still records `quality_proof=false`, `hardness_proof=false`, and `human_verdict=unverified`.
Consequences: technically preflight the exact A/B WAV, report its segment assignment, format, duration, peak/RMS/LUFS where available, silence/clipping, time-local waveform and spectral deltas, and audible contributor inventory. Then explain the non-priming purpose and obtain fresh readiness before playback. A `different_but_not_harder` verdict rejects and freezes F2 v1; no scalar retuning follows.
Status: accepted

---

### RBX-273

Date: 2026-08-11
Topic: accept the Stage-A F2-v1 human rejection and complete RIOTBOX-1430 source-pool evidence
Phase: P023 / Sound Excellence
Question: what follows when the only Matrix-v6 advanced survivor is heard in a hash-bound repeated A/B comparison and judged perceptually near-identical rather than meaningfully harder?
Decision: bind the human verdict to A/B artifact SHA-256 `384d7a977f7c1f4ccca84a24b2e64790ed3074d13fccdff9d46de4078b8cb368` and structured review SHA-256 `84e3cd67d0e764a33595ae0c880922b8bbf97a1dfc7cd3fb8969e6c3692b37ce`. Record `human_verdict=reject`, `source_recognition=source_clear`, and the failure `perceptual near-identity; B did not establish greater percussive hardness`. Freeze F2 `f2_exact_complementary_three_band_v1` for Stage-A `percussive_hard`; do not replay, scalar-retune, or reinterpret it. Accept RIOTBOX-1430's reopened source-pool slice as complete evidence work, but do not claim a positive hardness pass, product-path progress, or Stage-B eligibility.
Why: the listener heard the exact same candidate twice in a requested 10-second-A/10-second-B presentation and still found B barely different. Mechanical survival only made the comparison safe and identifiable; it cannot override the musician verdict. The ticket's corrected objective required a substantially larger lawful pool, fresh qualification, matrix execution, exact candidates, and structured human review. Those were delivered. Continuing to tune the rejected recipe inside the source-pool ticket would violate the frozen anti-retuning rule and repeat the earlier scope drift.
Evidence: the presentation contains 10.0 seconds of repeated raw source, 0.5 seconds exact-zero separation, 10.0 seconds of repeated F2 candidate, and 0.2 seconds exact-zero endpoint silence. It is 44.1 kHz stereo Float32, 20.7 seconds, unclipped, and uses an identical 121475-frame clean loop boundary for A and B. The source/candidate loop RMS values are `0.09445345038309058` and `0.09444391810932157`; no average-level advantage explains the judgment. Playback stopped with no remaining `pw-play` process after both hearings.
Consequences: RIOTBOX-1430 may proceed through branch review, CI, PR, and closeout with an explicit negative human outcome. RIOTBOX-1428 Stage B remains blocked. Any further Stage-A force attempt requires a newly versioned causal hypothesis and preregistered contract; F2 v1 is a negative calibration example, not a tuning starting point. RIOTBOX-1431 remains the separate workflow-document optimization follow-up.
Status: accepted

---

### RBX-274

Date: 2026-08-11
Topic: accept a post-execution Clippy-only spelling update in the Matrix-v6 offline runner
Phase: P023 / Sound Excellence
Question: how should the branch close when Rust 1.97 rejects the render-time runner's manual remainder spelling under `-D warnings` after all candidate and human evidence is already frozen?
Decision: preserve render-time runner SHA-256 `534f960b5b0afe98ece921d260f4fd30b8c616da8d3b7ab21cfe841f32bd4362` as the executable evidence identity for Matrix v6. Replace only `source.len() % channels != 0` with the Rust-1.97-equivalent `!source.len().is_multiple_of(channels)` and record the lint-clean current source SHA-256 `f686532c0860e73af9cd9bbd6805777a172b8cb816a1eb95d838aded13edc2c4`. Do not rerender, recompute candidate evidence, transfer a verdict to different audio, or change any algorithm/threshold/contract value.
Why: this is a standard-library predicate spelling change with identical Boolean behavior for the already-validated nonzero channel counts, not a source-result-driven algorithm change. Leaving the warning would fail the repository's mandatory Clippy gate; rerendering exact audio for a source-level lint spelling would create unnecessary duplicate evidence.
Evidence: the focused 46-test Percussive-Force suite and Matrix binary test target passed before the edit. After the edit, `cargo fmt --check` and `cargo clippy -p riotbox-audio --bin percussive_force_stage_a_matrix -- -D warnings` pass. The historical Matrix result, candidate WAV hashes, advanced results, A/B artifact, and structured human verdict remain unchanged.
Consequences: the Matrix-v6 result continues to cite the render-time source hash. Reviews must not mistake the post-execution current source hash for a different renderer generation. Full `just ci` must be rerun; no human replay is required because no audio bytes or recipe changed.
Status: accepted

---

### RBX-275

Date: 2026-08-11
Topic: freeze source-blind F4 source-native body sustain before Development candidate access
Phase: P023 / Sound Excellence
Question: what new causal Stage-A mechanism may follow the F1/F3 mechanical rejections and the F2 human near-identity rejection without retuning a failed family or using Development results to choose topology and constants?
Decision: freeze Protocol v6, Matrix v7, and F4 `f4_source_native_body_sustain_v1` before reopening any Development WAV. F4 preserves the physical attack and all samples outside the frozen body exactly, preserves sample order and `1.0x` playback, and selects one event-coupled source-native body band from fixed `55–180`, `180–560`, and `560–1120 Hz` analysis regions. A trusted band must exceed the unchanged `4x` lookbehind-noise or `16 LSB` numerical floor in both attack and body; selection maximizes body mean square over that floor. Only the selected body's decaying source samples receive the frozen bounded smooth gain `0.5*entry*exit*(0.35+0.65*sqrt(1-clamp(envelope/body_peak,0,1)))`, using an `8 ms` causal envelope, `2 ms` entry, and `10 ms` exit. There is no limiter, generated oscillator, delay, duplicate, resampling, transposition, filename branch, or source-specific constant. Reuse the mechanism-blind Protocol-v5 qualification and its exact four sources/eight events; execute only the eight new F4 conditions because F1–F3 remain immutable terminal evidence.
Why: the prior families changed attack/body level, complementary spectral balance, or dynamic residual yet produced no accepted force mechanism; the sole human-eligible F2 result remained perceptually near-identical. Registered primary evidence reports that higher-velocity snare strikes can retain loudness-controlled identity while showing greater sub-1 kHz energy and longer decay. F4 tests those coupled directions while leaving the recognizable attack untouched. The transfer is only an E2 falsifiable hypothesis across mixed drum sources; automation cannot turn it into a force verdict. Reusing the earlier mechanism-blind event catalog avoids needless source acquisition and qualification while the new exclusive matrix and access log preserve the post-rejection freeze boundary.
Evidence: F4 source SHA-256 is `85b6e4f3b19c292ee712a100bf563af90a4a780cac76734ffd36cb9ae782ef0a`; the v6/v7-capable matrix runner SHA-256 is `1eccec22f454f9a0309cb0e65aeaebb8b194d2356926fd0b55dd97aae21e932d`. Protocol v6 and Matrix v7 raw SHA-256 values are `e201d1a95936c17206ee1a1e151bcde32d593209b0d07cde7acb5b3aff32420a` and `4018ca070b7cb4193191a8a88c4279cd3bc878b25241c8f57f1f0eaa3227480d`. The unchanged qualification artifact SHA-256 is `f35f9412f8e07e6ced0922e6433d12cb9133e49003b257ef5850f2d72337f679`. Five source-independent tests pass: bit-identical attack/outside support with body change, three body frequencies selecting three distinct bands, deterministic repeats at 44.1/48/96 kHz, missing-body refusal, and headroom refusal without limiting. The v6 validator and four fail-closed mutation fixtures pass with `source_audio_accessed_by_validator=false`; strict Riotbox-audio Clippy passes. No Development candidate, holdout audio, commercial reference, or human playback was accessed during this freeze.
Consequences: Matrix v7 may now read each of the four exact registered Development paths once, verify hash/header identity, embed the bounded access log, and execute the eight conditions in declared order. The unchanged raw and attenuation-matched mechanical screens remain reject-only. Zero survivors stop without playback. A surviving exact candidate may receive one technically preflighted structured human comparison; `near_identity` or `different_but_not_harder` freezes F4 v1 without scalar retuning. A positive directional verdict must freeze the exact package before any Stage-B or holdout access.
Status: accepted

---

### RBX-276

Date: 2026-08-11
Topic: freeze the F4 advanced-access correction after Matrix-v7 basic screening
Phase: P023 / Sound Excellence
Question: how may the frozen full-source detector, anatomy, and confound screens run when Protocol v6 bounded the four renderer source reads but did not explicitly authorize the later source re-reads required by the already-established advanced-screen implementation?
Decision: freeze Protocol v7 before any advanced candidate or source audio access. Preserve F4 source SHA-256 `85b6e4f3b19c292ee712a100bf563af90a4a780cac76734ffd36cb9ae782ef0a`, every F4 equation and constant, all eight Matrix-v7 renders, the exact source/event binding, Protocol-v2 detector/anatomy/mechanical thresholds, and the raw-source/candidate event-identity correction from RBX-271. Bind Matrix-v7 result SHA-256 `70edb5d8604f12f634c4f4b0828cd3809af603144e94cd06d6294c391883a3c5`: six conditions passed basic screens; both Garzul conditions remain terminal strict-peak rejections because an unchanged source sample outside the frozen event reaches absolute `1.0`. Authorize exactly one contained regular-file re-read, in order, for DABRO, Dr Skitz, and Aikighost, plus one read of each of the six exact candidate WAVs. Embed the three-source access record in exclusive advanced result v3. For F4 only, test the actual smooth-processing boundaries at attack end, attack end plus entry, body end minus exit, and body end; this maps the unchanged boundary-discontinuity test to F4 topology and changes no tolerance.
Why: the advanced analyzer must recompute the frozen full-source detector/anatomy evidence and compare source and candidate in raw and attenuation-matched views; that cannot honestly happen without reopening the three source files owning basic survivors. Protocol v6's four renderer reads were complete, but silently treating them as authorization for later analysis would violate the explicit bounded-access rule. A versioned operational correction is safer than weakening provenance, skipping the full-source gate, or inventing another acquisition system. The candidate outcomes select only which already-rendered files require the next ordered gate; they do not tune the mechanism or its screens.
Evidence: Protocol v7 raw SHA-256 is `e92bbcb44ff0c7b14d43dc16173ee5fda5cd421f850d45a18d771fdff3ce9407`. Advanced wrapper and shared-screen SHA-256 values are `5c0778763a5b7a32e7612405ba294eb3666c74720d179af26a8bfc971f491b81` and `21bd63ea95d709231cd714c58d93a99ee20e98cc3d597375373885447d60b39c`. Protocol-v7 validation, four access/immutability mutations, Python compilation, and advanced `--validate-only` pass with `source_or_candidate_audio_accessed_by_validator=false`. The Matrix-v7 access log records four exact Development reads with matching raw SHA-256/header identities; holdout and commercial-reference access remain false.
Consequences: the pinned v7 advanced wrapper may now execute once. Any source/candidate/hash/cardinality/order/reanalysis/fit/boundary failure stops fail-closed without playback. Zero survivors stop Stage A. A survivor remains only technically reviewable and requires exact artifact preflight plus a fresh human readiness request; no automated result awards hardness or quality.
Status: accepted

---

### RBX-277

Date: 2026-08-11
Topic: accept two F4 advanced survivors and bind one first-order human comparison
Phase: P023 / Sound Excellence
Question: which exact F4 result may proceed to the single bounded human directional review after frozen advanced screening?
Decision: accept advanced-result-v3 SHA-256 `9b7536ba0526582ff9675401d313220474e2d95ba2a09fc9af42aea2801ad12a`. Exactly DABRO event ordinals 1 and 2 survive raw and attenuation-matched screens. Select event 1 solely because it is first in the frozen Matrix-v7 condition order; do not choose between survivors by musical metric. Bind source `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/05_dense_266735.wav` at raw SHA-256 `b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73` and candidate `artifacts/audio_qa/riotbox-1428/stage-a-v7-f4-matrix/f4_freesound_dabromusic_266735_event1.wav` at raw SHA-256 `36d9b1ed65233eb68b364908afa2a9c6fd909e1664d3caab45949de81e9cb6c1`. Permit one local ignored A/B artifact using the exact frozen lookbehind-through-tail window frames `14513..21281`: A repeats the raw source excerpt in a `0.5 s` unit for `10.0 s`; after `0.5 s` exact silence, B repeats the same candidate excerpt with frozen attenuation-match gain `0.9510122374043875` for `10.0 s`; append `0.2 s` exact endpoint silence. No other source, candidate, variant, or playback is authorized by this decision.
Why: both survivors have nearly identical technical evidence, and metric-based selection after results would turn reject-only screens into an implicit taste score. Matrix order is deterministic and source-blind. The level-controlled B view removes the small event-train RMS advantage while retaining F4's time-varying low-body/decay redistribution. Repetition gives the human enough exposures to judge a roughly `153 ms` composite source excerpt without asking them to identify a final-millisecond difference or listening to unrelated full-source duration.
Evidence: event 1 raw/matched near-identity deltas are `0.1071575336842887` and `0.09122748900692669`; zero-lag correlation is `0.9958420333203577`; attack spectral cosine is `1.0`; raw/matched body-energy ratios are `1.1149751352805153` and `1.0084105791416722`. Gain-only, static-EQ, static-distortion, event-integrity, body, identity, and boundary screens pass in both views. The advanced access log records exactly the three Protocol-v7 Development source re-reads in order. Dr Skitz and Aikighost conditions remain terminal `confound_screen_undefined` rejections; Garzul remains terminal at the basic strict-peak gate. Holdout and commercial-reference access remain false.
Consequences: technically construct and analyze the one exact A/B artifact before playback. Inventory it as a repeated composite excerpt from the Development source, not an isolated drum stem. Explain that B keeps the exact original attack before level control and changes only source-native low-body decay; lower/darker or merely different does not pass. Obtain fresh readiness before the single playback. Record perceptual near-identity, `different_but_not_harder`, identity loss, or body smear as a rejection that freezes F4 v1 without scalar tuning; only a clear recognizable more-forceful-strike verdict may unblock exact-package Stage-B planning.
Status: accepted

---

### RBX-278

Date: 2026-08-11
Topic: accept the F4 human near-identity rejection and diagnose the level-controlled cue collapse
Phase: P023 / Sound Excellence
Question: is the reported absence of a meaningful audible A/B difference caused by an A/B assignment or playback bug, or by F4 changing too little in the wrong perceptual domain?
Decision: bind the human verdict to A/B artifact SHA-256 `ab239a75fddb1d6a6752dd1bcc23aed70bafd84775d101541b6216581dc9fdad` and structured review SHA-256 `bb4f7e875aaaadc2f950805eb5fbb5396b0cd5ec8a2b80a9b961d5c53d555394`. Record `human_verdict=reject`, `source_recognition=source_clear`, strongest element `none`, and failure `perceptual near-identity; no difference heard between A and B`. Freeze F4 `f4_source_native_body_sustain_v1`; do not replay or scalar-retune it. Classify the failure as an ineffective mechanism design, not an A/B routing, segment-order, silence, or duplicate-artifact bug.
Why: the exact played blocks are not bit-identical, but their difference is both small and concentrated in a low-body decay redistribution. The frozen attenuation match reduces B's otherwise bit-identical attack by about `4.9%`, while the matched total body energy remains essentially unchanged. That leaves no stronger immediate strike cue and asks a listener to infer force from a subtle low-frequency color/decay change. Increasing the same gain would primarily make B darker/heavier and would violate the anti-retuning rule without addressing strike force.
Evidence: A and B contain twenty exact repeats of their declared `22050`-frame units; the A source excerpt matches exactly, the B candidate/gain assignment differs by at most one Float32 ULP, separator and endpoint silence are exact zero, and no `pw-play` process remained after the announced endpoint. A/B block RMS values are `0.08758846173155786` and `0.08757346556522888`; the difference RMS is `0.007937968263400248`, or `-20.854750817994613 dB` relative to A. Within the body the difference remains `-20.581023832645595 dB` relative to A. The attack correlation is `1.0`, but attenuation lowers its RMS from `0.28922214162824866` to `0.27505380135358365`. Matched total body energy ratio is `1.008410613585245`; the remaining redistribution raises `55–180 Hz` and `180–560 Hz` energy while slightly reducing upper-body/high energy. Every active excerpt sample differs because of the uniform match, proving non-identity but not perceptual relevance.
Consequences: Stage B and holdout access remain blocked. The next attempt must be a newly versioned source-blind topology that makes a level-controlled immediate attack cue and physical body cue move coherently. It must not reuse F4's scalar, treat extra low body as force, or answer the human rejection with another presentation of the same recipe. Existing Development evidence may motivate the new causal question, but its complete mechanism and constants must freeze before any new candidate access.
Status: accepted

---

### RBX-279

Date: 2026-08-11
Topic: close the F4 evidence boundary and route the unexecuted velocity-cue transfer follow-up
Phase: P023 / Sound Excellence
Question: what must the RIOTBOX-1428 branch review preserve after F4 was mechanically valid but human-rejected as perceptually near-identical, and which research follow-up remains unexecuted?
Decision: accept F4 `f4_source_native_body_sustain_v1` as terminal negative Stage-A evidence and preserve its frozen code, contracts, matrix, candidate, and human verdict without rerendering or scalar retuning. Constrain the historical `F4BodySustainPolicy.output_peak` field to its implemented meaning: maximum output observed while processing the modified body interval, not the full-candidate absolute peak. The independent Matrix-v7 `basic_metrics` full-candidate peak remains the authoritative complete-output headroom screen and correctly produced the two Garzul terminal rejections. Route the remaining research handoff through Linear issues RIOTBOX-1434 and RIOTBOX-1435: first validate only the six already registered natural-dynamic controls and freeze bounded multi-cue directions; then implement one structurally distinct source-relative velocity-cue transfer and run its exact eight-condition Development matrix plus human gate. This decision does not authorize opening control, Development, Holdout, or commercial-reference audio.
Why: the exact A/B assignment and playback were correct, and F4 changed only low-body decay while leaving the raw attack bit-identical; attenuation matching then reduced the attack level. The resulting failure is an ineffective single-domain mechanism, not a renderer-routing bug or a failure of the research model. The original research handoff separately proposed a coupled transformation of attack, decay, local resonance, and brightness after a bounded natural-reference sanity check; that path has not yet been executed. Recording the policy-field scope prevents a region-local diagnostic from being mistaken for complete-output safety while avoiding any post-result change to frozen F4 bytes.
Evidence: focused Riotbox-audio percussive-force tests, the matrix binary target, Protocol-v6/v7 fail-closed mutation fixtures, Matrix-v7 validate-only contracts, formatting, and strict Riotbox-audio Clippy pass on the reviewed branch. Matrix-v7 full-output analysis records Garzul event ordinals 1 and 2 as terminal because an unchanged source sample outside the event reaches absolute peak `1.0`; no candidate escaped that screen. The exact human-bound F4 artifact and structured review remain pinned by RBX-278.
Consequences: F4 cannot be replayed, renamed, or tuned into the next family. RIOTBOX-1434 is the sole contract enabler and must create its own versioned pre-access contract and Decision-Log authorization before any natural-control audio is opened. RIOTBOX-1435 is its exact audible follow-up and must freeze topology, equations, constants, and the full-output safety semantics before Development access. RIOTBOX-1428 Stage B and all Holdout access remain blocked until that audible slice earns the directional human pass.
Status: accepted

---

### RBX-280

Date: 2026-08-11
Topic: freeze the bounded natural-velocity control gate before any control-audio access
Phase: P023 / Sound Excellence
Question: which already-registered natural dynamic controls may RIOTBOX-1434 measure, with which fixed analysis and access boundary, before a source-relative multi-cue velocity transfer is designed?
Decision: freeze `riotbox.percussive_force_natural_velocity_controls.v1` and authorize one execution against exactly the registered mezzo-forte, forte, and fortissimo members of `philharmonia_snare_with_snares_025` and `philharmonia_whip_struck_together_025`, in that order. Open each exact repo-relative path at most once through the existing contained regular-file reader, verify its Matrix-v2 SHA-256, and write the access record before each open. Compute only fixed 1 ms RMS-envelope attack time, decay time, 40–2000 Hz body-resonance peak, and first-50-ms attack spectral centroid directions with the frozen passport. Record adjacent and extreme signs; a feature is technically monotonic only when both adjacent directions agree and are non-equal. The snare is the body-bearing control; the whip can establish only attack-edge direction. Filename dynamics are provisional labels, not truth. These measurements cannot select an algorithm, fit a perceptual threshold, prove hardness or quality, or establish universal calibration.
Why: RBX-279 identifies an unexecuted research handoff: test matched natural dynamic series before coupling attack, decay, local resonance, and brightness in a new Riotbox mechanism. Six controls are enough for a bounded directional sanity check but far too few for scientific calibration. Freezing the analysis and exact reads before access prevents the observed directions from choosing their own detector, feature window, source substitution, or retry policy while avoiding a new acquisition subsystem.
Evidence: contract raw SHA-256 is `f618144e02a6654b6a7c08b0773d91454b26e9b4cf8ac7cbcca723961783a78c`; runner raw SHA-256 is `b9df59203c4e78b064a30f28aa9ece8b0aada0870c8c2860043db28a18260552`; read-only validator raw SHA-256 is `26486d90b6e5dac8a201878150e0194b5780e031145c174d3f26554d250152a9`; predecessor Matrix-v2 remains `aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6`. Python compilation, twelve fail-closed in-memory mutation fixtures, repository validation, and runner `--validate-only` pass with `source_audio_accessed=false`. No control, Development, Holdout, or commercial-reference audio was opened before this freeze.
Consequences: after this decision and its contract, runner, and validator are committed, the runner may execute once at `artifacts/audio_qa/riotbox-1434/natural-velocity-controls-v1`. Any path, hash, format, cardinality, order, analysis-passport, or access-boundary failure stops without discovery, substitution, retry, or parameter change. Successful technical analysis still requires a separately constructed and technically analyzed randomized A/B plus reversed B/A human directional-sanity artifact before RIOTBOX-1434 may freeze any transfer direction. Any failure remains negative evidence. RIOTBOX-1435, Development access, candidate rendering, Stage B, and Holdout access remain blocked.
Status: accepted

---

### RBX-281

Date: 2026-08-11
Topic: correct only the RIOTBOX-1434 local-audio root after the v1 pre-open refusal
Phase: P023 / Sound Excellence
Question: may the natural-control gate proceed after v1 stopped before opening its first WAV because it prepended an undocumented repository-source root to a directional-control `local_path`?
Decision: accept the v1 access-log refusal at raw SHA-256 `0a33e8359c4932d72f66b2623d780236005372ed64d0031a212e597eccd47f2c` as terminal and forbid a v1 rerun. Freeze `riotbox.percussive_force_natural_velocity_controls.v2` with exactly one correction: resolve the unchanged Matrix-v2 control `local_path` values beneath `.download-examples`, the recommended ignored `RIOTBOX_LOCAL_AUDIO_ROOT` explicitly documented by pinned Matrix-v1 raw SHA-256 `3290011471bb1ae0fc66e54c8bb4e1382f82ceee6266245a44755d8f62f1f970`. Preserve the same six files, hashes, order, analysis passport, human protocol, claim boundary, and access limits. V2 receives a new exclusive output path and one execution only.
Why: v1 invented `data/test_audio/external` by analogy with product Development sources even though the directional controls use a separately documented local-audio-root contract. The contained opener failed while resolving the missing `RIOTBOX-1429` ancestor, before a file descriptor for any WAV existed or any source bytes were read. Correcting the already-documented root in a new version preserves fail-closed provenance without searching a source directory, substituting a file, changing analysis, or turning the failure into a retry.
Evidence: v2 contract raw SHA-256 is `a0485af6cb30e401a6bc3bd6e900e3a0f8afdb64ef3c44f6f8445a5386c4c14f`; v2 runner raw SHA-256 is `4a1f4f7ff2b12374eb768f2287c581938b387aefbfeee925835935a709f38370`; v2 read-only validator raw SHA-256 is `5b8f3ae20ef25a6cd2ff77969cbfde8b723d1d7a8769df927f45a75ee110efe8`. Python compilation, nine fail-closed in-memory mutations, repository validation, and runner `--validate-only` pass. V1 records `directory_discovery_performed=false`, all Development/Holdout/commercial-reference access flags false, zero source bytes read, and no control WAV opened. No source directory was enumerated to derive this correction.
Consequences: after this correction is committed, v2 may resolve and open only `.download-examples/RIOTBOX-1429/philharmonia_percussion/` plus each of the six already-registered filenames through the contained exact-file reader. Any mismatch stops without search, substitution, retry, parameter change, or further source access. All RIOTBOX-1435, Development, candidate, Stage-B, and Holdout gates remain unchanged.
Status: accepted

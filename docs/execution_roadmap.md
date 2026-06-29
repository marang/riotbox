# Riotbox Execution Roadmap

Version: 0.1  
Status: Draft  
Audience: product, realtime, MIR/ML, TUI, QA

Derived from:
- `docs/prd_v1.md`
- `plan/riotbox_masterplan.md`
- `plan/riotbox_liam_howlett_feral_addendum.md`

---

## 1. Purpose

This document defines the execution path from the current planning state to a stage-ready Riotbox product.

It is not a feature wish list. It defines:

- what happens first
- what can run in parallel
- what must not start too early
- which outputs each phase must produce
- how planning, research, implementation, testing, and benchmarking connect

---

## 2. Core Delivery Loop

Riotbox should be delivered through one repeating loop:

1. **Spec**
2. **Risk spike / research**
3. **Vertical slice implementation**
4. **Tests**
5. **Benchmarks**
6. **Review and freeze**
7. **Next slice**

No large subsystem should skip this sequence.

This is the product delivery loop. The operational Linear/branch/PR/CI/archive
loop that agents and contributors use to execute each bounded slice is maintained
in `docs/workflow_conventions.md`.

---

## 3. Product Spine

Every major decision should support the same user-facing spine:

1. load a track
2. inspect structure
3. hear a rebuild
4. mutate it
5. capture a useful moment
6. save the session
7. restore the session
8. perform it live
9. export a reproducible result

If a work item does not improve or unblock this spine, it should usually not be on the critical path.

---

## 4. Delivery Principles

- Build contracts before clever behavior.
- Resolve high-risk unknowns through bounded spikes, not open-ended research.
- Prefer vertical slices over isolated subsystems.
- Keep realtime work measurable from the start.
- Add Ghost autonomy only after action safety and replayability are real.
- Add feral depth as policy on top of stable core contracts.

---

## 5. Phase Order

### Phase A - Contract and planning freeze

Goal:
- remove ambiguity from the core contracts

Deliverables:
- PRD v1
- execution roadmap
- action lexicon
- validation and benchmark spec
- fixture corpus spec
- phase definition of done
- research / decision log

Must complete before:
- any substantial parallel implementation

### Phase B - Core technical spikes

Goal:
- answer only the questions that can still invalidate architecture choices

Critical spikes:
- audio I/O and latency spike
- sidecar RPC and failure-isolation spike
- deterministic replay spike
- analysis-provider bakeoff for beat / bars / sections / slice candidates
- source beat-grid verification spike:
  - automatic BPM estimate from arbitrary WAV input
  - beat, downbeat, bar, and phrase-grid candidate quality
  - confidence reporting and low-confidence fallback behavior
  - audio drift checks proving generated bass, MC-202, TR-909, and W-30 layers stay aligned to the source grid
  - follow the Rust-first all-lane direction in `docs/plans/source_timing_intelligence_plan.md`; external MIR/Python tools may inform research, but must not become the runtime timing contract

Output of every spike:
- decision
- measured results
- rejected options
- implication for implementation

### Phase C - Core skeleton

Goal:
- produce a stable, measurable realtime shell

Deliverables:
- transport
- scheduler
- session state
- action log
- snapshots baseline
- Jam-screen skeleton
- stable audio callback

### Phase D - Analysis vertical slice

Goal:
- make one track analyzable and visible inside the instrument

Deliverables:
- file load
- decode / normalize
- beat / bar grid
- automatic BPM estimate with confidence
- downbeat and phrase-grid candidates with confidence
- multiple timing hypotheses for half-time, double-time, and ambiguous material
- explicit timing warnings when the detected grid is weak or ambiguous
- source-grid drift report proving rendered lanes stay aligned to the analyzed grid
- all-lane timing proof for TR/Kick-Bass, MC-202, and W-30 using source-vs-output audio evidence
- P012 all-lane proof includes the Recipe 15 real-source Feral-grid auto/fallback
  contract, so cautious auto-BPM and conservative fallback behavior stay visible
  in the same phase-level gate; missing required Recipe 15 source fixtures must
  fail that phase-level gate rather than silently skipping the real-source proof
- P012 all-lane proof leaves a compact local summary artifact under
  `artifacts/audio_qa/local/` so reviewers can see the source-timing auto,
  fallback, and lane-metric outcomes without scraping command logs
- sections
- first slice / loop candidates
- Source Graph v1
- sidecar RPC path
- Jam / Source surface shows current beat, bar, phrase, tempo confidence, and degraded timing state when applicable
- adaptive Source Map that derives energy / peak buckets from decoded source
  audio, projects them through a usable timing hypothesis, and falls back to a
  time-uniform view when the grid is not usable
- user-confirmed timing hypothesis state after source audition, represented as a
  real action / session / replay contract rather than UI-local trust

### Phase E - Jam-first playable slice

Goal:
- make Riotbox feel like an instrument, even if still limited

Deliverables:
- Jam screen shows real values
- quantized action commit flow
- first capture path
- visible pending actions
- undo for recent actions
- source monitor mode for transport listening: `source`, `blend`, and `riotbox`
- bar and phrase seek controls that move the transport while preserving play /
  pause state
- musician-facing capture length selection for `1 bar`, `4 bars`, and `phrase`,
  with visible fallback when phrase evidence is unavailable
- capture -> raw audition -> promote -> hit flow that can be followed from a
  real source file

### Phase F - Device MVPs

Order:
1. TR-909 MVP
2. MC-202 MVP
3. W-30 MVP

Reason:
- 909 makes rebuilds immediately audible
- 202 adds identity and pressure
- W-30 becomes most valuable once grid, actions, and capture semantics are already stable

TR-909 MVP must explicitly include a punch / kick-reinforcement slice:
- kick body, click, tail, and pitch-envelope shaping
- source-kick plus 909-kick layering without destroying the source downbeat
- drum-bus saturation, drive, compression, and room pressure
- audible `reinforce`, `fill`, `slam`, and `takeover` contrast in both offline renders and the TUI path
- low-end and transient QA for peak, RMS, low-band RMS, crest factor, onset density, and source-grid drift
- fallback behavior when timing confidence is too weak to layer punch safely

### Phase G - Scene Brain

Goal:
- move from pattern generator to musical scene system

Deliverables:
- Scene Graph
- energy model
- strip / build / slam logic
- restore logic
- scene launch / recovery

### Phase H - Feral policy layer

Goal:
- add feral behavior without destabilizing core architecture

Deliverables:
- feral scoring
- break rebuild policy
- source-derived rebuild policy:
  - use the source beat grid, anchors, sections, transients, and candidates as musical reference material for a new Riotbox pattern
  - do not default to merely playing the original beat with generated layers on top
  - preserve downbeat kick identity or backbeat snare logic only when a mode explicitly promises anchor preservation
  - allow destructive, rebuilt, or replacement behavior when the gesture / scene / aggression level makes that intent explicit
  - keep MC-202, TR-909, W-30, and bass mutations aligned to the detected source grid even when the audible beat is newly generated
  - fail soft or require confirmation when timing confidence is too low for grid-locked rebuilds
- hook-fragment logic
- abuse-mix policy
- rebake / promotion logic

### Phase I - Ghost Watch / Assist

Goal:
- add useful, bounded agent behavior

Deliverables:
- Ghost tool schema
- budgets and locks
- watch / assist flows
- explainable action logs

`perform` is allowed only after watch / assist is stable and replay-safe.

### Phase J - Pro hardening

Goal:
- make Riotbox reliable enough for repeated performance use

Deliverables:
- deterministic replay
- crash recovery
- long-run soak tests
- benchmark baselines
- regression renders
- export consistency

---

## 6. Current Implementation Direction

The original immediate build sequence has largely landed: core contracts, Jam / Log / Source / Capture shells, device MVP seams, Ghost Watch / Assist, Feral policy, and replay/recovery hardening slices now exist in bounded form. The current active direction should be read with `docs/phase_definition_of_done.md`, the current reviews, and the active plans.

Near-next roadmap-aligned work:

1. treat `P011 | Pro Hardening` as closed for bounded MVP-spine hardening after the 2026-05-10 aggregate evidence gate
2. treat `P012 | Source Timing Intelligence` as closed for the bounded
   foundation phase after the 2026-05-28 all-lane source-grid output proof
3. treat `P013 | All-Lane Musical Depth` as closed for bounded representative
   showcase depth after the 2026-05-29 exit evidence review
4. treat `P014 | Arrangement / Scene System` as closed for bounded Arrangement /
   Scene behavior after the 2026-05-31 exit-candidate evidence and final stack
5. treat `P015 | Productization Alpha` as closed for bounded Jam
   productization after the 2026-05-31 exit review
6. keep the P011 aggregate gate, P012 all-lane source-grid proof, P013
   representative musical-quality gate, P014 scene-movement observer/audio
   proof, and P015 Jam taste/proof gate green as regression baselines while
   P016 starts from the existing product spine

Linear project / phase map:

1. `P011 | Pro Hardening` - closed bounded MVP-spine hardening baseline
2. `P012 | Source Timing Intelligence` - closed bounded Post-MVP timing foundation
3. `P013 | All-Lane Musical Depth` - closed bounded representative musical-depth baseline
4. `P014 | Arrangement / Scene System` - closed bounded Arrangement / Scene
   exit
5. `P015 | Productization Alpha` - closed bounded Productization Alpha baseline
6. `P016 | Pro Workflow / Export` - active immediate track
7. `P017 | Live Performance Readiness`
8. `P018 | Ghost + Feral Autonomy Expansion`
9. `P019 | Beta / Release Hardening`
10. `P020 | Riotbox 1.0 Release Cut`
11. `P021 | Audio Judge / Musical Fitness` - planned calibrated audio-quality
   judging track
12. `P022 | Professional Sound Output` - active professional-output proof
    track for source-backed rendered examples, listening verdicts, and weak
    output regression fixtures
13. `P023 | Sound Excellence / Production Quality` - planned 10/10
    sound-product track for release-grade musical output, demo quality, and the
    failure-to-fix loop beyond technical audio correctness

This is a project / phase overview, not a ticket list. Keep P012, P013, P014,
and P015 as regression baselines and keep P017-P020 coarse while P016 turns the
bounded product spine toward pro workflow and export behavior. P021, P022, and
P023 are the explicit quality ladder:

- P021 makes the technical audio-QA and musical-fitness judge calibrated enough
  to reject silence, clipping, fallback collapse, repeated placeholders, weak
  contrast, and other measurable regressions.
- P022 proves professional source-backed output through rendered examples,
  review packs, human verdict import, and negative fixtures that catch weak
  sound before it becomes a demo or PR claim.
- P023 raises the instrument itself to release-grade sound-product quality:
  compelling source transformation, memorable hooks, physical drums, bass
  pressure, destructive contrast, musician-useful demos, and concrete
  failure-to-fix loops.

P021 must not replace human listening or become a hidden taste oracle without
labeled Riotbox examples. P022 and P023 must not bypass Source Graph, Session,
Action Lexicon, queue / commit, replay, or existing audio-QA contracts.

### Product Intelligence Contract

This applies to every lane and product surface, not only MC-202.

Riotbox must distinguish scaffolding from intelligence. Hardcoded phrases,
fixed templates, scripted arrangements, fingerprint-only variation, and
source-aware mutations are allowed as diagnostics, controls, or architecture
proofs, but they do not count as product-quality source-derived behavior.

A feature may claim source-derived or intelligent behavior only when it proves:

- source evidence: real source timing, transients, low-band pressure, density,
  section role, hook/restraint context, slice identity, or captured audio
  material shaped the decision
- musical decision: Riotbox chose role, placement, density, contour, silence,
  destructive gesture, or arrangement move from that evidence
- product spine: the decision is represented through Source Graph, Session,
  Action Lexicon, queue / commit, replay, or the relevant documented contract
- audible consequence: the rendered output changes in a musician-audible way
- quality proof: same-source reproducibility and cross-source diversity are
  tested, with hardcoded/scripted artifacts labeled as non-quality proof

Reason:
- Replay/recovery/export reliability now has a bounded P011 regression gate.
- Source Timing Intelligence now has a bounded P012 regression gate for
  source-derived BPM, beat, downbeat, bar, phrase, drift, groove, confidence,
  degraded policy, Source Map, and all-lane output alignment.
- P013 now has a bounded representative showcase baseline for TR-909 source
  accent dynamics, W-30 source accent/slice/chop proof, MC-202 pressure/grid/
  source-contour proof, and all-lane generated-support mix movement.
- RIOTBOX-1035 reopens the documented MC-202 follow-up as a bounded source
  phrase planning track, anchored in
  `docs/plans/mc202_source_phrase_planning_plan.md`; it must promote primitive
  MC-202 pressure/answer support into source-derived phrase plans without
  weakening the P012 timing spine or P013 regression baseline.
- P014 now has a bounded Arrangement / Scene baseline for manual scene launch /
  restore movement through Session, replay, observer, and non-collapsed output
  proof.
- P015 now has a bounded Productization Alpha baseline for Jam taste/proof,
  first-run next-move guidance, Recipe 16, Help overlay readability, and
  explicit product-scope deferrals.
- Source Graph, session, actions, audio core, TUI, and QA contracts remain the controlling architecture; new timing or arrangement behavior must extend them instead of creating a parallel analysis or arranger path.

---

## 7. Parallel Work Rules

### Safe parallel work

- contract writing and fixture-corpus definition
- analysis-provider comparison and latency spikes
- benchmark harness setup
- golden-render harness setup
- TUI screen detail work after action concepts are stable

### Unsafe parallel work

- implementing Ghost tools before action schema is stable
- building feral-specific persistence before session spec exists
- deep W-30 behavior before capture semantics and action undo rules are fixed
- export-heavy workflows before deterministic replay is proven

---

## 8. Critical Path by Discipline

### Product

- PRD v1
- phase priorities
- acceptance criteria
- scope control

### Realtime

- audio callback
- scheduler
- timing model
- state handoff
- commit safety
- grid-lock safety: rendered and scheduled bass, MC-202, TR-909, and W-30 events must use the same timing authority as the source beat grid once that grid is trusted

### MIR / ML

- analysis provider selection
- confidence model
- Source Graph generation
- candidate scoring
- source beat-grid detection:
  - BPM estimate
  - beat, downbeat, bar, and phrase-grid candidates
  - confidence and warning model
  - drift detection between source anchors and generated lane events

### TUI / Interaction

- Jam-first information hierarchy
- action visibility
- pending vs committed state
- shortcut model
- timing trust visibility: Jam / Source should show beat, bar, phrase, BPM confidence, and degraded timing state before the user commits timing-sensitive gestures

### QA

- fixture corpus
- regression flows
- deterministic replay checks
- golden renders
- performance acceptance
- source-grid audio QA for bass and generated lane drift

---

## 9. Research Program

Research should be bounded and decision-oriented.

### Research item 1 - Audio latency and callback stability

Questions:
- what output path is acceptable on target laptops?
- what callback size and sample-rate combinations remain safe?
- which callback statistics must be recorded from day one?

Expected output:
- chosen baseline backend settings
- benchmark numbers
- rejection of unsafe defaults

### Research item 2 - Analysis provider bakeoff

Questions:
- which provider combination is acceptable for beat / bar / section / slice candidate quality?
- what confidence failures must be surfaced instead of hidden?
- how accurately can the provider detect BPM, beat, downbeat, bar, and phrase grid from unlabeled user audio?
- how do we detect and report when the generated Riotbox grid starts drifting against the source beat?
- what fallback behavior is safe when the beat grid is plausible but not trustworthy enough for bass or destructive mutation?

Expected output:
- baseline provider set
- failure modes
- fallback strategy
- fixture-backed BPM / downbeat tolerance thresholds
- source-grid drift metric and acceptance budget

### Research item 3 - Sidecar RPC and fault isolation

Questions:
- how are jobs queued?
- how are timeouts surfaced?
- what happens if the sidecar disappears mid-session?

Expected output:
- one transport choice
- one timeout policy
- one crash-recovery policy

### Research item 4 - Deterministic replay

Questions:
- what exactly must be logged?
- which state is recomputed vs persisted?
- what makes replay fail?

Expected output:
- replay-critical state list
- action log requirements
- deterministic replay acceptance test

---

## 10. Testing Program by Stage

### Stage 1 - Core skeleton

- startup / shutdown tests
- scheduler timing tests
- state transition tests
- save / load smoke tests

### Stage 2 - Analysis slice

- decode fixture tests
- beat / bar / section fixture tests
- BPM estimate tolerance tests
- downbeat-confidence fixture tests
- low-confidence timing fixture tests
- source-grid drift fixture tests that fail when generated bass or lane events audibly walk away from the source beat
- candidate reproducibility tests
- confidence-report tests

### Stage 3 - Jam slice

- pending-action visibility tests
- quantized commit tests
- capture action tests
- undo tests

### Stage 4 - Device MVPs

- TR-909 reinforcement behavior tests
- TR-909 punch-engine tests:
  - kick body / click / tail shaping changes the rendered buffer
  - source-kick plus 909-kick layering increases punch without masking the downbeat
  - drum-bus drive / compression / room settings produce measurable but bounded changes
  - `reinforce`, `fill`, `slam`, and `takeover` produce distinct audible outputs
  - low-band RMS, transient density, crest factor, and source-grid drift stay inside explicit budgets
- MC-202 follower and phrase mutation tests
- W-30 capture and pad reuse tests

### Stage 5 - Scene Brain and Feral

- scene transition tests
- variation-rate tests
- source-beat anchor preservation tests:
  - downbeat kick remains readable at low / medium aggression
  - backbeat snare logic remains readable at low / medium aggression
  - destructive behavior is explicit rather than accidental
- quote-risk and repetition checks
- capture-yield checks

### Stage 6 - Ghost

- action budget tests
- lock-respect tests
- explainability format tests
- replay safety under accepted Ghost actions

### Stage 7 - Hardening

- soak tests
- sidecar crash tests
- replay regression tests
- export reproducibility tests

---

## 11. Benchmark Program

Benchmarks are required from the first implementation slice onward.

### Realtime benchmarks

- callback timing
- xruns
- buffer underruns
- CPU peak
- memory growth
- action queue lag

### Analysis benchmarks

- analysis time per track
- memory consumption per job
- sidecar job latency
- candidate count stability
- BPM / beat-grid accuracy against fixture expectations
- downbeat confidence quality against fixture expectations
- source-grid drift budget for generated lane overlays

### Workflow benchmarks

- time to first playable Jam state
- time to first successful capture
- session save / load latency
- replay completion time

### Product-quality benchmarks

- capture yield per run
- variation density
- quote-risk ceiling
- feral scorecard consistency
- TR-909 punch quality:
  - kicks feel louder, deeper, and more forward without clipping
  - source downbeat remains readable unless destructive mode is explicit
  - `reinforce`, `fill`, `slam`, and `takeover` are distinguishable by ear and metrics
  - low-end energy improves without bass / kick drift against the source grid
- source-derived rebuild quality:
  - Riotbox creates a new musical result from source-derived timing, anchors, sections, transients, and candidates
  - original-beat preservation is required only in modes that explicitly promise anchor preservation
  - generated bass and device lanes stay grid-locked even when the audible beat is rebuilt or replaced
  - TUI makes tempo confidence, preservation mode, destructive mode, and timing degradation visible before the user commits a timing-sensitive gesture

---

## 12. Completion Gates

No phase is complete when code merely exists. A phase completes only when:

- the phase output exists
- required tests pass
- required benchmarks were run
- the Definition of Done document is satisfied
- findings were recorded in the research / decision log where applicable

---

## 13. 1.0 / Stage-Ready Product Readiness

Riotbox approaches a stage-ready 1.0 release only when all of the following are true:

- a user can move through the full product spine reliably
- replay is deterministic enough to trust saved work
- Jam mode is musically useful without deep study
- Ghost watch / assist is helpful and safe
- feral mode behaves as policy, not as architecture drift
- exports are reproducible
- long-run and crash tests are acceptable
- benchmark regressions are visible and actionable

### 10/10 Sound Product Gate

Riotbox is not a 10/10 sound product merely because it renders valid audio.
The versioned readiness contract lives in
`docs/specs/sound_product_readiness_rubric_spec.md` and the machine-checkable
fixture `scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json`.
The release-grade sound path is clear only when P022 and P023 prove all of the
following:

- real-source output regularly earns structured human `pass` verdicts across
  dense breaks, sparse drums, tonal riffs, pad / noise material, weak sources,
  and bad-timing sources
- a strong output has an identifiable hook within two bars: stab, riff, break,
  bass gesture, vocal hit, silence cut, or destructive transition
- the hardest musical element is obvious by ear and backed by metrics:
  snare / break transient, kick body, bass pressure, stab, or silence impact
- source character survives as transformed material instead of collapsing into
  fallback tones, polite pattern generation, or placeholder loops
- destructive gestures such as choke, stop, reverse, retrigger, pitch dive,
  filter slam, bitcrush, dropout, and restore produce immediate room-changing
  contrast
- `pressure_lift` and related rise / restore gestures are source-aware
  production decisions, not fixed bar-position recipes: dense breaks, sparse
  drums, tonal riffs, pad / noise material, weak sources, and bad-timing sources
  must be allowed to choose different lift shapes, bass movement, drum pressure,
  source treatment, and destructive contrast
  - current P022 diagnostic coverage exposes a `pressure_lift_policy` for dense
    break, tonal-hook, and sparse-bass-pressure sources plus a bounded
    `arrangement_policy` that derives first-six-bar hook/chop/pressure role
    placement from source/W-30 section candidates, compares against the old
    source-family scripted role order, and can produce different 8-bar
    role-order signatures across source families; it also exposes a bounded
    `mix_treatment_policy` that derives bus drive/slam/gain treatment from
    source/W-30 energy candidates and rejects collapse back to the old fixed mix
    recipe; eligible sources now also expose a bounded `tail_shape_policy` that
    derives dropout silence, stutter density/gain, and restore weight/drive from
    source/W-30 candidates and rejects collapse back to the old fixed tail
    recipe; role vocabulary remains bounded and
    `human_verdict: unverified`
  - current P022 diagnostic coverage also exposes a bounded
    `strongest_audible_element` proof across dense, matrix, professional
    source-WAV, and edge diagnostic reports. The proof labels whether kick,
    snare/break, bass pressure, stab/texture, silence, or restore impact is
    currently carrying the output, with score and margin gates that reject
    missing or ambiguous evidence while keeping the artifact diagnostic-only.
  - current P023 dense-break diagnostics gate physical drum pressure by
    requiring snare/break to be the strongest audible element and by reporting
    bounded snare-margin and combined drum-pressure scores; this protects
    harder break impact without claiming a musical pass.
  - current P022 diagnostic coverage also writes rebuild-only/source-layer-off
    professional-output WAVs and gates them for non-silence, useful RMS,
    source correlation, distinct source-on/source-off output, and a bounded
    source-character survival score based on rebuild-only spectral,
    transient, and RMS retention; this proves the current diagnostic render is
    not passing only because raw source bleed masks weak Riotbox generation and
    also catches generic fallback collapse, but it still keeps
    `quality_proof: false`
  - dense-break and tonal-hook diagnostics now derive W-30 hook/chop grain
    selection from scanned source/W-30 candidates, expose distance from the old
    static first-bar choice and hook/chop offset contrast, and reject collapsed
    static hook selection while still treating the scripted render as diagnostic
    only; current P023 coverage also requires stronger W-30/source hook
    presence and scores selected hook/chop/riff windows for source-character
    floor and variation so weak or too-narrow source-window choices route back
    to source selection instead of being hidden by later mix or routing reports
  - RIOTBOX-1295 strengthens that Hook/Chop path from selected offsets into
    audible riff playback proof: dense-break and tonal-hook diagnostics now
    require a source-derived hit pattern, minimum hit density, velocity
    contrast, reverse-hit presence, and suite-level Matrix/Source-WAV
    aggregation. Pad/noise and bad-timing source families are kept off the W-30
    hook-riff path and use texture/timing-cue behavior instead, preventing
    family-inappropriate Hook/Chop fallback from masking weak outputs.
  - RIOTBOX-1298 tightens source-first mix-bus masking proof by aligning
    automated-fitness and representative showcase source-first limits with the
    professional-suite `0.16` ceiling and adding a suite-level `0.09` masking
    headroom gate, so generated support cannot barely pass while still risking
    source-character burial.
  - dense-break and tonal-hook diagnostics now also derive destructive
    dropout/stutter/restore cue selection from scanned source/W-30 candidates,
    expose distance from old fixed destructive choices and stutter/restore offset
    contrast, and reject collapsed fixed destructive gestures while still
    treating the scripted render as diagnostic evidence
  - RIOTBOX-1297 strengthens destructive-gesture diagnostics from broad
    dropout/stutter contrast into explicit cut-and-restore impact proof:
    destructive reports now require the dropout silence to stay low against the
    stutter and the restore hit to slam back out of the cut, with rendered weak
    fixtures and suite mutation checks covering flat edits.
  - sparse-bass-pressure diagnostics now derive bass-pressure movement from the
    source low-band envelope and timing centroid, expose fixed-contour distance
    and frequency-span proof, require bass-led low-band pressure and a clear
    strongest-element dominance margin, and reject collapsed fixed movement
    while still treating the scripted render as diagnostic evidence
  - RIOTBOX-1296 strengthens sparse-bass-pressure diagnostics against
    midrange-phrase collapse: sparse reports now carry pressure-section
    low-band share and low/mid dominance gates through Source-WAV, Matrix, and
    professional-output suite validation, so a moving line cannot count as bass
    pressure unless the low band actually carries the section.
  - the MC-202 source phrase planning track must turn that diagnostic bass
    movement into replayable Session/Core phrase plans before MC-202
    bass/answer behavior can count as source-derived product behavior; the
    implementation plan lives in
    `docs/plans/mc202_source_phrase_planning_plan.md`
  - RIOTBOX-1278 adds a dense/non-dense MC-202 real-source listening pack that
    renders source windows, MC-202 stems, generated-support mixes, expression
    summaries, selected motif metadata, and non-product primitive A/B control
    evidence while keeping `human_verdict: unverified` and
    `quality_proof: false`
  - RIOTBOX-1279 adds `just mc202-producer-grade-closeout-smoke` as the
    MC-202 closeout boundary: dense and non-dense source-composed candidates
    can be technically reviewable, but `producer_grade_promotion_result` stays
    `blocked_for_human_promotion`, demo-bank promotion stays false, and
    RIOTBOX-1264 stays open until structured listening records human verdicts;
    primitive/template-only candidates remain production blockers, not proof
  - RIOTBOX-1285 lifts the tonal-hook MC-202 review candidate out of the
    primitive/template-only blocker by increasing source-derived Hold-contour
    bass support and tonal restore balance; dense, tonal, and sparse MC-202
    candidates are now all source-composed review candidates, while human
    verdicts still block producer-grade/demo-bank promotion
  - edge-source diagnostic coverage now includes pad/noise and bad-timing
    sources as weak/risky routed cases: source-timing, rendered WAV, metrics,
    source-family metadata, and concrete fix routing are present; pad/noise now
    takes an explicit `pad_noise` pressure-policy path plus a source-derived
    gated texture/stab policy instead of dense-break promotion, and bad-timing
    now takes an explicit `bad_timing` cautious arrangement / user-confirmation
    path that rejects confident bar-locked policy while timing is
    `candidate_ambiguous` / `manual_confirm_only`; in both cases
    `quality_proof: false` remains mandatory
- generated demo packs include curated source, rendered WAV, metrics, review
  prompt, human verdict, and a short reason why the example is demo-worthy or
  not demo-worthy yet
- the release-grade demo-bank contract is versioned in
  `docs/specs/release_grade_musician_demo_bank_spec.md` and validated by
  `scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json`
- source-family release-demo coverage must compare the P023 real-source corpus
  with the demo bank and block release-ready claims while any source family
  lacks a demo candidate, human verdict, or demo-ready human-pass entry; the
  gate must remain CI-safe without ignored local WAV files
- hardcoded or scripted audio generation may be used only as smoke, regression,
  or diagnostic evidence; it must not be presented as technical or musical
  quality proof until the relevant behavior is source-aware, policy-owned, and
  covered by non-hardcoded fixture evidence
- source-character survival proof must include real negative audio evidence,
  not only report mutation checks; the dense-break weak source-character smoke
  renders a weak rebuild-only WAV and requires
  `rebuild_only_source_character_not_surviving`
- RIOTBOX-1299 tightens source-selection proof with
  `rebuild_only_source_character_survival_margin >= 0.10` across dense,
  matrix, source-WAV, edge, professional-suite, and readiness-report
  diagnostics, and the weak-source WAV fixture must now fail both the survival
  floor and margin gates.
- RIOTBOX-1300 tightens hook/chop proof with W-30 headroom gates:
  generated dense/tonal professional diagnostics must keep
  `hook_chop_w30_to_source_margin >= 0.025`, tonal-hook fixture diagnostics
  must keep `w30_contribution_margin >= 0.050`, and the tonal generated path
  raises source-derived W-30 presence instead of accepting barely audible hook
  contribution.
- RIOTBOX-1301 tightens MC-202 source-composed bass movement proof at the
  render seam: SubPressureShove projection now carries bass body from the
  stored source-expression low-band contour / pressure evidence, pressure-vs-
  hook source-family tests require a clear bass-articulation margin plus
  stronger rendered low-band RMS/share, and neutralized source evidence must
  remain silent instead of leaking fallback MC-202 output.
- RIOTBOX-1302 moves MC-202 pressure composition beyond a fixed two-hit
  contour: SubPressureShove now derives root, secondary, and optional movement
  steps from source-expression low-band pressure / movement evidence, records
  the pressure-movement groove step in provenance, and tests high-vs-low
  low-band-movement pressure sources with the same source fingerprint basis for
  different phrase cells and rendered output.
- RIOTBOX-1303 makes MC-202 candidate selection use an explicit
  production-impact score: low-end impact, source-grid confidence, answer
  contrast, hook avoidance, phrase memory, destructive usefulness, and role fit
  now shape the winning candidate and are recorded in selected-candidate
  provenance, with pressure-vs-pickup regression coverage proving different
  source evidence chooses different phrase families and rendered output.
- RIOTBOX-1304 strengthens the MC-202 bass-pressure render seam for
  low-dominant sparse/drop sources: source-derived low-band dominance now
  applies a measured body-emphasis pass, deeper pressure reinforcement, and
  manifest provenance while tonal/hold material remains unboosted, so bass
  pressure is less likely to read as a midrange melody.
- RIOTBOX-1305 tightens W-30 hook/chop source-character policy for the first
  two bars: source-derived riff starts now prefer stronger character contrast
  when top-scoring candidates cluster too tightly, tonal-hook riff playback is
  brought forward without claiming quality proof, and dense/sparse pressure
  gates remain intact.
- RIOTBOX-1306 strengthens destructive gesture impact for stage-meaningful
  cuts: source-derived tail shaping now drives a deeper dropout silence, denser
  transient stutter, and slightly harder restore slam, while the professional
  destructive validator rejects the older flatter cut depth and keeps the
  evidence diagnostic-only until human listening accepts it.
- RIOTBOX-1307 tightens source-first mix-bus clarity: the Feral source-first
  path now uses less generated bleed and a stronger W-30/source weight, while
  professional-suite/readiness/family validators reduce the generated/source
  masking ceiling from `0.16` to `0.08` so generated support can hit hard
  without hiding transformed source identity.
- RIOTBOX-1308 starts source-window character selection before output
  promotion: Feral now scores real source windows from the selected search
  range, records the requested-vs-selected source-character window in reports
  and manifests, and keeps suite-level diagnostics for scanned/promoted cases
  so weak-head / stronger-later source material can move the rendered source
  decision without adding fallback product sound or claiming human quality.
- RIOTBOX-1309 strengthens TR-909 drum pressure in the rendered support path:
  the generated-support mix shifts more weight onto source-derived TR-909 body
  while reducing MC-202 support weight enough to preserve the `0.46`
  generated/source ceiling, and Feral reports now carry
  `tr909_rendered_drum_pressure` so kick/accent/source-grid evidence, support
  contribution, source-first headroom, and low-band output must all pass before
  drum pressure counts as surviving the rendered mix.
- RIOTBOX-1310 tightens weak-output promotion boundaries without overfitting
  tonal fixtures to dense-break thresholds: TR-909 rendered-pressure proof now
  derives support-contribution and low-band floors from the source-aware TR-909
  profile, `break_lift` keeps a transient/snare-appropriate low-band floor
  separate from stricter `drop_drive`, continuous tonal/high material stays
  `steady_pulse` unless it has real transient density, and the professional
  suite validates each Feral case against its manifest-recorded floors so weak
  evidence fails for the intended musical reason instead of being hidden by
  stale aggregate constants.
- RIOTBOX-1311 exposes source/timing confidence risk as a musician-visible
  cue on the existing Trust / Confidence surfaces: the cue derives from Source
  Graph confidence plus Source Timing degraded policy / grid use, distinguishes
  `trusted`, `degraded`, and `unavailable`, and tells the player whether grid
  moves are performable, should be confirmed/listened first, or need source
  recapture/loading.
- weak outputs create concrete follow-up work: source selection, chop policy,
  drum pressure, bass movement, mix-bus treatment, destructive gesture policy,
  fixture threshold, or UI cue
- weak-output reports must include a concrete proposed fix category and short
  musician-facing reason; unknown weak/fail codes must fail routing instead of
  silently landing in a generic bucket
- weak-output routing should also group routed failures into bounded P023
  production-fix candidates with artifact refs, software next step, and
  musician payoff, while keeping every candidate diagnostic-only
- professional listening-review packs must say why each candidate is worth
  review and why it is not demo-ready yet while `human_verdict` remains
  `unverified`
- P023 sound-quality readiness reports must aggregate rubric, source corpus,
  demo-bank, weak-output routing, and professional-output context into
  release blockers and next production fixes without acting as a hidden taste
  oracle or promoting scripted diagnostics to quality proof
- large professional-output JSON contracts move into named repo-local validators
  instead of growing opaque inline `jq` blocks in `Justfile`
  - RIOTBOX-1286 starts this extraction for the professional-output
    listening-pack and MC-202 producer-grade closeout gates: `Justfile` keeps
    the smoke commands short while Python validators preserve the same
    source-composed, no-primitive-template, and human-verdict-blocked contracts
  - RIOTBOX-1287 continues the extraction for edge-source and non-dense
    professional proof gates: validators own source-family coverage,
    diagnostic-only boundaries, rendered-artifact checks, mutation fixtures,
    weak-routing checks, and no-quality-claim safeguards for those reports
  - RIOTBOX-1288 continues the extraction for the professional-source WAV
    pack: its generator validate mode owns tonal/sparse source-family gates,
    source-aware policy checks, source-character thresholds, rendered-artifact
    requirements, mutation fixtures, and the diagnostic-only boundary
  - RIOTBOX-1289 continues the extraction for dense-break performance and
    agent musical review packs: the dense-break generator owns source-policy
    gates, physical drum-pressure checks, source-character survival,
    source-layer collapse mutations, and review visual-file validation while
    keeping the diagnostic-only/no-quality-proof boundary explicit
  - RIOTBOX-1290 makes broad professional-output QA runs artifact-race
    resistant: `just audio-qa-ci` acquires a repo-local broad-audio-QA lock
    before touching shared `artifacts/audio_qa/local-*` outputs, `just ci`
    inherits that protection, and concurrent broad runs fail clearly instead
    of deleting another run's artifacts
  - RIOTBOX-1292 keeps primitive-renderer lane and Feral-grid packs as
    non-product controls: the existing manifest boundary already blocks
    product-output and quality-proof claims, and the listening-manifest
    fixtures now also reject product-output-enabled or stale affected-path
    primitive boundaries
  - RIOTBOX-1293 requires source-derived TR-909 support evidence before
    representative showcase or automated-musical-fitness reports can pass:
    kick pressure must carry `pattern_origin: source_derived`, the
    `tr909_source_profile_and_accent_dynamics` evidence role, a `source_*`
    profile reason, enough pressure anchors, and source-derived accent dynamics
    with distinct accent levels and sufficient span. Generated-pack smoke tests
    assert the same fields so decorative or fixture-only TR-909 support cannot
    be promoted as product-quality drum pressure.
  - RIOTBOX-1294 makes persisted MC-202 source phrase plans replay truth: core
    replay now restores `ActionCommitRecord.mc202_source_phrase_plan` for
    MC-202 phrase actions, clears stale plans when older commit records lack a
    trusted plan, and the session save/load sample covers both lane-state and
    commit-record persistence. This protects restore from falling back to
    primitive or stale MC-202 material when source-composed plans are absent.
  - RIOTBOX-1295 requires W-30 Hook/Chop diagnostic output to prove the audible
    riff pattern itself is source-derived, not only the selected source window:
    dense-break/tonal reports carry source-derived hit-pattern, hit-count,
    velocity-span, and reverse-hit gates through dense, matrix, source-WAV, and
    professional-output suite validation while keeping all artifacts
    diagnostic-only.
- no PR, roadmap claim, or release note presents an audible feature as
  musician-ready when its human verdict is missing, weak, or explicitly
  unverified

### 10/10 Technical Audio-QA Gate

Riotbox is not a 10/10 technical audio-QA system merely because unit tests and
basic renders pass. The technical QA path is clear only when P021 and the
release-hardening phases prove all of the following:

- offline renders and live-path seams are checked for silence, near-silence,
  clipping, DC offset, channel imbalance, fallback collapse, identical-output
  collapse, timing drift, and unintended loudness swings
- golden, negative, and weak-output fixtures cover the major source families
  and failure classes used by the sound-product gate
- technical audio-QA distinguishes render-path smoke evidence from quality
  proof: hardcoded or scripted generation can prove that a harness runs and
  catches known regressions, but cannot prove production quality by itself
- audio judge thresholds are calibrated against Riotbox-owned human labels and
  report their confidence and known blind spots
- each audible ActionCommand remains covered through queue, commit /
  side-effect, Session / replay, observer, and output-QA surfaces
- deterministic replay and export reproduce the same intended musical result,
  not only the same action log
- real-session evidence records host, backend, device, latency, xrun, and soak
  provenance separately from sandbox-only execution
- CI blocks regressions that would produce silent, fallback-only, clipped,
  identical, rhythmically collapsed, or clearly weaker rendered output
- quality reports are useful to engineers and musicians: they explain what got
  worse, which artifact to hear, and which concrete follow-up category should
  fix it

### 20/10 Future-Idea Track

10/10 is the release gate. 20/10 is a deliberately separate idea track for
making Riotbox exceptional after the product can already be trusted. Capture
20/10 ideas without letting them obscure the 1.0 path. Candidate directions:

- an agent-assisted musical producer loop that compares multiple generated
  takes, listens through calibrated metrics plus human labels, and proposes the
  next concrete sound-design move instead of only reporting scores
- an owned source-to-performance index that remembers which source traits,
  chop policies, drum treatments, bass gestures, and destructive transitions
  produced strong human verdicts
- live resampling and self-abuse workflows where the player can capture
  Riotbox's own output, degrade it, retrigger it, and promote it into a new
  playable source without breaking replay
- musician-facing macro controls that reliably create stage-impact gestures:
  panic stop, pressure rise, dirty restore, break shred, bass punch, and
  source-mangle, all with predictable timing and undo
- taste-aware demo generation that searches for the best few takes, rejects
  boring ones automatically, and emits a listening pack with clear reasons
- cross-song or set-level performance memory where Riotbox preserves identity
  across a live set while still rebuilding each source into a new result
- optional ecosystem surfaces such as DAW export, stems, controllers, or
  external sync only after they preserve the 10/10 sound and QA gates

20/10 ideas should become Linear ideas or roadmap notes only when they identify
the musical payoff, the product spine they improve, the replay / realtime risk,
and the evidence required to prove they are more than impressive demos.

The versioned 20/10 contract lives in
`docs/specs/sound_product_2010_future_ideas_spec.md` and the machine-checkable
idea list lives in
`scripts/fixtures/sound_product_2010_future_ideas/ideas_v1.json`. That list is
validated in CI and explicitly keeps every idea non-blocking for the 1.0 sound
release gate unless a future issue promotes it into normal roadmap scope.

---

## 14. What Must Not Go on the Critical Path Yet

- full DAW export polish
- complex granular / spectral systems
- cloud AI
- plugin architecture
- full autonomous Ghost performance
- style-specific hardcoding outside policy / preset systems

---

## 15. Next Concrete Step

The immediate execution track is `P016 | Pro Workflow / Export`: start from the
P011 replay/recovery/export hardening baseline plus the P012, P013, P014, and
P015 product-spine proof surfaces, then add bounded pro workflow / export
behavior without weakening replay, timing, musical-depth, scene, or Jam
taste/proof regression gates.

As of 2026-06-05, product-quality work should prioritize the dense-break
8-bar source-backed performance Golden Path and the P021/P022/P023 quality
ladder before further TUI/export polish: the thing Riotbox delivers must sound
stronger before downstream workflow surfaces become valuable. P021 is the
future calibration layer for agent-assisted musical judgment; P022 captures
professional-output evidence and weak-output regressions; P023 owns the path to
10/10 sound-product quality and the 20/10 idea backlog. These tracks should
grow from real review packs, human labels, and reproducible audio artifacts,
not from generic metrics alone.

The old initial Core Skeleton sequence is complete enough that new work should
not restart from spec scaffolding. Use these live references instead:

- `docs/architecture_phase_map.md`
- `docs/reviews/p012_exit_review_2026-05-28.md`
- `docs/reviews/p013_exit_review_2026-05-29.md`
- `docs/reviews/p014_exit_candidate_review_2026-05-30.md`
- `docs/reviews/p015_exit_review_2026-05-31.md`
- `docs/specs/arrangement_scene_system_spec.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/specs/audio_qa_workflow_spec.md`
- `docs/specs/preset_style_spec.md`
- `docs/specs/tui_screen_spec.md`
- `docs/phase_definition_of_done.md`

The closed P014 stack built arrangement and scene behavior on the P012 timing
spine and P013 musical-depth baseline:

- introduce arrangement / scene behavior without weakening the P012 source-grid
  regression proof or the P013 representative musical-quality gate
- prove every audible arrangement or scene change with concrete metrics and
  non-collapsed audio evidence
- preserve fallback, manual-confirm, explicit-BPM, and locked-grid timing
  boundaries instead of pretending every source is grid-locked
- keep Jam / Source / observer timing language aligned through the shared Jam
  Source Timing summary whenever timing evidence is surfaced

P016 work may now proceed, but it must preserve the P015 bounded exit: no
automatic arranger, second Scene Graph, hidden replay/timing truth, product
taste oracle, host-audio soak claim, or full DAW/stem export claim without a new
Action Lexicon, Session/replay, observer, and output-QA contract.

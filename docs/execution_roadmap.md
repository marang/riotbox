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
4. keep the P011 aggregate gate, P012 all-lane source-grid proof, and P013
   representative musical-quality gate green as regression baselines while P014
   arrangement / scene work lands
5. start P014 from the existing Source Graph, Source Timing summary,
   observer/audio, lane-output, and representative musical-depth proof contracts
   instead of creating a second timing, arrangement, or mix truth
6. update specs and decision log when P014 changes the arrangement contract

Linear project / phase map:

1. `P011 | Pro Hardening` - closed bounded MVP-spine hardening baseline
2. `P012 | Source Timing Intelligence` - closed bounded Post-MVP timing foundation
3. `P013 | All-Lane Musical Depth` - closed bounded representative musical-depth baseline
4. `P014 | Arrangement / Scene System` - active next phase
5. `P015 | Productization Alpha`
6. `P016 | Pro Workflow / Export`
7. `P017 | Live Performance Readiness`
8. `P018 | Ghost + Feral Autonomy Expansion`
9. `P019 | Beta / Release Hardening`
10. `P020 | Riotbox 1.0 Release Cut`

This is a project / phase overview, not a ticket list. Keep P012 and P013 as
regression baselines and keep P015-P020 coarse until P014 proves arrangement /
scene expansion on top of the timing and musical-depth foundations.

Reason:
- Replay/recovery/export reliability now has a bounded P011 regression gate.
- Source Timing Intelligence now has a bounded P012 regression gate for
  source-derived BPM, beat, downbeat, bar, phrase, drift, groove, confidence,
  degraded policy, Source Map, and all-lane output alignment.
- P013 now has a bounded representative showcase baseline for TR-909 source
  accent dynamics, W-30 source accent/slice/chop proof, MC-202 pressure/grid/
  source-contour proof, and all-lane generated-support mix movement.
- Source Graph, session, actions, audio core, TUI, and QA contracts remain the controlling architecture; new timing behavior must extend them instead of creating a parallel analysis or arranger path.

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

The immediate execution track is `P014 | Arrangement / Scene System`.

The old initial Core Skeleton sequence is complete enough that new work should
not restart from spec scaffolding. Use these live references instead:

- `docs/architecture_phase_map.md`
- `docs/reviews/p012_exit_review_2026-05-28.md`
- `docs/reviews/p013_exit_review_2026-05-29.md`
- `docs/specs/arrangement_scene_system_spec.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/specs/audio_qa_workflow_spec.md`
- `docs/specs/preset_style_spec.md`
- `docs/specs/tui_screen_spec.md`
- `docs/phase_definition_of_done.md`

Near-next P014 slices should build arrangement and scene behavior on the P012
timing spine and P013 musical-depth baseline:

- introduce arrangement / scene behavior without weakening the P012 source-grid
  regression proof or the P013 representative musical-quality gate
- prove every audible arrangement or scene change with concrete metrics and
  non-collapsed audio evidence
- preserve fallback, manual-confirm, explicit-BPM, and locked-grid timing
  boundaries instead of pretending every source is grid-locked
- keep Jam / Source / observer timing language aligned through the shared Jam
  Source Timing summary whenever timing evidence is surfaced

Keep P015+ coarse until P014 proves arrangement expansion on top of the closed
P012 timing and P013 musical-depth foundations.

# Riotbox Source Timing Intelligence Spec

Version: 0.1
Status: Draft
Audience: core, audio, analysis, TUI, QA, Ghost

---

## 1. Purpose

This document defines the implementation-facing contract for Source Timing
Intelligence.

Source Timing Intelligence is the shared Rust-first timing surface that lets
TR/Kick-Bass, MC-202, W-30, Scene Brain, Feral policy, Ghost, replay, and QA
agree on where the source track's musical grid is, how trustworthy it is, and
how generated or rebuilt material should degrade when that grid is uncertain.

It turns the accepted plan in
`docs/plans/source_timing_intelligence_plan.md` into a contract that later code
slices can implement without inventing lane-specific timing systems.

---

## 2. Core Rule

The Source Graph is the only durable bridge between source analysis and
musical timing behavior.

Timing intelligence must be represented as bounded, serialized Source Graph
data before downstream lanes consume it.

Do not add:

- a lane-local beat-grid model
- provider-specific timing blobs in runtime state
- a Python-only runtime timing contract
- hidden timing authority in Ghost, TUI, or Feral policy
- analyzer decisions that cannot be replayed or inspected

---

## 3. Boundary With Audio Core

Source Timing Intelligence and the audio core solve different problems.

Source Timing Intelligence owns:

- source-derived BPM, beat, downbeat, bar, phrase, anchor, drift, groove, and
  confidence data
- multiple timing hypotheses and primary-hypothesis selection metadata
- degraded timing policy when confidence is weak or ambiguous
- replay-safe source timing provenance
- source-vs-output timing evidence for QA

The audio core owns:

- live transport position
- callback timing
- quantized commit boundaries
- bounded application of already-prepared actions
- realtime-safe lane rendering

Rules:

- analysis never runs inside the audio callback
- callback timing remains the live playback authority
- lanes may align generated events to Source Graph timing data, but they must
  still commit through the normal transport / queue / action boundary
- if Source Graph timing confidence is weak, lanes must degrade conservatively
  instead of pretending to be locked

---

## 4. Timing Contract Shape

The existing `TimingModel` should evolve from a single estimate into a
multi-hypothesis contract.

Target shape:

```text
TimingModel {
  bpm_estimate
  bpm_confidence
  meter_hint
  hypotheses
  primary_hypothesis_id
  quality
  warnings
  degraded_policy
}
```

`bpm_estimate`, `bpm_confidence`, and `meter_hint` remain compatibility fields
until all consumers read the richer contract directly.

### 4.1 Timing Hypothesis

```text
TimingHypothesis {
  hypothesis_id
  kind
  bpm
  meter
  confidence
  score
  beat_grid
  bar_grid
  phrase_grid
  anchors
  drift
  groove
  warnings
  provenance
}
```

`kind` should distinguish at least:

- `primary`
- `half_time`
- `double_time`
- `alternate_downbeat`
- `ambiguous`

Rules:

- plausible alternatives must remain visible instead of being discarded
- a selected primary hypothesis must be explicit
- low-confidence or ambiguous material must not silently upgrade to a
  high-confidence grid
- early probe-derived downbeat scoring may use onset/accent strength from the
  source timing probe, but weak or flat accents must still preserve alternate
  downbeat candidates instead of claiming a confident phase lock

### 4.2 Grids

Beat, bar, and phrase grids must be addressable by both time and musical index.

Minimum beat event:

```text
BeatGridEvent {
  beat_index
  time_sec
  confidence
  residual_ms
}
```

Minimum bar event:

```text
BarGridEvent {
  bar_index
  start_time_sec
  end_time_sec
  downbeat_confidence
  phrase_index
}
```

Minimum phrase event:

```text
PhraseGridEvent {
  phrase_index
  start_bar
  end_bar
  confidence
}
```

Rules:

- grids are source-analysis data, not live transport state
- downstream helpers may map source time to bar / phrase positions through the
  selected hypothesis
- missing grids are allowed, but absence must be visible through quality,
  warnings, and degraded policy
- early probe-derived phrase candidates may use 4-bar spans from stable bar
  timing evidence, but this must stay labelled as preliminary and must preserve
  `phrase_uncertain` when material is too short, downbeat scoring is ambiguous,
  or drift is high
- source timing QA reports must expose phrase-grid availability explicitly, not
  require reviewers to infer it from raw hypotheses. Early report statuses
  should distinguish unavailable primary timing, not-enough-material,
  ambiguous-downbeat, high-drift, and stable preliminary phrase evidence.
- source timing QA reports should also expose beat-period evidence behind a BPM
  candidate: candidate count, primary BPM/period, score, matched-onset ratio,
  median-distance ratio, alternate-candidate count, and whether the evidence is
  unavailable, weak, stable, or ambiguous. This is explanatory QA evidence, not
  permission for downstream lanes to treat the grid as production-locked.
- source timing QA reports should expose downbeat-phase evidence behind bar-grid
  selection: primary phase offset, score, alternate-phase count, and whether the
  evidence is unavailable, weak, stable, or ambiguous. This explains why a bar
  phase was selected without making it a separate timing authority.
- source timing QA may also expose a combined readiness report that folds
  candidate confidence, beat-period evidence, and downbeat-phase evidence into a
  compact status such as unavailable, weak, needs-review, or ready. This report
  is a QA / future-TUI summary only; it must not become a hidden timing source
  separate from the Source Graph timing hypotheses.

---

## 5. Source Anchors

Anchors are musical reference points extracted from the source. They help lanes
rebuild or answer the source without forcing the original beat to remain
audible.

Minimum anchor classes:

- `kick`
- `snare`
- `backbeat`
- `fill`
- `loop_window`
- `answer_slot`
- `capture_candidate`
- `transient_cluster`

Minimum anchor shape:

```text
SourceTimingAnchor {
  anchor_id
  anchor_type
  time_sec
  bar_index
  beat_index
  confidence
  strength
  tags
}
```

Rules:

- anchors may support source-derived rebuilds without playing the source as a
  continuous backing track
- lane policies decide whether to preserve, answer, replace, or destroy an
  anchor
- anchor confidence must affect lane behavior and UI warnings

---

## 6. Drift And Groove

Riotbox needs to know whether generated lanes stay aligned to the chosen
source grid.

Minimum drift report:

```text
TimingDriftReport {
  window_bars
  max_drift_ms
  mean_abs_drift_ms
  end_drift_ms
  confidence
}
```

Minimum groove residual:

```text
GrooveResidual {
  subdivision
  offset_ms
  confidence
}
```

Rules:

- drift must be measurable over 4, 8, 16, and 32 bar windows when the source is
  long enough
- early probe-derived timing candidates may emit 4/8/16/32-bar drift reports
  as soon as enough material exists for each window, but missing windows must
  mean "not enough material", not silent confidence
- groove data should preserve feel where possible instead of flattening every
  lane to a rigid grid
- output QA must check generated-lane drift against the selected source grid
  for timing-aware audio slices

---

## 7. Quality, Warnings, And Degraded Policy

Timing confidence must be user-visible and machine-checkable.

Minimum warning classes:

- `weak_kick_anchor`
- `weak_backbeat_anchor`
- `ambiguous_downbeat`
- `half_time_possible`
- `double_time_possible`
- `drift_high`
- `phrase_uncertain`
- `low_timing_confidence`

Minimum degraded policies:

- `locked`
- `cautious`
- `manual_confirm`
- `fallback_grid`
- `disabled`

Rules:

- `locked` requires high confidence and low drift for the selected use case
- current probe-derived auto-readiness may emit `locked` only when the primary
  BPM is finite, exactly one primary timing hypothesis exists, there are no
  timing warnings or alternate hypotheses, preliminary phrase evidence is
  present, and drift stays within the strict locked threshold of 35 ms
- `cautious` should reduce destructive lane suggestions
- `manual_confirm` is required when an action would make strong musical claims
  on ambiguous timing
- `fallback_grid` must be labelled as fallback, not presented as analyzed truth
- `disabled` is valid when the timing contract cannot safely support a lane

---

## 8. Lane Consumption Rules

All timing-aware lanes must consume the same Source Graph timing contract.

TR/Kick-Bass:

- align reinforcement, support hits, fills, and takeover accents to the primary
  timing hypothesis when confidence allows
- degrade conservatively when kick/downbeat confidence is weak
- prove low-end and transient alignment in audio QA

MC-202:

- place question/answer phrases in bar and phrase slots from the selected
  hypothesis
- use anchors as musical prompts, not as a requirement to quote source audio
- prove phrase timing and source-grid drift in generated output

W-30:

- derive chop, loop, capture, and pad candidates from transient, grid, and
  confidence data
- check loop closure and drift before presenting source-derived loops as stable
- expose weak candidate confidence instead of promoting blind captures

Scene Brain, Feral policy, and Ghost:

- may read timing only through Source Graph / view model surfaces
- must not create separate timing truth
- must explain degraded or rejected suggestions when timing is not trustworthy

---

## 9. Replay And Persistence

Timing data affects replay and restore, so it must be explicit.

Persistence rules:

- persist the selected timing hypothesis id
- persist enough hypothesis data for restore to avoid silent reanalysis drift
- persist the Source Graph hash or equivalent identity used by timing-aware
  actions
- keep compatibility fields stable until older consumers are migrated

Replay rules:

- a replayed timing-aware action must target the same selected hypothesis unless
  the session explicitly records a reanalysis / retarget decision
- if the Source Graph changes, replay must surface the mismatch instead of
  silently moving musical anchors
- timing warnings that affect action safety should be reconstructable from the
  persisted graph

---

## 10. QA Contract

Every Source Timing Intelligence slice must prove the control path and the
output path appropriate to its scope.

Control-path proof examples:

- serialization roundtrip for timing hypotheses
- primary-hypothesis selection tests
- degraded-policy tests
- lane projection tests showing that consumers read the Source Graph timing
  contract instead of local fallback timing

Output-path proof examples:

- source-vs-output drift metric
- onset / transient alignment check
- low-band attack offset check for TR/Kick-Bass
- MC-202 phrase-slot timing check
- W-30 loop-closure / chop-window timing check
- source-then-Riotbox A/B listening artifact for audible behavior changes

Minimum timing metrics:

- BPM error against synthetic or annotated fixtures
- beat hit rate within 35 ms and 70 ms
- downbeat accuracy
- phrase boundary agreement
- drift after 4, 8, 16, and 32 bars where applicable
- confidence calibration for weak and ambiguous fixtures

Rules:

- UI/log proof alone is not sufficient for audio-producing timing changes
- a slice that only adds contract/docs may use docs lint, grep checks, and
  targeted test commands, but follow-up implementation slices must add model or
  audio evidence
- if a proof seam is still aspirational, say so explicitly in the PR

---

## 11. Out Of Scope For The First Contract Slice

The first implemented analyzer step is a deterministic Rust skeleton that maps
fixture timing expectations into the `TimingModel` payload shape:

- `crates/riotbox-core/src/source_graph/timing_analysis.rs`
- `just source-timing-analyzer-skeleton-fixtures`

The first implemented evaluator step compares analyzer output back to fixture
expectations:

- `crates/riotbox-core/src/source_graph/timing_evaluation.rs`
- `just source-timing-fixture-evaluator`

The first implemented WAV probe step extracts deterministic onset-envelope
features from existing PCM WAV source loading:

- `crates/riotbox-audio/src/source_timing_probe.rs`
- `just source-timing-wav-probe`

The first implemented probe-diagnostics step maps those real-WAV input features
into a conservative `TimingModel` diagnostic:

- `crates/riotbox-core/src/source_graph/timing_probe_diagnostics.rs`
- rich onset evidence may only reach `Low` timing quality with
  `ManualConfirm`
- weak or silent evidence stays visibly degraded / disabled
- the diagnostic does not emit BPM, beat grids, bar grids, phrase grids, or a
  primary hypothesis

The first implemented BPM-candidate spike maps synthetic probe onset times into
candidate `TimingHypothesis` entries:

- `crates/riotbox-core/src/source_graph/timing_probe_candidates.rs`
- `just source-timing-bpm-candidates`
- it may emit a medium-confidence BPM candidate and beat grid for deterministic
  synthetic onset spacing
- it must preserve half-time / double-time alternatives when plausible
- downbeat and phrase confidence remain explicitly uncertain

The first beat-period scoring detector v0 scores candidate beat periods from
probe onset evidence before selecting a primary BPM candidate:

- candidate periods are derived from bounded onset-distance evidence
- primary selection is score-first but conservatively tie-broken toward the
  observed median adjacent onset distance
- half-time, double-time, and ambiguous period alternatives remain visible when
  their scores stay close to the primary
- hypothesis provenance includes
  `source-timing-probe.beat-period-score.v0`
- this is still a bounded onset-period scorer, not a production beat/downbeat
  detector

The preliminary downbeat ambiguity scaffold may score beat-phase candidates
inside the current meter and add `AlternateDownbeat` hypotheses when multiple
bar-start phases are similarly plausible. This is still not production downbeat
detection:

- `just source-timing-downbeat-ambiguity`
- alternate phases must stay visible instead of being silently collapsed
- the primary candidate may expose a shifted bar grid only when the bounded
  phase score is stronger
- `AmbiguousDownbeat` remains visible while scoring is only onset-presence based

Probe-derived anchors may now classify bounded musical grid roles inside a
primary timing hypothesis:

- stable bar-start evidence may classify beat-1 onsets as `Kick` anchors with
  `kick_anchor` and `downbeat` tags
- stable 4/4 beat-2 and beat-4 onsets may classify as `Backbeat` anchors with
  `backbeat_anchor` and `snare_style` tags
- ambiguous or weak downbeat evidence must keep onsets as `TransientCluster`
  anchors even when they are beat-aligned
- all classified probe anchors must preserve bar/beat indices when the onset is
  close enough to the selected grid, plus `anchor_classified_v0` provenance in
  tags
- this is source-timing evidence only; lane policies decide later whether to
  preserve, answer, replace, or destroy these anchors

The candidate confidence report summarizes this early detector state for QA:

- `source_timing_candidate_confidence_report(...)`
- `just source-timing-candidate-confidence-report`
- the report preserves BPM confidence, effective timing quality, degraded
  policy, hypothesis counts, alternate-downbeat counts, warning codes, and
  whether manual confirmation is still required
- after drift-report v0, the report also preserves primary drift status,
  available drift-window count, max/mean/end drift metrics, and drift
  confidence so QA/TUI consumers do not need to scrape raw hypotheses
- this report is for regression and review evidence only; it is not a user-facing
  confidence UI yet

The source-timing readiness report is the current boundary for allowing downstream
QA/listening tools to consume the timing estimate:

- `source_timing_probe_readiness_report(...)`
- `just source-timing-readiness-report`
- it combines beat evidence, downbeat evidence, confidence, drift, phrase
  stability, warnings, primary BPM, primary downbeat offset, and manual-confirm
  policy into one report
- `Ready` means the evidence is stable enough for bounded QA consumers; it does
  not mean Riotbox has a production-grade arbitrary-audio beat/downbeat detector

Generated Feral grid listening packs may use this readiness report as their
bounded BPM policy:

- explicit `--bpm` always wins and is recorded as `grid_bpm_source:
  user_override`
- without `--bpm`, a `Ready` report with a finite positive primary BPM may drive
  the pack grid only when it does not require manual confirmation; that case is
  recorded as `grid_bpm_source: source_timing`
- if readiness is weaker, the pack falls back to the static default BPM and is
  recorded as `grid_bpm_source: static_default`
- generated Feral grid manifests must also record
  `grid_bpm_decision_reason`, using stable values such as `user_override`,
  `source_timing_ready`, `source_timing_requires_manual_confirm`,
  `source_timing_not_ready`, `source_timing_missing_bpm`, and
  `source_timing_invalid_bpm`, so QA can tell why a static fallback occurred
- Feral grid manifests must record the readiness policy profile used for this
  decision, currently `source_timing.policy_profile:
  dance_loop_auto_readiness`, so diagnostic and auto-trust policies stay
  auditable
- manifests must preserve the source/grid BPM delta and whether the source timing
  agrees with the chosen grid, so QA can detect timing mismatch instead of only
  hearing a drifting or fallback-like render
- generated Feral grid packs should also expose a bounded output-drift smoke
  metric, currently `metrics.source_grid_output_drift`, so QA can catch obvious
  generated-support/grid misalignment before this becomes production beat/downbeat
  validation
- TR-909 support should expose the same bounded alignment evidence under a
  lane-specific key, currently `metrics.tr909_source_grid_alignment`, so the
  first all-lane proof is visible as TR-909 output evidence rather than only a
  generic pack-level drift smoke
- W-30 source chop should expose the same bounded alignment evidence under a
  lane-specific key, currently `metrics.w30_source_grid_alignment`, so the
  source-backed sample lane is auditable separately from TR-909 support
- MC-202 lane recipe manifests should expose bounded phrase-grid evidence under
  `metrics.mc202_phrase_grid` for each required MC-202 case, proving that
  generated candidate phrases start on the selected phrase boundary and that
  detected note onsets stay aligned to the sixteenth grid. This is a current
  lane-recipe QA proof, not a full source-derived MC-202 question/answer
  placement engine.
- observer/audio correlation should compare app-observed Source Timing readiness
  with manifest-side Source Timing evidence when both are present. The current
  summary contract reports this as `output_path.source_timing_alignment`, using
  BPM delta plus normalized warning-code overlap as bounded proof that the
  control path and generated output path are using compatible timing evidence.
- app observer snapshots should preserve compact Source Timing detail fields
  derived from Source Graph timing state, including `beat_status`,
  `beat_count`, `downbeat_status`, `bar_count`, `phrase_status`, and
  `phrase_count`, so QA can distinguish tempo-only, ambiguous-downbeat, and
  phrase-uncertain states without scraping TUI text.
- generated Feral grid QA should prove both the conservative fallback path and a
  strict auto-grid path: weak or manual-confirm timing must stay
  `static_default`, while long stable timing may use `source_timing` only when
  manual confirmation is false.

This skeleton is a contract/output-shape proof. It is not yet a production
BPM/downbeat detector and must not be presented as robust source analysis.

This spec still does not implement:

- a production-grade analyzer
- a new audio scheduler
- host-audio monitoring
- lane sound redesign
- Python runtime dependency
- full arrangement export

Those belong to later bounded P012+ slices and must reuse this contract.

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

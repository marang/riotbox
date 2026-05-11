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

### 4.3 Presentation Summary Contract

The Source Graph remains the durable timing truth, but musician-facing surfaces
must not each re-infer their own cue language from raw timing fields.

Use the shared Jam source timing summary, currently
`SourceTimingSummaryView`, as the presentation contract for:

- TUI Jam / Help / Source timing cues
- user-session observer cue, quality, degraded-policy, primary-warning, and
  compact primary-anchor evidence
- observer/audio correlation control-path timing readiness

That summary may collapse Source Graph detail into bounded musician language
such as `grid locked`, `needs confirm`, `listen first`, `fallback grid`, or
`not available`. Detailed diagnostics such as raw beat-grid count, bar-grid
count, phrase-grid count, hypothesis ids, and full warning-code lists should
still come directly from Source Graph timing state when a surface needs them.

When the shared summary exposes one `primary_warning`, it should pick the most
musically actionable timing risk, not the first warning in Source Graph storage
order. Current priority is:

1. `drift_high`
2. `ambiguous_downbeat`
3. `low_timing_confidence`
4. `weak_kick_anchor`
5. `weak_backbeat_anchor`
6. `half_time_possible`
7. `double_time_possible`
8. `phrase_uncertain`

Surfaces may show only this primary warning for focus, but raw observer/source
diagnostic fields such as `warning_codes` should still preserve the full Source
Graph warning list when space or QA context requires it.

Do not add another UI-only or observer-only timing-label mapper unless it wraps
the shared summary or is documented here as a temporary migration seam. Any new
policy label must keep the policy-to-cue contract aligned across TUI, observer
NDJSON, observer/audio JSON summaries, and validators.

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
- the current Rust probe may emit a small bounded `GrooveResidual` set from
  onset-to-grid residuals for common subdivisions; this is timing evidence for
  later lane policy, not a finished swing/groove playback engine
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
- source timing probe readiness may expose stable beat/downbeat evidence while
  still requiring manual confirmation; in that case the readiness status must be
  `needs_review`, not `ready`, so `ready` always means the compact probe summary
  is usable without a manual-confirm cue
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
- the source timing probe CLI JSON must expose a compact `anchor_evidence`
  summary so QA can distinguish stable kick/backbeat evidence from generic
  transient evidence before lane policies consume it
- `just source-timing-example-probe-report` may run the probe over local
  example WAVs when they exist and emit a compact Markdown table for review.
  Missing local WAVs must be reported as skipped rows, not CI failures, because
  the example audio files are intentionally not committed. The committed fixture
  smoke is `just source-timing-example-probe-report-fixtures`.
- the example report may use an optional expectations file for conservative
  local regression checks. Expectations should cover stable review fields such
  as cue, readiness, manual-confirm, BPM tolerance, beat/downbeat/phrase status,
  and warning-code presence; missing local WAVs must remain skipped instead of
  failing fresh clones.
- `just source-timing-example-probe-report-local` uses the tracked local-example
  expectations file for the documented Beat/DH examples. It is an optional local
  regression command because the source WAV files are deliberately outside Git.
- the source timing probe CLI JSON and example report expose `grid_use` as a
  conservative QA classification derived from the existing readiness evidence.
  Stable short drum loops with reliable beat/downbeat evidence but insufficient
  phrase material may report `short_loop_manual_confirm`; that means the grid is
  review-useful but still requires confirmation and must not be presented as
  full phrase lock or production arbitrary-audio detection.

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
- a `NeedsReview` report may also drive the pack grid only when the primary BPM
  is finite and the beat/downbeat evidence is stable, manual confirmation is
  still required, the confidence result is cautious, and no alternate timing
  evidence is present; this case is recorded as
  `grid_bpm_source: source_timing` with
  `grid_bpm_decision_reason: source_timing_needs_review_manual_confirm`
- if readiness is weaker, the pack falls back to the static default BPM and is
  recorded as `grid_bpm_source: static_default`
- generated Feral grid manifests must also record
  `grid_bpm_decision_reason`, using stable values such as `user_override`,
  `source_timing_ready`, `source_timing_needs_review_manual_confirm`,
  `source_timing_requires_manual_confirm`, `source_timing_not_ready`,
  `source_timing_missing_bpm`, and `source_timing_invalid_bpm`, so QA can tell
  whether source timing was trusted, used cautiously, or rejected
- Feral grid manifests must record the readiness policy profile used for this
  decision, currently `source_timing.policy_profile:
  dance_loop_auto_readiness`, so diagnostic and auto-trust policies stay
  auditable
- manifests must preserve the source/grid BPM delta and whether the source timing
  agrees with the chosen grid, so QA can detect timing mismatch instead of only
  hearing a drifting or fallback-like render
- generated Feral grid manifests must preserve compact primary anchor evidence
  under `source_timing.anchor_evidence`, including total primary anchors plus
  kick, backbeat, and generic transient counts; this keeps downstream listening
  QA from trusting a readiness label without seeing what musical timing anchors
  supported it
- generated Feral grid packs should also expose a bounded output-drift smoke
  metric, currently `metrics.source_grid_output_drift`, so QA can catch obvious
  generated-support/grid misalignment before this becomes production beat/downbeat
  validation
- TR-909 support should expose the same bounded alignment evidence under a
  lane-specific key, currently `metrics.tr909_source_grid_alignment`, so the
  first all-lane proof is visible as TR-909 output evidence rather than only a
  generic pack-level drift smoke
- trusted Feral-grid TR-909 support may consume compact primary groove residual
  evidence as a bounded whole-lane timing offset, recorded as
  `metrics.tr909_groove_timing`. This is a first source-timing groove consumer,
  not a production swing engine; weak/manual-confirm/static timing must keep the
  offset inactive and auditable.
- W-30 source chop should expose the same bounded alignment evidence under a
  lane-specific key, currently `metrics.w30_source_grid_alignment`, so the
  source-backed sample lane is auditable separately from TR-909 support
- W-30 source chop should also expose bounded loop-closure evidence under
  `metrics.w30_source_loop_closure`, proving that the selected source-backed
  preview is non-silent, still points at its source window, and has repeat-safe
  faded edges inside budget before QA treats it as a usable chop/loop unit. This
  is the current micro-loop proof, not final automatic loop detection.
- strict observer/audio correlation treats Feral-grid pack-level
  `source_grid_output_drift`, `tr909_source_grid_alignment`, and
  `w30_source_grid_alignment`, plus W-30 `w30_source_loop_closure`, as required
  P012 output evidence; if any are missing, malformed, or outside budget, the
  output path is not passing
- MC-202 lane recipe manifests should expose bounded phrase-grid evidence under
  `metrics.mc202_phrase_grid` for each required MC-202 case, proving that
  generated candidate phrases start on the selected phrase boundary and that
  detected note onsets stay aligned to the sixteenth grid.
- MC-202 lane recipe manifests should also expose bounded Source Graph phrase
  slot evidence under `metrics.mc202_source_phrase_slot`, proving that the
  generated candidate consumes a selected source phrase-grid slot and starts at
  that source phrase boundary. The current proof may use a synthetic source
  timing contract; it is a P012 bridge proof, not a full production
  question/answer placement engine.
- observer/audio correlation should compare app-observed Source Timing readiness
  with manifest-side Source Timing evidence when both are present. The
  app-observed cue, quality, degraded policy, primary warning, and compact
  primary-anchor evidence should come from the shared Jam source timing summary.
  The primary warning is priority-selected by musician-facing timing risk, while
  raw grid/hypothesis counts and full warning-code lists remain Source Graph
  diagnostics. The current summary contract reports this as
  `output_path.source_timing_alignment`, using BPM delta plus normalized
  warning-code overlap as bounded proof that the control path and generated
  output path are using compatible timing evidence.
- observer/audio correlation should also compare compact app-observed and
  manifest-side Source Timing anchor evidence as
  `output_path.source_timing_anchor_alignment`. This is a bounded consistency
  proof for primary/kick/backbeat/transient anchor classes; it must not demand
  exact anchor-count equality while the current probes can report different
  evidence density on each path.
- observer/audio correlation should also compare compact app-observed and
  manifest-side Source Timing groove evidence as
  `output_path.source_timing_groove_alignment`. This is a bounded consistency
  proof for primary groove residual presence, maximum absolute residual offset,
  and short residual previews; it must not require exact residual-offset
  equality while the current observer and manifest producers expose different
  evidence density. Strict evidence should reject clear contradictions, such as
  locked observer groove residuals with no manifest residuals, while keeping
  missing or non-comparable evidence `partial`.
- app observer snapshots should preserve compact Source Timing detail fields
  derived from Source Graph timing state, including `beat_status`,
  `beat_count`, `downbeat_status`, `bar_count`, `phrase_status`, and
  `phrase_count`, so QA can distinguish tempo-only, ambiguous-downbeat, and
  phrase-uncertain states without scraping TUI text.
- app observer snapshots should also preserve compact primary timing-anchor
  evidence as `anchor_evidence`: total primary anchors plus kick, backbeat, and
  transient-cluster counts from the shared Jam source timing summary. This is a
  control-path proof that the app observed real anchor evidence or honestly
  observed none; it is not a replacement for manifest-side output comparison.
- app observer snapshots should preserve compact primary groove evidence as
  `groove_evidence`: residual count, max absolute offset, and a short preview
  from the shared Jam source timing summary. This keeps control-path session
  logs aligned with probe/manifest groove evidence without asking observer
  consumers to inspect raw timing hypotheses.
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

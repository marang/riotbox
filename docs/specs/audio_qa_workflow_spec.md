# Riotbox Audio QA Workflow Spec

Version: 0.1  
Status: Draft  
Audience: audio, realtime, QA, product

---

## 1. Purpose

This document defines how Riotbox should validate audio-producing behavior in a way that is:

- technically strict
- musically honest
- reproducible
- usable both in CI and by a human operator

It exists so the project does not drift into either of these failure modes:

- "tests pass, but the output is musically useless"
- "the output sounds interesting once, but cannot be reproduced or improved"

---

## 2. Core Rule

Riotbox must not treat audio quality as either:

- numbers only
- subjective listening only

Audio quality must be validated through both:

1. automated gates
2. repeatable human listening review

Neither layer replaces the other.

---

## 3. Validation Stack

Riotbox audio QA should run at four layers:

1. hard technical gates
2. musical contract gates
3. fixture-backed golden render review
4. human listening review

### 3.1 Hard technical gates

These checks prevent obviously broken audio behavior:

- no silent output where activity is expected
- no unexpected active output in idle cases
- peak range stays inside expected limits
- no obvious clipping
- no click / pop regressions for covered transitions
- transport and commit timing remain stable
- callback timing stays inside benchmark limits

### 3.2 Musical contract gates

These checks validate behavior against product intent rather than "beauty":

- `fill` increases event density relative to idle or support states
- `release` reduces energy relative to `drive`
- takeover is more assertive than support
- capture and promoted playback remain materially usable
- variation exists over time and does not collapse into identical bars

### 3.3 Fixture-backed golden render review

For stable fixture, seed, action list, and render config:

- the system should render deterministic review artifacts
- those artifacts should be compared against known baselines
- deltas should be visible before they become production drift

### 3.4 Human listening review

Humans must be able to listen to the same deterministic outputs that automation validated.

Manual listening is required because:

- timing can be technically valid but still feel awkward
- variation can exist numerically but still feel trivial or annoying
- support layers can pass signal checks while still sounding cheap

---

## 4. Two Execution Modes

Riotbox audio QA should support two official modes.

### 4.1 CI mode

Fast, deterministic, non-interactive checks:

- unit and integration tests
- buffer-level audio regression checks
- metric extraction and threshold comparison
- replay / action-sequence consistency
- benchmark pass / fail reporting

CI mode is for:

- merge safety
- regression prevention
- enforcing minimum quality floors

### 4.2 Local listening mode

Operator-facing manual review:

- render deterministic WAV outputs for a known fixture pack
- write metrics beside those renders
- compare candidate output to baseline output
- let the operator listen before approving a change

Local listening mode is for:

- musical judgment
- product taste
- identifying weak but technically legal outputs

---

## 5. Required Harnesses

Riotbox should maintain three audio QA harnesses.

### 5.1 Buffer regression harness

This is the current lowest-level signal gate.

It should validate render-state inputs against expected output ranges such as:

- active sample count
- peak absolute value
- optional RMS or band-energy ranges

This harness is already appropriate for:

- callback-facing lane renderers
- support / takeover / fill state comparisons
- quick regression checks in CI

### 5.2 Offline WAV render harness

Riotbox should add a deterministic offline render harness that can:

- load a known fixture or render-state case
- apply a fixed seed and fixed action list
- render reviewable WAV files
- emit sidecar metrics as JSON or Markdown

This harness must exist so a human can hear:

- baseline output
- candidate output
- the practical effect of a code change

### 5.3 Listening pack harness

Riotbox should support named listening packs such as:

- `tr909-smoke`
- `capture-smoke`
- `w30-preview-smoke`
- `feral-review`

Each listening pack should render a small fixed set of review cases to one output directory.

---

## 6. Output Layout

Local audio QA output should use a stable structure.

Recommended shape:

```text
artifacts/
  audio_qa/
    2026-04-18/
      tr909-smoke/
        fills_phrase_drive/
          baseline.wav
          candidate.wav
          metrics.json
          notes.md
        takeover_controlled_phrase/
          baseline.wav
          candidate.wav
          metrics.json
          notes.md
```

Every rendered case should include:

- fixture or case ID
- seed
- action list or render-state source
- baseline reference if one exists
- metrics
- optional human review notes

---

## 7. First Metrics To Enforce

The first audio QA implementation should start with bounded, explainable metrics.

### 7.1 Signal metrics

- `peak_abs`
- `rms`
- `crest_factor`
- `active_sample_ratio`
- `silence_ratio`
- `dc_offset`

### 7.2 Rhythm and variation metrics

- `onset_count`
- `event_density_per_bar`
- `bar_similarity`
- `identical_bar_run_length`
- `variation_density`

### 7.3 Spectral and energy metrics

- `low_band_energy_ratio`
- `mid_band_energy_ratio`
- `high_band_energy_ratio`
- `spectral_centroid_range`
- `energy_delta_between_sections`

### 7.4 Product-facing metrics

- `capture_yield`
- `usable_break_variant_count`
- `quote_risk`
- `source_retention_estimate`

For early phases, metrics should use ranges rather than fake precision.

---

## 8. First Listening Rubric

Every manual listening case should be scored against a short fixed rubric.

Recommended fields:

- rhythmic clarity
- energy appropriateness
- transition quality
- variation usefulness
- support-layer tastefulness
- capture-worthiness
- artifact severity

Recommended scale:

- `1` unacceptable
- `2` weak
- `3` acceptable
- `4` strong
- `5` excellent

Short comments should also note concrete failure classes such as:

- too empty
- too busy
- cheap-sounding support
- awkward phrasing
- weak impact
- over-repetitive
- capture not worth keeping

---

## 9. First Fixture Packs

The first practical audio QA system should define a small stable listening corpus.

### 9.1 Initial review fixtures

- `clean_128_house`
- `clean_140_breaks`
- `dense_break_chopped`
- `dense_hybrid_rave`
- `hook_vocal_short`
- `hook_synth_stab`
- `low_confidence_soft_attacks`
- `feral_stress`

### 9.2 Initial action or render packs

Each fixture should support a small review set such as:

- idle / baseline
- support
- fill
- break reinforce
- takeover
- capture

Not every fixture needs every pack, but the assignment must be explicit.

---

## 10. Release Gates For Audio-Producing Changes

An audio-producing change should not be considered complete without:

- relevant unit and integration tests passing
- log, state, or action-history assertions proving that the intended action path landed
- relevant buffer or offline-output regression cases proving that the rendered audio output changed or stayed stable as intended
- the affected listening pack rendered locally
- at least one human listening pass on candidate output
- benchmark notes recorded when behavior changed materially

For small low-risk changes, the listening pass may be limited to the directly affected pack.

For larger changes, a broader smoke pack is required.

For every new or changed audio-producing function, the minimum test shape is:

- one control-path assertion, such as action log, render-state, queue/commit, or provenance state
- one output-path assertion, such as non-silence, peak/RMS range, source-vs-fallback metric delta, or a fixture-backed WAV artifact comparison

If the function only prepares state and cannot produce audio by itself, the output assertion must cover the nearest downstream render seam that consumes that state.

Do not accept "the log says it happened" as sufficient proof for audible behavior.

---

## 11. Improvement Loop

Riotbox should improve audio quality through an explicit closed loop.

### 11.1 Capture failures, do not hand-wave them away

When a render sounds bad but still passes technical checks, record the failure.

Use stable failure classes such as:

- too empty
- too monotonous
- too chaotic
- wrong section energy
- weak transition impact
- bad support taste
- unhelpful capture outcome

### 11.2 Turn failures into fixtures or thresholds

Every repeated failure should lead to at least one of:

- a new fixture case
- a stronger metric threshold
- a better profile or policy weight
- a better listening-pack case

When a user reports that two gestures sound the same, prefer adding or tightening a source-vs-control output comparison over adding only more UI/log assertions.

### 11.3 Improve policies, not hidden magic

Audio quality should primarily improve through:

- better deterministic engines
- better profile weights
- better thresholds and budgets
- better scene and action policies

The system should avoid pushing quality responsibility into opaque prompt behavior.

### 11.4 Re-render and compare

After an audio change:

- render baseline and candidate
- compare metrics
- compare listening notes
- keep the new baseline only if the change is actually better

---

## 12. Role Of Agents And Ghost

Agentic or Ghost-driven behavior must not be allowed to bypass audio QA.

Agents may:

- choose actions
- choose profiles
- bias weights within bounded ranges
- propose or perform quantized mutations

Agents must not:

- directly define unbounded audio output outside tested engines
- bypass replay-safe action paths
- introduce hidden render behavior that cannot be fixture-tested

This keeps Riotbox instrument-like, reproducible, and debuggable.

### 12.1 Future user-session observer

Riotbox should add an opt-in user-session observer when manual TUI/audio testing stays ambiguous.

The observer should attach through an explicit local socket, debug endpoint, or equivalent host-session bridge and help distinguish:

- user input timing errors
- unclear TUI timing or commit feedback
- control-path success with fallback-like audio output
- audio device or output path failure
- technically valid output that is musically weak

Useful observer evidence includes:

- exact launch command and source file
- keypress/action timeline
- queued and committed action timeline
- transport position and boundary timeline
- render-state snapshots
- audio callback health
- output metrics or monitored audio capture when available

Guardrails:

- require explicit user opt-in
- keep observer and capture work outside the realtime audio callback
- avoid storing unnecessary raw user audio when metrics or short deterministic artifacts are enough
- record whether evidence came from sandbox, real user session, offline render, or host audio monitor

Initial operational slice:

- `riotbox-app --observer <events.ndjson>` writes an opt-in local NDJSON event stream for an interactive terminal run
- current observer events include launch context, audio-runtime start or failure, keypress outcomes, queue / history snapshots, transport state, render-state summaries, and boundary commit observations
- this first slice is file-backed, not socket-backed, and does not record raw user audio
- use it to separate user input timing, queued-vs-committed state, runtime status, and render-state projection before claiming an audio-output bug or user-timing mistake

---

## 13. Current Repo Status

Today the repo already has:

- validation and benchmark guidance
- fixture corpus guidance
- callback-facing audio regression fixtures for `TR-909`, `W-30 preview`, and `W-30 resample`
- W-30 preview fixture checks for active samples, peak, and optional source-window sum / RMS ranges
- focused app/runtime regressions for source-backed W-30 reuse, including the promoted `[w] hit` path that verifies `LiveRecall` keeps non-empty source-window preview samples when a decoded capture window is available
- an initial local-only W-30 preview render helper that writes one deterministic source-window smoke WAV plus sibling Markdown metrics, with optional PCM16/PCM24 WAV source-window input
- an initial W-30 preview smoke listening-pack convention under `docs/benchmarks/`
- an initial local baseline-vs-candidate audio artifact convention under `docs/benchmarks/`
- an initial local W-30 preview smoke metrics comparison helper for baseline-vs-candidate Markdown metrics that also writes a local `comparison.md` report
- a W-30 source-vs-fallback control wrapper that renders synthetic fallback as baseline, source-backed WAV preview as candidate, and requires minimum RMS / sum deltas so fallback collapse is caught
- an opt-in file-backed user-session observer for `riotbox-app` that writes launch, keypress, queue / commit, transport, and runtime evidence to NDJSON outside the realtime audio callback

Today the repo does not yet have a full official workflow for:

- general deterministic offline WAV render generation across fixture packs
- generated listening packs beyond the first W-30 preview smoke convention
- automated baseline vs candidate WAV comparison
- automated waveform or perceptual audio comparison
- socket-backed host-session observation or monitored host audio capture
- a standard listening rubric stored with benchmark artifacts

Those gaps should be treated as near-term QA work, not optional polish.

---

## 14. Near-Term Build Order

The next bounded audio QA slices should land in this order:

1. widen signal metrics on the existing buffer regression fixtures
2. add deterministic offline WAV render support for fixture-backed review cases
3. add a first listening-pack manifest and output directory convention
4. add a short listening-review template and archive path
5. connect the most stable metric checks to CI while keeping listening review local-first

---

## 15. Success Condition

The audio QA system is doing its job when Riotbox can say all of the following honestly:

- broken audio behavior is caught automatically
- deterministic outputs can be re-rendered and compared
- humans can listen to the same cases that automation validated
- repeated weak outputs become fixtures or stronger thresholds
- audio quality improves by iteration instead of wishful thinking

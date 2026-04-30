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
- an initial local W-30 preview smoke metrics comparison helper for baseline-vs-candidate Markdown metrics that writes local `comparison.md` and `manifest.json` reports
- a W-30 source-vs-fallback control wrapper that renders synthetic fallback as baseline, source-backed WAV preview as candidate, and requires minimum RMS / sum deltas so fallback collapse is caught
- a CI-safe generated W-30 source-vs-fallback smoke that uses deterministic synthetic source material, checks minimum source-vs-fallback deltas, validates the generated listening manifest, and runs under `just audio-qa-ci`
- a CI-safe first-playable Jam probe, `just first-playable-jam-probe`, that combines synthetic source material, W-30 source-vs-fallback output evidence, and a generated app-level observer probe for the current `space -> capture -> raw audition -> promote -> W-30 hit` user path
- a CI-safe stage-style Jam probe, `just stage-style-jam-probe`, that uses generated app-level multi-boundary observer evidence, generated W-30 source-vs-fallback output evidence, and summary-level commit boundary assertions for `Phrase`, `Bar`, and `Beat`
- a bounded repeated stage-style stability smoke, `just stage-style-stability-smoke`, that runs the generated stage-style restore-diversity observer/audio path multiple times, validates observer and summary contracts for every run, rejects collapsed output metrics, and requires the generated full-grid mix WAV hash to remain stable across repetitions
- an explicit stronger stage-style stability gate, `just stage-style-stability-gate`, that reuses the same generated observer/audio path with more repetitions and a longer generated source/grid budget; it is still CI-safe and deterministic, but is a bounded gate rather than a real host-audio soak
- a CI-safe interrupted-session recovery probe, `just interrupted-session-recovery-probe`, that creates real adjacent session/temp/autosave files, emits the same recovery observer envelope, validates it, and proves the drill remains read-only with no selected restore candidate
- a CI-safe missing-target recovery probe, `just missing-target-recovery-probe`, that covers a missing requested session path plus adjacent autosave clue without silently choosing the autosave
- an opt-in file-backed user-session observer for `riotbox-app` that writes launch, keypress, queue / commit, transport, and runtime evidence to NDJSON outside the realtime audio callback
- a shared local listening-review template and `just audio-qa-notes <path>` helper for writing ignored `notes.md` files beside generated audio QA artifacts
- MC-202 audio proof cases in the lane recipe listening pack, covering follower-vs-answer, touch low-vs-high, follower-vs-pressure, follower-vs-instigator, follower-vs-mutated-drive, neutral-vs-lift contour, and direct-vs-hook-response contrasts without claiming a finished synth engine
- a first live MC-202 callback/mix seam that projects committed MC-202 role/follower/answer/pressure/instigator state into typed render state, mirrors it through `AudioRuntimeShell`, and verifies active bass output at the mixbuffer seam
- a live MC-202 touch-control regression that proves the same committed phrase changes buffer energy when the performer raises or lowers touch
- a quantized MC-202 phrase-mutation regression that proves a committed phrase variant changes the render buffer against the follower-drive control
- a first MC-202 note-budget regression that proves density can be reduced without silencing the phrase
- a first MC-202 source-section contour regression that proves a section-derived contour hint changes the rendered phrase without relying on UI/log state alone
- a first MC-202 hook-response regression that proves hook-like sections can force answer-space restraint instead of doubling the same follower phrase
- a first MC-202 recipe replay regression that drives the musician-facing follower/answer/pressure/instigator/mutation/touch flow through queue, commit, render state, and audio-buffer deltas
- a first MC-202 undo rollback regression that restores committed lane state from session undo state and proves the rendered buffer returns to the previous audible seam
- an initial lane recipe listening pack that writes baseline/candidate WAVs, metrics, Markdown comparisons, pack summary, and `manifest.json` for TR-909, Scene-coupled TR-909, and MC-202 cases
- sample-by-sample signal delta RMS checks in that pack, so shape differences with similar loudness are not hidden by plain RMS comparison
- a first local Feral before/after render pack that writes a source excerpt, Riotbox-transformed after render, before-then-after listening file, W-30 / TR-909 / MC-202 stems, metrics, comparison report, README, and `manifest.json` for a source WAV without committing generated audio
- a first local grid-locked Feral demo render pack that writes MC-202 question/answer, TR-909 beat/fill, W-30/Feral source-chop, and combined mix WAVs from one shared beat/bar/frame grid, then checks stem activity, MC-202 question-vs-answer signal delta, and low-band support
- first machine-readable `manifest.json` files beside the W-30 preview smoke, Feral grid demo, lane recipe, and Feral before/after pack outputs, recording pack metadata, artifact paths, thresholds, key metrics, and pass status
- a first shared `riotbox-audio` listening-manifest helper for local pack artifact records, signal/render metric records, and pretty JSON writes, currently used by the W-30 preview smoke comparison, Feral grid, lane recipe, and Feral before/after pack runners
- widened signal diagnostics across the current local QA outputs, including active/silence ratios, DC offset, onset count, first grid-aware event-density-per-bar diagnostics for lane recipe and Feral grid outputs, first Feral grid bar-variation diagnostics for bar similarity and identical-bar runs, and first Feral grid spectral energy ratio diagnostics for low/mid/high-band shape
- a schema version 1 compatibility policy for generated audio QA manifests, captured in `docs/benchmarks/listening_manifest_schema_policy_2026-04-29.md`
- a CI-safe Feral grid manifest smoke gate that renders from synthetic input and asserts manifest schema version, artifact roles and files, metrics files, thresholds, pass status, and non-collapsed output metrics without depending on ignored local example audio
- a local observer/audio correlation notes template and `just observer-audio-correlation-notes <path>` helper for pairing `riotbox-app --observer <events.ndjson>` control-path evidence with generated audio QA `manifest.json` output evidence
- a local observer/audio correlation summary helper, `just observer-audio-correlate <events.ndjson> <manifest.json> <summary.md>`, that extracts launch mode, audio-runtime status, key outcomes, first commit boundary, commit count, commit boundary coverage, pack result, artifact count, and key output metrics into Markdown
- an explicit CI-safe `just audio-qa-ci` smoke gate, mirrored as a named GitHub Actions step, that runs the stable W-30 preview, lane recipe, Feral before/after, Feral grid, and observer/audio-correlation helper tests without generating or committing local listening artifacts
- a committed synthetic observer/audio correlation fixture smoke that proves the summary helper reads both control-path observer events and output-path manifest metrics without depending on ignored local artifacts
- an optional strict `observer_audio_correlate --require-evidence` mode that fails when committed control-path evidence or passing output-path manifest metrics are missing
- a strict committed-fixture CLI smoke, `just observer-audio-correlate-fixture`, wired into `just audio-qa-ci` and the named GitHub Actions audio QA step without writing local artifacts
- strict observer/audio output evidence now rejects collapsed zero-level metrics even if a manifest incorrectly reports `result: pass`
- strict observer/audio output failures report the missing or collapsed metric names and the active metric floor
- observer/audio Markdown summaries surface the same output-evidence issue list for non-strict local QA review
- observer/audio correlation can emit opt-in JSON summaries for machine-readable QA verdicts and metric inspection
- a `just observer-audio-correlate-json <events.ndjson> <manifest.json> <summary.json>` helper exposes the machine-readable summary path
- the committed-fixture JSON summary path is smoke-tested in `just audio-qa-ci` and the named GitHub Actions audio QA step
- observer/audio JSON summaries include a top-level `schema` and `schema_version` marker plus control-path `commit_count` and `commit_boundaries` fields so automation can reject unexpected summary shapes and assert boundary coverage before making QA decisions
- the committed-fixture JSON smoke requires both `control_path.present` and `output_path.present`, keeping the machine-readable path aligned with the control-plus-output proof rule
- the observer/audio JSON summary v1 contract is documented in `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- the observer/audio JSON fixture smoke also runs the repo-local `scripts/validate_observer_audio_summary_json.py` contract validator without adding an external schema dependency
- validator fixtures cover both a valid failure summary with `null` metrics and a rejected invalid schema marker
- a repo-local `scripts/validate_user_session_observer_ndjson.py` helper validates the `riotbox.user_session_observer.v1` event stream shape, including recovery snapshot candidate `decision` labels and optional read-only `manual_choice_dry_run` evidence when snapshots are present
- strict observer/audio correlation now rejects malformed observer stream evidence before accepting committed control-path evidence
- `just user-session-observer-validator-fixtures` validates the committed observer fixture streams plus valid and invalid recovery-snapshot fixtures, and is wired into `just audio-qa-ci`
- a shared manifest v1 envelope validator that checks stable top-level fields and artifact records for current local audio QA producer shapes while leaving pack-specific metrics flexible
- optional Feral scorecard validation inside the shared manifest v1 validator, so generated Feral grid manifests must carry well-typed scorecard evidence when they emit a `feral_scorecard` block
- strict observer/audio correlation now validates that shared manifest v1 envelope before treating pack-specific output metrics as acceptable evidence
- observer/audio strict Markdown and JSON correlation is smoke-tested against a freshly generated Feral grid manifest built from a deterministic synthetic break WAV and a generated app-level Feral-grid observer probe
- observer/audio strict JSON correlation also accepts W-30 preview source-diff manifests as output-path evidence, using candidate RMS, active-sample ratio, and RMS delta to reject silent or fallback-collapsed first-playable output
- the listening manifest v1 field-level JSON contract is documented in `docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md`
- a repo-local `scripts/validate_listening_manifest_json.py` helper and `just listening-manifest-validator-fixtures` fixture matrix validate the listening manifest v1 envelope without freezing pack-specific metrics
- `just audio-qa-ci` validates freshly generated W-30 preview, lane recipe, Feral before/after, and Feral grid manifests against the listening manifest v1 envelope
- `just recipe2-observer-audio-gate` correlates a headless app-level documented Recipe 2 MC-202 observer path with a freshly generated lane recipe listening-pack manifest, and requires that the generated observer stream carries the same transport / queue / runtime / recovery snapshot envelope used by the live `riotbox-app --observer` path
- generated-pack manifest validation can require referenced artifact and metrics files to exist via `--require-existing-artifacts`
- `just offline-render-reproducibility-smoke` is a CI-safe bounded reproducibility check that renders the same deterministic source-backed W-30 output twice and compares WAV hashes; it is an offline render smoke, not the full export workflow
- `just full-grid-export-reproducibility-smoke` is a CI-safe bounded export reproducibility check that renders the deterministic Feral grid/full-mix pack twice from generated source material, validates both listening manifests, rejects collapsed full-mix output metrics, and compares the exported full-mix WAV hashes; it is still not the full arrangement export workflow
- `just interrupted-session-recovery-probe` is a CI-safe file-backed drill for observer recovery evidence; it is still not automatic startup recovery and does not execute a restore
- `just missing-target-recovery-probe` is the sibling file-backed drill for a missing normal load target with a parseable autosave clue; it keeps the same read-only manual recovery boundary
- `just stage-style-stability-smoke` is a CI-safe repeated-run smoke around the generated stage-style restore-diversity path; it catches obvious intermittent observer/audio collapse and full-mix nondeterminism, but remains a bounded smoke rather than a real host-audio soak or multi-hour endurance test
- `just stage-style-stability-gate` is the stronger bounded version of the same path. It defaults to three repeated runs, eight generated bars, and a longer generated source window while preserving the same observer/audio correlation and stable full-mix hash assertions. It is suitable for PRs that touch P011 stability gates, but it is not a substitute for future live-device soak testing.

Today the repo does not yet have a full official workflow for:

- general deterministic offline WAV render generation across fixture packs
- generated listening packs beyond the W-30 preview smoke, lane recipe, Feral before/after, and Feral grid-demo conventions
- a complete JSON Schema document or compatibility test matrix across every helper version
- CI-gated grid-locked multi-lane Feral demo rendering across the full fixture corpus beyond the current synthetic manifest smoke gate
- broad automated baseline vs candidate WAV comparison outside the current bounded helper cases
- automated waveform or perceptual audio comparison
- socket-backed host-session observation or monitored host audio capture
- live or CI-wide observer-vs-audio-manifest correlation across documented recipes
- automated enforcement of human listening-review rubrics

Those gaps should be treated as near-term QA work, not optional polish.

---

## 14. Near-Term Build Order

The next bounded audio QA slices should land in this order:

1. widen signal metrics on the existing buffer regression fixtures
2. add deterministic offline WAV render support for fixture-backed review cases
3. keep the shared listening-pack manifest helper/schema stable while new pack runners are added
4. connect the most stable metric checks to CI while keeping listening review local-first
5. automate recipe replay / observer correlation once the local notes workflow is proven

---

## 15. Success Condition

The audio QA system is doing its job when Riotbox can say all of the following honestly:

- broken audio behavior is caught automatically
- deterministic outputs can be re-rendered and compared
- humans can listen to the same cases that automation validated
- repeated weak outputs become fixtures or stronger thresholds
- audio quality improves by iteration instead of wishful thinking

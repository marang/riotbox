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
- source-derived rebuilds remain musically usable when the original source layer is muted
- source-layer modes are explicit and optional, not an implicit requirement for Riotbox to sound complete
- anchor-preservation modes keep promised kick / snare anchors readable, while destructive or replacement modes are allowed to rebuild the beat

Every musical pattern used by a listening pack, benchmark, demo, or generated
artifact must carry an explicit origin. The allowed origin labels are:

- `source_derived`: derived from Source Graph, Source Timing, capture, source
  windows, anchors, transient evidence, or section evidence
- `user_confirmed`: explicitly accepted or performed by the musician
- `primitive_renderer`: transparent engine or preset vocabulary, useful as a
  renderer/control surface but not proof of source-aware musical intelligence
- `fixture`: deterministic QA material created to exercise one specific seam
- `fallback`: degraded or safe placeholder chosen because better evidence is
  unavailable
- `compatibility_silent`: an output slot kept for manifest/schema continuity
  while the musical implementation is intentionally silent

Generated packs must not present `primitive_renderer`, `fixture`, `fallback`, or
`compatibility_silent` output as source-derived behavior. A source showcase can
include those lanes only when the manifest and README keep the origin visible and
when source-independent support is not loud enough to mask source-backed output.
If a fixed pattern is a renderer vocabulary or preset, label it as such; if a
pattern is claimed to react to a source, prove the source relation in the
manifest.

### 3.2.1 Source-derived rebuild gates

Riotbox must distinguish three related but different output modes:

- `rebuild-only`: the original source file is not audible as a continuous backing track; Riotbox output is generated or reconstructed from source-derived timing, anchors, sections, transients, slices, captures, and candidates
- `source-layer`: the original source is intentionally audible beside generated Riotbox lanes, such as for loop accompaniment, A/B checking, hybrid performance, or transition support
- `anchor-preserve`: selected source anchors, such as downbeat kick or backbeat snare identity, are intentionally preserved or reinforced while the surrounding pattern may still be rebuilt

QA must not let `source-layer` mask weak generation.

For every source-derived arrangement or rebuild feature, include at least two of
these comparison renders when the seam exists:

- `rebuild_only.wav`: source layer muted, generated Riotbox lanes active
- `source_layer_on.wav`: same render with the source layer intentionally audible
- `source_reference.wav`: the original or looped source reference for listening comparison only
- `anchor_preserve.wav`: optional mode-specific render proving promised downbeat / backbeat anchors remain readable
- `destructive_or_replace.wav`: optional mode-specific render proving replacement behavior is explicit and still grid-locked

The `rebuild_only` render is the primary product proof. It must pass non-silence,
timing, variation, and musical-contract checks without relying on the original
beat underneath. `source_layer_on` may sound better or different, but it must not
be the only passing case unless the feature is explicitly a source-layer feature.

Minimum checks:

- source layer mute state is represented in control-path state, manifest metadata, or render config
- generated lanes remain aligned to the trusted source beat grid
- output is not silent, fallback-collapsed, or one-bar identical across the review window
- source-retention or source-correlation metrics are reported when available
- listening notes explicitly say whether the result works without the source layer

Failure classes to record:

- needs original source to sound complete
- source-layer masks weak generation
- rebuilt beat loses grid
- anchor-preserve mode destroys promised kick or snare identity
- destructive mode was not explicit

### 3.2.1.1 Source transport monitor gates

Source transport and Source Map work must prove the musician can hear, see, and
capture the intended material.

Minimum gates:

- `source` monitor mode is non-silent for a decoded source
- `blend` and `riotbox` monitor states are distinguishable from `source` when
  generated lanes are active
- bar / phrase seek changes the audible source excerpt and preserves the current
  play / pause state
- source playback does not perform file I/O or analysis in the realtime callback
- confirmed-grid state survives save / restore and replay without changing the
  original Source Timing evidence
- manual source-timing confirmation has a CI-safe observer probe that presses
  the real `C` control, records the immediate commit, and proves the observer
  exposes confirmed runtime state while analyzer cue / warning evidence remains
  unchanged
- source-map snapshots show energy, peaks, bars or time fallback, playhead, and
  capture range without relying on color alone
- capture length intents produce source windows that match `1 beat`, `1 bar`,
  `4 bars`, or phrase fallback expectations
- source-window consumers distinguish analyzer-locked timing, user-confirmed
  timing, manual-confirm-required timing, fallback timing, and unavailable
  timing through typed readiness; unconfirmed manual-confirm timing must not
  silently create a bar-accurate source-window reuse claim
- observer snapshots expose the same Source Map projection used by the TUI,
  including capture-range availability. QA checks should use that observer
  evidence when validating whether the visible capture target is bar-accurate or
  intentionally unavailable.
- user-session observer probes should assert `source_map.capture_range_available`
  for locked/bar-grid and fallback/untrusted paths so this visual capture target
  contract is covered outside unit-only snapshots.
- observer snapshots should expose the latest committed capture source window
  when one exists, including source id, start/end seconds, duration, and frame
  bounds, so capture-length and boundary QA can correlate the visible `cap`
  preview with the committed source-window provenance.

### 3.2.2 Multi-source showcase diversity gates

Source-showcase listening packs must prove source reflection across multiple
input files. Passing non-silence, reproducibility, and same-source stability is
not enough if different sources produce effectively the same Riotbox result.

For any pack presented as a source showcase:

- validate reproducibility within the same source separately from diversity
  across different sources
- reject identical or near-identical full mixes across distinct source files
- reject source-backed stems that are byte-identical across distinct source
  files unless the fixture explicitly proves those sources contain the same
  selected window
- reject source-independent generated stems, such as fixed TR-909 or MC-202
  support, when they are loud enough to dominate the source-backed material
- record whether generated support is intentionally common across sources or
  whether it is supposed to react to source timing, density, energy, anchors, or
  section role

The current lightweight command is:

```bash
just source-showcase-diversity "PACK_A PACK_B ..."
```

The deterministic synthetic showcase is a fixture / developer-QA pack, not a
musician-facing listening demo:

```bash
just synthetic-fixture-showcase
```

That command writes ignored artifacts under
`artifacts/audio_qa/local-synthetic-fixture-showcase/`, including raw source
comparison windows, W-30 source chops, source-first mixes, generated-support
mixes, source-diversity output, reproducibility evidence, and an observer/audio
correlation summary. Its sources are generated by
`scripts/write_synthetic_showcase_sources.py` and are intentionally repeatable.
Do not use this command as the answer to "what can Riotbox already do?"

The old `just representative-source-showcase` target remains as a deprecated
compatibility alias for the same synthetic fixture path.
The representative showcase generator refuses to reset output directories
outside repo-local `artifacts/audio_qa/` or `/tmp/riotbox-*` paths unless the
caller passes the explicit `--force-output-reset` escape hatch.

For musician-facing local review, use the real-source listening showcase:

```bash
just real-source-listening-showcase
```

That command is manifest-driven, starts from local example WAVs under
`data/test_audio/examples/`, writes source windows as separate before/after
files, renders Riotbox stems and mixes, and emits a report that separates
`technical_status` from `musical_verdict`. A technically valid render may still
receive a weak or failed musical verdict.

The synthetic fixture showcase can still run the musical-quality review gate:

```bash
just representative-source-showcase-musical-quality
```

The gate is intentionally separate from source-diversity and non-silence checks.
It marks at least one pack as a `musically_convincing_candidate` only when the
case keeps source-first masking under control, makes generated TR-909 support
audible rather than decorative, preserves W-30 source-chop energy, requires
source-derived W-30 accent dynamics, proves MC-202 source-section contour and
all-lane mix movement, exposes source-anchor evidence, carries low-end support,
and avoids a fully static bar loop. This is a fixture review aid, not automatic
taste scoring and not a product listening verdict.

The full synthetic fixture showcase stays a local review pack because it is
larger than a normal CI smoke. The aggregate audio QA gate instead includes
`just syncopated-source-showcase-smoke`, which generates the same deterministic
syncopated-snare source family in a temp directory, runs `feral_grid_pack`, and
validates source-timing plus source-grid output evidence so scorer/order
regressions fail before manual showcase generation.

It hashes referenced audio artifacts and compares manifest metrics across
multiple `manifest.json` files or pack directories. It is a blocker gate for
source-showcase packs, not a replacement for listening review. A passing result
means the pack avoided the known identical-output false positive; it does not
mean TR-909, W-30, MC-202, or future bass policies are musically complete.

When `--json-output` or `--markdown-output` is provided, the command also emits a
source-diversity summary with:

- per-role artifact hash groups
- pairwise normalized RMS deltas
- pairwise low-band RMS deltas
- pairwise spectral-energy distance when manifest spectral metrics exist
- pairwise waveform correlation when referenced artifacts are readable PCM16 WAV
- generated-to-source-backed dominance ratios
- stable failure codes such as `full_mix_identical_across_sources`,
  `full_mix_cross_source_correlation_too_high`, and
  `generated_stem_dominates_mix`

Early P011 guardrail defaults:

- identical full-mix hashes across different sources always fail
- source-backed stems with identical hashes across different sources fail unless
  the fixture explicitly proves the same selected source window
- full-mix normalized RMS delta below `0.05` is treated as too similar when no
  spectral-energy evidence is available or the spectral-energy distance is also
  below `0.02`
- full-mix waveform correlation at or above `0.995` is treated as too similar
- identical generated support with generated/source-backed RMS ratio at or above
  `0.75` is treated as dominant
- Feral grid packs expose explicit lane stems plus two listening mixes so source
  extraction is not judged from a drum-dominant render:
  `04_riotbox_source_first_mix.wav` leads with the source-backed W-30 chop, while
  `05_riotbox_generated_support_mix.wav` keeps generated support secondary and
  records generated/source RMS ratios in `metrics.mix_balance`
- Feral grid pack-level `metrics.source_grid_output_drift` is measured from the
  complete generated-support mix, not an individual lane. Lane-specific timing
  evidence remains separate under `metrics.tr909_source_grid_alignment`,
  `metrics.mc202_source_grid_alignment`, and
  `metrics.w30_source_grid_alignment`.
- Feral grid TR-909 support must not be fixed only by BPM/grid when the pack is
  presented as source-aware. The generated manifest records
  `metrics.tr909_source_profile` with the measured source-window energy/onset
  evidence, chosen support profile, pattern adoption, phrase variation,
  drum-bus level, slam intensity, and reason label so reviewers can see why the
  support pattern changed.
- Feral grid TR-909 support must expose source-derived accent dynamics under
  `metrics.tr909_source_accent_dynamics`, proving that kick/support accents have
  enough distinct source-shaped levels to avoid a flat decorative pulse while
  staying on the source grid.
- Feral grid generated-support mixes must expose explicit all-lane mix movement
  proof under `metrics.all_lane_mix_movement`, showing that the source-first and
  generated-support listening mixes are audibly distinct and that TR-909,
  MC-202, and W-30 all contribute measurable energy instead of passing only
  aggregate mix-balance or non-silence checks.
- Feral grid MC-202 support must expose bounded source-section contour evidence
  under `metrics.mc202_source_contour` before being treated as deeper P013 bass
  behavior. The proof may shape contour, touch, and support level from source
  energy/density and must compare against the primitive neutral support control;
  it is not a source-derived phrase planner and must not be described as
  extracted MC-202 question/answer placement.
- Feral grid W-30 source-chop output must carry audible source identity, not only
  prove that a source window exists. The generated manifest records
  `metrics.w30_source_chop_profile` with source-window RMS, selected segment
  RMS, normalized preview RMS/peak, selected source frame, gain, and reason label
  so reviewers can tell whether the W-30 stem used an articulate source segment
  and did not collapse back to a generic preview/control tone.
- Feral grid W-30 source-chop output must expose source-derived accent dynamics
  under `metrics.w30_source_accent_dynamics`. The proof checks that selected
  source offsets produce multiple trigger velocities and enough velocity span
  to avoid a flat repeated chop while staying on the same source grid.
- Feral grid W-30 source-chop output must also expose bounded repeat-safety
  evidence under `metrics.w30_source_loop_closure`. The first proof checks that
  the selected preview is non-silent, maps back to the selected source window,
  and has faded edges inside edge-delta / edge-absolute budgets. This is a
  micro-loop/chop-window QA proof, not the final W-30 loop detector.

### 3.2.3 Automated musical fitness gate

Automated musical fitness sits inside the existing audio QA stack as a
deterministic rejection layer. It is stronger than hard technical validity
because it can reject outputs that are non-silent but musically broken. It is
weaker than human listening review because it cannot certify taste, hook
strength, emotional impact, or whether a musician would keep using the result.

The report schema is `riotbox.automated_musical_fitness.v1`. Generated reports
and any QA report that embeds the automated result must use the same language:

- `technical_status`: whether the selected render or candidate passed basic
  technical sanity, such as non-silence and clipping checks
- `automated_musical_fitness_status`: whether the automated musical-fitness
  gate rejected a known bad-output mode
- `human_verdict`: the human listening state; this must remain `unverified`
  until a person has listened and recorded a verdict
- `selected_candidate`: the candidate or render path the automated report
  selected for compact review
- `failure_codes`: stable machine-readable failure codes
- `score_breakdown`: compact per-section scores and failure codes, suitable for
  CI logs and report summaries

The automated gate can reliably catch known bad-output modes when the manifest
or report carries the required evidence:

- silence, near-silence, clipping risk, and missing full-mix metrics
- fallback collapse or byte/metric identity collapse
- source masking or fake source-derived contour evidence
- static loops, missing W-30 trigger/slice/accent variation, and identical bars
- lane imbalance where placeholder or weak lanes are hidden by a stronger lane
- weak low-end, weak transient pressure, and decorative drum/bass support
- weak source-grid alignment or large peak offsets
- identical response signatures across different source cases

The automated gate cannot certify:

- that the hook is memorable
- that the break, bass, stab, chop, or silence cut has taste
- that a technically varied loop is not annoying
- that the output has enough live-performance impact
- that a source-reactive response is the best musical response
- that a generated pack is approved for musician-facing demos

Manual listening is still required when a change materially affects audible
character, claims a candidate is musically convincing, ships a real-source
review pack, changes drum/bass/chop policy, or promotes an output from
automated evidence into a product example. A passing automated report means "no
known bad-output mode was caught"; it does not mean "this sounds good".

The current deterministic command is:

```bash
just automated-musical-fitness-fixtures
```

For local/manual showcase review, generate the automated report beside the
showcase artifacts:

```bash
just automated-musical-fitness showcase=artifacts/audio_qa/local-representative-source-showcase
```

When `validation/automated-musical-fitness.json` exists, the representative
showcase musical-quality report embeds the compact automated fields while still
keeping its own candidate result separate. Absence of the automated report is
backward-compatible; it means the automated layer did not run for that report,
not that the output passed or failed.

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

Structured listening review records the human layer as explicit artifact data,
not chat memory and not CI-only truth. For audio-producing slices, the local
workflow is:

```bash
just listening-review-pack RIOTBOX-123
just listening-review-record artifacts/audio_qa/local/listening-reviews/RIOTBOX-123/review.json \
  keep kick source_transformed_but_present clear
```

The pack command writes a local review directory with:

- `prompt.md`: one-question-at-a-time listening prompt
- `review.json`: structured verdict data, initially `human_verdict: unverified`
- `metrics.json`: compact source/candidate file presence and byte metadata
- `README.md`: local artifact ownership notes

The record command updates `review.json` and writes `review-summary.md`.
Required verdict fields include:

- ticket, PR, command, source file, and seed/config when available
- `technical_status`
- `automated_musical_fitness_status`
- `human_verdict`: `keep`, `reject`, `technically_ok_but_musically_weak`, or
  `inconclusive`
- strongest element: `kick`, `snare`, `bass`, `stab`, `chop`, `vocal`,
  `silence`, or `none`
- source-recognition verdict
- hook verdict after two bars
- failure reason
- preferred direction
- avoid list
- concrete follow-up

PRs that affect audible behavior must say whether a listening-review pack
exists, whether a human verdict was recorded, or why the change remains
`human_verdict: unverified`. The structured verdict complements automated
musical fitness; it does not replace deterministic metrics, and it must not be
stored only in agent memory.

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
- `source_layer_dependency`
- `rebuild_only_usability`
- `anchor_preservation_score`
- `grid_drift_budget`

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
- rebuild-only usefulness
- source-layer dependency
- anchor-preservation honesty
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
- only works with original source underneath
- generated layers drift against the source grid
- promised anchor was lost
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
- rebuild-only
- source-layer-on
- anchor-preserve
- destructive / replace

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

For source-derived rebuild, arrangement, TR-909 reinforcement, MC-202 phrase, W-30
slice/capture, bass, or Feral policy changes, the release gate must also state
which source-audibility mode was tested:

- rebuild-only
- source-layer
- anchor-preserve
- destructive / replace

If a feature claims to generate a new Riotbox result, `rebuild-only` evidence is
required. If `source-layer` is used, it must be documented as an optional layer or
transition tool, not as hidden support for a weak rebuild.

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
- a CI-safe source timing confirmation probe, `just source-timing-confirmation-probe`, that presses the real `C` control against a manual-confirm Source Graph, validates the normal observer stream, asserts the immediate `source_timing.confirm_grid` commit, and proves `grid_confirmed` runtime state appears without changing analyzer cue / warning evidence
- a CI-safe source transport map/capture probe, `just source-transport-map-capture-probe`, that starts in manual-confirm listen-first mode, confirms the grid, seeks the Source Map, captures a bar-aligned source window, raw-auditions, promotes, triggers W-30, and correlates the observer path with W-30 source-vs-fallback output evidence
- a CI-safe stage-style Jam probe, `just stage-style-jam-probe`, that uses generated app-level multi-boundary observer evidence, generated W-30 source-vs-fallback output evidence, and summary-level commit boundary assertions for `Phrase`, `Bar`, and `Beat`
- a CI-safe stage-style snapshot convergence smoke, `just stage-style-snapshot-convergence-smoke`, that drives a supported Scene / MC-202 / TR-909 stage-style sequence, restores from a mid-run snapshot payload, asserts latest-snapshot replay summary readiness, rejects unsupported suffix commands, and compares the replayed final mix buffer against the committed final mix
- a bounded repeated stage-style stability smoke/proof, `just stage-style-stability-smoke` / `just stage-style-stability-proof`, that runs the generated stage-style restore-diversity observer/audio path multiple times, validates observer and summary contracts for every run, rejects collapsed output metrics, requires the generated full-grid mix WAV hash to remain stable across repetitions, and validates normalized proof data for run count, commit-boundary coverage, observer/audio evidence, and stable output hash
- an explicit stronger stage-style stability gate, `just stage-style-stability-gate`, that reuses the same generated observer/audio path with more repetitions and a longer generated source/grid budget; it is still CI-safe and deterministic, but is a bounded gate rather than a real host-audio soak
- a CI-safe interrupted-session recovery probe, `just interrupted-session-recovery-probe`, that creates real adjacent session/temp/autosave files, emits the same recovery observer envelope, validates it, and proves the drill remains read-only with no selected restore candidate
- a CI-safe missing-target recovery probe, `just missing-target-recovery-probe`, that covers a missing requested session path plus adjacent autosave clue without silently choosing the autosave
- an opt-in file-backed user-session observer for `riotbox-app` that writes launch, keypress, queue / commit, transport, and runtime evidence to NDJSON outside the realtime audio callback
- a shared local listening-review template and `just audio-qa-notes <path>` helper for writing ignored `notes.md` files beside generated audio QA artifacts
- MC-202 audio proof cases in the lane recipe listening pack, covering touch low-vs-high, follower-vs-pressure, follower-vs-instigator, follower-vs-mutated-drive, and neutral-vs-lift contour contrasts without claiming a finished synth engine; `mc202.generate_answer` stays control-path only until source-derived phrase planning exists
- a first live MC-202 callback/mix seam that projects committed MC-202 role/follower/pressure/instigator state into typed render state, mirrors it through `AudioRuntimeShell`, and verifies active bass output at the mixbuffer seam; answer remains a control-path intent until source-derived phrase planning exists
- a live MC-202 touch-control regression that proves the same committed phrase changes buffer energy when the performer raises or lowers touch
- a quantized MC-202 phrase-mutation regression that proves a committed phrase variant changes the render buffer against the follower-drive control
- a first MC-202 note-budget regression that proves density can be reduced without silencing the phrase
- a first MC-202 source-section contour regression that proves a section-derived contour hint changes the rendered phrase without relying on UI/log state alone
- a regression guard that hook-like sections no longer inject a hardcoded MC-202 answer phrase before the source-derived question/answer placement engine exists
- a first MC-202 recipe replay regression that drives the musician-facing follower/pressure/instigator/mutation/touch flow through queue, commit, render state, and audio-buffer deltas, while answer asserts control-path state without synthetic output
- a first MC-202 undo rollback regression that restores committed lane state from session undo state and proves the rendered buffer returns to the previous audible seam
- an initial lane recipe listening pack that writes baseline/candidate WAVs, metrics, Markdown comparisons, pack summary, and `manifest.json` for TR-909, Scene-coupled TR-909, and MC-202 cases
- sample-by-sample signal delta RMS checks in that pack, so shape differences with similar loudness are not hidden by plain RMS comparison
- a first local Feral before/after render pack that writes a source excerpt, Riotbox-transformed after render, before-then-after listening file, W-30 / TR-909 / MC-202 stems, metrics, comparison report, README, and `manifest.json` for a source WAV without committing generated audio
- a first local grid-locked Feral demo render pack that writes TR-909 beat/fill, W-30/Feral source-chop, primitive MC-202 source-grid proof bass pressure, and combined mix WAVs from one shared beat/bar/frame grid, then checks stem activity and low-band support without injecting a hardcoded MC-202 question/answer phrase or presenting the MC-202 proof phrase as source-derived phrase planning
- first machine-readable `manifest.json` files beside the W-30 preview smoke, Feral grid demo, lane recipe, and Feral before/after pack outputs, recording pack metadata, artifact paths, thresholds, key metrics, and pass status
- a first shared `riotbox-audio` listening-manifest helper for local pack artifact records, signal/render metric records, and pretty JSON writes, currently used by the W-30 preview smoke comparison, Feral grid, lane recipe, and Feral before/after pack runners
- widened signal diagnostics across the current local QA outputs, including active/silence ratios, DC offset, onset count, first grid-aware event-density-per-bar diagnostics for lane recipe and Feral grid outputs, first Feral grid bar-variation diagnostics for bar similarity and identical-bar runs, and first Feral grid spectral energy ratio diagnostics for low/mid/high-band shape
- a schema version 1 compatibility policy for generated audio QA manifests, captured in `docs/benchmarks/listening_manifest_schema_policy_2026-04-29.md`
- a CI-safe Feral grid manifest smoke gate that renders from synthetic input and asserts manifest schema version, artifact roles and files, metrics files, thresholds, pass status, and non-collapsed output metrics without depending on ignored local example audio
- a local observer/audio correlation notes template and `just observer-audio-correlation-notes <path>` helper for pairing `riotbox-app --observer <events.ndjson>` control-path evidence with generated audio QA `manifest.json` output evidence
- a local observer/audio correlation summary helper, `just observer-audio-correlate <events.ndjson> <manifest.json> <summary.md>`, that extracts launch mode, audio-runtime status, key outcomes, first commit boundary, commit count, commit boundary coverage, pack result, artifact count, grid-BPM decision evidence, source/grid BPM agreement, and key output metrics into Markdown
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
- observer/audio JSON summaries include a top-level `schema` and `schema_version` marker plus control-path `commit_count`, `commit_boundaries`, and optional observer-side Source Timing Intelligence readiness fields so automation can reject unexpected summary shapes and assert boundary/timing coverage before making QA decisions
- the committed-fixture JSON smoke requires both `control_path.present` and `output_path.present`, keeping the machine-readable path aligned with the control-plus-output proof rule
- observer snapshots include P014 Scene evidence: active / restore / next scene,
  landed movement intent, Arrangement Scene contract readiness, source-locked
  movement permission, and Source Monitor scene-anchor state
- observer/audio JSON summaries include `observer_scene_movement` and
  `scene_movement_audio_evidence`; strict evidence rejects source-locked scene
  movement when the observer lacks a Source Monitor anchor or output metrics are
  missing / collapsed
- `just p014-scene-movement-observer-probe` is wired into `just audio-qa-ci` and
  proves a headless `scene.launch` path through observer NDJSON validation and
  strict observer/audio JSON correlation
- observer/audio summaries can surface Feral-grid `source_grid_output_drift`
  evidence and strict correlation requires Feral-grid manifests to include
  pack-level `source_grid_output_drift` plus lane-specific
  `tr909_source_grid_alignment`, `mc202_source_grid_alignment`,
  `w30_source_grid_alignment`, and `w30_source_loop_closure`; missing or
  out-of-budget metrics fail the output path instead of being treated as an
  optional note
- observer/audio summaries can compare observer-side Source Timing readiness with
  manifest-side Source Timing evidence as `output_path.source_timing_alignment`;
  strict correlation treats real mismatches as output-path failures while keeping
  missing or non-comparable evidence reviewable for older/non-Feral packs
- observer-side Source Timing readiness fields used for cue, quality,
  degraded policy, primary warning, and compact primary-anchor evidence should
  come from the shared Jam source timing summary, not from a separate observer
  mapper. Beat/downbeat/phrase counts and full warning-code lists remain raw
  Source Graph diagnostics when included in the observer stream.
- observer/audio summaries can also compare compact observer-side and
  manifest-side Source Timing anchor evidence as
  `output_path.source_timing_anchor_alignment`; this records partial, aligned,
  and contradictory anchor evidence without requiring exact anchor-count equality
- observer/audio summaries can also compare compact observer-side and
  manifest-side Source Timing groove evidence as
  `output_path.source_timing_groove_alignment`; this records partial, aligned,
  and contradictory groove-residual evidence without requiring exact
  residual-offset equality. Strict correlation treats clear contradictions as
  output-path failures, such as locked observer groove residuals with zero
  comparable manifest residuals, while missing or density-mismatched evidence
  stays reviewable as `partial`.
- generated Feral grid listening manifests carry compact
  `source_timing.anchor_evidence` counts for primary, kick, backbeat, and
  transient-cluster anchors, so QA can audit whether timing readiness is backed
  by musically meaningful anchors instead of only a readiness/status label
- observer/audio JSON summaries surface Feral-grid `grid_bpm_source` and
  `grid_bpm_decision_reason` plus `source_timing_bpm_delta`, so reviewers can
  distinguish trusted source timing, explicit user override, manual-confirm
  fallback, missing/invalid timing, and conservative static-default fallback
  without opening the raw manifest
- the observer/audio JSON summary v1 contract is documented in `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- the observer/audio JSON fixture smoke also runs the repo-local `scripts/validate_observer_audio_summary_json.py` contract validator without adding an external schema dependency
- validator fixtures cover both a valid failure summary with `null` metrics and a rejected invalid schema marker
- a repo-local `scripts/validate_user_session_observer_ndjson.py` helper
  validates the `riotbox.user_session_observer.v1` event stream shape,
  including recovery snapshot candidate `decision` labels, compact
  replay-family diagnostics, optional Source Timing Intelligence readiness plus
  musician-facing timing `cue` when a Source Graph is attached, compact
  source-timing anchor-evidence counts and source-timing groove-evidence
  previews from the shared Jam source timing summary, policy-to-cue
  consistency, and optional read-only
  `manual_choice_dry_run` evidence when snapshots are present
- `just source-timing-probe-json-validator-fixtures` validates the source timing probe CLI JSON contract, including cue/readiness consistency, machine-readable score fields, and primary anchor-evidence shape, and is wired into `just audio-qa-ci`
- `just generated-source-timing-probe-json-smoke` runs the real source timing probe CLI against a deterministic generated WAV, validates the emitted JSON contract, and asserts stable grid-locked timing plus visible kick/backbeat anchor evidence before the aggregate audio QA gate can pass
- `just generated-degraded-source-timing-probe-json-smoke` runs the same CLI contract against generated silence and asserts degraded/manual-confirm evidence so weak material cannot falsely pass as grid-locked
- `just generated-ambiguous-source-timing-probe-json-smoke` runs a flat-pulse generated source with strong beat evidence but weak downbeat/phrase evidence and asserts it remains manual-confirm with generic transient anchors instead of falsely becoming grid-locked or semantically classified
- `just syncopated-source-showcase-smoke` runs the deterministic syncopated source showcase case through `feral_grid_pack` and validates source timing, source-grid output drift, TR-909/W-30 lane alignment, primitive-renderer MC-202 proof output with lane alignment, loop closure, and non-silent full-grid audio before `just audio-qa-ci` can pass
- strict observer/audio correlation now rejects malformed observer stream evidence before accepting committed control-path evidence
- `just user-session-observer-validator-fixtures` validates the committed observer fixture streams plus valid and invalid recovery-snapshot fixtures, and is wired into `just audio-qa-ci`
- a shared manifest v1 envelope validator that checks stable top-level fields and artifact records for current local audio QA producer shapes while leaving pack-specific metrics flexible
- optional Feral scorecard validation inside the shared manifest v1 validator, so generated Feral grid manifests must carry well-typed scorecard evidence when they emit a `feral_scorecard` block
- strict observer/audio correlation now validates that shared manifest v1 envelope before treating pack-specific output metrics as acceptable evidence
- observer/audio strict Markdown and JSON correlation is smoke-tested against a freshly generated Feral grid manifest built from a deterministic synthetic break WAV and a generated app-level Feral-grid observer probe
- generated Feral grid observer/audio correlation now gates on aligned source
  timing evidence: observer readiness and manifest timing must stay within BPM
  tolerance, share normalized warning evidence, and report no alignment issues
- the same generated Feral grid gate also proves the conservative fallback path:
  weak/unavailable source timing must report `grid_bpm_source: static_default`
  with an explicit fallback `grid_bpm_decision_reason`, while the observer/audio
  summary still preserves aligned warning evidence and non-collapsed output
- the same generated Feral grid gate also proves the explicit user override
  path: an override must report `grid_bpm_source: user_override`,
  `grid_bpm_decision_reason: user_override`, numeric
  `source_timing_bpm_delta`, and matching
  `source_timing.bpm_agrees_with_grid` evidence while the output path remains
  non-collapsed
- the generated Feral grid gate also proves the timing-risky user override
  path: an out-of-tolerance override must still report
  `grid_bpm_source: user_override` and
  `grid_bpm_decision_reason: user_override`, but its numeric
  `source_timing_bpm_delta` must exceed the agreement tolerance and
  `source_timing.bpm_agrees_with_grid` must be `false` in both the manifest and
  observer/audio summary while output remains non-collapsed
- generated Feral grid observer/audio correlation now also reports source timing
  anchor and groove alignment separately, so QA can distinguish a grid/BPM match
  from missing musical anchor evidence or missing groove-residual evidence
- the generated locked Feral-grid observer/audio path also proves that observer
  Source Timing detail fields can carry a real locked grid shape: `beat_status`
  `grid`, nonzero beat count, `downbeat_status` `bar_locked`, nonzero bar
  count, `phrase_status` `phrase_locked`, and nonzero phrase count
- strict observer/audio correlation rejects locked observer timing when the
  generated output path still reports static/default or manual-confirm Source
  Timing policy, so control-path grid lock cannot silently mask fallback output
- `just observer-audio-correlate-locked-grid-json-fixture` keeps a committed
  observer/manifest fixture pair for locked-grid Source Timing alignment and
  asserts locked observer grid use, locked manifest grid use, aligned
  grid-use compatibility, aligned anchor evidence, and aligned groove evidence
  before `just audio-qa-ci` can pass
- observer/audio strict JSON correlation also accepts W-30 preview source-diff manifests as output-path evidence, using candidate RMS, active-sample ratio, and RMS delta to reject silent or fallback-collapsed first-playable output
- the listening manifest v1 field-level JSON contract is documented in `docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md`
- a repo-local `scripts/validate_listening_manifest_json.py` helper and `just listening-manifest-validator-fixtures` fixture matrix validate the listening manifest v1 envelope without freezing pack-specific metrics
- `just audio-qa-ci` validates freshly generated W-30 preview, lane recipe, Feral before/after, and Feral grid manifests against the listening manifest v1 envelope
- `just recipe2-observer-audio-gate` correlates a headless app-level documented Recipe 2 MC-202 observer path with a freshly generated lane recipe listening-pack manifest, and requires that the generated observer stream carries the same transport / queue / runtime / recovery snapshot envelope used by the live `riotbox-app --observer` path
- observer/audio JSON summaries include the required `lane_recipe_cases` field
  and expose populated case evidence for lane recipe
  manifests, including MC-202 phrase-grid and Source Graph phrase-slot proof,
  so strict lane timing evidence is inspectable instead of only affecting the
  internal pass/fail verdict
- the generated Recipe 2 observer/audio gate asserts the generated summary's
  `lane_recipe_cases` evidence for the required MC-202 cases, so the visible
  JSON summary and the generated lane recipe manifest must agree on phrase-grid
  and Source Graph phrase-slot proof
- the same Recipe 2 gate now rejects MC-202 lane recipe cases whose generated
  candidate lacks `mc202_phrase_grid` evidence; that metric proves the current
  offline candidate starts on the phrase boundary and its detected note onsets
  stay aligned to the sixteenth grid
- the same Recipe 2 gate also rejects MC-202 lane recipe cases whose generated
  candidate lacks `mc202_source_phrase_slot` evidence; that metric proves the
  current offline candidate consumes a Source Graph phrase-grid slot and starts
  at the selected source phrase boundary. The lane recipe pack now builds that
  phrase grid from generated PCM source evidence through the normal Source
  Timing probe and probe-BPM TimingModel path, but it is still a bounded
  CI-safe proof rather than a production phrase arranger
- generated-pack manifest validation can require referenced artifact and metrics files to exist via `--require-existing-artifacts`
- `just offline-render-reproducibility-smoke` is a CI-safe bounded reproducibility check that renders the same deterministic source-backed W-30 output twice and compares WAV hashes; it is an offline render smoke, not the full export workflow
- `just p011-exit-evidence-manifest` validates the current machine-checkable P011 evidence index across replay, recovery, export reproducibility, and stage-style stability categories, including proof-file existence and repo-local `just` recipe references; it is an evidence index, not an execution gate by itself
- `just p011-exit-evidence-manifest-validator-fixtures` keeps the P011 evidence-index validator honest with the live manifest plus a negative fixture for a missing `just` recipe reference
- `just p011-exit-evidence-gate` is the aggregate bounded executable category gate for the P011 evidence index. It validates the manifest, selects every category, globally deduplicates repeated proof commands, and runs them without shell expansion. It is the CI entrypoint for category-level P011 exit evidence, not a host-audio soak, full arrangement export gate, or endurance gate.
- `just p011-replay-evidence-gate` is the first bounded executable category gate for the P011 evidence index. It validates the manifest, selects the `replay` category, deduplicates that category's proof commands, and runs them without shell expansion. It is CI-safe replay evidence, not a full all-category exit gate.
- `just p011-recovery-evidence-gate` is the second bounded executable category gate for the P011 evidence index. It validates and runs the `recovery` category's proof commands, covering generated recovery observer drills and the recovery-surface test family without claiming automatic startup recovery or real interrupted-host-session rehearsal.
- `just p011-export-evidence-gate` is the third bounded executable category gate for the P011 evidence index. It validates and runs the `export_reproducibility` category's proof commands, covering stable normalized manifest data and audio artifact hashes for the current deterministic Feral grid product-export seam without claiming full arrangement export readiness.
- `just p011-stage-style-evidence-gate` is the fourth bounded executable category gate for the P011 evidence index. It validates and runs the `stage_style_stability` category's proof commands, covering generated repeated-run restore-diversity observer/audio evidence and stable full-mix hashes without claiming host-audio soak or multi-hour endurance coverage.
- `just p011-exit-evidence-category-gate-fixtures` keeps the category runner honest with a replay dry-run and a negative missing-category fixture.
- `just full-grid-export-reproducibility-smoke` / `just product-export-reproducibility-smoke` is a CI-safe bounded export reproducibility check that renders the deterministic Feral grid/source-first plus generated-support pack twice from generated source material, validates both listening manifests, rejects collapsed full-mix output metrics, compares the exported generated-support WAV hashes, and validates a normalized product-export proof with temp paths removed; it is still not the full arrangement export workflow
- `riotbox-core::export_readiness` turns the current product-export
  reproducibility proof into a typed `ExportReadinessContract` for P016. The
  first contract scope is `product_mix`; the first boundary is the
  deterministic Feral-grid generated-support export with `full_grid_mix` as the
  current export role, `feral-grid-demo` as the current pack/recipe identity,
  `reproducible` status, and explicit unsupported-scope flags for stem package
  export, live recording export, DAW export, and host-audio soak.
- the first export action boundary is `export.product_mix`, which writes a
  `full_grid_mix` product artifact plus proof receipt only after the existing
  product-export reproducibility proof and artifact hash check succeed. It is not a
  stem package, live recording, DAW session, host-audio capture, automatic
  arranger export, or automatic Ghost export gate.
- the observer export surface derives `requested`, `started`, `completed`, and
  `failed` lifecycle records from the existing `export.product_mix`
  ActionCommand, queue/action history, and export receipts. Completed records
  include receipt id, export scope, pack id, role, artifact/proof paths,
  hashes, the full-grid WAV artifact-set entry, the product-export proof JSON
  artifact-set entry, per-artifact normalized manifest hash evidence,
  per-artifact source graph lineage when the Session has it, per-artifact
  confirmed timing-grid lineage when the Session has it, per-artifact audio
  metrics and WAV format evidence when the written local product artifact can
  be decoded safely, QA gate id/result evidence, readiness status, and
  unsupported scopes; failed records include the action id and failure reason.
  This is an observer projection, not a second export truth.
- wider P016 export scopes require stronger gates before they are claimed:
  stem packages require per-stem non-silence, role labeling, hash stability, and
  source/capture lineage checks against the per-artifact evidence fields; live recordings require real-session
  host-audio evidence with callback-gap and stream-error summaries; DAW session
  export requires tempo-map and arrangement placement validation against the
  Source Graph/Session timing truth. None of those are covered by the current
  product-export reproducibility smoke.
- `export.stem_package` remains reserved until an implementation can provide a
  package receipt whose `artifact_set[]` contains every claimed stem role, the
  package manifest/proof entries, per-stem hashes, per-stem WAV format/audio
  metrics, and the policy-required source/capture lineage and fallback
  comparison evidence. A UI, Ghost, or CLI path must not show it as ready while
  those gates are absent.
- `ExportScope::StemPackage` is a reserved typed receipt scope only. It lets
  future receipts state `export_scope: stem_package` explicitly, but it does not
  remove unsupported-scope flags or turn the current QA skeleton into full
  stem-package export readiness.
- `riotbox-core::session::validate_stem_package_receipt_readiness` is the
  current receipt-level guard: missing, failed, or deferred
  `stem_package_artifact_set_evidence` gates keep readiness blocked, and even a
  passed gate stays blocked while the receipt still carries the
  `stem_package` unsupported-scope flag.
- Stem-package QA gates must fail when a claimed role is missing, duplicated,
  mislabeled, hashless, locationless, silent by metrics, or missing required
  lineage/fallback evidence. Hash stability and non-silence must be checked per
  stem, not only on a package-level manifest.
- `riotbox-core::export_qa::validate_stem_package_artifact_set_evidence`
  is the current CI-safe stem-package gate skeleton. It validates only
  structure: claimed roles must be stem roles, each claimed role must have
  exactly one artifact-set entry, and each entry must carry a location plus
  sha256. When a claimed stem artifact includes audio metrics, the gate fails
  metrics that prove silence or do not contain enough activity evidence. Missing
  metrics keep per-stem non-silence deferred/aspirational. Callers may enable
  the structural lineage policy to require each claimed stem artifact to carry
  source graph, source capture, or capture-lineage evidence before a wider stem
  scope is accepted. The default gate remains compatible with current
  product-mix callers. Callers may also enable the structural fallback policy
  to require typed source-vs-fallback comparison evidence before accepting a
  claimed stem artifact. Enabled structural policies reject blank lineage
  identities, blank fallback reference identities, and fallback comparison
  payloads with no metric fields, but threshold interpretation and real render
  comparison remain deferred. Passing this skeleton does not claim a full stem
  package export.
- `riotbox-core::session::ExportReceiptQaGateResult::stem_package_artifact_set_evidence`
  records that skeleton as `stem_package_artifact_set_evidence` in receipt
  `qa_gates[]`. Structural failures become `failed`; structural acceptance with
  deferred audio/fallback proof becomes `deferred`, not `passed`, so receipts
  can explain why a stem package is blocked without claiming readiness.
- `riotbox-core::export_qa::validate_stem_package_hash_stability_evidence`
  is the current CI-safe per-stem hash identity gate. It requires one nonblank
  SHA-256 identity for each claimed stem role, fails missing / duplicate /
  hashless / non-stem claims, and records
  `stem_package_per_stem_hash_stability` in receipt `qa_gates[]`. Successful
  identity evidence remains `deferred`, not `passed`, until a package writer or
  repeated render proof can compare stable hashes across actual outputs.
- `riotbox-core::export_qa::validate_stem_package_non_silence_evidence`
  is the current CI-safe per-stem non-silence receipt gate. It records
  `stem_package_per_stem_non_silence` as `passed` only when every claimed stem
  has audio metrics that prove activity, `deferred` when metrics are absent,
  and `failed` when metrics prove silence, cannot prove activity, or claimed
  roles are missing, duplicated, or non-stem. It is metrics evidence, not a
  package writer or listening-pack approval.
- `riotbox-core::export_qa::validate_stem_package_lineage_evidence`
  is the current CI-safe per-stem lineage receipt gate. It records
  `stem_package_per_stem_lineage` as `passed` only when every claimed stem
  artifact carries source graph, source capture, or capture-lineage evidence
  from Session/Core receipt fields. Missing, duplicate, non-stem, lineage-free,
  or blank lineage identities record `failed`. It validates traceability
  evidence only and does not claim package writing or listening approval.
- `riotbox-core::export_qa::validate_stem_package_fallback_comparison_evidence`
  is the current CI-safe per-stem source-vs-fallback comparison receipt gate.
  It records `stem_package_per_stem_fallback_comparison` as `passed` only when
  every claimed stem artifact carries a nonblank fallback reference identity
  and at least one comparison metric field. Missing, duplicate, non-stem,
  comparison-free, blank, or metricless evidence records `failed`. It is
  structural comparison evidence only; real render thresholds remain separate.
- `riotbox-core::session::validate_stem_package_receipt_readiness` requires the
  receipt to be `export_scope: stem_package`, to have no remaining
  `stem_package` unsupported-scope flag, and to carry `passed` status for all
  required stem-package QA gates: artifact-set evidence, per-stem hash
  stability, per-stem non-silence, per-stem lineage, and per-stem fallback
  comparison. Missing, `deferred`, or `failed` gates keep readiness blocked with
  typed blockers.
- `riotbox-core::stem_package_manifest::StemPackageManifest` is the current
  CI-safe schema contract for future stem-package manifests. It serializes
  stable schema id/version, `export_scope: stem_package`, receipt/action
  references, claimed stem roles, typed per-stem WAV artifact identities,
  manifest JSON identity, proof JSON identity, and QA gate summaries. Its
  constructor enforces identity consistency, but it is not a writer, not a
  render path, and not evidence that the actual stem audio is non-silent or
  fallback-safe.
- `StemPackageManifest::from_receipt` is the current CI-safe bridge from
  Session receipt truth to that manifest value. It accepts only
  `export_scope: stem_package`, uses the
  `stem_package_artifact_set_evidence` gate for claimed roles, requires the
  corresponding stem WAV artifact identities plus manifest/proof JSON
  identities in `artifact_set[]`, and carries the receipt QA gate summaries
  forward. Receipt-side manifest/proof JSON file hashes stay in
  `artifact_set[]`; the manifest payload carries only JSON role, location, and
  media type for those identities. This still proves identity wiring only;
  audio non-silence, fallback-safety, and future file writing remain separate
  gates.
- `StemPackageManifest::normalized_json_bytes` is the current CI-safe proof
  input helper. It returns deterministic in-memory pretty JSON bytes and proves
  that stable manifest values serialize identically while artifact identity
  changes alter the proof input. It does not write files or claim package
  export readiness.
- `StemPackageManifest::normalized_json_sha256` is the current CI-safe proof
  identity helper. It hashes `normalized_json_bytes` directly so future
  manifest/proof artifacts can use the same deterministic identity without a
  parallel serializer. Embedded manifest/proof JSON identities omit their own
  eventual file hashes, so this manifest hash is non-circular. It does not write
  package files or claim stem export readiness.
- `riotbox-core::stem_package_proof::StemPackageProof` is the current CI-safe
  stem-package proof JSON schema contract. It records package, receipt/action,
  manifest SHA-256, claimed roles, and manifest/proof JSON identities, but it
  remains an in-memory proof payload only. It does not write files, render
  stems, or make `export.stem_package` ready for musicians.
- `StemPackageProof::from_manifest` builds that proof payload from
  `StemPackageManifest` and its `normalized_json_sha256` helper. It is proof
  identity wiring only; it does not embed the eventual proof-file SHA, write a
  proof JSON file, or claim package export readiness.
- The current stem-package manifest fixture is in-memory and CI-safe: it uses
  claimed drums and bass stems, manifest/proof identities, and deferred QA gate
  evidence, then roundtrips the manifest JSON, derives and roundtrips the proof
  JSON payload, and checks receipt readiness stays blocked. It is a contract
  fixture, not a listening pack, package writer, or proof that
  `export.stem_package` is ready for musicians.
- Future stem-package writer QA contract:
  - reusable product-export evidence: local artifact hashing, local proof file
    hashing, receipt-side `artifact_set[]` projection, source graph and
    timing-grid receipt evidence, safe post-write WAV metric extraction,
    recovery preflight for local artifact paths, and observer export lifecycle
    projection from queue/history plus Session receipts
  - new evidence required before readiness: one written WAV per claimed stem
    role, per-stem format metrics, per-stem non-silence, per-stem hash
    stability across repeated writer/render output, per-stem source/capture or
    capture-lineage evidence when policy requires it, per-stem source-vs-
    fallback comparison evidence when policy requires it, manifest JSON file,
    proof JSON file, and a package/render profile identity distinct from the
    current product-mix Feral-grid proof
  - gate order: render/write stems outside realtime audio; decode/measure
    written WAVs; hash stems; attach lineage and fallback evidence from
    Session/Core; build manifest and proof payloads from receipt evidence; write
    and hash manifest/proof JSON; run stem-package artifact-set, hash-stability,
    non-silence, lineage, and fallback-comparison gates; then commit the receipt
    only if the scope is no longer unsupported and all required gates pass
  - realtime boundary: filesystem writes, hashing, decoding, metric extraction,
    QA comparison, Ghost/model calls, and observer emission must remain outside
    the realtime audio callback
  - replay/restore boundary: replay may validate package metadata and artifact
    availability, but must not regenerate stems or rewrite package files without
    a fresh explicit export request
  - manifest/proof identity rule: receipt `artifact_set[]` entries own written
    manifest/proof JSON file hashes; manifest/proof payload identities own only
    JSON role, location, and media type. The writer must keep this boundary so
    proof `manifest_sha256` and the eventual proof-file SHA are computed from
    final payload bytes without self-hash cycles.
- Observer export snapshots project those receipt `qa_gates[]` values as-is,
  including non-product stem-package evidence. For `export_scope: stem_package`
  receipts, observer snapshots also project `stem_package_readiness` with the
  Core-derived readiness status, typed blockers, and blocker labels from
  `validate_stem_package_receipt_readiness`. The observer surface is evidence
  projection from Session/Core receipt truth, not a second readiness engine and
  not permission to surface `export.stem_package` as runnable.
- `just stage-style-snapshot-convergence-smoke` is a CI-safe app-level replay convergence check for the current supported stage-style seam. It proves a mid-run snapshot payload can hydrate and replay a Scene / MC-202 / TR-909 suffix to the same final mixed buffer as the originally committed path. It is not a broad crash-recovery drill, host-audio run, or proof that every possible stage gesture is replay-supported.
- `just interrupted-session-recovery-probe` is a CI-safe file-backed drill for observer recovery evidence; it is still not automatic startup recovery and does not execute a restore
- `just missing-target-recovery-probe` is the sibling file-backed drill for a missing normal load target with a parseable autosave clue; it keeps the same read-only manual recovery boundary
- `just stage-style-stability-smoke` / `just stage-style-stability-proof` is a CI-safe repeated-run smoke around the generated stage-style restore-diversity path; it catches obvious intermittent observer/audio collapse and full-mix nondeterminism, validates a normalized proof JSON whose top-level proof hash excludes run-local audit hashes, but remains a bounded smoke rather than a real host-audio soak or multi-hour endurance test
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

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
- source-derived dropout, stutter, and restore tails expose candidate counts,
  distance from fixed recipes, and output contrast instead of passing as one
  hardcoded destructive ending

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

Any listening manifest that still contains `pattern_origin: "primitive_renderer"`
must also include `primitive_renderer_boundary` with
`evidence_role: non_product_diagnostic_control`,
`product_output_allowed: false`, `demo_readiness: unverified`,
`quality_proof: false`, `promotion_blocked: true`, and exact affected paths for
the primitive origins. Missing or stale primitive-boundary metadata is a
manifest validation failure, because otherwise a hardcoded renderer can slip
back into musician-facing proof as if it were source-derived output.

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

This rule applies across the whole product. A hardcoded phrase, fixed template,
scripted arrangement, fingerprint-only variation, or source-aware mutation may
be useful diagnostic evidence, but it is not source-derived quality proof by
itself. Any lane that claims source-derived intelligence must show that source
features changed the musical decision and rendered output.

For any pack presented as a source showcase:

- validate reproducibility within the same source separately from diversity
  across different sources
- reject identical or near-identical full mixes across distinct source files
- reject source-backed stems that are byte-identical across distinct source
  files unless the fixture explicitly proves those sources contain the same
  selected window
- reject source-independent generated stems, such as fixed TR-909 or MC-202
  support, when they are loud enough to dominate the source-backed material
- reject source-derived claims where removing the source feature vector leaves
  the same musical role, step placement, contour, destructive gesture, or
  arrangement decision
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

For MC-202 producer-grade review scaffolding, use the dense/non-dense
real-source listening pack:

```bash
just mc202-real-source-listening-pack-smoke
```

This writes source windows, MC-202 stems, generated-support mixes, listening
review packs, source-expression summaries, selected motif metadata, and a
primitive A/B control that is explicitly non-product evidence. The control must
keep `product_fallback_allowed: false`; it is not fallback music and cannot
support a product-quality claim.

For MC-202 producer-grade closeout, run:

```bash
just mc202-producer-grade-closeout-smoke
```

This gate aggregates the professional output listening pack, the real-source
listening scaffold, and the MC-202 source-composed review gate. It must pass
only as a technical closeout while keeping `quality_claim_allowed: false`,
`demo_bank_promotion_allowed: false`, and `parent_ticket_state: keep_open`
until structured listening records a human pass/weak/fail verdict. A primitive
or template-only MC-202 candidate remains a production blocker, not a product
fallback or proof of musician-ready quality.

The professional-output listening-pack and MC-202 closeout smoke recipes keep
their JSON contracts in repo-local validators instead of long inline `jq`
expressions. Validator extraction is QA maintainability only: it must preserve
or tighten the same source-composed evidence, no primitive/template product
output, artifact identity, and human-verdict blocking semantics.

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
- source-first feral mixes with generated/source-backed RMS ratio above `0.16`
  are treated as masking the source-backed W-30 lane
- generated-support feral mixes must keep generated/source-backed RMS ratio
  between `0.16` and `0.46`, so support stays audible without becoming a
  source-masking render
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
- The professional-output suite aggregates Feral grid `metrics.mix_balance`
  across its child manifests. It must fail when source-first renders let
  generated support mask the source, when generated-support renders bury support
  below a useful audible floor, or when generated support dominates the source
  window. This remains diagnostic evidence, not an automated musical pass.
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
- Professional source-WAV tonal-hook diagnostics must keep the W-30 source-chop
  strong enough to carry the hook: the tonal case fails when
  `proof.w30_to_source_rms_ratio` falls below `0.22`. This protects hook
  audibility only; it remains diagnostic evidence with
  `human_verdict: unverified`.
- Dense-break Hook/Chop diagnostics use the same `0.22` W-30/source floor for
  hook-forward proof. Sparse-bass-pressure diagnostics keep the lower general
  W-30 floor so bass remains the strongest element instead of being obscured by
  a hook-forward policy meant for dense/tonal material.
- Sparse-bass-pressure diagnostics must prove more than source-derived
  movement: movement must be at least `1.25 Hz` away from the fixed contour,
  span at least `8.0 Hz`, keep pressure low-band lift at or above `1.60`, and
  leave bass as the strongest audible element with at least `0.08` dominance
  margin. These remain scripted diagnostic gates, not musical-pass claims.
- Destructive-variation professional diagnostics require a hard dropout/stutter
  contrast and an impact restore: `dropout_to_stutter_rms_ratio <= 0.10`,
  `stutter_to_hook_transient_ratio >= 1.20`, and
  `restore_to_pressure_rms_ratio >= 1.22`. These thresholds prove the
  diagnostic output did not collapse to a flat edit; they do not approve the
  render as product-quality audio without human listening.
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

Professional-output listening packs may carry `audio_judge_label` metadata so a
later recorded human verdict can be imported into the human listening label
corpus without re-identifying artifacts by hand. The metadata is not a verdict.
Import must reject `human_verdict: unverified`; when artifact hash checking is
requested, import must also reject stale or missing performance-report,
agent-review, source-window, and full-performance artifacts.

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

### 3.2.4 Audio judge spike

P021 adds `riotbox.audio_judge_spike.v1` as an offline QA spike for deciding
whether a future audio judge is worth building from Riotbox-owned metrics plus
optional CLAP/MERT-style music embeddings.

The spike is not a runtime dependency and not a taste oracle. It must keep:

- `human_verdict: unverified` unless structured human listening has been
  recorded
- deterministic Riotbox metrics as the baseline provider
- optional model providers isolated to offline QA
- confusion or coverage examples, not only average scores
- a recommendation of `useful`, `too_weak`, `too_expensive`, or `not_ready`

The current implementation reports `not_ready` when the label corpus is too
small, weak/fail labels are not matched to generated review packs, or optional
providers are unavailable. This is expected for the first spike; the value is
making calibration gaps explicit instead of letting agents claim musical pass
from metrics alone.

Run:

```bash
just audio-judge-spike-fixtures
just audio-judge-spike-generated-smoke
```

### 3.2.5 Musical pass gate policy

P021 defines `riotbox.musical_pass_gate_policy.v1` as the verdict-language
contract for technical, automated, human, and future calibrated-agent musical
quality claims.

Allowed states:

- `technical_fail`: technical path failed; no sound-quality claim allowed.
- `technical_pass`: technical validity only; no musical quality claim allowed.
- `agent_fail`: automation caught known bad output; block or fix.
- `agent_weak`: output renders but musical guardrails are weak; diagnostic only.
- `agent_promising`: known weak modes were not caught; useful merge evidence but
  still `human_verdict: unverified`.
- `human_musical_pass`: structured listening review approved the audible result.
- `human_musical_fail`: structured listening rejected the result or marked it
  technically OK but musically weak.
- `calibrated_agent_musical_pass`: future offline judge approval for a bounded
  source family after label coverage and validation.

Only `human_musical_pass` and `calibrated_agent_musical_pass` may claim musical
pass. `calibrated_agent_musical_pass` is not human approval: it must keep
`human_verdict: unverified`, remain offline-QA-only, require matched
pass/weak/fail labels, and include confusion or failure examples.

PR language rules:

- Metrics or logs may say `technical_pass`; they must not say "sounds good".
- Automated reports may say `agent_fail`, `agent_weak`, or `agent_promising`;
  they must not say `musical_pass`.
- A human listening pack may say `human_musical_pass` or `human_musical_fail`
  only after a recorded structured verdict.
- A future judge may say `calibrated_agent_musical_pass` only inside the
  documented source-family boundary and only after the policy fixture validates
  the label and provider requirements.

P022 professional-output diagnostics may also expose
`strongest_audible_element` as bounded machine evidence. Allowed automated
labels are `kick`, `snare`, `bass`, `stab`, `silence`, and `restore`, with
score, margin, candidate-count, and ambiguity fields. This answers "what is
currently hitting hardest?" for software and musician-facing review packs; it
does not approve the output as musical quality and must remain paired with
`quality_proof: false` and `human_verdict: unverified` until a structured human
or calibrated-agent gate approves it.

P023 dense-break diagnostics additionally gate physical drum pressure for the
source family that claims snare/break impact. A `dense_break` report must keep
`strongest_audible_element == "snare"` and expose bounded
`dense_break_snare_pressure_margin` and
`dense_break_physical_drum_pressure_score` fields. Dense reports must also
expose `dense_break_pressure_transient_to_hook_ratio` so the pressure-lift
section proves it kept enough break/snare transient relative to the hook/chop
section instead of becoming only low-band support. These fields prove that the
current scripted render has a dominant snare/break transient with low-band
support; they still do not turn the artifact into a musical quality proof.

P022 rebuild-only/source-layer-off diagnostics may expose transformed-source
survival evidence. The current bounded fields are
`rebuild_only_source_spectral_similarity`,
`rebuild_only_source_transient_retention`,
`rebuild_only_source_rms_retention`, and
`rebuild_only_source_character_survival_score`. These fields prove only that
some source-like spectral/transient character survives after raw source masking
is removed and raw-copy correlation is still rejected; they do not prove that
the result is musically good.

P023 Hook/Chop source selection may also expose
`hook_chop_source_character_score_floor`,
`hook_chop_source_character_score_mean`, and
`hook_chop_source_character_score_span`. These fields prove that source-backed
hook/chop/riff windows were selected from windows with enough source identity
and enough variation across selected offsets. They are a bounded selection
contract, not a musical pass.

The dense-break professional diagnostic also keeps a real weak-WAV regression
for this boundary:

```bash
just dense-break-weak-source-character-fixture-smoke
```

That smoke renders an intentionally weak `06_rebuild_only_performance.wav` and
requires the report validator to reject it with
`rebuild_only_source_character_not_surviving`. It is negative diagnostic
evidence only; it must keep `quality_proof: false` and
`human_verdict: unverified`.

Weak professional-output routing must turn known failure codes into concrete
sound-fix categories instead of leaving engineers with raw metric names. Current
categories are `source_selection`, `chop_policy`, `drum_pressure`,
`bass_movement`, `mix_bus`, `destructive_gesture`, `fixture_threshold`, and
`ui_cue`. Each routed case must include a short `musician_fix_reason`, and
unknown weak/fail codes must fail routing with an unknown-route error until a
stable category is added. Rebuild-only source-character failures route first to
`source_selection` because the musician-facing fix is to pick or expose source
material whose identity survives the rebuild-only path.

Professional output listening packs must include compact demo-readiness reasons
for every review case:

- `demo_readiness`: currently `unverified` unless a structured human verdict
  has promoted the artifact
- `demo_worthy_reason`: why the artifact is worth human review, based on
  existing proof such as strongest audible element, source-character survival,
  pressure, restore, or bass/chop target
- `not_demo_worthy_reason`: why the artifact is not demo-ready yet, usually
  because `human_verdict` is still `unverified` and scripted diagnostics cannot
  claim product quality

The same reasons must appear in `review.json` and the human review prompt so a
reviewer hears the candidate with the intended musical target and the current
quality boundary in view.

Release-demo human-review queues use
`riotbox.release_demo_human_review_queue.v1`. They are review worklists, not
quality proof. Every queued candidate must remain
`human_verdict: unverified`, `demo_readiness: unverified`, and
`quality_claim: false` until a structured listener verdict is recorded. Each
queue entry must carry enough musician-facing context for an outside reviewer to
listen with intent:

- source family and demo-bank source-family alias
- strongest audible element
- source-character summary
- hook-within-two-bars summary
- destructive contrast, bass/drum pressure, live-triggerability, and eight-bar
  replay-value summaries
- `demo_worthy_reason`
- `not_demo_ready_reason` that explicitly names the unverified human verdict and
  blocked quality claim
- review blockers such as unverified verdict/readiness, blocked quality claim,
  and missing family coverage
- required listening questions covering strongest element, source survival,
  first-two-bar hook, live gesture contrast, demo-worthiness, and concrete
  follow-up
- required verdict path for `pass`, `weak`, and `fail`, with the current state
  fixed at `human_verdict:unverified/demo_readiness:unverified`

Queue validation must reject missing source-character or strongest-element
context, incomplete listening questions, stale verdict state, or any quality
claim. This keeps the queue useful for human review without promoting
unlistened artifacts into release-ready evidence.

P023 sound-quality readiness reports must aggregate the current
release-demo human-review queue when it exists. The readiness report should
surface queue availability, priority counts, source families, review blockers,
and per-candidate review context: strongest audible element, source character,
demo-worthy reason, not-demo-ready reason, required verdict state, and required
listening questions. Queue entries remain claim blockers while they are
`human_verdict: unverified`; a readiness report must reject stale queue verdict
state, missing review blockers, or any queued candidate with
`quality_claim: true`. This makes the readiness report useful as a release
blocker and reviewer worklist without turning unreviewed artifacts into quality
proof.

Large professional-output JSON contracts belong in named repo-local validators,
not in oversized inline `jq` blocks inside `Justfile`. `just` recipes may keep
small smoke assertions and compact negative mutations, but cross-report musical
thresholds, evidence-boundary checks, artifact existence checks, and failure-code
names should live in validator scripts so future sound-quality gates stay
reviewable without weakening proof.

The P023 edge-source diagnostics and non-dense professional proof pack follow
the same rule: their smoke recipes must call report validators with mutation
fixtures instead of duplicating source-family coverage, human-verdict,
diagnostic-only, artifact, silence, identity-collapse, and weak-routing checks
as shell `jq`. Passing those validators proves the diagnostic boundary and
contract shape only; it does not promote scripted edge/non-dense renders to
product-quality proof.

Run:

```bash
just musical-pass-gate-policy-fixtures
```

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
  `silence`, `restore`, or `none`
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

Human labels intended for future audio-judge calibration must use
`riotbox.human_listening_label_corpus.v1`. The corpus stores labels by
review-pack identity and SHA-256 artifact hashes so local source audio does not
need to be committed. It distinguishes `pass`, `weak`, `fail`, and
`inconclusive` human labels from technical or agent-promising status.

Structured `riotbox.listening_review.v1` reviews can be imported into the label
corpus only when they carry explicit `audio_judge_label` metadata with source
family/id, review pack identity, artifact identity hashes, created date, and
reason tags. The importer maps structured listening verdicts to label verdicts:
`keep` to `pass`, `technically_ok_but_musically_weak` to `weak`, `reject` to
`fail`, and `inconclusive` to `inconclusive`. It must reject `unverified`
reviews and reviews missing the audio-judge metadata so human labels cannot be
created from chat memory or vague listening notes.

Run:

```bash
just listening-review-label-import-fixtures
```

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
- `dense-break-performance-pack`
- `agent-musical-review-pack`

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
- one output-path assertion, such as non-silence, peak/RMS range, source-vs-control metric delta, or a fixture-backed WAV artifact comparison

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
- a W-30 source-vs-control wrapper that renders a synthetic non-product baseline, source-backed WAV preview as candidate, and requires minimum RMS / sum deltas so control collapse is caught
- a CI-safe generated W-30 source-vs-control smoke that uses deterministic synthetic source material, checks minimum source-vs-control deltas, validates the generated listening manifest, and runs under `just audio-qa-ci`; existing command names may still say `source-vs-fallback` for compatibility, but the baseline is diagnostic control only
- a CI-safe first-playable Jam probe, `just first-playable-jam-probe`, that combines synthetic source material, W-30 source-vs-control output evidence, and a generated app-level observer probe for the current `space -> capture -> raw audition -> promote -> W-30 hit` user path
- a CI-safe source timing confirmation probe, `just source-timing-confirmation-probe`, that presses the real `C` control against a manual-confirm Source Graph, validates the normal observer stream, asserts the immediate `source_timing.confirm_grid` commit, and proves `grid_confirmed` runtime state appears without changing analyzer cue / warning evidence
- a CI-safe source transport map/capture probe, `just source-transport-map-capture-probe`, that starts in manual-confirm listen-first mode, confirms the grid, seeks the Source Map, captures a bar-aligned source window, raw-auditions, promotes, triggers W-30, and correlates the observer path with W-30 source-vs-control output evidence
- a CI-safe stage-style Jam probe, `just stage-style-jam-probe`, that uses generated app-level multi-boundary observer evidence, generated W-30 source-vs-control output evidence, and summary-level commit boundary assertions for `Phrase`, `Bar`, and `Beat`
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
- a first local grid-locked Feral demo render pack that writes TR-909 beat/fill, W-30/Feral source-chop, primitive MC-202 source-grid proof bass pressure, and combined mix WAVs from one shared beat/bar/frame grid, then checks MC-202 pressure role, low-band RMS, and low/mid-band dominance without injecting a hardcoded MC-202 question/answer phrase or presenting the MC-202 proof phrase as source-derived phrase planning
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
- The current internal stem-package local CI proof uses a separate receipt and
  package identity: `export_scope: stem_package`,
  `pack_id: stem-package-local-ci`, `export_role: package_manifest`, and
  boundary `stem_package.local_ci_package_v1`. Its readiness proof is the
  stem-package artifact-set, hash-stability, non-silence, lineage, and
  fallback-comparison gate set; it must not reuse the product-mix
  `feral-grid-demo/full_grid_mix` identity.
- `just stem-package-local-ci-report-smoke` is the CI-safe operator-report
  smoke for that proof path. It creates a temp local CI package through the CLI,
  validates the ready read-only report, removes one stem file, and validates the
  blocked missing-file report. It proves report/readiness plumbing only, not
  final DAW export UX or listening approval.
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
  source/capture lineage checks against the per-artifact evidence fields; live
  recordings require real-session `live_recording_host_audio_refs[]` evidence
  with host/device, callback-gap, stream-error, and duration summaries plus a
  Core/Session readiness report that blocks missing evidence, blank host/device
  identity, zero duration, callback-gap threshold breaches, stream errors, and
  still-unsupported live-recording scope flags; DAW session export requires
  tempo-map and arrangement placement validation against the Source
  Graph/Session timing truth. None of those are covered by the current
  product-export reproducibility smoke.
- live-recording observer receipt snapshots project
  `live_recording_host_audio_readiness` from the Core/Session receipt report
  alongside `live_recording_host_audio_refs[]`. This projection explains
  blocked/ready evidence states for real committed receipt lifecycles only; it
  does not make `export.live_recording` runnable or synthesize lifecycle from
  observer-only state.
- `riotbox-app --live-recording-readiness-report --session <session.json>` is a
  read-only operator report over Session receipts. It writes no files, emits no
  observer events, launches no host, captures no audio, and does not make
  `export.live_recording` runnable; it only reports whether the latest
  `export_scope: live_recording` receipt has sufficient host-audio evidence for
  the current Core/Session readiness contract.
- `just live-recording-readiness-report-smoke` exercises that report through
  the built `riotbox-app` binary for ready and blocked receipt evidence.
- `export.stem_package` remains reserved until an implementation can provide a
  package receipt whose `artifact_set[]` contains every claimed stem role, the
  package manifest/proof entries, per-stem hashes, per-stem WAV format/audio
  metrics, and the policy-required source/capture lineage and fallback
  comparison evidence. A UI, Ghost, or CLI path must not show it as ready while
  those gates are absent.
- `ExportScope::StemPackage` is no longer only a future receipt label: the
  current app has an internal `stem_package.local_ci_package_v1` commit proof for
  deterministic drums/bass CI stems. That proof may remove the stem-package
  unsupported-scope flag only when the written artifact set, per-stem hashes,
  non-silence metrics, lineage, and fallback-comparison gates pass. It still
  does not make a full musician-facing DAW/stem export workflow ready.
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
- The current local CI package writer records the hash-stability gate as
  `passed` only for its deterministic repeated fixture proof boundary. Wider
  stem renderers must supply their own repeated-output hash evidence before they
  can claim the same gate.
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
- The CI-safe ready stem-package receipt fixture exercises that positive
  readiness path with explicit stem artifact identities, active audio metrics,
  lineage, fallback comparison, manifest/proof identities, and all required
  gates passed. It is a contract fixture only: no files are written, no package
  writer runs, no audio proof or listening verdict is produced, and the
  artifact-set/hash-stability gates are fixture writer-proof placeholders until
  real writer and repeated-render evidence exist.
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
  - first allowed writer boundary: `stem_package.local_ci_package_v1`, a future
    local app-side package writer for an explicit Session export request. Its
    stem source is deterministic offline stem render providers declared by
    role; the first implementation boundary should start with roles that
    already have receipt/fixture proof, and must reject unsupported role claims
    instead of producing placeholder or fallback-only stems.
  - current code-level skeleton only plans that boundary. It validates local
    destination and bounded drums/bass claims, then returns final artifact
    identities without writing WAV/JSON files, hashing outputs, running audio
    metrics, or producing listening-review evidence.
  - current CLI dry-run exposes that plan through
    `riotbox-app --stem-package-local-ci-dry-run`. It requires an explicit
    local destination and claimed roles, reports planned artifact paths,
    supported/unsupported role claims, and readiness blockers, and must keep
    `writes_files: false`. This proves only the control-plane plan and negative
    blocking surface; it cannot satisfy stem-package audio QA gates or claim
    musician-ready export output.
  - current CLI execute exposes the bounded writer proof through
    `riotbox-app --stem-package-local-ci-execute`. It requires an explicit
    Session, local destination, and supported claimed roles, writes the
    deterministic local CI package through the committed writer/action path,
    saves the Session receipt, and can emit observer evidence. This satisfies
    the internal local CI writer proof for drums/bass only; it is still not a
    structured listening-review pack, DAW export workflow, or general
    musician-facing `export.stem_package` control.
  - current CLI operator report exposes the written proof summary through
    `riotbox-app --stem-package-local-ci-report --session <session.json>`. It is
    read-only: it reports the latest stem-package Session receipt, stem roles,
    manifest/proof identities, QA gate status, Core readiness blockers, local
    artifact availability, and missing package files without writing observer
    events, regenerating artifacts, or treating product-mix receipts as stem
    packages. This helps operators inspect the internal proof boundary; it is
    still not a listening review, DAW export workflow, or musician-facing export
    readiness claim.
  - current CI writer proof emits files for that boundary inside
    `riotbox-app::jam_app::stem_package_writer`: deterministic drums/bass WAVs
    are written through a staging directory and promoted into the final package
    layout, decoded for format and non-silence metrics, hashed from the final
    paths, represented in `artifact_set[]`, and paired with manifest/proof JSON
    hashes. A repeated fixture run proves stable per-stem hashes for identical
    inputs. No structured listening-review pack exists for this proof; it is
    CI-safe written-artifact evidence with `human_verdict: unverified`.
  - current musician-facing surface gate is separate from receipt readiness:
    a ready local CI stem-package receipt can satisfy the internal writer proof,
    per-stem QA gates, observer projection, operator report, and package
    identity, but the TUI/Ghost/user export surface remains disabled while the
    package is developer-proof-only, DAW placement is missing, or structured
    listening review is not verified. Reserved UI/Ghost-style attempts must
    reject with those blockers and must not write files or commit a receipt.
  - current arrangement / DAW placement contract skeleton reserves
    `daw_session` receipt identity and `arrangement_placement_refs[]` so future
    DAW export QA can prove scene/bar/beat placement separately from local file
    availability. It is a Session receipt contract and observer/recovery report
    surface only; it writes no DAW files and does not approve an audible DAW
    handoff.
  - current DAW tempo-map contract skeleton adds `daw_tempo_map_ref` to
    `daw_session` receipts so future DAW export QA can prove tempo-map evidence
    separately from arrangement placement and local file availability. It stores
    receipt evidence for the existing confirmed timing boundary; it writes no
    DAW tempo map and does not approve an audible DAW handoff.
  - current DAW operator report exposes the combined receipt/preflight status
    through `riotbox-app --daw-export-readiness-report --session
    <session.json>`. It is read-only: it reports the latest DAW-session receipt,
    placement readiness, tempo-map readiness, unsupported-command blockers,
    local artifact preflight, missing/unreadable files, proof-gate status for
    JSON package integrity, writer proof, host-import proof, and audible-output
    proof, and release blockers without writing DAW files, observer events, or
    musician-facing export state. `developer_proof_only` remains fixed; writer,
    host-import, and audible-output blockers clear only when their own proof
    gates pass. The report's `proof_stack` summary makes stacks explicit when
    they are complete but still developer-proof-only. A `ready_for_writer`
    report means the receipt is ready for the next DAW-writer implementation
    gate only.
  - `just daw-export-readiness-report-smoke` is the CI-safe operator-report
    proof for that path. It runs the real binary against a temporary Session,
    validates the ready-for-writer report, validates a complete
    developer-proof-only proof stack that still leaves musician-facing DAW
    export disabled, removes the manifest file, and validates the missing-file
    blocker.
  - current DAW writer plan skeleton exposes deterministic planned identities
    through `riotbox-app --daw-session-writer-plan --session <session.json>
    --daw-session-destination <dir>`. It is a dry-run only: it reuses the
    operator readiness report, reports planned arrangement manifest, tempo-map,
    and DAW-session proof JSON paths under `daw_session/`, carries placement
    refs and tempo-map refs forward, and keeps `daw_writer_missing` explicit.
    It writes no files, creates no destination directory, emits no observer
    events, and is not a musician-facing DAW export action.
  - `just daw-session-writer-plan-smoke` is the CI-safe proof for that planning
    path. It runs the real binary, validates the ready-for-writer dry-run plan,
    removes the arrangement manifest source file, validates the missing-file
    blocker, and proves the destination directory was not created.
  - current DAW-session manifest/tempo-map/proof payload contracts live in
    `riotbox-core::daw_session_manifest`,
    `riotbox-core::daw_session_tempo_map`, and
    `riotbox-core::daw_session_proof`. They provide deterministic normalized
    JSON/hash contracts for the planned DAW manifest, tempo map, and proof from
    receipt evidence, placement refs, tempo-map refs, source artifact
    identities, and planned DAW JSON identities where applicable. They are
    schema and proof-input contracts only: no DAW files are written, no Session
    is mutated, no audio output is produced, and no musician-facing DAW export
    is enabled.
  - current DAW writer plan payload preview exposes those contracts through the
    read-only CLI report. A ready preview reports planned manifest/tempo/proof
    paths, normalized manifest and tempo-map hashes, and proof/manifest hash
    linkage; a blocked preview carries typed upstream blockers and emits no
    hashes. This proves payload shape only, not DAW file export completion, DAW
    placement correctness in a host, or audible output.
  - current DAW JSON writer proof is internal and CI-safe only. It writes the
    manifest, tempo-map, and proof JSON payloads through staging, promotes the
    package, hashes final JSON files, and verifies proof/manifest linkage. This
    proves local JSON file emission only, not DAW audio output, host import
    correctness, observer lifecycle, Session mutation, or a musician-facing
    `export.daw_session` workflow.
  - current DAW JSON package execute CLI exposes that proof explicitly through
    `riotbox-app --daw-session-json-package-execute --session <session.json>
    --daw-session-destination <dir>`. It writes only the local JSON package,
    reports hashes plus the package report, and keeps Session mutation,
    observer lifecycle, host import correctness, audible output, and
    musician-facing DAW export out of scope. `just
    daw-session-json-package-execute-smoke` is the bounded proof for that
    real-binary path.
  - current DAW JSON package report reads a local `daw_session/` package
    without writing, validates expected schema ids, verifies proof/manifest hash
    linkage, reports final JSON SHA-256 values, and surfaces typed blockers for
    missing files, invalid JSON, schema mismatch, or hash mismatch. This proves
    local JSON package integrity only, not DAW host import correctness or
    audible output.
  - current DAW JSON package receipt evidence records that local package result
    in Session/Core receipt truth via `artifact_set[]` entries for
    `export_manifest`, `daw_session_tempo_map`, and `product_export_proof`, plus
    the `daw_session_json_package_integrity` QA gate. This proves receipt
    handoff of JSON package evidence only; it still does not prove DAW host
    import correctness, observer lifecycle completion, or audible output.
    `riotbox-app --daw-session-json-package-evidence-apply --session
    <session.json> --daw-session-destination <dir>` is the current real-binary
    path for that handoff, and `just
    daw-session-json-package-evidence-apply-smoke` proves the saved Session
    receipt evidence after the package execute path.
  - current DAW session surface gate keeps musician-facing DAW export disabled
    even when receipt and JSON package evidence are ready. The remaining
    blockers explicitly name developer-only status, missing DAW writer, missing
    DAW host-import proof, and missing audible-output proof so JSON package
    success is not mistaken for a playable DAW export.
  - current DAW host-import proof evidence is a reserved receipt QA gate:
    `daw_session_host_import_proof`. Missing or failed proof keeps
    `daw_host_import_proof_missing` visible; passed proof removes only that
    blocker and still does not prove audible output or make `export.daw_session`
    runnable. The gate may pass only after a passed `daw_session_writer_proof`
    exists on the same receipt; out-of-order host-import evidence records
    `daw_writer_proof_missing` and remains failed.
  - current DAW host-import proof apply path is explicit and evidence-only:
    `riotbox-app --daw-session-host-import-proof-apply --session
    <session.json> --daw-session-host-import-proof <proof.json>` reads a local
    `riotbox.daw_session_host_import_proof` JSON report and mutates only the
    latest DAW-session receipt's host-import QA gate. It is not a DAW host
    runner, not a DAW writer, not an observer lifecycle event, and not audible
    output proof.
  - current `export.daw_session` host-import-proof action boundary is
    `host_import_proof_v1`. It commits an existing local
    `riotbox.daw_session_host_import_proof` JSON report through queue/history,
    Session action log, commit record, and matching receipt evidence only after
    the same receipt has passed writer proof. It attaches only
    `daw_session_host_import_proof`, emits observer lifecycle evidence, writes
    no files, launches no host, captures no audio, and leaves
    `developer_proof_only` plus `audible_output_proof_missing` visible.
  - current DAW audible-output proof evidence is a reserved receipt QA gate:
    `daw_session_audible_output_proof`. Missing or failed proof keeps
    `audible_output_proof_missing` visible; passed proof removes only that
    blocker and still does not launch a host, capture audio, write DAW files, or
    make `export.daw_session` runnable. The gate may pass only after passed
    `daw_session_writer_proof` and `daw_session_host_import_proof` gates exist
    on the same receipt; out-of-order audible-output evidence records missing
    prerequisite blockers and remains failed.
  - current DAW audible-output proof apply path is explicit and evidence-only:
    `riotbox-app --daw-session-audible-output-proof-apply --session
    <session.json> --daw-session-audible-output-proof <proof.json>` reads a
    local `riotbox.daw_session_audible_output_proof` JSON report and mutates
    only the latest DAW-session receipt's audible-output QA gate. It is not a
    DAW writer, not a host runner, not live audio capture, and not an observer
    lifecycle event. `just daw-session-audible-output-proof-apply-smoke` proves
    the real-binary Session mutation while `export.daw_session` stays disabled.
  - current `export.daw_session` audible-output-proof action boundary is
    `audible_output_proof_v1`. It commits an existing local
    `riotbox.daw_session_audible_output_proof` JSON report through
    queue/history, Session action log, commit record, and matching receipt
    evidence only after the same receipt has passed writer proof and
    host-import proof. It attaches only `daw_session_audible_output_proof`,
    emits observer lifecycle evidence, writes no files, launches no host,
    captures no live audio, and leaves `developer_proof_only` visible.
  - first DAW session writer/action boundary is reserved as
    `daw_session.local_project_writer_v1` for the first bounded
    `export.daw_session` local-writer commit path. It sits after the CI-safe
    `daw_session.json_package_writer_v1` JSON package proof and before
    host-import or audible-output proof. Current code also has a typed reserved
    `export.daw_session` queue-history guard that rejects attempts with the DAW
    surface-gate reason and records destination/receipt intent without writing
    files, receipts, observer lifecycle records, host checks, or proof
    artifacts.
  - current local-writer `export.daw_session` commit path uses the same
    `daw_session.local_project_writer_v1` boundary only after the DAW writer
    plan is ready and `daw_session_json_package_integrity` has passed. It runs
    the existing staged local writer proof, records a committed action and
    commit record, attaches `daw_session_writer_proof` to the matching receipt,
    and still emits no host-import proof, audible-output proof, live capture, or
    final musician-facing export enablement.
    `just daw-session-writer-export-execute-smoke` runs the real binary through
    this queue/commit path, saves the Session mutation, and verifies the
    optional observer lifecycle while still writing only local writer proof
    JSON files.
  - current observer export snapshots include `export.daw_session` lifecycle
    records only when a real queued DAW-session action exists. Rejected reserved
    attempts produce requested / started / failed records without a receipt;
    committed local-writer proof actions produce requested / started /
    completed records with the matching DAW-session receipt and proof-gate
    summary; committed host-import-proof actions do the same for
    `host_import_proof_v1`; committed audible-output-proof actions do the same
    for `audible_output_proof_v1`. This is observer evidence, not live capture,
    host launch proof, or musician-facing DAW export readiness.
    `just daw-session-host-import-proof-export-execute-smoke` proves the
    host-import action path through the real binary without launching a host or
    writing DAW files.
  - current DAW-session writer proof skeleton writes only bounded local proof
    artifacts through `riotbox-app --daw-session-writer-proof-execute
    --session <session.json> --daw-session-destination <dir>`. The proof
    requires a passed JSON package gate, uses staging, emits
    `daw_session_writer/local_project_skeleton.json` and
    `daw_session_writer/writer_proof.json`, mutates no Session, emits no
    observer events, launches no host, and captures no audio. `just
    daw-session-writer-proof-smoke` proves that real-binary path.
  - current DAW-session writer proof apply path mutates only receipt evidence:
    `riotbox-app --daw-session-writer-proof-apply --session <session.json>
    --daw-session-destination <dir>` attaches `daw_session_writer_proof` and a
    writer-proof artifact entry to the latest DAW-session receipt. That writer
    proof is surfaced by the DAW operator report under
    `proof_gates.writer_proof`, including gate status and matching artifact
    availability. It is still not host-import proof and not audible-output
    proof.
  - current observer export snapshot also projects the latest DAW-session
    receipt's `proof_gates` and `proof_stack` so observer consumers see the same
    JSON package, writer-proof, host-import, and audible-output proof state as
    the operator report. This read-only receipt summary is not an
    `export.daw_session` lifecycle claim; lifecycle records are emitted only
    when a real queued action exists.
  - DAW-session release blockers are cleared only by their own proof layer:
    JSON package evidence clears only JSON package blockers, writer proof
    clears only `daw_writer_missing`, host-import proof clears only
    `daw_host_import_proof_missing`, audible-output proof clears only
    `audible_output_proof_missing`, and `developer_proof_only` stays visible
    until a later musician-facing release policy removes it. Any PR that
    implements the writer must state whether structured listening review exists
    or whether the handoff remains `human_verdict: unverified`.
  - reusable product-export evidence: local artifact hashing, local proof file
    hashing, receipt-side `artifact_set[]` projection, source graph and
    timing-grid receipt evidence, safe post-write WAV metric extraction,
    recovery preflight for local artifact paths, and observer export lifecycle
    projection from queue/history plus Session receipts
  - current live-recording export is a reserved contract only:
    `export.live_recording`, `export_scope: live_recording`,
    `live_recording.receipt_contract_v1`, `live_recording_capture`, and
    `live-recording-receipt-contract` are stable typed identities, but no live
    capture, WAV writer, observer completion, Session receipt mutation, or
    musician-facing runnable command exists yet. The app queue guard rejects
    attempts with a typed `export.live_recording` action and failed observer
    lifecycle so operators can see the boundary; because it writes no audio,
    current QA is control-path and side-effect proof only.
    `just live-recording-reserved-action-lifecycle-smoke` proves that rejected
    lifecycle path stays distinct from read-only live-recording receipt
    projection: it creates no receipt, writes no destination, and reports the
    explicit future-capture-writer reason.
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
  - minimal output proof before any UI, Ghost, or CLI path may surface
    `export.stem_package` as runnable: a CI-safe writer proof that writes the
    final package layout, records format and audio metrics for every claimed
    stem, proves per-stem non-silence, proves repeated writer/render hash
    stability for identical inputs, proves per-stem source/capture lineage, and
    proves source-vs-fallback comparison evidence for every claimed stem. If the
    writer changes audible behavior beyond exporting already-proven buffers, it
    also needs the current structured listening-review status or an explicit
    `human_verdict: unverified` note.
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
  For `export_scope: daw_session` receipts, observer snapshots also project the
  latest DAW-session receipt as top-level `daw_session_receipt` evidence so
  package/placement/tempo-map proof plus proof-stack state is visible before
  `export.daw_session` exists. This must remain read-only evidence projection
  and must not invent DAW export lifecycle records.
  Current app observer lifecycle projection includes `export.stem_package` and
  `export.daw_session` actions from action log, queue history, and pending queue
  state. Failed reserved attempts have typed failure reasons and no receipt;
  completed receipts expose readiness only from the Session receipt.
  Observer snapshots also include the shared stem-package musician surface gate
  with `status`, `runnable`, typed blockers, and musician labels. That gate
  explains product surfacing and is not permission to infer or mutate package
  artifacts.
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

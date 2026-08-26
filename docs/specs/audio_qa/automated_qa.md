# Automated Audio QA

Parent: [Audio QA Workflow Spec](../audio_qa_workflow_spec.md)

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

An artifact runner must durably write the complete computed per-gate metrics
and identities before enforcing an aggregate technical pass assertion. Its
fail-closed record must therefore retain the exact failed gate values even when
the runner aborts. A missing per-gate record is an evidence failure and cannot
be repaired by guessing, replay, or an unregistered analysis retry.

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

Explicit live gestures must be measured in a short, role-appropriate window
against the immediate no-gesture counterfactual. Report both an absolute signal
delta and a relative delta so a quiet but proportionally different gesture and
a loud but perceptually buried gesture cannot pass the same coarse whole-render
gate. Shared pressure, level, or slam floors must not erase the explicit
gesture; a fill, slam, trigger, launch, or restore needs its own audible
articulation rather than only a raised baseline.

The dense-break live Fill must earn its contrast through arrangement
articulation rather than a global TR-909 gain increase. Its exact RuntimeMix
counterfactual keeps source, W-30, MC-202, transport, and pre-Fill state fixed;
the final-beat review must show the source and non-TR-909 bed moving out of the
way without using global Fill gain. Also prove that Source-only output is
sample-identical, non-Fill and silent/wrong-route states do not duck, the
envelope resets at the bar boundary without a hard edge, and full-block/offline
versus canonical callback-block rendering agrees.

If the confirmed source bar has a non-zero transport phase, preflight must
prove that both the Fill recipe and its focus envelope consume the same
confirmed anchor. A four-beat recipe must remain ordered as build-up followed
by final-beat pause/payoff inside the source bar; a transport-zero rotation
that places the payoff first is a product-path failure even when every event
and aggregate metric remains valid. The manifest records the derived anchor,
resolved render input, and affected RuntimeMix phase path.

The exposed `PhraseDrive` drum phrase must then be judged separately from the
arrangement cut. Its trigger proof counts sounding owners rather than null
events, preserves the intentional trigger-policy rests, distinguishes a
non-sounding callback-local choke from a sounding owner, and checks
independently decaying kick, snare, and hat voices. For the supported
`MainlineDrive + PhraseDrive` signature close, preflight must additionally
prove a click-safe three-hit setup into a callback-local choke, two real rest
slots, a late pitch-diving kick, the fixed delayed snare crack, and a smooth
RuntimeMix bed pocket that reaches silence before the stomp. Compare the
isolated final beat against the rejected composite control using absolute
40-120 Hz drum-thump, 120-500 Hz drum-body, and 2-10 kHz attack evidence; a
better relative spectral share alone is insufficient. A candidate replacing a
reviewed weak recipe must also use a phase-identical exact RuntimeMix control
and report the time-local silence duration, pause RMS ratio, payoff RMS, and
payoff body/attack deltas. Prove clean pre-limiter headroom, deterministic
127/128-frame rendering, fresh Fill-to-Fill and Fill-to-Break boundaries, and
sample-exact legacy output for non-Fill modes. These checks prove deterministic
contrast and physical drum structure, not musical quality or live usefulness.
Unless the typed policy assigns a bass owner separately, 40-120 Hz drum thump
is not a bass-pressure claim.

After a gain/accent hierarchy passes measurement but earns repeated human
"no difference" feedback, do not iterate another small trigger-weight change.
The next candidate must alter at least two perceptual dimensions such as
rhythm/space, pitch trajectory, timbre, or articulation. Its exact A/B
preflight must focus on the locally changed slots as well as whole-render
metrics and state plainly what event the listener should be able to identify.

For explicit drum gestures, a peak-only delta is insufficient. The exact
audible Blend counterfactual must also report loudness-aware waveform
correlation and the share of 10 ms windows whose delta RMS exceeds both 10% of
the candidate-window RMS and an absolute `1e-5` floor. The dense-break Golden
Path currently requires fill coverage `>= 0.15` and slam coverage `>= 0.10`,
with correlation `<= 0.99` for both. These are deterministic anti-collapse
gates, not musical-quality proof; structured human listening remains required.

For Golden Path review, variation must include musical macro-development, not
only sample-level motion or short silence gaps. A near-identical short loop
repeated across an eight-bar review window fails unless the reviewed mode
explicitly promises a held loop and the underlying hook has already earned a
human pass. The preferred proof establishes a source-derived hook, raises
pressure, creates a destructive role swap or drop, and returns with a materially
changed payoff. A render that is merely cleaner, louder, darker, or busier does
not prove recognizable Riotbox character.

A compact Golden Path arc and a reusable loop answer different QA questions.
A scripted sequence that forces hook, pressure lift, fill, scene change, and
return every one or two bars may prove gesture reachability and
macro-development while still sounding like a crowded medley. It must not be
used by itself to claim loopability, reusable source material, or performer
freedom. Those claims additionally require a sustained isolated audition of the
relevant hook, capture, lane, or mutation, with an explicit human keep/reject
decision before roles are combined on demand.

Bass-pressure review must name the typed bass owner first. When bass is
assigned, report absolute low-band energy or lift as well as relative spectral
share; a high relative low-band ratio can otherwise describe a quiet, dull mix.
When bass ownership is `unassigned`, absent bass pressure is not itself a
failure. Source comparison should also catch unintended high-band collapse.
Where practical, provide both raw-level and loudness-matched A/B renders: the
matched render isolates timbre and arrangement, while the raw render preserves
the product-level gain verdict.

Commercial reference recordings are local listening and measurement material
only. They must stay ignored and uncommitted and must never become Riotbox
product sources, fixtures, generated assets, or redistributed review-pack
content.

Source-aware tuning and promotion must also separate development material from
fresh acceptance evidence. The versioned
`docs/benchmarks/source_holdout_rotation_v2.json` contract, whose v1
predecessor remains immutable, requires a
multi-family development matrix plus disjoint unseen and reserve holdout sets.
At least five eligible development sources across four typed families must
inform a candidate before a family-specific success claim. At least two
different-family holdouts must then test it without having selected the
algorithm or constants. Once holdout output changes the next implementation,
that source is consumed, must be recorded in rotation history, and cannot be
presented as unseen again. `just source-holdout-rotation-fixtures` enforces
family diversity, partition disjointness, rotation, provenance, license and
reference boundaries. RIOTBOX-1428 Stage A additionally uses exact-case,
development-only access with a bounded access log; it must reject holdout
identity, path, or hash collisions before opening any selected file and must
not discover source directories. The v2 registry may preserve explicitly
registered native-rate PCM16 or PCM24 development files while inherited v1
entries retain their 48 kHz PCM16 contract. This remains
`quality_proof: false`.

The isolated RIOTBOX-1428 percussive-force screen is a narrower preregistered
role test: exactly four positive packs from four authors across dense break,
sparse drums, and electronic drums, with two qualified events per pack. It
does not claim the broader five-source/four-family source-aware readiness floor
above. Its exact detector, event anatomy, source-contrast partition, F1--F3
equations, controls, level matcher, reject-only screens, and blind ordering are
preserved as immutable historical evidence in
`docs/benchmarks/percussive_force_stage_a_protocol_v1.json` and
`docs/benchmarks/percussive_force_development_matrix_v2.json`; the predecessor
`docs/benchmarks/percussive_force_development_matrix_v1.json` is historical too.
The later
Protocol-v2 / Matrix-v3 execution is likewise immutable fail-closed historical
evidence in `docs/benchmarks/percussive_force_stage_a_protocol_v2.json` and
`docs/benchmarks/percussive_force_development_matrix_v3.json`, not an active
source of values for another run. Any future acquisition, qualification, or
execution requires a new versioned preregistration and targeted Decision before
source access or recomputation. Mechanical success grants only a human
listening request; it cannot award `percussive_hard`.

### CI layers

Broad validation should be layered so engineers get fast feedback without
weakening audio proof:

- fast layer: formatting, Rust tests, clippy, and lightweight contract checks
- audio layer: deterministic offline renders, metrics, and targeted audio-QA
  smokes for touched seams
- full layer: broader report generation, listening-pack fixtures, and release
  readiness diagnostics

Running a smaller layer is not permission to skip relevant output-path proof for
an audio-producing change. It only scopes the command set to the slice.

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
- gates that reuse an existing source-backed render or manifest must validate
  it with that artifact's stored timing identity. Generated-fixture BPM, sample
  rate, source, or anchor constants must not be imposed on real-source evidence.

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

After a bounded Development exploration earns a provisional human keep, shared
audible DSP, mix, pattern, and performance-policy qualification requires a
frozen rebuild and bounded source matrix before the formal Golden Path product
verdict. The early exploratory usefulness check is not promotion evidence and
is never imported as the formal verdict. The matrix remains a regression and
overfitting gate. It must include at least three contrasting real sources and
reject exact-path failure, clipping/limiter concealment, silence, timing
regression, or near-identical source-backed hook envelopes. Existing shared
behavior and regressions enter at this qualification gate directly. The
historical dense command remains as a compatibility alias:

```bash
just dense-break-live-source-matrix
```

Beat08, Beat20, and DH BeatC now classify as sparse rather than dense, so the
alias delegates to the controlled character-aware matrix below instead of
forcing all four sources through dense-only gesture gates. Git history retains
the older dense-only envelope validator as historical evidence.

Controlled dense/tonal/sparse held-state expansion uses:

```bash
just controlled-source-live-matrix
```

The dense control renders once; the selected tonal and sparse sources each
render twice through the exact callback-block RuntimeMix. The matrix requires
byte-identical same-source WAVs, three distinct typed character decisions and
WAVs, no clipping or limiter concealment, and non-collapsed pairwise 20 ms RMS
envelopes. The held-state artifact is deliberately separate from the dense
scripted Alpha arc: it proves a loopable committed instrument state, not a fixed
composition. Tonal review assigns midrange/hook leadership to W-30 and
intentional MC-202 stay-out; sparse review assigns drum/transient impact to
TR-909, not bass. All three current cases declare bass owner `unassigned`. The
report remains `quality_proof: false` and `human_verdict: unverified` until the
tonal and sparse candidates receive structured human verdicts. Generated report
state does not absorb those verdicts: store each human decision as a separate
`riotbox.listening_review.v1` pack. RIOTBOX-1404 records `keep` for both held
loops and both destructive variants. Its rejected sparse destructive control
is retained as evidence that deterministic technical success does not excuse
source kicks drifting between fixed-grid drums; the accepted replacement must
prove `1.0x` playback, a nonzero bounded grid gate, deterministic WAV identity,
and a fresh listening pass.

The current lightweight command is:

```bash
just source-showcase-diversity "PACK_A PACK_B ..."
```

Feral-grid renderer packs also have a direct WAV-level gate:

```bash
just feral-grid-render-diversity "PACK_A PACK_B ..."
```

This gate compares the rendered product roles themselves:
`04_riotbox_source_first_mix.wav`, `05_riotbox_generated_support_mix.wav`,
`stems/01_tr909_beat_fill.wav`, `stems/02_w30_feral_source_chop.wav`, and
`stems/03_mc202_bass_pressure.wav`. It rejects identical hashes and
near-identical cross-source waveforms, so a pair of different sources cannot
pass if the result collapses into the same clicky or placeholder-like output.

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

Each MC-202 real-source listening-pack case must also carry a compact
`mc202_role_evidence` review target. Sparse bass-pressure material is reviewed
as source-derived bass pressure; dense/non-dense material is reviewed as a
pressure-answer; tonal-hook material is reviewed as hook-restraint or
stab-answer behavior. This field tells the human listener what musical job to
judge, but it remains `quality_proof: false` and `human_verdict: unverified`
until structured listening records a verdict.
The same pack must expose `selected_motif.rhythm_signature` for the rendered
MC-202 stem and report-level `mc202_rhythm_diversity`. This is a folded 16-step
RMS signature, not a taste verdict: it catches repeated MC-202 rhythm feel
across dense, sparse, and tonal real-source cases while preserving
`quality_proof: false` and `human_verdict: unverified`.

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
- source-first feral mixes with generated/source-backed RMS ratio above `0.08`
  are treated as masking the source-backed W-30 lane
- generated-support feral mixes must keep generated/source-backed RMS ratio
  between `0.145` and `0.46`, so support stays audible without becoming a
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
  window. Source-first renders must also keep at least `0.04` headroom below
  the `0.08` generated/source masking ceiling, so a barely-passing source-first
  mix cannot hide source character risk. This remains diagnostic evidence, not
  an automated musical pass.
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
  movement: movement must be at least `1.75 Hz` away from the fixed contour,
  span at least `17.0 Hz`, keep pressure low-band lift at or above `2.70`,
  keep pressure low-band share at or above `0.36`, keep low/mid pressure ratio
  at or above `2.45`, and leave bass as the strongest audible element with at
  least `0.20` dominance margin. These remain scripted diagnostic gates, not
  musical-pass claims.
- Destructive-variation professional diagnostics require a hard dropout/stutter
  contrast and an impact restore: `dropout_to_stutter_rms_ratio <= 0.0065`,
  `dropout_silence_to_stutter_rms_ratio <= 0.0065`,
  `stutter_to_hook_transient_ratio >= 1.55`,
  `restore_to_hook_transient_ratio >= 1.60`,
  `restore_to_pressure_rms_ratio >= 1.36`, and
  `restore_to_dropout_silence_rms_ratio >= 6.00`. These thresholds prove the
  diagnostic output did not collapse to a flat edit: the cut must actually get
  out of the way, the stutter must hit, and the restore must slam back from the
  cut. They do not approve the render as product-quality audio without human
  listening.
- Dense-break render reports use a stronger active restore/pressure floor
  (`1.18`) than the generic floor (`1.12`). Sparse-bass-pressure diagnostics keep
  the generic floor because their pressure section is intentionally large; the
  dedicated destructive-variation report carries the stricter `1.36` floor for
  the stage-meaningful cut/restore proof.
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

P023 rendered TR-909 drum pressure must also survive the generated-support
mix, not only the isolated dense-break scorecard. Feral grid manifests expose
`tr909_rendered_drum_pressure` with source-derived origin, support-mix
contribution, low-band RMS, source-first masking headroom, support/source
ratio, and source-grid alignment. The professional suite currently requires
every rendered TR-909 case to keep support contribution at or above `0.05`.
Drop-drive and break-lift profiles require low-band RMS at or above `0.0030`;
steady-pulse profiles require at least `0.0017` after their lighter source
role is strengthened. Break-lift policy must also carry enough low body for
syncopated/high-transient sources instead of passing as click-only lift.
Source-first generated/source masking must stay under `0.08` and
generated-support source ratio under `0.46`. These gates prove the drum
support is not decorative or buried, while still keeping the evidence
diagnostic and `quality_proof: false`.

P022 rebuild-only/source-layer-off diagnostics may expose transformed-source
survival evidence. The current bounded fields are
`rebuild_only_source_spectral_similarity`,
`rebuild_only_source_transient_retention`,
`rebuild_only_source_rms_retention`, and
`rebuild_only_source_character_survival_score`. P023 diagnostics also expose
`rebuild_only_source_character_survival_margin`, which must stay at least
`0.10` above the `0.70` survival floor. These fields prove only that some
source-like spectral/transient character survives after raw source masking is
removed and raw-copy correlation is still rejected; they do not prove that the
result is musically good.

P023 Hook/Chop source selection may also expose
`hook_chop_source_character_score_floor`,
`hook_chop_source_character_score_mean`, and
`hook_chop_source_character_score_span`. These fields prove that source-backed
hook/chop/riff windows were selected from windows with enough source identity
and enough variation across selected offsets. They are a bounded selection
contract, not a musical pass. Dense-break and tonal-hook professional
diagnostics must keep `hook_chop_source_character_score_floor >= 0.64`; tonal
replacement logic must not trade away the existing selected source-character
floor merely to widen the span.

P023 source-character window selection also exposes
`source_character_window_selection` in Feral grid manifests and professional
suite/readiness summaries. A valid promotion must scan more than the requested
window, raise the source-character score, and keep
`rms_retention_ratio >= min_rms_retention_ratio` (`0.98` today) so the selector
does not chase a weak transient peak. The professional suite requires at least
three searched cases and at least one promoted case, and the readiness report
mirrors those gates. Source-family policy may still request a longer window
for tonal-hook or bad-timing review material when a short request would damage
the family-specific hook/caution contract. These metrics prove source-window
selection is active and energy-preserving; they remain diagnostic evidence with
`quality_proof: false`.

P023 dense-break product-path source-window policy must also expose
`source_selection_policy` in the dense performance report, professional-output
suite, and readiness report. This policy is family-specific: dense-break rescue
may move to a higher-character transient window only when it avoids excessive
high-band/stab tilt, sparse-bass-pressure selection prioritizes low-band share
and low/mid weight over generic transient score, and tonal-hook material with no
extra searchable source duration may keep its requested full window without
failing candidate-count gates. Candidate-count floors apply only when the search
duration is larger than the selected window. The policy must preserve
`quality_proof: false` and `automated_musical_approval: false`; it proves
source-window policy execution, not human musical approval.
When readiness keeps `source_selection` as the current product priority, it
must compare the candidate's source families with policy-covered families.
Dense-only policy evidence is not enough to imply tonal-hook or other family
coverage; uncovered families must stay explicit in the priority detail and next
implementation step.
Professional-output source-selection policy coverage must aggregate the dense
product path and professional source-WAV policy cases for tonal-hook and
sparse-bass-pressure material. Candidate-count floors apply only to cases whose
search window is larger than the selected window; a verified full-window tonal
case may have one candidate when no additional searchable source duration
exists. Once promotion-allowed policy families cover the current
source-selection candidate families, `source_selection` becomes a stale
regression control rather than the current product gap.

P023 source-selection risk must also expose actionable demotion evidence for
edge material. Bad-timing and pad/noise diagnostics may stay source-backed, but
they must not be promoted as release-quality demos while timing confidence,
texture suitability, or human verdict state remains unresolved. Professional
suite/readiness reports therefore require blocked edge families, demotion reason
counts, and concrete review actions such as confirming timing before bar-locked
moves or auditioning pad/noise texture before demo promotion. This is a
musician-facing safety boundary, not a musical approval gate.
When `source_selection` becomes the current P023 product priority, the readiness
report must also expose a concrete priority detail instead of only a generic
category: triggering case ids, primary case ids, source families, artifact refs,
demotion/review reason linkage, software next step, and musician-facing action.
It must also show that the dense product-path source-window policy is applied
while edge source families remain unavailable/degraded for promotion.
The detail must preserve `quality_proof: false` and
`automated_musical_approval: false` until source choice is proven by output and
human verdict.

P023 readiness may keep static weak-output fixtures as negative controls, but
it must reconcile their routing priority against current professional-suite
evidence. If a weak fixture still routes `chop_policy` while dense, matrix, and
tonal W-30 response gates pass, readiness should mark that category as a stale
fixture-only risk and point implementation toward the next current product gap.
The same rule applies to `bass_movement`: sparse-bass negative fixtures remain
regression controls, but they should not stay current priority when matrix and
source-WAV sparse movement, low-band lift/share, low-to-mid ratio, and bass
dominance gates all pass.
The same rule applies to `destructive_gesture`: flat-stutter negative fixtures
remain regression controls, but they should not stay current priority when
current dropout/stutter silence, stutter transient, and restore-impact gates
all pass. If a source-selected dense hook raises the hook transient baseline,
destructive review may also satisfy the transient contract with source-referenced
floors: `stutter_to_source_transient_ratio >= 5.50` and
`restore_to_source_transient_ratio >= 6.00`, with high rebuild-only
source-character survival proving the source is still transformed but present.
The same rule applies to `mix_bus`: source-masked or support-buried negative
fixtures remain regression controls, but they should not stay current priority
when current professional-suite generated support is loud enough, source-first
masking stays below the ceiling, masking headroom stays above the floor, and
support never overwhelms the source.
The same rule applies to `drum_pressure`: old weak drum-pressure fixtures remain
regression controls, but they should not stay current priority when current
dense-break snare pressure and rendered TR-909 gates pass. The required current
proof is the same bounded professional-suite contract: strongest dense element
is `snare`, dense pressure score/margin/transient floors pass, rendered TR-909
support contribution and low-band RMS meet their required floors, and generated
support stays below the source-masking ceilings. This demotion is only
diagnostic prioritization; it does not claim human musical approval or release
quality.
This preserves regression controls without letting old hookless examples hide
the instrument's current audible state.

`riotbox.sound_quality_readiness_report.v2` separates that diagnostic
reconciliation from release-quality ownership. A live release-demo bank remains
immutable evidence: an older weak/fail entry may stop being an active production
blocker only when a frozen `riotbox.release_demo_evidence_reconciliation.v1`
names a later successor with the same source family and source path. The
successor must be an eligible non-fixture human `pass`, be `demo_ready`, and
carry a passing exact-product-path gate. The old verdict, fix category,
artifact, review, and hashes remain visible under
`superseded_weak_or_fail_entries`; they are never deleted or relabeled.

For covered-scope release quality, that reconciliation must bind exact
RuntimeMix human-pass entries for `dense_break`, `sparse_drums`, and
`tonal_riff`, plus reviewed no-fallback degraded/unavailable product outcomes
for `bad_timing`, `pad_noise`, and `weak_source`. The professional-output suite
remains required current diagnostic context and must keep
`scripted_generation: true` and `quality_proof: false`; it cannot grant or veto
the human product-quality decision once the complete validated product set is
present. Fixture-calibration reconciliation may exercise lifecycle mutations
but must always return `quality_proof: false`. Any stale bank, contract, review,
source identity, product gate, family coverage, unresolved weak/fail entry, or
queued candidate fails closed.

P023 Hook/Chop riff playback diagnostics also expose
`hook_chop_riff_hit_pattern_source_derived`,
`hook_chop_riff_hit_count`, `hook_chop_riff_velocity_span`, and
`hook_chop_riff_reverse_count`. P023 also requires
`hook_chop_w30_to_source_margin >= 0.10` in dense/tonal generated
professional diagnostics, and tonal-hook fixture reports require
`w30_contribution_margin >= 0.050` above their W-30 contribution floor.
Dense-break and tonal-hook reports must prove that selected source offsets
drive a non-static hit pattern with enough hit density, velocity contrast, at
least two reverse gestures, and enough W-30 headroom above the hook-presence
floor. Current dense/tonal generated diagnostics require at least six source
offsets, ten riff hits, and `hook_chop_riff_velocity_span >= 0.25` before the
hook/chop path may pass. The rendered W-30 riff layer must not buy that
diversity by masking dense-break drum pressure; dense-break diagnostics still
require snare/break to stay the strongest element with its documented margin.
P023 response-signature diagnostics additionally expose
`hook_chop_response_delta_ratio`, `hook_chop_response_correlation`, and
`hook_chop_response_transient_ratio`. Dense-break and tonal-hook reports must
prove that the rendered hook/chop response differs audibly from a raw source
copy while still carrying source transient attack: current floors are response
delta `>= 0.35`, response correlation `<= 0.92`, and response transient ratio
`>= 0.58`. These gates reject source-copy or hookless output; they remain
scripted diagnostic evidence, not a musical approval.
Pad/noise and bad-timing
source families must not use this W-30
hook/chop riff as a hidden product fallback; they use their family-specific
texture or timing-cue paths instead. These fields remain diagnostic and must
keep `quality_proof: false` until structured listening review accepts the
result.

P023 sparse-bass-pressure diagnostics also expose
`sparse_pressure_low_band_share` and
`sparse_pressure_low_to_mid_ratio` alongside the existing source-derived bass
movement fields. Sparse reports must prove that the pressure section is not
only a moving midrange phrase: the low band must carry enough of the pressure
section and must dominate the mid band by the documented ratio. Current sparse
diagnostics require at least `1.75 Hz` fixed-contour distance, `17.0 Hz`
frequency span, `2.70x` low-band lift, `0.36` low-band share, `2.45x`
low/mid ratio, and `0.20` strongest-element dominance margin before this path
can pass. These are technical pressure-shape gates, not musical approval.

`just diverse-test-source-wavs` writes deterministic generated example/test
sources and must cover at least twelve distinct source families. Those WAVs are
for examples, regression tests, and collapse detection only; their manifests
must keep `quality_proof: false`.

The dense-break professional diagnostic also keeps a real weak-WAV regression
for this boundary:

```bash
just dense-break-weak-source-character-fixture-smoke
```

That smoke renders an intentionally weak `06_rebuild_only_performance.wav` and
requires the report validator to reject it with
`rebuild_only_source_character_not_surviving` and
`rebuild_only_source_character_margin_too_low`. It is negative diagnostic
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
When readiness reports `ui_cue` as the current product priority, it must expose
`ui_cue_priority` rather than only a generic category. The detail must name the
case ids, source families, artifact refs, cue surface, cue reasons, software
next step, musician-facing action, and required player cues. The required cue
surface is timing/source risk before confident bar-locked or live-trigger moves:
Riotbox should show unavailable/degraded state and the reason before a musician
trusts a risky move. This remains diagnostic prioritization with
`quality_proof: false` and `automated_musical_approval: false`.
Once that cue is implemented, readiness must not demote `ui_cue` merely because
the plan says so. It must consume a current Jam perform-risk cue contract from
the app (`riotbox.jam_perform_risk_cue_contract.v1`) proving degraded and
unavailable timing both expose `bar/live?` on the Trust surface, with
`quality_proof: false` and `automated_musical_approval: false`. If that
contract is missing or the cue regresses, `ui_cue` remains the current product
risk; if it passes, `ui_cue` becomes a stale regression control and readiness
advances to the next non-stale gap.
`fixture_threshold` must follow the same current-evidence discipline. A
fixture-threshold route may be demoted only when it is secondary
negative-control evidence, has no primary routed cases, carries the expected
`source_report_not_passed` fixture signal, and the related current output proof
already passes. A primary threshold case, unknown routing, or missing current
proof keeps `fixture_threshold` as the current implementation risk.
When all weak-output categories are stale and
`current_product_top_candidate_category` is `none`, the readiness report's main
Next Actions must prioritize source-family review and human/demo coverage. Stale
weak-output categories remain visible as regression controls in current-evidence
reconciliation, but they must not obscure the active review path in the main
action list.

MC-202 producer-grade closeout routing extends that category vocabulary for the
MC-202 lane with `answer_bite`, `hook_restraint`, and
`destructive_articulation`. Structured demo-bank promotion may consume
`mc202_producer_fix_candidates` only after matching the exact review case and
rendered WAV hash. The closeout category `human_listening` is allowed in the
closeout worklist but must not become a demo-bank production fix after a human
verdict has been recorded.

Edge-source diagnostics for pad/noise and bad-timing material must also carry a
source-selection promotion gate. The gate must keep `promotion_allowed: false`,
name the blocked source families, preserve `quality_proof: false`, and explain
the musician-facing reason: risky timing/source material remains review or
routing evidence until source selection, timing confirmation, or human listening
clears it. The professional-output suite and P023 readiness report must surface
that gate so these cases cannot silently become demo-ready examples.


## Validator ownership

Large cross-report contracts stay in named repo-local validators. The manifest
and artifact boundary is specified in
[`manifests_and_artifacts.md`](./manifests_and_artifacts.md).


Run:

```bash
just musical-pass-gate-policy-fixtures
```


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


The listening-pack harness is specified in
[`listening_review.md`](./listening_review.md).

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


## Current executable automated profiles

These current-profile rules were formerly hidden in the status inventory and
are normative here.

- a CI-safe generated W-30 source-vs-control smoke that uses deterministic synthetic source material, checks minimum source-vs-control deltas, validates the generated listening manifest, and runs under `just audio-qa-ci`; existing command names may still say `source-vs-fallback` for compatibility, but the baseline is diagnostic control only
- a CI-safe first-playable Jam probe, `just first-playable-jam-probe`, that
  checks a generated app-level observer journey for
  `source -> capture -> audition -> promote -> preset/role preparation ->
  w/s/f/y -> Y+D changed return`
  alongside the exact callback-block RuntimeMix dense-break pack; correlation
  is explicitly limited to the typed action contract because source fixture,
  Session, and transport timeline differ. The pack proves
  `source`, `blend`, and `riotbox` routes plus per-gesture counterfactual deltas
  through Source Monitor and the master limiter, while remaining
  `evidence_role: diagnostic`, `scripted_generation: true`,
  `quality_proof: false`, and `human_verdict: unverified`
  - every exact-mixer monitor mode, performance stage, gesture counterfactual /
    candidate, full mix, and isolated lane stem carries pre- and post-limiter
    metrics plus changed-sample count; the current clean-path gate permits no
    limiter activity, so post-limiter `clip_count == 0` cannot conceal a hot mix
  - Source, Blend, and Riotbox monitor references each render four bars so a
    human reviewer can judge balance and source character without extrapolating
    from a too-short one-bar excerpt
  - the legacy full-mix and isolated-lane regression is frozen immediately after
    the committed `w` hit while TR-909 is still `break_reinforce`, before the
    later live `f`, `s`, `y`, and `Y` gestures; the TR-909 pressure stem keeps
    its established minimum RMS gate instead of validating the later fill/slam
    state under the old filename
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
- musician-visible performance risk cues should use the same shared Jam source
  timing summary plus Source Graph confidence. The cue must distinguish
  `trusted`, `degraded`, and `unavailable` states, and must point to a player
  action such as performing grid moves, confirming/listening first, or
  recapturing/loading source material instead of hiding source/timing risk in
  logs.
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
- observer/audio strict JSON correlation also accepts W-30 preview source-diff
  manifests as output-path evidence for the narrower W-30 probes, using
  candidate RMS, active-sample ratio, and RMS delta to reject silent or
  diagnostic-control-collapsed output; the first-playable Jam probe no longer
  substitutes that narrower seam for its Blend / multi-gesture product-mixer
  evidence
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

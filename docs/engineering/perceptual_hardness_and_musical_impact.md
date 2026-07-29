# Perceptual Hardness and Musical Impact

Status: accepted research guidance; implementation hypotheses remain unverified

Date: 2026-07-26

Primary context: RIOTBOX-1422 / P023

## Purpose

This document turns the repeated listening failure "`Hard` is different, louder,
or choppier, but it is not harder" into reusable audio-engineering guidance.
It also records adjacent findings that help Riotbox become more coherent,
punchy, interesting, loopable, and useful in live performance.

This is not a fixed DSP recipe. It separates:

- published perceptual evidence
- established production mechanisms documented by audio-tool vendors
- Riotbox implementation hypotheses that still require multi-source technical
  proof and structured human listening

Commercial recordings may inform local listening and measurement comparison,
but they are not research stimuli for this document and must never become
Riotbox product sources, fixtures, generated assets, or committed artifacts.

## Triggering Failure

The exact committed RIOTBOX-1422 Base-to-Hard artifact at commit `edd90406`
kept the accepted grid-safe Base in its first two bars and used the then-current
`source_transient_chop` Hard policy in its final two bars:

```text
/tmp/riotbox-1422-hard-review-edd90406/
  06_w30_resample_base_to_hard_live_gesture.wav
SHA-256:
  1ca4b7bdf5029ff60bfc8cc2cb2bb7b24639e04c2010c6e2ff56cd14b5c32ee4
```

FFmpeg `loudnorm` analysis of the two exact listened A/B halves reported:

- Base: `-16.85 LUFS`, `-4.09 dBTP`
- Hard: `-19.08 LUFS`, `-2.26 dBTP`

The standalone Hard sibling was technically distinct from Base and safe:

- standalone Hard silence ratio: `45.75%`
- Hard crest factor: `7.789`
- Hard peak: `0.709396`
- clips: `0`

The exact A/B Base half is sample-identical to the corresponding first two bars
of the standalone Base sibling. Its Hard half is intentionally not
sample-identical to the standalone Hard sibling: the product-path A/B keeps
callback state and applies the Base-to-Hard transition, whereas standalone Hard
starts with fresh callback state. The exact A/B Hard half measures `46.38%`
near-silence with `|mono| < 1e-4`; the standalone four-bar Hard sibling measures
`45.75%`. This distinction does not change the diagnosis, but it must remain
explicit whenever standalone and live-transition evidence are compared.

The structured human verdict was nevertheless `reject`: the second half was
audibly choppier and emptier, but not harder. This disproves three tempting
shortcuts:

1. a larger waveform delta does not prove hardness
2. louder peaks or a higher crest factor do not prove hardness
3. more silence or shorter gates do not prove hardness

An onset- and band-local audit of the same `1ca4b7...` artifact explains the
verdict. Relative to Base, Hard raised only the first `0–10 ms` onset RMS by
`77.6%`, then removed `48.3%` at `10–40 ms`, `69.0%` at `40–120 ms`, and
`91.0%` at `120–200 ms`. Its attack/body ratio jumped from `6.8` to `39.0`.
Hard also lost Hann-windowed band magnitude in `20–80 Hz` (`-38.4%`),
`80–250 Hz` (`-21.1%`),
`250 Hz–2 kHz` (`-18.6%`), and `2–6 kHz` (`-12.8%`), while only the
low-magnitude `6–20 kHz` band increased (`+53.7%`). A higher peak therefore hid
a short bright click followed by missing body and phrase energy. The human
"choppier/emptier" judgment was not subjective ambiguity; it accurately
described the signal.

The reviewed Base remains the control. Research must improve Hard without
silently rewriting the already-correct timing and harmonic foundation.

### Reproducible measurement method

The onset and band deltas above are diagnostic measurements of the exact
`06_w30_resample_base_to_hard_live_gesture.wav`, not values copied from the
standalone siblings:

- decode the stereo PCM16 artifact at `48 kHz` and split it at frame `177231`
  into the exact two-bar Base and Hard halves
- form mono as the arithmetic mean of left and right
- use the committed Hard mask `11010111`, whose active eighth-note slots are
  `0, 1, 2, 4, 6, 7`; repeat those six anchors across both bars
- concatenate samples from every active anchor for `0–10`, `10–40`, `40–120`,
  and `120–200 ms`, compute RMS for Base and Hard, then report
  `(Hard / Base - 1) * 100`
- define the reported attack/body diagnostic as active-anchor `0–10 ms` RMS
  divided by active-anchor `40–120 ms` RMS
- for band deltas, remove each complete half's mean, apply a Hann window,
  compute the real FFT, integrate squared magnitudes inside each stated band,
  take the square root, then compare Hard with Base
- use FFmpeg `loudnorm` on the exact two segments for integrated loudness and
  true peak; use Riotbox `signal_metrics` for standalone peak, crest, clip, and
  silence values, where the active threshold is `1e-4`

The FFT result is a relative band-magnitude proxy, not calibrated sound
pressure or an auditory-model output. The active-slot aggregation includes the
real first Base-to-Hard transition because that is part of the performer-facing
gesture the listener judged.

## Working Vocabulary

Riotbox must name the intended domain before using words such as `hard`,
`punch`, or `pressure`.

| Term | Working meaning | Not sufficient |
| --- | --- | --- |
| Timbral hardness | A source-relative combination of fast/strong onset evidence, broad or elevated attack spectrum, midband presence, and percussive character | gain, clipping, or brightness alone |
| Drum/transient punch | Perceived dynamic power or weight at short time scales, including onset, frequency band, duration, transient-to-body relation, and masking | whole-file crest factor or LUFS |
| Midrange/hook bite | A memorable source-backed attack or riff that cuts through in the presence region without becoming a foreign layer | global treble lift or unrelated click |
| Bass/low-end pressure | Absolute, role-owned low-band energy and physical body | relative low-band share without a typed bass owner |
| Roughness | Fast amplitude fluctuation that can increase salience, aversion, buzz, or grit | a synonym for punch, hardness, or quality |
| Arrangement impact | A performance event that changes expectation and the room: stop, choke, fill, drop, role swap, or forceful return | empty space without a payoff |
| Harmonic coherence | New energy remains recognizably related to the source and its musical context, or creates deliberate, controlled tension | indiscriminately removing all roughness or dissonance |
| Loopability | The material has a stable pulse and identity, enough repetition to learn, and enough bounded surprise to reward another cycle | novelty at every step or exact repetition without a hook |

These domains interact but cannot substitute for one another. A break can have
strong transient punch without owning bass. A bass gesture can have body
without a sharp attack. Silence can create arrangement impact without making
the sounding material harder.

## Evidence Synthesis

### Timbral hardness is multidimensional

Pearce, Brookes, and Mason trained a perceptual hardness model on 202 stimuli
from 32 source types. Six combined features predicted listener ratings:
maximum bandwidth, attack spectral centroid, midband level,
percussive-to-harmonic ratio, onset strength, and log attack time. The model
reached `R² = 0.76` on training material and `R² = 0.57` on a new dataset.
Maximum attack bandwidth over the first `100 ms` was the strongest single
feature in that model; its midband feature covered roughly `7–14 Bark`
(`~700–2150 Hz`). No single feature was presented as hardness by itself, and
the authors note that sibilance, clicks, or recording defects can sometimes be
rated as hard. Riotbox must therefore guard against winning the metric with an
artifact.

Earlier mallet-hardness work also obtained a useful perceptual fit from several
attack-local measurements together: time-varying spectral level, its slope,
spectral centroid, and time-weighted spectral centroid over the first `325 ms`
of a struck sound.

Riotbox consequence: Hard QA needs onset-local temporal and spectral evidence.
Whole-render loudness, silence, RMS, peak, or one spectral ratio can only be
safety or anti-collapse evidence.

### Punch is microdynamic and band-dependent

Fenton, Lee, and Wakefield describe punch as perceived dynamic power or weight.
Their work separates transient, steady-state, and residual components and
analyzes octave bands rather than treating the complete mix as one waveform.
Listener perception correlated with onset times, frequency band, signal
duration, and dynamic range. Their analysis also shows that steady-state or
residual energy can mask transient power.

Riotbox consequence: preserve and measure the relationship between the attack
and the body that follows it. Removing the body can produce a click with high
crest factor but less physical impact. Raising the body indiscriminately can
mask the attack.

### Perceived attack is not the raw rise time

A saxophone study found that articulation and performed dynamics influenced
perceptual attack time, while the simple acoustic rise-time measure had only a
weak correlation with the perceptual result. The attack is therefore a shaped
spectrotemporal event, not merely the first non-zero sample or the steepest
amplitude slope.

Riotbox consequence: align a source's perceptual attack, not only its trigger
event or coarse source slot. A technically quantized layer can still feel late
or detached when its audible attack has preroll, a soft spectral bloom, or a
foreign transient.

### Roughness can add salience, but it is not hardness

Studies of screams and alarm-like signals place a salient roughness region
around `30–150 Hz` amplitude modulation. One controlled study found especially
strong aversion around `40–80 Hz`; newer roughness-estimation work likewise
uses modulation energy in a bounded roughness range rather than ordinary audio
frequency energy.

Riotbox consequence: source-derived modulation in this range is a plausible
ingredient for bite or alarm-like urgency, but it is high-risk. Too much can
become buzzy, fatiguing, or unpleasant. It must be bounded, time-local,
source-related, and judged separately from punch, tonal pitch, and output
level. A fixed synthetic roughness oscillator would violate the source-backed
product contract.

### Distortion can support aggression, but there is no universal THD target

A loudness-matched study of strongly compressed rock vocals found very high
correlations among perceived distortion, roughness, and aggression. The
distorting compressor variants sounded more aggressive than clean compression
in that specific material. The study also showed why a single THD scalar is a
poor recipe: different even/odd harmonic structures can share a THD value but
sound different, and the tested material does not establish a universal
threshold for drums, breaks, bass, or full mixes.

Riotbox consequence: controlled nonlinear energy is a stronger hardness
hypothesis than another gate, but measure its harmonic distribution,
intermodulation, roughness, residual, and aliasing as diagnostics. Select and
blend it by source role. Never turn a context-specific distortion amount into
one global magic number.

### Production tools separate attack, sustain, spectrum, and body

Official production-tool documentation reinforces the perceptual research:

- Native Instruments' Transient Master changes attack and sustain contours
  independently of absolute level and describes stronger transients as more
  powerful and aggressive.
- Ableton's Drum Buss treats mid/high distortion, transient contour, low-end
  enhancement, decay, compression, and dry/wet balance as separate controls.
  Its positive transient direction adds attack and sustain for a fuller punch;
  the negative direction adds attack while reducing sustain for a tighter,
  crisper sound.
- Ableton's Saturator and Roar provide band-selective or multiband nonlinear
  processing, parallel routing, dry/wet control, tone shaping, and output
  compensation. Their documentation explicitly shows why full-band distortion
  is not the only or safest route: low energy can be kept out of a shaper while
  mids/highs are saturated, preserving low-end impact.

Riotbox consequence: transient shortening and transient strengthening are
different operations. The current Hard gate performed the first but did not
reliably perform the second. A useful Hard path will likely need parallel,
band-aware, source-relative processing rather than one envelope multiplied
over the complete signal.

### Bass-drum hardness and physical body are related but distinct

A bass-drum listening study recruited 23 participants and analyzed 18 after
exclusions. Compression, equalization, high-frequency excitation, and
low-frequency excitation influenced hardness judgments in the tested
conditions. High-frequency excitation increased perceived hardness, while
low-frequency excitation tended to reduce it.

Riotbox consequence: do not infer that more low end makes a bass drum harder.
For an explicitly assigned kick or bass owner, a clearer head is a hardness
hypothesis, while retained or enhanced body is a separate punch/weight
hypothesis that must not be deleted accidentally. For an unassigned W-30 break
role, absent bass pressure is not a failure and must not be faked with a
generic low oscillator.

### Musical interest needs a learnable pulse and bounded surprise

The cited syncopation paradigms report an inverted-U relationship: moderate
syncopation tended to produce more pleasure and urge to move than very low or
very high syncopation. This is not a universal law for every definition of
musical complexity. A strong beat supports groove; complexity without metric
clarity can reduce it.

Riotbox consequence: a loop should make its pulse and hook learnable, then add
one or a few meaningful prediction changes. Repeating an unchanged weak phrase
is boring, while continual retrigger changes, collisions, or off-grid attacks
become confusing. Destructive variation should bend an established groove,
not erase the listener's model of it.

### Harmonic coherence and roughness must be treated separately

Consonance research associates perceptual judgments with more than one
acoustic contributor, including roughness, harmonicity, register, sharpness,
and familiarity. Roughness can be musically intentional; harmonic output can
still be dull; a pitch-related layer can still feel foreign when its attack or
role is wrong.

Riotbox consequence: "`harmonic enough`" is a contextual listening contract,
not one universal scalar. Generated harmonics should normally come from the
actual source waveform, and tonal transforms should use trusted source pitch or
scale evidence when available. For noisy or break material, coherence is more
often established by timing, spectral continuity, and source recognition than
by forcing a pitched consonance score.

## Mechanism Matrix

| Mechanism | Intended contribution | Common failure | Required proof |
| --- | --- | --- | --- |
| Source-local transient emphasis | clearer, more forceful attack | louder but unchanged; click detached from body | onset-local attack strength, centroid/bandwidth, body retention, listening |
| Transient/sustain parallel split | attack can be driven without deleting the phrase body | phase smear, doubled hit, thin click, excessive sustain masking | alignment, no pre-echo/double onset, transient-to-body delta |
| Band-limited saturation or clipping | controlled harmonic density and midrange bite | full-band fuzz, lost low end, aliasing, hollow/foreign edge | per-band onset lift, source recognition, native and loudness-matched A/B |
| Dry/processed parallel blend | retains identity and body while adding aggression | comb filtering, level-only benefit, timid wet path | phase-safe reconstruction, gain-matched delta, source stability |
| Source-derived roughness modulation | alarm-like salience or grit | buzz, fatigue, tonal whine, fixed synthetic signature | bounded `30–150 Hz` modulation change, no cross-source collapse, listening |
| Low/body enhancement | kick or bass physical weight | boom, masking, false bass ownership | typed owner plus absolute low-band lift and decay control |
| Choke/gate/dropout | arrangement contrast and room for a payoff | choppier/emptier but not harder | explicit silence window plus stronger return; never hardness proof alone |
| Retrigger/resequence | live rhythmic mutation | off-grid echo, duplicate attack, lost pulse | perceptual-onset alignment, stable base grid, moderate complexity |
| Pitch dive/reverse | destructive identity and gesture drama | hollow, merely slower/higher, harmonically foreign | typed role, source relation, clear gesture window, loop usefulness |
| Macro stop/drop/return | stage impact and renewed expectation | scripted medley, too many short sections, weak payoff | isolated loop pass plus time-local contrast and return proof |

Nonlinear branches should oversample only where the chosen shaper and realtime
budget justify it, with an explicit anti-alias filter and latency contract.
Oversampling without appropriate filtering does not automatically remove
aliasing. Parallel branches must be sample/latency aligned; otherwise their
supposed body reinforcement can create comb filtering, a hollow center, or a
second perceived attack.

## Riotbox Hardness Contract

### What `Hard` must mean

A Riotbox Hard variation must be an immediately recognizable,
performer-triggered increase in the intended impact domain while preserving
enough source identity and pulse to remain useful.

For `source_transient_chop`, Hard should normally combine:

1. a source-local perceptual attack aligned to the performance grid
2. measurable attack-spectrum or onset-strength change
3. retained body after the attack, unless the performer explicitly chose a
   thin choke/stab role
4. a bounded rhythmic or articulation contrast
5. source recognition and a stable underlying pulse

For `source_texture_bite`, Hard should normally combine:

1. continuous source flow
2. source-derived nonlinear or rough timbre
3. controlled spectral focus
4. preserved recognizable texture and useful level
5. no invented trigger grid

No universal requirement says that Hard must add bass. Bass pressure is judged
only when a typed owner assigns that role.

### What cannot pass as `Hard`

- a native-level gain difference that disappears when loudness matched
- more peak level, crest factor, silence, or waveform delta by itself
- shortening every hit until only ticks remain
- full-band distortion that removes body or source identity
- an unrelated global attack, oscillator, or noise layer
- pitch change described as aggression when it is only higher, slower, hollow,
  or louder
- off-grid retriggers that create echoes or missing-beat illusions
- a hardcoded sequence tuned to one source
- a technically safe render with no musician desire to trigger or loop it

## Current RIOTBOX-1422 Implementation Audit

The following findings explain the current artifact and constrain the next
implementation. They are code-audit results, not published psychoacoustic
claims:

1. The transient-chop envelope decays immediately. At `130 BPM`, the current
   `0.55`-of-an-eighth path reaches approximately `-4.8 dB` after `20 ms`,
   `-12 dB` after `50 ms`, and `-19 dB` after `80 ms`. It has no distinct
   source-adaptive attack, hold, or body stage.
2. A selected slice retrigger jumps the source cursor, but the edge detector's
   history is not primed from the sample immediately preceding that new local
   onset. `sample - last_character_input` can therefore classify a cursor
   discontinuity as musical attack and amplify an artificial click.
3. The current source policy reduces the source to its strongest positive
   `20 ms` envelope rise relative to a global mean, then chooses
   `source_transient_chop` or `source_texture_bite`. It does not yet distinguish
   kick body, snare crack, tonal hook, noise texture, or sustained body.
4. The live-path renderer's current promotion gate proves only non-silence,
   headroom, missing-source silence, and a Base/Hard waveform delta. A signal
   with less body and worse musical impact passes that gate.
5. Fresh stopped activation and running seek/restore are not fully modeled for
   this resample callback. A stopped state can begin sounding without a manual
   trigger, and an already-active callback can retain stale beat, step, and
   cursor state across a transport seek. Starting mid-bar in an inactive Hard
   slot can also leak slot-zero audio. Active source replacement has no separate
   source/artifact revision, so new PCM can inherit the old cursor and edge
   history when mode and variation remain unchanged. These need explicit
   regressions before the next review-ready product-path candidate.
6. The callback currently snapshots all `16,384` atomic source samples for
   every audio buffer. At `48 kHz` with `128`-frame buffers that is about
   `6.14 million` atomic sample loads per second plus a roughly `64 KiB`
   snapshot copy per callback. The branch increased this payload from the
   earlier deliberately bounded `4,096` samples without adding callback-time,
   xrun, or stack evidence. It is lock-free, but the branch needs either a
   bounded immutable PCM handoff or real-session worst-case evidence before
   merge and before adding decomposition, crossover, oversampling, or extra
   parallel paths.
7. Source projection uses arithmetic stereo-to-mono summing and uniformly
   selects at most `16,384` frames without an anti-alias filter. Opposed stereo
   content can lose body. The exact `81,237`-frame / `44.1 kHz` capture becomes
   a proxy with an effective rate of about `8.89 kHz` and a Nyquist limit near
   `4.45 kHz`. Original attack content above that limit cannot survive this
   representation faithfully; unfiltered point decimation aliases it, while
   reconstruction/interpolation images, later cursor jumps, and nonlinearity
   can create new `6–20 kHz` output that must not be mistaken for preserved
   source attack.

The immediate implementation order is therefore:

1. preserve the accepted Base output
2. correct fresh-start, seek/restore, and inactive-slot state semantics
3. add a bounded source/artifact revision and reset or phase-map cursor/history
   with a click-safe transition when active PCM changes
4. prime source-local edge history so cursor jumps cannot manufacture attack
5. replace immediate decay with a source-adaptive hold/release and a
   click-safe, phase-aligned attack/body plan
6. strengthen the product-path QA gate with onset/body, boundary-jump, and real
   Base-to-Hard transition evidence
7. benchmark the snapshot path and choose a bounded immutable handoff before
   expanding DSP cost
8. only then test band-limited nonlinear bite

## Next Multi-Source Hypotheses

These are ordered experiments, not accepted constants.

### H1: source-adaptive transient/body parallel path

Outside the realtime callback:

- locate each source-local perceptual onset
- derive a bounded attack region from its energy and spectral evolution rather
  than using one fixed sample count for every source
- estimate attack centroid, bandwidth, onset strength, midband level, and
  transient-to-harmonic relation
- retain a time-aligned, explicitly band-limited body path from the original
  source
- retain bounded full-rate or otherwise bandwidth-declared onset-local attack
  material; the current roughly `4.45 kHz` proxy bandwidth cannot test
  broadband attack preservation by itself

Inside the callback:

- render the dry/body path continuously or according to the typed retrigger
- add a bounded attack-only path
- avoid a second delayed copy of the onset

Hypothesis: driving a source-adaptive attack path while retaining body will
sound harder than multiplying the entire hit by the rejected `0.55`-step gate.

### H2: source-relative, band-limited nonlinear attack path

Select one attack-relevant band or bounded crossover plan from source evidence
outside the callback. Apply controlled saturation/clipping to that band on the
attack path, then blend it with the dry/body path and compensate output level.
Keep low energy out of the nonlinear path unless the typed owner and source
evidence call for low-body hardness.

Hypothesis: attack-local harmonic density and midband presence will add bite
without the hollow, foreign, or low-end-erasing result of full-band drive.

#### H2 development result: rejected

The first H2 development candidate selected a source-relative attack band,
added a level-compensated nonlinear parallel branch only during the
source-adaptive attack, and preserved the accepted Base hash. On Beat03 it
measured a `2.228x` `0–10 ms` attack lift, a `1.358x` selected-band lift, and
retained `1.040x` body energy in both the `40–120 ms` and `120–200 ms`
windows. The exact Base and Hard WAVs were technically different
(`0.966` correlation), but the project musician heard no audible difference
when they were played separately.

That verdict rejects the mechanism as a musician-facing Hard state. Concentrated
difference energy inside repeated short attack windows can pass directional
signal gates while remaining perceptually negligible across the performance
window. Do not rescue this candidate by raising its parallel-mix or output-gain
constant. A successor must change one causal dimension: extend a source-derived
nonlinear or destructive timbre through a bounded, perceptible gesture while
retaining the accepted body, timing, and source identity. Its gate must measure
gesture-duration body/timbre contrast in addition to attack-local change.

### H3: bounded source-derived roughness

Only after H1/H2 have an accepted source-relative foundation, test a small
amount of source-envelope-derived modulation in the roughness region. Never
activate it as a missing-source fallback and never use one fixed oscillator as
the audible content.

Hypothesis: bounded roughness can add salience to suitable texture or hook
material. It is expected to fail on some tonal and already-dense sources, so
the source policy needs an explicit unavailable/no-roughness choice.

### H4: role-owned body support

For kick/drum or assigned bass sources only, test a separate decay-controlled
body path. Tune its frequency from source evidence and preserve attack
clarity. Do not add this path to unassigned W-30 break material merely because
the listener used the general word "pressure."

Hypothesis: assigned low-body support can turn a sharp but thin hit into
physical punch. Excess decay or untuned resonance will instead sound boomy and
mask the hook.

### H5: one decisive performance move after the sound itself passes

Once the isolated Base/Hard material is desirable to loop, test one explicit
choke, stop, or destructive return. Do not use arrangement events to conceal a
weak underlying Hard sound.

Hypothesis: a learnable loop plus one bounded surprise produces more live value
than either an unchanged repetition or a medley of constant changes.

## Experiment and QA Protocol

### Source matrix

Before another human listening request:

- keep Beat03 as the exact taste target and preserve the accepted Base hash
- use at least five real development sources across at least four typed source
  families
- keep at least two different-family holdouts from choosing the algorithm or
  constants; use only causally untouched cases after the rotation described
  below
- include transient break, sparse drum/percussion, tonal/riff, and
  texture/pad/noise behavior where the typed policies are eligible
- fail unavailable sources honestly instead of generating replacement sound

The RIOTBOX-1422 audit found that `oga_illin_robotic` was rendered as the sole
`holdout-dense` target while Hard changed across five successive v3–v6 tuning
iterations. Its final `15.3%` texture delta was then explicitly used to raise
the shared texture regression threshold from `0.005` to `0.02` and freeze the
rules. It therefore changed the next implementation and is consumed
development material, not a reserve holdout. On `2026-07-26` the manifest
retired it with explicit rotation history and replaced it in `holdout_b` with
the independent CC0 `oga_riintron_fat_groove_drums` source. The replacement is
local, ignored, technically format-checked, unheard, and still
`provisional_unheard_holdout`; acquisition is not quality proof.
`oga_illin_robotic` cannot support another holdout/generalization claim.
`oga_bretbernhoft_beatloops` and `oga_akikazer_menu` were also rendered under
four changing algorithm generations. Their causal influence could not be
proved, so the manifest conservatively retires them rather than claiming they
remain fresh, replacing them with the unheard CC0
`oga_hornpipe2_prehistoric_drums` and `oga_centurion_chippy_melody` cases.

### One causal question per experiment

Do not change gate, timing, distortion, level, pattern, and body at once.
Compare one mechanism family at a time:

1. Base versus transient/body split
2. split versus split plus band-limited nonlinear path
3. accepted nonlinear path versus optional roughness
4. isolated accepted material versus one performance gesture

This keeps listening feedback actionable and prevents another chain of
hardcoded floats whose audible responsibility is unknown.

When the first implementation seam supports it, use a small factorial
diagnostic rather than serial taste-tuning:

1. chop/space only
2. attack bandwidth/perceptual-onset treatment only
3. body/transient-weight treatment only
4. controlled nonlinear/roughness treatment only
5. the intended combined candidate

Rate `harder`, `punchier`, `more aggressive`, `brighter`, `better hook`, and
`fatiguing` separately. Do not collapse them back into a generic `pressure`
question.

### Pre-registration and promotion gate

The first H1/H2 matrix is `technical_characterization`, not acceptance.
Numeric perceptual thresholds are not calibrated yet and must not be invented
after seeing holdout output.

Before rendering that matrix, freeze in its manifest:

- the exact mechanism version and unchanged settings used for every source
- the typed policy and intended audible domain for each eligible source family
- the expected direction of every mechanism-specific diagnostic, including
  attack, body retention, selected-band change, boundary jump, and timing
- hard safety failures: non-finite output, clipping, unauthorized stopped
  sound, missing-source sound, unstable pulse, unbounded boundary jump,
  Base-hash change, or callback-budget failure
- the rule that every family is reported separately; an aggregate average
  cannot hide one collapsed family
- whether each comparison is native level, loudness matched, or both

For the exact Beat03 control, H1 must at minimum preserve the accepted Base
hash, increase retained `40–120 ms` body relative to the rejected recipe,
reduce cursor-boundary artifacts, and keep a source-local attack. H2 must add
the declared selected-band attack/bite change without erasing that H1 body or
source identity. These are directional anti-regression conditions, not proof
that the result is musically Hard.

Development results calibrate candidate thresholds. Freeze those thresholds
and the algorithm before any valid holdout run. A family-level safety or
directional failure rejects the technical hypothesis. A surprising holdout
result may reject it or begin a new development cycle, but must never tune the
current candidate in place; if its output informs the next implementation, the
case is consumed and must be rotated. Only structured human listening can turn
a technically review-ready candidate into a musical pass.

### Required technical views

For each eligible source, record:

- non-silence, clipping, true peak, DC, channel balance, and deterministic hash
- native-level and loudness-matched Base/Hard comparisons
- source recognition or source-relative similarity evidence
- source-local onset alignment and loop-boundary stability
- onset strength and attack duration
- attack spectral centroid, maximum bandwidth, and midband level
- transient-to-harmonic or transient-to-body relation
- body-retention ratio after the attack
- time-local per-band delta rather than only full-render band share
- processed-minus-dry residual, harmonic distribution, intermodulation, and
  aliasing when nonlinear processing is intentional
- parallel-path delay, polarity, mono compatibility, and correlation
- declared proxy sample rate/bandwidth plus anti-alias and stereo-cancellation
  evidence
- optional `30–150 Hz` modulation energy when roughness is intentionally used
- cross-source diversity and holdout behavior

These metrics are diagnostic. They may reject silence, collapse, timing
failure, or a hypothesis that did not alter its intended dimension. They cannot
promote musical quality without structured listening.

### Listening assignment

Analyze and bind the exact WAV before playback. Tell the listener:

- which half is Base and which is Hard
- the exact duration
- whether bass is assigned
- whether the intended change is transient attack, midrange bite, body,
  roughness, or arrangement impact
- the one event or property that should be obvious

Listen both at native product level and loudness matched when level could bias
the result. Ask whether the Hard gesture is immediately harder, remains in
time, preserves useful source identity, and is desirable to trigger or loop.
A "`different but not harder`" result is a reject for the hypothesis.

## Realtime and Product-Spine Boundary

Research does not authorize a second action system or analysis inside the audio
callback.

- `w30.apply_damage_profile` remains the performer gesture owner unless a
  separate product decision changes it.
- Source analysis, onset localization, band choice, decomposition, modulation
  statistics, and recipe selection happen outside the callback.
- The callback consumes only a bounded typed render plan and fixed-size state;
  it performs no file I/O, allocation, model call, or unbounded analysis.
- Large immutable PCM payloads should use a bounded handoff rather than
  per-callback copies. The current path still recopies `16,384` atomics and
  remains blocked pending such a handoff or real-session worst-case
  callback/xrun/stack proof.
- Parameter changes are smoothed. Lookahead, oversampling, and parallel-path
  latency must be explicit parts of the realtime/latency contract rather than
  hidden production conveniences.
- Session/replay must retain the committed action, source/capture identity,
  policy version, and actual selected render inputs.
- Missing or ineligible source material stays unavailable/silent.
- Observer and listening manifests must expose the actual policy and source
  evidence, not only a hardness-looking recipe identifier.

## Reusable Guidance for Adjacent Sound Problems

### To sound more harmonic or coherent

- start new energy from the real source waveform rather than a foreign layer
- align the perceptual attack, not only the transport event
- keep a dry/source path when nonlinear processing threatens identity
- distort selected bands rather than forcing the entire spectrum through one
  shaper
- use trusted pitch/scale evidence for tonal transforms; otherwise prefer
  rhythm, filtering, or source-derived timbre over invented notes
- distinguish deliberate tension from accidental roughness, beating, or
  duplicated onsets

### To sound punchier or more physical

- protect the leading transient from masking
- keep enough body and duration after the head
- analyze short-time bands and transient/body relations instead of whole-file
  LUFS or crest factor
- add low-body support only for an assigned role
- use output compensation and loudness-matched comparison so louder is not
  mistaken for punchier

### To become more interesting and loopable

- establish one recognizable pulse and hook first
- use moderate, source-related syncopation rather than maximum event density
- repeat enough for prediction, then violate one expectation deliberately
- separate an isolated keepable loop from a compact gesture-demonstration arc
- make destructive variation lead to a worthwhile return

### To become more playable and performant

- give each control one immediate, named room-level consequence
- quantize committed intent so the TUI is not a reaction-time game
- preserve a stable Base that the performer can return to
- prefer a few strong orthogonal gestures over many subtle float changes
- expose role and expected sound so the musician can judge the right property

## Riotbox H7 Result and H8 Consequence

H7 proved that an audible, deterministic, source-dependent transform can still
fail the promised musical role. Its 8 kHz/63-level grit recipe made the result
dirtier and more destroyed, but structured human review found no harder
source-kick or low-end impact. Therefore:

- `source_grit_slam_v1` remains Damage/lo-fi vocabulary evidence, not proof of
  a Hard variation;
- roughness, destruction, level, and spectral difference must not stand in for
  low-end or drum-transient impact;
- a Hard candidate may claim source-kick impact only when analysis assigns a
  source-owned low transient;
- unassigned material must not receive a synthetic kick or bass fallback.

H8 introduces `source_low_transient_punch_v1`. Control-plane analysis requires
all three source-relative conditions before assigning it: meaningful 45–180 Hz
attack share, a low-band attack that exceeds its following body, and material
low-band attack energy relative to the whole source. During the exact live
callback path, the versioned recipe returns only that filtered source band in
parallel during the detected attack, after the destructive chain. Base bypasses
the recipe exactly.

Its fixed parallel gain is part of the versioned recipe, not an anonymous taste
float. Technical QA separately requires:

- absolute low-band attack energy for assigned material;
- relative low-band attack lift over Base;
- preserved later body, bounded total level, continuity, and no clipping;
- explicit `unavailable` ownership for material without qualifying source
  evidence;
- structured human confirmation that the relevant kick/body actually feels
  harder, since the measurements alone cannot establish that verdict.

## Riotbox H9 Result and H10 Consequence

H9 removed Damage from the assigned source-kick-impact path and added a short
source-owned 45–180 Hz body return plus a 900–3,600 Hz attack head. The frozen
candidate passed development and fresh holdout gates, but structured listening
found Base and H9 perceptually identical. The comparison was not a playback or
artifact-assignment error: the concatenated review payload contained the exact
three registered PCM files in order, and Base/H9 had distinct hashes and
waveforms.

The failure exposed a metric and mechanism mismatch:

- only about `5.71%` of the 7.385-second review render differed from Base by
  more than `0.01` full scale, about `0.422` aggregate seconds or roughly
  `18 ms` per selected hit;
- H9 raised the selected 0–10 ms attack but the 40–120 ms and 120–200 ms body
  windows only reached `1.040x` Base, approximately `0.34 dB`;
- the validator treated `>= 0.95x` as sufficient body evidence, so preservation
  could be mislabeled as punch;
- the `selected_band_attack` diagnostic still followed the historical
  Damage-bite band even though H9 bypassed that processing;
- the focused unit test proved only that one synthetic output sample increased.

Therefore H10 must not retune H9's parallel gains. It needs a different
source-local hit mechanism with separate, typed roles:

- an attack-head articulation with its own intended-band, absolute, relative,
  and duration evidence;
- a following low/body articulation spanning a musically meaningful portion of
  roughly 20–100 ms, with a required audible lift rather than preservation
  alone;
- a phase-coherent dry/body topology or band-replacement topology so parallel
  filter phase cannot silently erase the intended lift;
- native-level and loudness-matched comparison, plus a regression that fails
  when the large-difference region collapses into a sub-20-ms click;
- unchanged groove, source timing, Base hash, and missing-source silence.

## Riotbox H10 Result and H11 Consequence

H10's direct-path source hit shaper passed the development matrix, but the
frozen holdout rejected the candidate before human listening:

- a weak, intermittently active source was assigned transient-chop ownership,
  then failed because its later 120–200 ms body was absent;
- a pad selected `source_texture_bite` and reached about `1.400x` Base RMS, but
  that policy bypassed the transient-only directional validator and therefore
  bypassed the intended `1.15x` level ceiling;
- the failure was not evidence that another head/body gain tweak was needed.
  It exposed missing source suitability and incomplete policy-wide QA.

H11 therefore makes two contract changes before another listening request:

- typed source suitability is evaluated from original mono PCM before Hard
  ownership. A source below `0.04` RMS or below `0.60` active-frame share at a
  `0.001` activity floor receives explicit `Hard = unavailable`;
- every audible Hard policy passes the same bounded whole-gesture level check.
  Transient-chop retains its stricter attack/body/continuity gates, while
  texture-bite can no longer bypass level safety.

An unavailable Hard policy is not a weak candidate. It is a truthful product
outcome: trigger mask, attack windows, grit, and Hard gain remain inactive,
while the source-backed Base output is preserved sample-identically. The H11
QA pass also corrected the focus renderer so Source Monitor PCM cannot
contaminate the isolated W-30 Base/Hard delta.

## Riotbox H11 Result and H12 Consequence

H11 correctly rejected the frozen candidate before human listening, but its
new safety surfaces exposed two different proxy failures on the fresh holdout:

- a lower-level pad selected `source_texture_bite`, and the fixed `0.79`
  compensation still produced about `1.878x` Base RMS instead of remaining
  below the frozen `1.15x` ceiling;
- a transient source satisfied the source-analysis low-attack-over-body
  ownership proxy and received `source_hit_shaper_v3`, but the actual rendered
  20–100 ms body reached only `1.095x` Base instead of the required `1.15x`.

The first failure means nonlinear level compensation cannot be one global
number across sources. The second means source-band ownership evidence and
renderer-window outcome are related but not interchangeable. H12 therefore
must not adjust the H11 `0.79` or `0.94` values and retry. It needs a typed,
source-calibrated transform plan:

- predict the selected transform at projection time using the same DSP
  topology and source proxy as the live renderer;
- derive a bounded, explicit output compensation from that prediction and
  carry it through product state to the callback;
- retain `source_hit_shaper_v3` only when renderer-aligned prediction supports
  the required head/body outcome, otherwise keep the source-backed transient
  policy without falsely claiming low-impact ownership;
- preserve missing-source silence, unsuitable-source Base identity, source
  timing, groove, and the frozen policy-wide QA gates.

This calibration is not automatic mastering and must not make all sources
equally loud. It only prevents a fixed nonlinear recipe from creating
source-dependent level explosions and prevents an indirect feature ratio from
claiming an audible body result it cannot deliver.

The implemented H12 plan removes the fixed `0.79` texture compensation and
derives a bounded per-source gain targeting a predicted `1.05x` Base RMS. Its
projection proxy and callback share the same nonlinear source-character
topology. The versioned `source_hit_shaper_v3` recipe keeps its existing
`0.94` output gain; H12 does not retune that number. Instead, H12 measures the
selected source hit windows through the same peaking-filter topology, matches
the transformed proxy to a `1.20x` level target for comparison, and requires a
predicted level-matched 20–100 ms body ratio of at least `1.15x` before the V3
recipe may own Hard. Failing that evidence demotes the candidate to the normal
source-backed transient policy rather than manufacturing a body claim.

For texture policies, predicted raw and compensated level ratios describe the
whole callback proxy. For V3, they describe only the selected transform
windows and are not claims about whole-gesture output loudness; the
level-matched body ratio is the ownership signal. The exact-path renderer
continues to enforce the unchanged whole-gesture level ceilings and
directional transient gates. Its non-collapse check is now source-relative:
Hard-minus-Base delta RMS must be at least `0.12x` Base RMS, replacing the
fixed absolute `0.01` threshold that unfairly rejected otherwise meaningful
lower-level sources.

### H12 Listening Result

The frozen H12 holdout technically passed, but structured listening returned
`technically_ok_but_musically_weak`. The listener heard a less dull, clearer
transient and audible damage, but not a convincing or keep-worthy Hard gesture.
Exact four-second A/B analysis explains the result:

- Hard was only `0.6 LUFS` louder and reached a `1.057x` RMS level ratio;
- Hard/Base energy was nearly unchanged at 40–120 Hz (`1.027x`) and changed
  modestly at 120–500 Hz (`1.085x`);
- 500 Hz–2 kHz rose to `1.169x`, while 2–10 kHz nearly doubled to `1.991x`;
- the aligned waveform correlation remained `0.952`.

H12 therefore solved its calibration and truthful-ownership problem, but not
the broader musical Hard problem. The audible difference is dominated by
brightness and damage while low/body energy, rhythm, and gesture topology stay
too similar. The next candidate must preserve H12 source-relative calibration
and change at least two non-level perceptual dimensions: source-local
transient/body articulation plus a stage-useful rhythmic, pitch, or destructive
gesture. Another brightness, loudness, or generic-damage scalar adjustment is
not an acceptable next experiment.

## Frozen H13 Hypothesis: Source Reverse Into Impact V1

H13 tests one typed mechanism rather than another collection of independent
taste constants. It is available only for a suitable
`source_transient_chop` policy with source PCM and a committed `HardDamage`
action:

1. rank the already source-selected trigger slots by their local 0–20 ms head
   and 20–100 ms body evidence;
2. name exactly one strongest source impact and the immediately preceding
   eighth-note slot;
3. during the end of that preceding slot, read a bounded window of the same
   source impact backwards so its transient converges on the grid boundary;
4. return to the same impact forwards on the boundary and apply a bounded,
   source-relative body gain only inside its 20–100 ms body window.

The body gain is derived from the measured head/body relationship of the
selected impact. It may lift a weak body toward 90% of its own head RMS, is
bounded to `1.12..=1.40`, and cannot create a kick, bass, or unrelated layer.
The same measured head/body RMS and window durations derive a separate
impact-level compensation, so the local articulation cannot pass by raising
whole-gesture level. The callback does not lengthen or replace the selected H12
attack slice: after the existing H12 voice is rendered, H13 applies its bounded
2.5 ms-ramped body articulation only to the selected 20–100 ms output window.
The reverse pickup is normalized against the source energy already present at
the destination slot tail, then ramps to the compensated impact level over its
final 10 ms for continuity. It uses the same source cursor and is inactive
outside its declared pickup slot. Other selected hits retain the H12 path.

This changes two causal dimensions:

- **transient/body articulation:** the selected source hit develops from head
  into a more present source-owned body instead of receiving another broadband
  brightness adjustment;
- **arrangement/performance impact:** a source-derived reverse approach creates
  anticipation and a clear return boundary rather than repeating the same chop
  topology at a different level.

H13 does not claim bass pressure. Its intended domains are drum/transient or
midrange/hook body, plus arrangement/performance impact. A valid technical
candidate must prove the registered pickup and impact occur in their typed
slots, the reverse window actually runs backwards into the forward cursor,
the selected 20–100 ms body changes at matched whole-gesture level, Base and
unavailable output remain unchanged, cross-source slot/body choices differ
where source evidence differs, and the exact callback output remains bounded
and continuous. Brightness, waveform delta, loudness, or generic damage alone
cannot pass H13.

### H13 Holdout Result

The frozen H13 candidate failed holdout before human listening. On
`oga_iamoneabe_tryme`, Base-to-Hard RMS reached `1.45012x` against the frozen
`1.30x` maximum. Artifact forensics separated inherited and new behavior:

- Base-to-H12 was already `1.41339x`;
- H12-to-H13 added only `1.02599x` (`0.223 dB`);
- the level failure therefore belongs primarily to H12
  `source_hit_shaper_v3` calibration, not the reverse/body gesture.

The independent tonal holdout `oga_matiasvme_crazy` passed H13 with a
`1.1144x` selected-body ratio and `2.9243` pickup delta at `1.0144x`
Base-to-Hard level. Three degraded timing cases correctly could not confirm a
grid and stopped before candidate rendering. The mixed evidence is useful but
cannot promote H13: the whole frozen generation is technically rejected, no
human playback is requested, and holdout-b must not be used for retuning.

## H14: Exact Callback Calibration With Hit-Window Preservation

H14 treats the H13 rejection as a calibration/contrast problem, not permission
to weaken the release gates. For each new source revision and trusted tempo,
the control plane renders the exact W-30 callback over four complete
eight-step cycles. It compares Base with the typed Hard recipe, searches a
bounded between-hit output gain, and caches the result for subsequent runtime
refreshes. No measurement, allocation, file I/O, or search enters the realtime
callback.

The callback then applies three distinct responsibilities:

1. keep the source-selected 0–100 ms primary hit at unity, including 20 ms of
   real source context before its grid boundary and a 2.5 ms click-safe lead-in
   before that full-unity context;
2. retain the existing H12 `0.94` calibration through the 100–200 ms following
   body and fade out over 10 ms;
3. apply the source/tempo-specific calibrated gain only between those owned
   hit windows.

This is an arrangement and microdynamics decision: the physical hit stays
present while surrounding material yields, increasing usable attack contrast
without claiming bass pressure or hiding failure behind global loudness.
Sources without `source_hit_shaper_v3`, trusted tempo, or source PCM keep their
existing typed behavior and never receive a synthetic fallback.

The former H13 holdout failure `oga_iamoneabe_tryme` is now consumed
development material. H14 changes its fixed H12 output gain from `0.94` to
`0.639841`, retains the selected hit locally, and measures Base-to-H12
`1.25878x`, filtered head `1.16759x`, filtered body `1.30463x`, and
H12-to-H13 impact body `1.05539x`. The complete Base-to-H13 level is about
`1.29208x`, below the unchanged `1.30x` ceiling. Beat03, dense-full-mix stress,
sparse percussion, tonal, and pad development cases also pass; weak and
texture cases retain unavailable or continuous-policy behavior.

This technical pass is not a human musical verdict. H14 must still be frozen
before one untouched multi-family holdout set is rendered, and only a complete
technical holdout pass may produce a structured listening pack.

## H17–H18: Runtime-Synchronous Body Ownership

The H17 reachability-qualified `oga_congusbongus_lasso_lady` holdout exercised
the intended `source_hit_shaper_v3` path but failed before listening. Its
frozen callback render reached a Base-to-Hard level ratio of `1.31278`, above
the unchanged `1.30` ceiling, while its apparent late-body result was being
measured against the requested 140 BPM rather than the product-owned
`140.62514 BPM` grid. `oga_yd_oriented` was preflight-ineligible because the
low-impact recipe remained unavailable. Neither case received a human verdict.

H18 corrects the causal path rather than loosening either gate:

1. H12 exact calibration owns the between-hit gain and its head/body floors.
2. The same gain is verified separately through the complete H13 callback, so
   an H13 level reduction cannot conceal an H12 overshoot.
3. The existing typed hit-window compensation now reaches the realtime
   snapshot. When the exact callback proves a 120–200 ms body shortfall, the
   control plane derives the smallest local late-body target that restores the
   unchanged `0.95` floor.
4. H13 no longer attenuates the selected impact after source-local body
   articulation. The derived body gain remains bounded at `1.75`, and the
   unchanged full-path `1.30` ceiling prevents this from becoming a global
   loudness fix.
5. Product-path QA measures against `hard_state.tempo_bpm`, the same tempo used
   by the callback.

The post-change development matrix spans eight sources across dense break,
dense-full-mix stress, sparse drums, tonal riff, pad/noise, and weak/unavailable
behavior. Beat03, Bertsz, and Lasso exercise exact callback calibration and
pass the full H12/H13 gates. Their selected H13 impact-body ratios are
`1.0633`, `1.0720`, and `1.1410`; their full Base-to-Hard levels are `1.2690`,
`1.2340`, and `1.1652`. Cinameng, Marwan, Fupi, and Beard remain within the
legacy `1.15` ceiling; Pauliuw remains explicitly unavailable without fallback
audio.

This is technical development evidence, not a musician-facing quality pass.
Fresh multi-family acceptance and structured listening remain mandatory.

## H25 V5: Source-Aligned Impact Instead Of Scalar Tuning

Repeated listening rejected several superficially different Hard renders for
the same perceptual reason: louder, darker, dirtier, more hollow, or more
damaged was not the requested harder drum impact. Delayed attacks, repeated
foreign clicks, lost body, and unchanged rhythmic identity could not be
repaired responsibly by another gain, drive, or EQ scalar.

`source_aligned_impact_v5` therefore changes the decision algorithm:

1. inspect each real source onset on a 2 ms analysis hop over a 50 ms region;
2. reject a candidate whose strongest presence rise starts more than 15 ms
   after its intended grid point;
3. require at least two aligned source hits so one accidental click cannot own
   an eight-step performance;
4. choose a real hit by balancing source-relative presence attack and attack
   crest rather than maximizing either value alone;
5. repeat that selected source hit only on the source-derived trigger mask;
6. preserve the dry source body and add only a causal 900–3600 Hz nonlinear
   presence residual inside the selected attack head;
7. if the full source rhythm violates the unchanged level, causal-head, or
   crest contracts, try an anchored alternating mask and then an anchored
   two-hit floor; and
8. surface source mismatch or unavailable state when no candidate passes.

An evaluated rejection keeps its source-derived V5 selector evidence only for
negative-cache matching. The projected Hard policy is `unavailable`, so the
callback cannot trigger the rejected recipe; repeated view refreshes reuse the
same rejection instead of performing the exact control-plane search again.

This turns earlier musician feedback into falsifiable behavior:

- “only louder” is rejected by level-matched causal head and crest checks;
- “dumpfer/übersteuert” cannot pass without retained crest;
- “außerhalb vom Takt” is rejected by source-local peak-to-grid alignment;
- “zu monoton” is addressed by preserving the source trigger topology and
  choosing density from three typed candidates rather than repeating every
  step;
- “kein Bassdruck” is not silently reinterpreted as failure because this
  recipe owns transient/midrange impact and leaves bass ownership unassigned.

The development matrix spans the Beat03 dense-break Golden Path, a
dense-full-mix stress case, a second dense break, sparse percussion, and a
tonal riff. Beat03 and Bertsz pass the exact callback. Cinameng and Marwan
correctly fail local attack-over-body ownership. Fupi reaches the exact V5
solver but fails level/crest preservation and therefore becomes unavailable
instead of emitting fallback audio.

The fresh Continue holdout is the first exact V5 pass: whole-window level
`1.01426`, causal presence head `1.14549`, and crest `1.00119`. Three tonal
holdouts correctly reject either performer impact ownership or exact level
preservation. That is only one prequalified source family, however. H25
therefore remains a technical partial pass with `human_verdict: unverified`;
it does not meet the required two-source/two-family holdout gate and is not a
musical-alpha claim.

Machine-readable evidence is frozen in
`docs/benchmarks/w30_resample_h25_characterization_v1.json`.

## Sources

Primary perceptual and audio research:

- Pearce, Brookes, Mason, [Modelling Timbral
  Hardness](https://doi.org/10.3390/app9030466), 2019.
- Freed, [Auditory correlates of perceived mallet hardness for a set of
  recorded percussive sound
  events](https://pubmed.ncbi.nlm.nih.gov/2299041/), 1990.
- Fenton, [Audio Dynamics—Towards a Perceptual Model of
  Punch](https://eprints.hud.ac.uk/id/eprint/32629/), 2017.
- Fenton, Lee, Wakefield, [Hybrid Multi-resolution Analysis of
  Punch](https://eprints.hud.ac.uk/id/eprint/24358/1/AES%20138_Fenton_complete.pdf),
  2015.
- Fenton, Lee, Wakefield, [Towards a Perceptual Model of Punch in Musical
  Signals](https://eprints.hud.ac.uk/id/eprint/26288/9/AES%20139_Perceptual_Punch_Model_Fenton.pdf),
  2015.
- Fenton and Lee, [A Perceptual Model of Punch Based on Weighted Transient
  Loudness](https://doi.org/10.17743/JAES.2019.0017), 2019.
- Bechtold and Senn, [Articulation and Dynamics Influence the Perceptual Attack
  Time of Saxophone
  Sounds](https://doi.org/10.3389/fpsyg.2018.01692), 2018.
- Arnal et al., [Human Screams Occupy a Privileged Niche in the Communication
  Soundscape](https://doi.org/10.1016/j.cub.2015.06.043), 2015.
- Arnal et al., [The rough sound of salience enhances aversion through neural
  synchronisation](https://doi.org/10.1038/s41467-019-11626-7), 2019.
- Anikin, [Acoustic estimation of voice
  roughness](https://pubmed.ncbi.nlm.nih.gov/40295423/), 2025.
- Wang et al., [The impact of audio effects processing on the perception of
  hardness of Bass
  Drum](https://doi.org/10.1049/ccs2.12060), 2022.
- Moore, [Dynamic Range Compression and the Semantic Descriptor
  Aggressive](https://doi.org/10.3390/app10072350), 2020.
- Hafezi and Reiss, [Autonomous multitrack equalization based on masking
  reduction](https://doi.org/10.17743/jaes.2015.0021), 2015.
- Kahles, Esqueda, and Välimäki, [Oversampling for Nonlinear Waveshaping:
  Choosing the Right
  Filters](https://doi.org/10.17743/jaes.2019.0012), 2019.
- Witek et al., [Syncopation, Body-Movement and Pleasure in Groove
  Music](https://doi.org/10.1371/journal.pone.0094446), 2014.
- Matthews et al., [The sensation of groove is affected by the interaction of
  rhythmic and harmonic
  complexity](https://doi.org/10.1371/journal.pone.0204539), 2019.
- Lahdelma et al., [Register impacts perceptual consonance through roughness
  and sharpness](https://pmc.ncbi.nlm.nih.gov/articles/PMC9166839/), 2022.
- Margulis, [Aesthetic responses to repetition in unfamiliar
  music](https://doi.org/10.2190/EM.31.1.c), 2013.

Official production documentation:

- Ableton, [Live 12 Audio Effect
  Reference](https://www.ableton.com/en/live-manual/12/live-audio-effect-reference/)
  (`Drum Buss`, `Roar`, `Saturator`, `Limiter`).
- Native Instruments, [Transient Master
  Manual](https://www.native-instruments.com/ni-tech-manuals/transient-master-manual/en/welcome-to-transient-master).

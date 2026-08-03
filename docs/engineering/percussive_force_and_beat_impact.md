# Percussive Force and Beat Impact

Status: active P023 research paper and future-validation brief

Owner: RIOTBOX-1429

Direct audible follow-up: RIOTBOX-1428

Scope: understanding percussive events, beats, groove, force, softness,
heaviness, and arrangement impact before any further implementation

## Decision Summary

Riotbox will not implement `Hard` as a global preset, one scalar, or a synonym
for damage. The failed RIOTBOX-1428 iterations exposed a more fundamental gap:
the project did not yet distinguish an isolated hard strike from a hard beat,
a heavy arrangement, a punchy mix, an aggressive timbre, or a compelling
groove. This paper closes that conceptual gap before another DSP pass.

The narrow meaning retained for **percussive force** is:

> The candidate is recognizably the same source hit in the same rhythmic place,
> but it feels as if it was struck more forcefully.

The claim needs a typed owner, causal signal evidence, and a human directional
verdict. A candidate is rejected when it is mainly louder, globally pitched,
darker, brighter, dirtier, clipped, shorter, doubled, delayed, or simply
different. Mechanical metrics may reject collapse and confounds; they cannot
award the musical verdict.

RIOTBOX-1429 is a `contract_enabler`. Its deliverable is research, vocabulary,
an analysis rubric, and falsifiable hypotheses. It does **not** change Rust,
DSP, product behavior, the realtime callback, or generate another listening
candidate. The algorithm families and experiment design later in this paper
are a handoff for RIOTBOX-1428 Stage A. They are not implemented mechanisms
and do not claim instrument progress.

## Typed Vocabulary

Words such as `hard`, `punch`, and `pressure` are too ambiguous without a
domain. Riotbox uses the following typed meanings:

| Construct | Musician-facing meaning | Not sufficient |
| --- | --- | --- |
| `percussive_hard` | Same recognizable hit and rhythmic identity, perceived as a more forceful strike | gain, global pitch/rate change, a click, flam, darkness, or distortion |
| `percussive_soft` | Same recognizable hit and rhythmic identity, perceived as a gentler strike | lower gain, darkness, blurred onset, shorter tail, or collapsed body |
| `drum_punch` | Short-time attack/body impact or dynamic immediacy | peak, crest factor, LUFS, or sustained weight alone |
| `hook_hard` | More immediate midrange bite and source-backed riff presence | unrelated treble or a foreign stab |
| `bass_hard` | More controlled low-end impact from an explicitly assigned bass owner | relative low-band share without absolute energy and ownership |
| `aggressive_grit` | Controlled roughness, nonlinear density, or aversive salience | proof of strike force or punch |
| `destructive_damage` | Deliberate breakage, reverse, rate movement, aliasing, or bit damage | an acceptable `percussive_hard` substitute |
| `arrangement_impact` | A stop, choke, fill, drop, role swap, or return that changes expectation and the room | isolated-hit force |
| `loopability` | Stable pulse and identity with enough bounded development to reward repetition | force, novelty density, or a scripted medley |

These constructs may reinforce one another but never substitute for one
another. A forceful snare need not own bass. A distorted hit may be aggressive
without being more forceful. Silence can create arrangement impact without
changing the struck event.

## Evidence Tags

Evidence directness and replication are recorded separately. A direct single
study must not look replicated merely because its construct is relevant:

- **E1 — direct construct evidence:** a controlled perceptual task directly
  measured the named construct for its stated stimuli and listener population.
- **O — observational or performed evidence:** measurements describe recorded
  behavior, performance, or acoustics without directly testing the named
  listener perception.
- **E2 — transfer evidence:** controlled evidence concerns another source type
  or adjacent construct and supports a mechanism or measurement, not the
  Riotbox verdict.
- **E3 — engineering mechanism:** documentation or signal theory explains what
  processing does but does not validate perception.
- **Replication:** `single_study`, `single_study_with_internal_validation`,
  `independent_related_studies`, `multiple_related_studies`, `replicated`,
  `unknown`, or `not_applicable`, recorded alongside E1–E3/O rather than
  encoded into them. Sample size, corpus, and mixed-method limitations stay in
  prose rather than becoming extra replication labels.
- **R — Riotbox observation:** exact project artifact plus mechanical and/or
  human evidence. It is valid negative or local evidence, not a universal law.
- **H — unverified hypothesis:** a causal candidate to test. It must not be
  mirrored into agent skills as established behavior.

## A Beat Is a Multi-Scale Object

A beat is not a waveform, a BPM value, or a list of onset timestamps. It is an
organized field of expectations and sounding roles that unfolds over several
time scales. Riotbox must keep these layers separate in analysis and may then
reason about their interaction:

| Layer | Primary question | Relevant evidence | Typical false conclusion |
| --- | --- | --- | --- |
| event articulation | What kind of physical or synthetic gesture does one event imply? | attack evolution, body, decay, local spectrum, velocity/articulation cues | a higher peak means a harder strike |
| pulse and meter | Can a listener infer where the beat and stronger metric positions are? | beat salience, periodicities, stable anchors, meter confidence | every detected onset is equally important |
| pattern topology | Which expected positions sound, sustain, or remain empty? | role-labelled onset grid, inter-onset structure, density, syncopation, pickups, omissions | more notes or more syncopation means more groove |
| performance field | How are dynamics and timing organized around the grid? | role-relative velocity hierarchy, repeatable microtiming, swing ratio, push/pull relations | random jitter and random velocity equal human feel |
| orchestration and interlock | Which roles reinforce, answer, or leave space for one another? | kick/bass/hook alignment, composite attacks, call/response, masking, negative space | a drum stem can be judged without its intended partners |
| phrase and arrangement | How does expectation change across repetitions? | invariant anchors, bounded variation, fills, stops, drops, returns, section contrast | an identical one-bar loop proves development, or constant novelty proves playability |
| mix and playback | Does the intended gesture remain legible and physical in context? | absolute and relative band energy, transient masking, proximity, headroom, dynamics, playback chain | louder, darker, wider, or more distorted means harder |
| listener and use | Does the result invite movement, retriggering, looping, or stage action? | blinded comparison, body response, musician task, familiarity and context | a machine score awards musical fitness |

No scalar may collapse these layers into a universal `beat_quality` or
`hardness` score. A system may use layer-specific diagnostics to reject a
failure, but the requested musical construct and its owner must be stated
before analysis begins.

### Relational vocabulary beyond one hit

| Construct | Working meaning | Distinguish it from |
| --- | --- | --- |
| strike force | one event is perceived as a more forceful gesture while identity and rhythmic place remain | level, pitch, distortion, duplicate onsets |
| articulation hardness | a fast, short, temporally precise-seeming attack quality | strike force, punch, aggression, or loudness |
| punch | short-time attack/body impact that remains legible in its relevant bands | sample peak or crest factor alone |
| weight | sustained or repeated bodily impression of mass, often involving low/mid body and temporal spacing | bass ownership, loudness, or darkness alone |
| drive | forward motion produced by pulse clarity, anticipation, event flow, and role interaction | speed or density alone |
| groove | pleasurable urge to move with a rhythm | timing deviation, syncopation count, or familiarity alone |
| heaviness | contextual experience combining sonic, performative, structural, and affective factors | a single drum property or low tuning |
| aggression | threat, friction, abrasion, urgency, or aversive salience | strike force, punch, or heaviness |
| softness | gentler articulation or lower-impact organization with intact pulse, identity, and expressive hierarchy | simply turning the signal down or blurring it |
| interest | attention sustained by recognizable identity plus interpretable change | maximum novelty or maximum event density |

The same passage can be soft but grooving, heavy but slow, aggressive but thin,
punchy but not bass-heavy, or rhythmically hard-hitting while every isolated
sample is modest. Human review briefs must therefore name the intended layer
and construct rather than ask whether something has generic `pressure` or is
generic `hard`.

## Evidence Model

### Perceived hardness is multidimensional

Pearce, Brookes, and Mason modeled semantic timbral hardness using 202 stimuli
from 32 source types. Their six-feature regression used maximum attack
bandwidth, attack spectral centroid, midband level, percussive-to-harmonic
ratio, onset strength, and log attack time. It reached `R² = 0.76` on training
material and `R² = 0.57` on a new dataset. Maximum bandwidth in the first
`100 ms` contributed most strongly. Clicks, sibilance, and recording defects
also caused over-prediction. **Directness E1 for broad timbral metadata and E2
for Riotbox perceived strike force; replication
`single_study_with_internal_validation`.**

Consequence: use several attack-local temporal and spectral features together,
and add explicit artifact controls. Do not optimize a single hardness score or
copy the paper's empirically chosen thresholds into Riotbox.

Freed modeled perceived mallet hardness from spectral-level mean and slope,
spectral-centroid mean, and a time-weighted centroid across the first `325 ms`
of struck sounds (`R² = 0.725`). **Directness E1 for mallet hardness and E2 for
mixed drums and breaks; replication `single_study`.**

Consequence: hardness-related timbral cues evolve through the attack and early
body. A full-file spectrum or one instantaneous peak is inadequate.

Wang and colleagues found that compression, equalization, and high-/low-band
excitation changed hardness ratings for processed bass-drum stimuli in their
23-participant study. High-frequency excitation increased rated hardness most,
while their low-frequency excitation condition reduced it; the authors related
harder cases to a quicker, more pronounced sound head rather than to extra bass
alone. The single kick, coupled processor changes, and fixed settings do not
establish a cross-source force or EQ recipe. **Directness E1 for processed
bass-drum hardness and E2 for Riotbox perceived strike force; replication
`single_study`.**

Lakatos found rise time and spectral centroid useful in broad harmonic and
percussive timbre spaces. Those are descriptive dimensions, not measurements
of strike force. **Directness E2; replication `single_study`.**

### Physical impact force uses multiple cues

Lutfi, Liu, and Stoelinga asked listeners to discriminate force of impact from
physically modeled struck bars. Listener strategies varied substantially;
percussionists generally performed better, and force judgments depended on
combinations of partial level and frequency rather than one ideal cue.
**Directness E1 for modeled struck bars and E2 for recorded drums; replication
`single_study`.**

Cheshire, Stables, and Hockman showed that participants could distinguish
high- and low-velocity snare strikes after loudness-disparity removal. Attack
time, decay time, fundamental frequency, and brightness differed significantly
between recorded velocities. This supports velocity discrimination, not a
direct judgment that one strike sounded harder. **Directness E1 for recorded
snare-velocity discrimination and E2 for Riotbox perceived strike force;
replication `single_study`.**

Consequence: `1.0x` and no global transposition are invariants against an
algorithmic pitch cheat. Natural performance can covary with local resonance
or fundamental movement, so a source-consistent local change is not an
automatic reject. It remains an unvalidated Stage-A cue, cannot itself prove
force, and must not be frozen numerically from this study.

Acoustic drum construction further conditions those changes. Strike position,
head and shell modes, snares, cavity coupling, and tension alter the response;
the system is not a generic envelope generator. Acoustical studies support the
source dependence of resonant behavior but do not provide a Riotbox hardness
transform. **Directness O and E2; replication `multiple_related_studies`.**

Consequence: source-adaptive processing must infer what body and resonances are
actually present. A universal envelope or EQ gesture cannot be assumed to
represent a stronger strike across snares, kicks, electronic drums, and dense
breaks.

### Punch is transient-local and band-dependent

Fenton and Lee's PM95 model combines signal separation, onset detection,
low-level features, and perceptually derived octave-band/time weights. Against
subjective punch judgments it achieved Pearson `r = 0.849` and Spearman
`rho = 0.833`. **Directness E1 for punch; replication `single_study`.** Riotbox
has not reproduced or validated PM95 and must not label a simple attack/body
ratio as that model.

Consequence: the harness needs event-local band and time views. Whole-render
RMS, LUFS, crest factor, correlation, and spectral share remain safety or
collapse screens.

### Physical onset, PAT, P-centre, and ATC are different constructs

Physical onset is a signal boundary. Gordon's perceptual attack time (PAT) is a
listener-estimated attack location relative to physical onset for an isolated
tone. His best orchestral-tone model correlated `0.995` with measured values
and depended strongly on rise behavior and listening level. Bechtold and Senn
later found that articulation, dynamics, and their interaction explained much
more PAT variance for saxophone tones than simple rise time; rise time
correlated only `r = 0.143`. **Directness E1 for PAT in the studied tones and
E2 for drum-event alignment; replication `independent_related_studies` for the
boundary on universal rise-time prediction, not for one universal PAT model.**

A P-centre is the perceived temporal location of an event in a rhythmic
alignment task. Danielsen and colleagues directly studied how attack duration
and shape can move that location. Attack temporal centroid (ATC), meanwhile,
is a timbre descriptor computed inside an independently detected attack
interval. It is neither PAT nor P-centre and must not define the interval over
which it is itself calculated.

Consequence: a cursor and physical-onset invariant prevents gross timing
errors. Riotbox may report a separately named `rhythmic_location_proxy_v1` to
reject large movement, but it cannot label that value PAT, P-centre, or groove
validation until a human alignment experiment calibrates it.

Danielsen and colleagues also found that expert drummers instructed to play a
rock pattern pushed, on-beat, or laid-back changed not only onset placement but
snare level and temporal/spectral centroid; tempo influenced those sound
features too. **Directness O and E2 for Riotbox transformation; replication
`single_study`.**

Consequence: moving an existing event earlier or later is not a complete model
of a performed timing intention. Articulation, dynamics, timbre, and relative
placement may form one gesture, and all must be analyzed before calling a
result `pushed`, `laid_back`, or `harder`.

### Aggression and roughness may covary without proving force

Moore found strong associations among perceived distortion, roughness, and
aggression for two strongly compressed rock-vocal mixes. The tested material
does not establish a drum-force recipe or universal total-harmonic-distortion
target. **Directness E1 for those vocal stimuli and E2 for aggressive drum
color; replication `single_study`.**

Consequence: a bounded nonlinear branch may add aggression or bite, but it
must be measured and judged separately from force. Full-band distortion is a
mandatory false-positive control.

### Production mechanisms are not perceptual proof

Official Ableton and Native Instruments documentation separates transient
attack, sustain/body, distortion, low-end enhancement, and output gain. This
supports distinct processing controls and parallel or band-aware hypotheses.
It does not prove that any setting sounds harder. **Directness E3; replication
`not_applicable` to the perceptual claim.**

Nonlinear paths require explicit oversampling/filtering and latency decisions.
Kahles, Esqueda, and Välimäki show that waveshaping oversampling quality depends
on the interpolation and decimation filters; oversampling alone does not remove
aliasing. **Directness E3; replication `not_applicable` to the perceptual
claim.**

### Mix survival and phase are contextual checks

Parker and Fenton's transient/steady-state/residual masking model correlated
with subjective mix clarity, while other mix research shows that experienced
productions can contain intentional masking. **Directness E1 for mix clarity
in the tested material and E2 for Riotbox event survival; replication
`multiple_related_studies`.**

A small AES drum-alignment study found that only a minority of listeners
reliably detected time-aligned versions and did not establish a preference for
the corrected mixes. **Directness E1 for those mixes; replication
`single_study`.**

Consequence: inspect cancellation, flams, overlap, and masking when a role
fails in context, but do not optimize abstract `clarity` or phase alignment as
universal quality. The question is whether the intended gesture survives in
the declared mix without erasing productive interlock or source character.

## Cross-Genre Findings

The purpose of comparing metal, jazz, funk, and sample-based dance music is not
to average them into one style. Each exposes a different failure in the idea
that a beat can be improved by turning one generic amount knob.

### Metal: heaviness is relational, not an isolated drum setting

Herbst and Mynett's systematic metal-production research treats heaviness as a
compound of sonic, performative, structural, and affective factors. Their
production analyses and practitioner accounts repeatedly separate punch,
weight, rhythmic intensity, ensemble precision, clarity, density, distortion,
and contrast. This is qualitative genre evidence rather than a controlled
Riotbox listening experiment. **Directness E2 for the concept of musical
heaviness; replication `multiple_related_studies`.**

The transferable findings are:

- a fast, dense passage and a slow, spacious breakdown can both feel heavy for
  different reasons;
- at high event rates, shorter and more sharply articulated drum events can
  preserve legibility, whereas longer low-frequency bodies may mask adjacent
  hits and turn force into a wash;
- at slower rates, wider spacing can accommodate more body and low-frequency
  sustain, so perceived mass may increase without adding notes;
- kick, bass, and guitar articulation form an ensemble gesture; precise
  reinforcement can make the composite feel larger than any isolated member;
- `in your face` proximity requires transient legibility and audibility of the
  contributing gestures, not merely reduced dynamic range;
- a slowdown, half-time feel, dropout, or return derives much of its impact
  from contrast with what preceded it. Constant maximum density removes that
  relational leverage.

Consequently, metal does not establish a `hard drum` recipe for Riotbox. It
establishes that **event force**, **ensemble lock**, **spectral room**, and
**arrangement contrast** must be assessed separately and then related. A dense
fast break may need less per-hit body than a half-time slam, even when both are
intended to hit hard.

### Jazz: stable reference plus controlled freedom

The cited jazz studies do not support either a universal swing constant or
unconstrained random humanization.
Friberg and Sundström found tempo-dependent swing ratios and ensemble timing;
later studies likewise show that a notated `2:1` triplet ratio is not a
universal performance rule. Datseris and colleagues found that removing
natural random-scale microtiming from twelve piano performances did not reduce
swing ratings overall, while expanding those deviations reduced ratings.
**Directness E1 for the tested jazz stimuli; replication
`independent_related_studies`.**

Nelias and colleagues then isolated a particular *relational* pattern:
slightly delayed soloist downbeats paired with offbeats synchronized to the
rhythm section increased swing ratings among professional and semiprofessional
jazz musicians. Their analysis of 456 improvisations found a tempo-dependent
trend but not universal use by every performer. This does not authorize a
global drum delay. **Directness E1 for the tested soloist/rhythm-section
relation; replication `single_study_with_internal_validation`.**

Dahl's percussion-performance study found that accented strokes were prepared
from greater stick height and delivered with greater striking velocity; the
interval following an accent was also commonly lengthened. The performers used
different movement strategies. **Directness O and E2 for audio-only Riotbox
generation; replication `single_study`.**

The safe transfer is:

- preserve a clear timing reference before introducing freedom around it;
- model timing as relations between roles and metric positions, not independent
  random offsets per hit;
- model accents as a hierarchy of articulation, dynamics, and local phrasing,
  not just louder samples;
- allow stylistic timing to depend on tempo and role;
- use quiet strokes and omissions to make primary accents legible. A soft beat
  can retain strong groove because softness is not rhythmic vagueness.

Ghost-note research provides a useful warning: listener ratings changed with
ghost-note treatment, but ghost notes did not universally improve groove and
their effect depended on pattern and context. They are named connector,
anticipation, or texture roles inside an accent hierarchy—not generic filler
for empty grid cells. **Directness E1 for the tested patterns; replication
`single_study`.**

`Hard articulation` in jazz descriptions often means a fast, short,
temporally precise-seeming onset. It must not be silently translated into
Riotbox `percussive_hard`, transient punch, bass pressure, distortion, or
arrangement impact.

### Funk and breakbeat: the rhythmic fingerprint survives the solo break

Ainsworth measured more than one thousand onset deviations in fourteen early
funk recordings. The study found repeatable microrhythmic contours and related
many of them to bass, organ, guitar, vocal, and arrangement structure. In the
`Amen Brother` analysis, the first two drum-break bars mirror the preceding
organ rhythm and dynamics; the later bars vary the pattern. The break therefore
contains evidence of roles that are temporarily absent. The paper is an
observational analysis, not proof that copying its deviations improves groove.
**Directness O and E2 for generative transfer; replication `single_study`.**

Sioros and colleagues tested ten reconstructed polyphonic funk/rock excerpts
with 35 listeners. Removing original syncopation reduced groove, but adding
pseudorandom `25%`, `50%`, or `70%` syncopation did not recreate the original
effect. Their structural comparison found differences in which instruments
and metrical positions carried syncopation, in counter-metric figures, and in
pickup patterns. **Directness E1 for those stimuli; replication
`single_study`.** The exact percentages are experimental conditions,
not production constants.

Witek and colleagues found an inverted-U relation between syncopation and
pleasure/urge to move in funk drum breaks; a later movement study found little
spontaneous synchronization at high syncopation. The broader literature is
not unanimous about every groove predictor, and listener familiarity matters.
**Directness E1 for the tested groove stimuli; replication
`multiple_related_studies`.**

The safe transfer is:

- a source break's fingerprint includes role-labelled onsets, accent and
  ghost-note hierarchy, local timbre, repeatable timing relations, pickups,
  omissions, and phrase contour—not merely BPM plus a waveform slice;
- chopping may preserve that fingerprint, deliberately mutate one named
  dimension, or explicitly destroy it. Randomly moving hits is not neutral;
- the stable anchor and the counter-rhythm must remain distinguishable. If all
  roles syncopate or fill space equally, the meter and the surprise both lose
  meaning;
- a variation should answer an established loop. Constant novelty becomes a
  medley, while exact short repetition without meaningful contrast can become
  inert;
- a stop, break, or return is a phrase event whose force depends on prior
  expectation. Silence is not an isolated-hit hardener.

Score-informed drum-separation research demonstrates that recovering
individual events from a mixed historical break is a separation and transient-
restoration problem, not exact sample extraction. It therefore motivates
explicit artifact and crosstalk checks but does not validate a Riotbox chop
policy. **Directness E3; replication `single_study`.** A future chop record
should retain role, physical onset, pre-onset look-behind, attack/body/tail,
source timing mode, accent rank, and boundary confidence. Separation artifacts
and crosstalk are gates before musical tuning. Repeating one canonical hit at
every matching grid position is source reuse, but not preservation of the
performed break's articulation.

### Sample-based rave: beat and stage impact come from coordinated roles

Empirical EDM work found that groove-related responses were not explained by
an isochronous bass alone; rhythmic properties in high, mid, and low bands,
dynamic fluctuation, and timbral fluctuation separated different excerpt
clusters. Bass-filter experiments also show that timbre can affect groove and
liking, but do not establish a universal bass curve. **Directness E1 for the
tested EDM excerpts and E2 for Riotbox; replication `multiple_related_studies`.**

For Riotbox's rave-punk direction, the practical model is therefore a
coordinated but performer-separable set of roles:

- one or more unmistakable pulse anchors;
- a memorable source-backed hook or break fingerprint;
- a deliberately assigned low-end owner when bass pressure is intended;
- attack/body space that lets the hardest event speak;
- one interpretable counter-rhythm or disruption rather than indiscriminate
  busyness;
- phrase-scale stops, fills, destructive mutations, and returns that remain
  triggerable choices instead of a forced script.

This is a quality reference model, not a request to imitate a commercial track
or to turn commercial reference audio into product material.

### Local commercial reference boundary

The product owner's highlighted passages are taste coordinates, not training
data or reconstruction targets. `Their Law` around minute five primarily
anchors phrase/arrangement impact; `Full Throttle` around minute one anchors a
memorable melodic hook; `Voodoo People` anchors rhythmic identity. The broader
full-era reference set, including `Spitfire` and `Firestarter`, contributes
contrasting examples of abrasive hook presence, urgency, physical drums,
negative space, and live-room usefulness. **Evidence R: product-owner taste
orientation, not scientific validation.**

No one track or passage defines Riotbox. Commercial recordings remain local,
ignored, uncommitted listening/measurement references and may never become
product sources, fixtures, generated assets, or redistributed excerpts. The
analysis rubric compares dimensions and relationships; it does not optimize
toward a commercial recording's waveform or numbers.

### Stops and low end need explicit context

Structured silent-event positions carried prediction-related neural responses
in the studied melodies. Loudness research also found that sufficiently long
gaps can reset temporal weighting toward the following sound. These adjacent
constructs do not prove that a particular dropout makes a Riotbox return hit
harder. **Directness E2; replication `multiple_related_studies`.** A stop must
be described by metric location,
duration in beats and time, depth, muted owners, and explicit return owner.
Milliseconds reported by one experiment are not a universal drop threshold.

A live electronic-concert study found more participant movement when
very-low-frequency energy was present even though a follow-up suggested it was
not consciously detected. **Directness E1 for movement in that concert and E2
for Riotbox stage impact; replication `single_study`.** This supports a
separate stage/playback-path question, not a kick recipe. Declare a bass owner
first, measure its absolute as well as relative low-band output, and do not
claim stage pressure from a small-monitor or headphone review alone.

### Limits of the transfer

Most reviewed experiments do not test distorted rave-punk sample chops, a live
TUI instrument, memorability, stage impact, or the exact Riotbox source corpus.
Several use small performer sets, synthesized patterns, fixed tempi, quantized
backings, or self-report. Natural commercial recordings also confound source,
performance, production, familiarity, and listener preference. No reviewed
study independently manipulates every relevant role, timbre, timing,
articulation, dynamic, tempo, and arrangement factor.

Therefore the literature establishes dimensions, causal candidates, and known
confounds. It does not award Riotbox character or a product recipe.

## Beat Analysis Rubric

The rubric is diagnostic. It produces a profile and an evidence trace, never a
single taste score. Before listening or measuring, write the intended sentence,
for example: `the snare should feel more forcefully struck`, `the loop should
drive harder while retaining its hook`, or `the return should feel heavier
because the prior bar creates space`.

### 1. Intent and ownership

Record the intended construct, time scale, source-backed owner, supporting
roles, and expected absence. If bass is `unassigned`, absent bass pressure is
not a failure. If the claim is `arrangement_impact`, an unchanged isolated hit
is not a failure. An ambiguous target stops the review before playback.

### 2. Pulse, meter, and phrase frame

Establish BPM/tempo confidence, candidate meter, beat and subdivision anchors,
bar/phrase boundaries, and any ambiguity. Report whether the pulse is carried
by kick, snare/backbeat, hat/ride, bass, hook, or a composite. A transform must
not silently change the reference frame and then claim improved groove.

### 3. Event and role inventory

For each perceptually meaningful event, record onset, role, metric position,
accent class, articulation class, event duration, and audible contributors.
Distinguish a single composite hit from a flam, echo, ghost note, pickup, or
second role. Do not infer instrumental identity more confidently than the
source separation supports.

### 4. Event anatomy

Analyze attack, body, and tail using adaptive boundaries plus fixed diagnostic
views. Track temporal and spectral evolution rather than only full-file
statistics. A hard-strike claim needs retained identity and body; a soft-strike
claim needs a coherent gentler articulation, not just attenuation or blur.

### 5. Accent and dynamic hierarchy

Measure within-role and cross-role level/energy ranks, not only averages.
Identify primary accents, secondary accents, ghost/support strokes, and phrase
peaks. Constant velocity is a distinct condition, not a neutral baseline. A
hierarchy must be musically interpretable and must survive the actual mix.

### 6. Timing relations

Report deviations relative to the inferred metric grid *and* to named role
anchors. Look for repeated relational patterns across bars. Separate deliberate
push/pull, swing subdivision, and role lead/lag from drift and independent
jitter. Never add a universal millisecond offset based on a genre label.

### 7. Pattern topology and expectation

Report event density by role and metrical level, syncopation location and role,
pickups, omissions, counter-rhythmic figures, and the balance between stable
anchors and violations. A syncopation count is not enough. Ask whether the
listener can maintain a prediction that the deviations meaningfully challenge.

### 8. Interlock, masking, and physical space

Examine composite attacks, kick/bass/hook reinforcement, spectral and temporal
masking, tail overlap, and headroom. A role may be strong in solo and weak in
context, or deliberately modest in solo while making a powerful composite.
Evaluate both views when the product claim concerns the ensemble.

### 9. Repetition and development

Across at least the promised listening window, mark invariants, bounded
variations, fills, stops, drops, returns, and changed role ownership. Identify
whether change clarifies a phrase or merely adds events. A loopability verdict
asks whether the musician would voluntarily sustain, retrigger, or mutate the
material—not whether an automated eight-bar script contains activity.

### 10. Mix, playback, and human use

Record raw and level-controlled evidence, playback chain, audibility, clipping,
and endpoint silence. Mechanical analysis may reject timing collapse,
near-identity, artifacts, masking, or a mismatched claim. Only a bounded human
task can decide force, groove, desirability, memorability, and live usefulness.

### Failure vocabulary

Use concrete diagnoses rather than another float change:

| Failure | Meaning |
| --- | --- |
| `intent_ambiguous` | the listener was not told which role and construct to judge |
| `role_unassigned` | the requested property has no product owner; do not fake one |
| `near_identity` | the relevant musical dimension did not change enough to review |
| `level_or_pitch_confound` | the apparent difference is dominated by gain or global pitch/rate |
| `transient_without_body` | a click or sharp edge replaced physical event body |
| `body_smear` | sustain or low-mid energy masks the onset or adjacent events |
| `dynamic_flatness` | accents and support strokes lack an interpretable hierarchy |
| `randomized_timing` | deviations lack a stable role-, meter-, or phrase-relative pattern |
| `anchor_loss` | syncopation, deletion, or mutation makes the pulse/metric model unstable |
| `density_without_hierarchy` | additional events compete without a clear anchor or counter-role |
| `source_fingerprint_erased` | recognizable source timing/timbre/phrase relations no longer guide output |
| `contrast_absent` | a supposedly hard/heavy moment has no preparation, space, or return contrast |
| `scripted_medley` | forced rapid section changes replace musician-controlled component choice |
| `exact_repeat_fatigue` | repetition establishes no further reward after the hook is learned |
| `mix_masked` | the intended role exists in solo evidence but not in the declared product mix |

Each diagnosis must point to the failed layer and a future causal question. It
must not immediately prescribe a constant.

## What Prior Riotbox Work Actually Proved

The H27–H31 listening chronology is negative product-owner evidence, not an
accepted algorithm family. The synthesis below preserves the recorded failure
classes. RIOTBOX-1422 now preserves stable hash-bound H27–H30 manifests and a
bounded closeout. H27 and H30 carry structured-review records; H28 and H29
carry artifact-bound human observations but no standard review pack. Their
audio remains local, so none is an artifact-complete executable backtest. The
H31 narrative and local review/WAV hashes are likewise committed only as a
hash-bound historical observation, not an executable control.

The H27–H30 evidence boundary is recorded in
`docs/reviews/riotbox_1422_h27_h30_rejected_experiment_closeout_2026-08-02.md`.

| Failure family | What changed | Human or decisive outcome | Durable lesson |
| --- | --- | --- | --- |
| choke/gate emphasis | more silence and a sharp first few milliseconds | choppier and emptier, not harder | shortening is not force |
| low-band reinforcement | source-relative low attack energy | bass/kick still not harder or source had no bass owner | assign role before judging low-end pressure |
| grit/damage | roughness, distortion, and waveform delta | dirtier or more destroyed, not harder | aggression is orthogonal |
| repeated parallel overlays | local attack/body metrics | repeated variants sounded the same or merely duller | quiet residual ratios can hide an unchanged dominant object |
| H31 v1 local takeover | `~0.78x` processed body with technical gate pass | lower-pitched second half, no increase in force | global rate/pitch stability is a semantic invariant |
| H31 v2 pitch-stable reconstruction | attack ratios `1.144–1.243`, body ratios `0.810–0.880`, correlations `0.971–0.984` | mechanically rejected before playback | near-identity and body loss do not merit another listening request |

The committed H31 summary lives in
`docs/reviews/riotbox_1428_h31_stage_a_rejected_experiment_2026-08-02.md`.
Its numeric values are local diagnostics, not calibrated perceptual thresholds,
and its local hashes establish identity rather than future availability.

## What RIOTBOX-1429 Can and Cannot Conclude

This research can freeze:

- the multi-scale beat model and typed vocabulary;
- known confounds and failure vocabulary;
- the requirement for role-, meter-, source-, and phrase-aware analysis;
- the need for multi-source causal experiments and human musical verdicts;
- a set of falsifiable algorithm and validation hypotheses for RIOTBOX-1428
  Stage A.

It cannot freeze:

- a universal hardness, groove, punch, or loopability score;
- DSP constants copied from papers, commercial tools, or one source;
- a genre-wide timing offset, swing ratio, syncopation amount, EQ curve, or
  transient target;
- a winning renderer topology;
- a claim that the new legal source inventory contains suitable events before
  mechanism-blind qualification;
- a product or live-instrument improvement.

Numbers such as study window lengths, tested timing displacements, correlation
coefficients, and historical Riotbox ratios are evidence metadata. They are not
product parameters. Any later operating range must be derived from declared
source evidence, bounded by failure controls, held constant across contrasting
development sources, and tested on untouched holdouts.

The future matrix has three numeric freeze classes:

1. RIOTBOX-1429 freezes corpus quotas, event ordinal partitions, stop rules,
   holdout denial, and the two-generation listener-fatigue rule as experiment-
   design invariants. They are anti-overfit choices, not scientific thresholds,
   and need a new versioned research decision before Stage-A qualification to
   change.
2. RIOTBOX-1428 Stage A must define and passport the source-distance formula,
   missing/zero semantics, cluster thresholds, deterministic negative-control
   renderers, and mechanical falsifiers before their declared qualification or
   candidate boundary.
3. Source-adaptive DSP values may be resolved only from registered development
   evidence. They freeze before Golden Path human review and may never change
   after that review or after holdout access. No Stage-B product constants are
   selected here.

## Future Validation Corpus Design

This section is a preregistration draft for RIOTBOX-1428 Stage A.
RIOTBOX-1429 may inventory legal evidence and freeze the design, but it does
not render algorithm candidates or execute the listening gate.

### Partitions

RIOTBOX-1428 Stage A must use four distinct evidence roles:

1. **Natural dynamic/velocity reference controls:** same-instrument,
   same-articulation recordings at labeled performance dynamics. These are
   local directional sanity checks, not measured physical-force ground truth.
2. **Positive development sources:** legal source loops/events from at least
   four packs spanning at least three registered source families plus a
   mechanism-blind acoustic-contrast gate. These may select algorithms and
   constants.
3. **Refusal and stress cases:** dense full mix, tonal, pad/noise, weak-level,
   and bad-timing material. Correct output may be `unavailable`, `degraded`, or
   `reject`; these cases never earn positive percussive coverage by force.
4. **Holdouts:** no research or development process may read, hash, render,
   classify, or listen to active `holdout_a` or `holdout_b` audio. Only
   RIOTBOX-1428 Stage B may use them, after event/algorithm/threshold/package
   hashes are frozen, every positive development source passes mechanically,
   and the isolated mechanism earns a blinded human directional pass.

Before admission, RIOTBOX-1428 Stage A has one narrow lane for the two explicitly
named local candidates. It first rejects their declared IDs, paths, and hashes
against the active holdout union without reading holdout audio, then permits
raw-source-only human suitability review. It may not qualify events, transform
audio, or discover a directory in this lane.

After a suitability pass, Stage A must version the source-registry schema and
validator, admit development-only candidates, prove the active holdout
IDs/paths/hashes are unchanged, and write a new matrix snapshot pinned to the
new schema and manifest hash. Only then may the runner accept those entries for
qualification. Any other manifest-hash change fails closed. The runner also
dynamically rejects the active holdout union and the explicit denylist and may
not glob the external source directory. RIOTBOX-1429 does not consume holdout
audio or run holdout-local-file discovery.

The canonical preregistration is
`docs/benchmarks/percussive_force_development_matrix_v1.json`. It records the
proposed source identities, hashes, roles, quotas, controls, cross-product, and
holdout snapshot. Event eligibility, exact event frames, feature distances, and
mechanical thresholds remain explicitly pending for RIOTBOX-1428 Stage A. The
current manifest hash is a research snapshot, not an execution freeze.

### Natural directional reference controls

The Philharmonia Orchestra percussion library provides natural recordings of
the same instrument and articulation at labeled dynamics. RIOTBOX-1429 inventories
the `mezzo-forte`, `forte`, and `fortissimo` snare-with-snares and whip pairs
as local reference controls. The library permits use in musical work but
forbids redistribution as samples or a sampler instrument. Therefore:

- the archive and decoded WAVs remain local, ignored, and uncommitted;
- they are not Riotbox product sources, fixtures, generated assets, or demo
  material;
- filename dynamics remain ordinal performance labels; a future blinded
  product-owner check may confirm a local perceived ordering but cannot
  establish measured striking force or a general perceptual calibration;
- license, upstream URL, decode process, and hashes are recorded in the local
  experiment manifest.

These six files provide local directional sanity checks, not threshold-setting
ground truth. The snare is the body-bearing reference; the whip is an
impulsive attack-edge reference and cannot establish retained drum body. A
larger repeated-take/listener corpus or an additional body-rich kick/tom set is
required before claiming scientific perceptual calibration.

### Positive development quota

Before any future algorithm comparison, mechanism-blind qualification must
find at least four legal source packs from four authors, spanning dense break,
sparse drums, and electronic drums, with at least two confident events per
pack. A frozen source-feature contrast gate must also yield at least three
distinct clusters; metadata labels alone do not satisfy contrast. Freeze no
more than three events per source:

- events 1 and 2 may inform development;
- event 3, if present, remains untouched until the family and thresholds are
  frozen in RIOTBOX-1428 Stage A; it is not claimed historically fresh;
- event boundaries are selected before any candidate is rendered.

The proposed matrix starts with registered Cinameng dense-break and Marwan
sparse-percussion development sources. The local William Hector war-drum and
frosty ham electronic-drum files are only source-admission candidates: they do
not count until RIOTBOX-1428 Stage A records a human source-family suitability
verdict, versions the source-registry schema/validator to support the
`electronic_drums` core family, admits the accepted development sources, proves
the active holdout union unchanged, and repins the matrix before qualification.
None claims holdout freshness. Beat03 remains a historical taste coordinate and
negative-evidence source only; its local-use license boundary excludes it from
algorithm selection and promotion evidence. The future Stage-A Golden Path is
the mechanism-blindly qualified Cinameng dense-break event ordinal 1. If that
event or the four-source, two-event, or three-cluster qualifier fails, stop and
version a revised preregistration after expanding the legal development corpus;
never choose a post-result fallback or borrow a holdout.

### Mechanism-blind event record

Each frozen event records:

- case ID, source family, source path, source hash, license, and partition;
- exact start/end frames at original sample rate;
- physical-onset frame and detector version;
- `rhythmic_location_proxy_v1` frame, method, and confidence, explicitly not
  PAT or P-centre ground truth;
- attack, body, and tail boundaries with the evidence used to choose them;
- event role and eligibility or typed refusal reason;
- source peak, RMS, DC, channel balance, and clipping state;
- whether the event may count toward positive coverage.

No filename branch or per-source processing constant is allowed after this
record is frozen.

## Proposed Future Measurement Contract

The measurements below are falsification and diagnosis requirements for
RIOTBOX-1428 Stage A. They are not evidence that RIOTBOX-1429 implemented an
analyzer or calibrated a perceptual score.

### Event anatomy

Fixed windows such as `0–10 ms` and `40–120 ms` are useful historical views,
not universal event anatomy. The future harness should derive boundaries from a smoothed
wideband and multiband envelope:

1. **look-behind:** enough pre-onset material to prove no precursor or edit
   discontinuity;
2. **attack:** physical onset through an independently detected envelope
   turnover/decay boundary around the local peak; compute ATC only after this
   interval is frozen;
3. **body:** the post-attack region while source energy remains materially
   above its tail/noise estimate;
4. **tail:** the remaining bounded decay used to detect truncation, ringing,
   or a second event.

For comparability, the manifest also reports fixed diagnostic windows at
`0–10`, `10–40`, `40–120`, and `120–250 ms`, clipped to the event. Decisions
must say whether a value uses adaptive or fixed windows. These proposed widths
are diagnostic protocol values, not universal event anatomy or product
thresholds; RIOTBOX-1428 Stage A must give them numeric passports and freeze
their version before event qualification.

### Mechanical invariants and safety screens

Every candidate must pass:

- finite samples, deterministic hash, declared sample rate and channel count;
- no clipping and declared sample peak; true peak is reported only when the
  versioned oversampled/BS.1770-capable analyzer actually measures it;
- no unauthorized limiter in loudness matching;
- one detected onset; no detected pre-echo, delayed duplicate, flam, or
  boundary discontinuity;
- no global resampling or transposition; product playback rate remains `1.0x`;
- physical-onset and `rhythmic_location_proxy_v1` movement inside the frozen
  mechanical tolerance;
- aligned identity proxies above their frozen screens;
- dry/wet polarity, parallel delay, stereo correlation, and mono compatibility;
- retained body and controlled tail rather than a winning attack click.

The human layer separately determines whether there is one audible event,
whether source identity remains recognizable, and whether timing still feels
correct. Mechanical proxies may reject a candidate; they cannot award those
perceptual claims.

Pitch stability means no global algorithmic pitch/rate shortcut. A local
source-consistent spectral or resonant change is not an automatic reject, but
it remains an unvalidated Stage-A cue and cannot itself prove force.

### Diagnostic feature groups

The harness reports raw values and candidate/source ratios; it does not collapse
them into an unvalidated universal score.

| Feature group | Required views | Interpretation limit |
| --- | --- | --- |
| time/envelope | attack temporal centroid, rise, peak time, body and tail energy, attack/body relation | cannot alone prove force |
| transient loudness | event-local, time-varying band contributions and aggregate proxy | not PM95 unless PM95 is independently reproduced and validated |
| attack spectrum | centroid, bandwidth, midband level, per-band flux over the first `100 ms` | clicks and sibilance can win falsely |
| source identity | aligned correlation, spectral-envelope similarity, residual energy | similarity can hide no audible change |
| timing | physical onset, `rhythmic_location_proxy_v1`, secondary-onset count, train interval stability | the proxy is neither PAT nor human P-centre ground truth |
| body/tail | adaptive and fixed-window retention, decay centroid, truncation/ring checks | extra sustain can mask attack |
| nonlinear residual | harmonic distribution, intermodulation and aliasing proxies, oversampling/filter version | aggression is not force |
| low end | absolute and relative low-band attack/body energy, only for assigned owner | no bass owner means no bass-pressure failure |
| whole artifact | BS.1770 integrated loudness and true peak, RMS, peak, DC, silence | safety and level context only |

Integrated BS.1770 loudness is meaningful for the complete review artifact,
but a `20–120 ms` hit is too short for it to become a transient-force metric.
The proposed `event_rms_attenuation_match_draft` computes channel-aware RMS
across the complete frozen event or registered repeated-event train, attenuates
only the louder side by `min(rms_a, rms_b) / rms_louder`, then applies equal
attenuation to both sides if either lacks sample-peak headroom. It uses no
limiter, amplification, frequency weighting, or hidden window. RIOTBOX-1428
Stage A must passport and freeze the exact scope before use. This remains a
level-control view, not transient loudness or PM95. Both raw and matched
comparisons are proposed. Until a perceptually calibrated transient matcher
exists, a raw pass plus matched failure is
`matching_method_inconclusive_or_level_dependent`, not automatically
`merely_louder`.

### False-positive controls

Run the following controls on one frozen event from each positive development
source. No control may earn `percussive_hard`:

- hidden exact A/A;
- gain only;
- one Stage-A-frozen global rate or pitch change;
- darkness/low-pass only;
- brightness-only;
- distortion only;
- a registered delayed duplicate/flam control whose audible delay is frozen by
  RIOTBOX-1428 Stage A;
- a detached early click plus collapsed body.

The committed H31 report is a historical observation that motivates the
global-rate/pitch confound control; it is not an executable backtest and its
approximately `0.78x` body operation must not be mislabeled as a whole-event
recipe. The landed H27–H30 manifests are registered only as hash-bound
historical observations: H28/H29 have no standard review pack, and none has the
complete committed audio needed for executable replay. A dirty worktree or
chat chronology is not a backtest input. The matrix intentionally leaves new
control parameters unresolved until RIOTBOX-1428 Stage A gives each value a
numeric passport and freezes the renderer before candidate generation.

A proposed metric or threshold is invalid unless it rejects every frozen false
control before it sees a new Golden Path candidate. A later matrix may also
require a stable historical failure only after a complete, landed artifact
bundle registers it as executable. The current draft requires no historical
artifact replay.

## Future Algorithm Hypotheses

These families are falsifiable implementation hypotheses for RIOTBOX-1428
Stage A, not selected designs. RIOTBOX-1429 changes no renderer and claims no
surviving mechanism. RIOTBOX-1428 may reject all of them and must not preserve
a family merely because it is documented here.

The first future comparison should use at least three structurally distinct,
deterministic, source-general families. A source filename may never select
topology or constants.

### F1: source-synchronous attack/body redistribution

Derive fast and slow source envelopes, estimate attack/body/tail boundaries,
and redistribute energy inside the one event. Preserve the aligned dry event
and construct a force candidate by strengthening the source attack while
retaining or deliberately shaping body.

Expected causal change: earlier/stronger event-local attack evidence with
retained body and no new spectrum unrelated to the source.

Primary failure: a thin click, body loss, or near-identity after gain matching.

### F2: phase-coherent complementary multiband shaping

Split the event with complementary, reconstructing bands. Apply
source-relative attack/body treatment only where the source contains trusted
event energy, then reconstruct a sample-aligned event.

Expected causal change: a broader or more present attack and controlled body
without low-pass masking or comb filtering.

Primary failure: hollow phase cancellation, detached high-band click, or one
fixed band signature across sources.

### F3: onset-conditioned parallel dynamics/nonlinear residual

Create a source-derived, onset-local residual through compression and/or
bounded nonlinear excitation. Retain an aligned dry body, compensate output,
and keep roughness/aggression metrics separate.

Expected causal change: denser source-related attack/body microdynamics and
midrange bite without turning the whole event into damage.

Primary failure: merely dirtier, louder, clipped, aliased, or masked body.

### Optional F4: velocity-cue transfer

After the natural reference set passes its limited directional sanity check,
estimate transformations of attack, decay, local resonance, and brightness
from matched lower/higher-dynamic pairs and transfer only source-relative
directional cues. This is a research
hypothesis, not a learned product model, until the corpus is large enough and
cross-source validation passes.

Expected causal change: several natural dynamic/velocity cues move coherently
rather than one hand-tuned float.

Primary failure: copying one instrument's spectral fingerprint onto unrelated
sources or inferring force labels from filenames alone.

### Beat-level hypothesis families

The broader research suggests additional families for later, independently
scoped tickets. None may be smuggled into the isolated force experiment:

- **H-PATTERN-1 — anchor and counter-rhythm:** preserve an explicit
  timekeeper/backbeat anchor and place source-supported syncopation or pickups
  in named counter-roles. Compare with the same event count and aggregate
  syncopation distributed pseudorandomly.
- **H-CHOP-1 — articulation-preserving reuse:** select among source-derived
  event variants by role and source-relative accent rank instead of replaying
  one canonical hit. Compare on the same onset grid.
- **H-TIMING-1 — explicit timing modes:** compare `raw_source`,
  `warp_preserved`, `requantized`, and independent random-jitter controls while
  keeping pattern and timbre fixed. No mode is assumed to win universally.
- **H-LAYER-1 — collision-local space:** when a named owner is masked at a
  typed attack collision, alter only the competing event in a short,
  source-relative region. Compare with global EQ and global gain changes.
- **H-STOP-1 — typed stop and return:** compare full silence, bounded
  attenuation, and a micro-dropout at a named phrase boundary with a declared
  return owner. Duration remains beat-, BPM-, source-, and context-dependent.
- **H-STAGE-1 — playback-path survival:** recheck an accepted bass-owned loop
  on a declared full-range path and preserve any disagreement with the
  headphone/nearfield verdict rather than hiding it in `pressure`.

Contextual masking reduction is a mix and arrangement mechanism. It cannot win
the isolated percussive-force claim. Likewise, pattern, stop, and stage
hypotheses require their own claims and tests.

## Proposed Future Experiment Protocol

This protocol is a handoff, not work executed by RIOTBOX-1429.

### Pre-registration

Before rendering candidates, freeze:

- corpus manifest, source hashes, partitions, event frames, and eligibility;
- algorithm versions and one shared typed mapping per family from
  `versioned source features -> bounded policy values -> renderer inputs`,
  including bounds, refusal/default behavior, and every actual resolved value;
- declared feature directions for each family;
- matching method and tolerances;
- safety, invariant, false-positive, near-identity, and collapse screens;
- blinded ordering seed and artifact duration;
- maximum two review-ready generations per mechanism family;
- rule that one family failure cannot be repaired by scalar retuning unless a
  new causal mechanism is named and pre-registered.

Natural references supply limited local directional sanity checks. Historical
negative evidence constrains failure labels and stopping rules; it is not a
numeric threshold-fitting corpus. Every Stage-A threshold then freezes before
Golden Path output. Six reference files do not justify universal perceptual
thresholds. Aggregate averages may not hide a failed source family.

### Ordered family screen and factorial diagnosis

The order is fixed: mechanism-blind catalog and reference/control sanity check
-> mechanical F1–F3 cross-product -> preliminary blinded family comparison ->
identical ablation for every survivor -> cross-source directional confirmation
-> optional blinded product preference among fully passing survivors -> product
handoff. No later stage may change earlier event selection or thresholds.

For each family, isolate one causal question in its first render. A preliminary
family survives only if its raw and level-control Golden Path comparisons both
receive `clearly different`, `more forcefully struck`, recognizable identity,
retained-body, and retained-bite verdicts without a false-control explanation.
Every family satisfying that rule proceeds; zero survivors stop Stage A.
Raw/level-control disagreement is inconclusive, never a tie-break. No
mechanical aggregate may select a musical winner.

Every preliminary human survivor receives the same ablation:

1. source control;
2. attack operation only;
3. body operation only;
4. reconstructed attack plus body.

Global transposition, global playback rate, event onset, train timing, and
nominal matched level remain fixed. This identifies whether attack, body, their
interaction, or level caused the verdict. `percussive_soft` requires its own
versioned hypothesis, feature direction, controls, falsifiers, and experiment;
it is not an ablation and must not be invented as the mathematical inverse of a
surviving hard operator.

Every ablation survivor then runs unchanged on the complete qualified positive
matrix. If more than one mechanism independently earns the full directional
pass, present those mechanisms in a final blinded preference comparison on the
same preregistered Golden Path event and ask which the musician would rather
trigger as `Hard`. A unique preference selects the Stage-B candidate. A tie
preserves all scientific/product evidence but keeps Stage B blocked; it does
not authorize retuning or a post-result metric discriminator.

### Artifact set

For every F1–F3 family and development event, produce only the family-screen
artifacts:

- isolated native source and candidate events;
- registered repeated-event raw trains at identical intervals;
- `event_rms_attenuation_match_draft` level-control trains;
- source-to-candidate and candidate-to-source A/B artifacts;
- one manifest binding exact files, hashes, provenance, metrics, contributors,
  and intended role.

Every preliminary survivor receives attack-only, body-only, and combined-hard
artifacts. Cross-source confirmation then renders each frozen ablation survivor
for every qualified positive pack. If a pack has a mechanism-blindly frozen
event ordinal 3, that untouched event becomes a mandatory post-freeze
confirmation case; absence grants no within-source-confirmation claim but does
not fail the two-event minimum.

Derive the declared match from the complete registered train using the frozen
Stage-A equation. If sample-peak headroom is insufficient, attenuate both sides
equally after matching; do not limit one side. Preserve raw artifacts because a
forceful candidate that is unusably quiet remains a product failure.

### Mechanical promotion gate

A family becomes `review_ready` only when:

- all invariants and safety checks pass on every positive source;
- natural-reference checks do not end as `reference_order_inconclusive`, and
  false controls do not earn force;
- at least two events from every qualified positive source pack survive;
- each survivor is measurably non-identical without body collapse;
- the intended attack/body feature group changes in its registered direction;
- refusal and stress cases emit their declared typed outcome without fallback;
- the Stage-A access log records the required canonical holdout-manifest
  metadata comparison while proving that no active holdout audio file was
  opened, read, hashed, rendered, classified, or played and no holdout directory
  discovery occurred.

This gate grants a listening request, not a musical pass.

## Proposed Future Human Listening Gate

This is an **N=1 product-owner directional gate**, not scientific perceptual
validation. It uses an ISO-5495-inspired randomized paired comparison because
the product question is directional: which event sounds more forcefully
struck? The procedure does not claim ISO conformance or population validity.
Listening device, gain setting, environment, date, and blinded-order seed are
recorded. Scientific calibration would additionally require multiple
assessors, repeated randomized trials, controlled level/equipment, and a
predeclared statistical analysis.

### Review questions

For every pair, record separately:

- which event sounds more forcefully struck;
- clearly different: yes/no;
- same recognizable source identity: yes/no;
- one audible event with no flam, echo, or detached duplicate: yes/no;
- same rhythmic place and perceived timing: yes/no;
- physical body retained: yes/no;
- source-related bite retained: yes/no;
- more punchy, brighter, darker, dirtier, louder, lower, doubled, delayed, or
  fatiguing: independent tags;
- desirable to trigger: yes/no;
- confidence: low/medium/high.

`Different but not harder`, `only louder`, and `harder only because lower` are
rejects.

Confidence is anchored to the directional task: `low` means uncertain,
internally conflicting, or dependent on guessing; `medium` means a stable
choice with a describable force cue and no dominant confound; `high` means the
direction is immediate and unmistakable while identity/body/bite remain clear.
Confidence does not replace the separate property fields.

### Bounded schedule

1. **Natural-reference sanity:** present randomized A/B and reversed B/A views
   of the blinded `mezzo_forte` and `fortissimo` endpoints for both the snare
   and whip sets. The two orders must agree on one perceived direction;
   intermediate comparisons are optional diagnostics. The filename labels are
   not ground truth: disagreement or inability to choose records
   `reference_order_inconclusive`, pauses the force task, and requires the
   reference set, listening setup, and question to be replaced or rechecked.
   Never tune an algorithm or force the label order to make this check pass.
2. **False-control sanity:** independently present hidden A/A and the frozen
   gain, rate/pitch, dark, bright, distortion, delayed-duplicate, detached-click,
   and stable historical controls needed to verify that the review question
   rejects known confounds. Natural-reference and false-control judgments are
   separate blocks; neither selects an algorithm family.
3. **Golden Path family screen:** present raw and the exact Stage-A-frozen
   event-RMS attenuation-match views for every mechanically review-ready F1–F3
   family on the same preregistered Golden Path event. Every family that passes
   independently proceeds; zero survivors stop Stage A.
4. **Survivor ablation:** every preliminary survivor receives the identical
   source, attack-only, body-only, and combined-hard comparisons, including a
   hidden reversed repeat. Freeze or reject each family whose causal
   contribution remains ambiguous.
5. **Cross-source confirmation:** run every frozen ablation survivor on every
   qualified positive source pack and registered development event, plus every
   available frozen ordinal-3 confirmation event. Include one reversed-order
   repeat per source and hidden A/A controls. A failure on one source cannot be
   averaged away by the others.
6. **Final product preference:** if multiple mechanisms independently pass,
   compare them blind on the same Golden Path event using the preregistered
   trigger-preference question. A unique choice selects the Stage-B candidate;
   a tie blocks Stage B without invalidating either result or authorizing
   retuning.

Loop desirability is deliberately absent from this isolated Stage-A gate.
RIOTBOX-1428 Stage B must test the frozen selected mechanism in the exact
musician-controlled RuntimeMix/TUI loop; that later live-path verdict cannot be
replaced by an offline repeated-event train.

Isolated comparisons are at most `5 s`; use `2–3 s` when enough. State the
exact artifact and expected property, show the conspicuous readiness cue, and
wait for fresh confirmation before every playback. Split blocks before the
listener loses the preceding comparison; stop immediately on fatigue. Repeated
`same` ends that comparison rather than triggering another scalar revision or
another nominally new render.

For hidden A/A, the expected answer is `clearly different: no`; the forced
direction choice is ignored because it is necessarily arbitrary. Low
confidence or `inconclusive` does not count toward promotion. A conflict
between a primary pair and its reversed repeat is a failed repeat, not a value
to average away.

### Musical pass

The isolated mechanism passes the N=1 product gate only when all of the
following are true:

- every qualified positive source pack has at least one registered event whose
  raw and matched comparisons select the candidate as more forcefully struck;
- no qualified source has all of its registered development events fail;
- every available frozen ordinal-3 confirmation event agrees with the
  mechanism's direction; absence grants no confirmation claim;
- every reversed-order repeat preserves its primary directional judgment;
- every passing event retains recognizable source identity, physical body, and
  bite;
- every passing event remains one audible event without flam/echo and retains
  the same perceived rhythmic place;
- every promotion-bearing judgment has medium or high confidence; low or
  `inconclusive` evidence never counts;
- no passing choice is explained mainly by a false-control tag;
- every hidden A/A is reported `clearly different: no`;
- raw and matched evidence agree on the direction for every promotion-bearing
  event;
- aggregate counts, means, or scores do not rescue a failed source.

A raw-level pass with a matched failure is
`matching_method_inconclusive_or_level_dependent` and cannot promote until the
confound is resolved. This gate is a Riotbox product/taste decision only.

One structural revision is allowed after a human failure. Another failure, two
review-ready generations without a meaningful distinction, or listener
fatigue freezes the family. If no family passes, stop RIOTBOX-1428 before
Stage B as `no mechanism earned product promotion`.

## Staged Product Handoff

RIOTBOX-1429 hands its research paper and preregistration directly to the
already-existing RIOTBOX-1428. Only RIOTBOX-1428 Stage A may implement the
experimental mechanisms, build candidates, and run the validation gate.
RIOTBOX-1428 Stage B remains blocked until Stage A produces an isolated human
pass. The frozen Stage-A package must contain:

- typed role and semantic contract;
- algorithm family/version and complete typed
  `features -> bounded policy -> renderer inputs` mapping with actual values;
- analysis schema/version and actual source/event evidence;
- eligible/refused decision and confidence;
- calibration, false-control, development, and blinded human results;
- bounded prepared-event representation suitable for control-plane work;
- realtime budget assumptions, latency, smoothing, oversampling/filtering, and
  callback-safe state requirements;
- Action Lexicon, queue, commit, Session/replay, observer, and QA consequences;
- exact holdout protocol for 1428, including consumption and rotation rules.

Source analysis, FFTs, onset/event selection, decomposition, and recipe
selection remain outside the callback. The callback may consume only bounded,
typed, render-ready state and perform fixed-cost sample work. Missing,
ineligible, rejected, or stale source evidence produces explicit unavailable
state and silence, never replacement music.

RIOTBOX-1428 must then prove the same accepted mechanism in a sustained
isolated component, a musician-controlled loop, and the exact RuntimeMix/TUI
path. A compact scripted performance arc is not instrument proof.

## Skill Handoff

This research does not justify a new standalone skill. The canonical document
and future versioned experiment artifacts own the detailed evidence. After the
future validation experiment closes:

- mirror only validated musician-facing force vocabulary, false-positive
  controls, and listening stop rules into
  `.codex/skills/riotbox-rave-punk-production/SKILL.md`;
- mirror only validated engineering, evidence, and anti-retuning rules into
  `.codex/skills/riotbox-development/SKILL.md`;
- keep hypotheses, bibliographies, datasets, and changing thresholds here and
  in versioned manifests;
- re-read both skills after editing and mirror durable repo-facing rules into
  `AGENTS.md` or `docs/specs/audio_qa_workflow_spec.md`;
- do not create a home-directory copy or shadow skill.

A dedicated skill becomes appropriate only if later work establishes an
independent, recurring workflow with stable inputs, outputs, and validation
that is not already owned by these two skills.

## Primary Sources

### Event acoustics, force, punch, and timing

- Pearce, Brookes, and Mason, [Modelling Timbral
  Hardness](https://doi.org/10.3390/app9030466), 2019.
- Freed, [Auditory correlates of perceived mallet
  hardness](https://doi.org/10.1121/1.399298), 1990.
- Fenton and Lee, [A Perceptual Model of Punch Based on Weighted Transient
  Loudness](https://doi.org/10.17743/jaes.2019.0017), 2019.
- Cheshire, Stables, and Hockman, [Investigating timbral differences of varied
  velocity snare drum strikes](https://www.open-access.bcu.ac.uk/13026/), 2020.
- Lutfi, Liu, and Stoelinga, [Auditory discrimination of force of
  impact](https://doi.org/10.1121/1.3543969), 2011.
- Gordon, [The perceptual attack time of musical
  tones](https://doi.org/10.1121/1.395441), 1987.
- Bechtold and Senn, [Articulation and Dynamics Influence the Perceptual Attack
  Time of Saxophone Sounds](https://doi.org/10.3389/fpsyg.2018.01692), 2018.
- Danielsen et al., [Where is the beat in that note? Effects of attack,
  duration, and frequency on perceived
  timing](https://doi.org/10.1037/xhp0000611), 2019.
- Kazazis, Depalle, and McAdams, [Attack temporal centroid in timbre
  spaces](https://doi.org/10.1121/10.0006788), 2021.
- Lakatos, [A common perceptual space for harmonic and percussive
  timbres](https://doi.org/10.3758/BF03212144), 2000.
- Wang et al., [The impact of audio effects processing on the perception of
  hardness of bass drum](https://doi.org/10.1049/ccs2.12060), 2022.
- Moore, [Dynamic Range Compression and the Semantic Descriptor
  Aggressive](https://doi.org/10.3390/app10072350), 2020.
- Kahles, Esqueda, and Valimaki, [Oversampling for Nonlinear Waveshaping:
  Choosing the Right Filters](https://doi.org/10.17743/jaes.2019.0012), 2019.
- Danielsen et al., [Effects of instructed timing and tempo on snare drum sound
  in drum kit performance](https://doi.org/10.1121/1.4930950), 2015.
- Parker and Fenton, [Musical Mix Clarity Prediction Using Decomposition and
  Perceptual Masking Thresholds](https://doi.org/10.3390/app11209578), 2021.
- Weidman, Sweeney, and Bulla, [The Perceptual Impact of Automatic Drum
  Microphone Time Alignment and Polarity
  Correction](https://secure.aes.org/forum/pubs/conventions/?elib=22248),
  2023.
- Skrodzka, Hojan, and Proksza, [Vibroacoustic investigation of a batter head
  of a snare drum](https://acoustics.ippt.pan.pl/index.php/aa/article/view/674),
  2006.
- Bilbao, [Time domain simulation and sound synthesis for the snare
  drum](https://doi.org/10.1121/1.3651240), 2012.
- Worland, [Normal modes of a musical drumhead under non-uniform
  tension](https://doi.org/10.1121/1.3268605), 2010.

### Performance, meter, groove, and genre

- Dahl, [The playing of an accent: Preliminary observations from temporal and
  kinematic analysis of
  percussionists](https://kth.diva-portal.org/smash/record.jsf?pid=diva2%3A11292),
  2000.
- Friberg and Sundstrom, [Swing Ratios and Ensemble Timing in Jazz
  Performance](https://doi.org/10.1525/mp.2002.19.3.333), 2002.
- Fruhauf, Kopiez, and Platz, [Music on the timing
  grid](https://doi.org/10.1177/1029864913486793), 2013.
- Witek et al., [Syncopation, Body-Movement and Pleasure in Groove
  Music](https://doi.org/10.1371/journal.pone.0094446), 2014.
- Senn et al., [The Effect of Expert Performance Microtiming on Listeners'
  Experience of Groove in Swing or Funk
  Music](https://doi.org/10.3389/fpsyg.2016.01487), 2016.
- Witek et al., [Syncopation affects free body-movement in musical
  groove](https://doi.org/10.1007/s00221-016-4855-6), 2017.
- Senn et al., [Groove in drum patterns as a function of both rhythmic
  properties and listeners' attitudes](https://doi.org/10.1371/journal.pone.0199604),
  2018.
- Datseris et al., [Microtiming Deviations and Swing Feel in
  Jazz](https://doi.org/10.1038/s41598-019-55981-3), 2019.
- Nelias et al., [Downbeat delays are a key component of swing in
  jazz](https://doi.org/10.1038/s42005-022-00995-z), 2022.
- Sioros et al., [Syncopation and Groove in Polyphonic Music: Patterns
  Matter](https://doi.org/10.1525/mp.2022.39.5.503), 2022.
- Stupacher et al., [The sweet spot between predictability and
  surprise](https://doi.org/10.3389/fpsyg.2022.906190), 2022.
- Duvel, [Den leisen Schlagen auf der Spur: Ghostnotes und Groove in
  Schlagzeug-Patterns der popularen
  Musik](https://doi.org/10.5771/9783828851474), 2024.
- Ainsworth, [Microtiming in Early Funk: A Microrhythmic Analysis of Fourteen
  Influential Funk Grooves](https://doi.org/10.31751/1224), 2025.
- Herbst and Mynett, [Toward a Systematic Understanding of Heaviness in Metal
  Music Production](https://doi.org/10.1080/19401159.2022.2109358), 2022.
- Herbst and Mynett, [Metal Music and the Aesthetics of Heaviness: Sonic,
  Structural, and Affective
  Perspectives](https://doi.org/10.1080/19401159.2025.2535100), 2025.
- Wesolowski and Hofmann, [There's More to Groove than Bass in Electronic Dance
  Music](https://doi.org/10.1371/journal.pone.0163938), 2016.
- Lustig and Tan, [All about that bass: Audio filters on basslines determine
  groove and liking in electronic dance
  music](https://doi.org/10.1177/0305735619836275), 2019.

### Source manipulation, repetition, silence, and stage context

- Dittmar and Muller, [Reverse Engineering the Amen Break: Score-Informed
  Separation and Restoration Applied to Drum
  Recordings](https://doi.org/10.1109/TASLP.2016.2567645), 2016.
- Livingstone, Palmer, and Schubert, [Emotional response to musical
  repetition](https://doi.org/10.1037/a0023747), 2012.
- Di Liberto, Marion, and Shamma, [The Music of Silence: Part II: Music
  Listening Induces Imagery
  Responses](https://doi.org/10.1523/JNEUROSCI.0184-21.2021), 2021.
- Fischenich et al., [The effect of silent gaps on temporal weights in loudness
  judgments](https://doi.org/10.1016/j.heares.2020.108028), 2020.
- Cameron et al., [Undetectable very-low frequency sound increases dancing at
  a live concert](https://doi.org/10.1016/j.cub.2022.09.035), 2022.

## Replication Assets

- Pearce, Brookes, and Mason, [Timbral Hardness Modelling
  Dataset](https://doi.org/10.5281/zenodo.1548721), 2019.
- Fenton and Lee, [Perceptual Punch Evaluation
  Tool](https://doi.org/10.5281/zenodo.4560084), 2021.

## Standards, Reference Material, and Engineering Documentation

- ITU-R, [BS.1770-5: Algorithms to measure audio programme loudness and
  true-peak audio level](https://www.itu.int/rec/R-REC-BS.1770-5-202311-I/en),
  2023.
- ISO, [ISO 5495:2005 Sensory analysis — paired
  comparison](https://www.iso.org/standard/31621.html), confirmed 2023.
- Philharmonia Orchestra, [Sound samples and licensing
  terms](https://philharmonia.co.uk/resources/sound-samples/).
- Ableton, [Live 12 Audio Effect
  Reference](https://www.ableton.com/en/live-manual/12/live-audio-effect-reference/).
- Native Instruments, [Transient Master
  Manual](https://www.native-instruments.com/ni-tech-manuals/transient-master-manual/en/overview.html).

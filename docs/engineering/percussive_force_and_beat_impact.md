# Percussive Force and Beat Impact

Status: active P023 semantic contract and research router

Owner: RIOTBOX-1429

Validation lineage: RIOTBOX-1428, RIOTBOX-1430

Scope: understanding percussive events, beats, groove, force, softness,
heaviness, and arrangement impact for implementation and validation

## Authority and Document Map

The versioned benchmark JSON contracts and their exact Decision-Log decisions
own executable Stage-A algorithms, thresholds, source partitions, and stop
rules. This explanatory document must not be used to retune a frozen contract.
Any such change requires a new version and decision before recomputation.

Read only the module needed for the task:

- [Research Evidence](./percussive_force/research_evidence.md) owns the evidence
  model, cross-genre findings, primary sources, and replication material.
- [Stage-A Design History](./percussive_force/stage_a_design_history.md)
  preserves the RIOTBOX-1429 experiment design, hypotheses, historical
  measurements, and proposed listening gate. It is not current execution
  authority.

Legacy section links land here and then route to the owning module:

<a id="evidence-model"></a>
<a id="cross-genre-findings"></a>
<a id="primary-sources"></a>
<a id="replication-assets"></a>
<a id="standards-reference-material-and-engineering-documentation"></a>
<a id="what-prior-riotbox-work-actually-proved"></a>
<a id="what-riotbox-1429-can-and-cannot-conclude"></a>
<a id="future-validation-corpus-design"></a>
<a id="proposed-future-measurement-contract"></a>
<a id="future-algorithm-hypotheses"></a>
<a id="proposed-future-experiment-protocol"></a>
<a id="proposed-future-human-listening-gate"></a>

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
candidate. The algorithm families and experiment design in
[Stage-A Design History](./percussive_force/stage_a_design_history.md) are a
handoff for RIOTBOX-1428 Stage A. They are not implemented mechanisms and do
not claim instrument progress.

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
- keep hypotheses, bibliographies, datasets, and changing thresholds in the
  linked research/history modules and versioned manifests;
- re-read both skills after editing and mirror durable repo-facing rules into
  `AGENTS.md` or `docs/specs/audio_qa_workflow_spec.md`;
- do not create a home-directory copy or shadow skill.

A dedicated skill becomes appropriate only if later work establishes an
independent, recurring workflow with stable inputs, outputs, and validation
that is not already owned by these two skills.

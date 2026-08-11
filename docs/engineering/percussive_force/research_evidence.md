# Percussive Force Research Evidence

Status: supporting research evidence for the active semantic contract

Canonical entry point: [Percussive Force and Beat Impact](../percussive_force_and_beat_impact.md)

This module owns the evidence model, cross-genre synthesis, bibliography, and
replication material. It does not own executable Stage-A algorithms,
thresholds, or current frozen contracts.

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

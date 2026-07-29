# RIOTBOX-1422 W-30 Resample Level and Hard-Variation Review

Date: 2026-07-23

Classification: `audible_vertical_slice`

First candidate human verdict: `reject`

Full-phrase base human verdict: `pass` / loopable

Latest heard hard candidate verdict: `reject`

Post-research H25 successor: `technical partial` / `human_verdict: unverified`

## Outcome And Rejected First Candidate

The existing source-backed W-30 resample seam now distinguishes a
source-preserving `base` state from a performer-triggered `hard_damage` state.
No new action system was added. A committed `w30.apply_damage_profile` action
activates the hard variation only when it occurs after the current resample
and targets that resample or one of its lineage captures.

The active resample no longer disappears when the damage action moves W-30 pad
focus back to its lineage source. The app projects the most recently
materialized lineage-ready resample only while focus is the resample or one of
its lineage captures; unrelated later material deactivates it. Capture history
and the committed action log remain Session/replay truth.

The first review-ready implementation still represented only the strongest
`8192` source frames (about `186 ms` of the Beat03 resample input). Direct
source/base/hard listening rejected the candidate:

- source recognition: `source_lost`
- strongest element: `none`
- hook after two bars: `missing`
- base: musically dull
- hard damage: not perceptibly harder
- performer use: neither state felt playable

Technical level and raw-signal delta therefore produced a musical false
positive. The validated verdict is
`/tmp/riotbox-1422-listening-review/verdict/review.json`.

## Base Recovery And Hard-Candidate Evolution

- The recovered base callback payload contains `16384` mono samples and represents the
  complete committed resample artifact as an evenly sampled full-duration
  proxy rather than selecting one transient grain.
- Base playback is a continuous phrase anchor. Grid progress no longer resets
  it before later source material becomes audible.
- Direct listening accepted this base as recognizable, improved, and useful
  enough to loop. Its Beat03 WAV remains byte-stable while hard candidates
  evolve.
- Earlier `hard_damage` attempts based on rate sweeps, holes, random cursor
  order, edge emphasis, and quantization were rejected because they sounded
  slower, incomplete, hollow, higher, or merely louder instead of harder.
- The later clipped-attack candidate removed pitch/rate movement, holes, random ordering, and
  global hard-mode gain. It keeps the base body and crossfades about `40 ms` of
  each beat into a hard-clipped onset retained separately from the same
  committed source.
- Missing or invalid source audio remains typed unavailable and digitally
  silent. No oscillator or replacement music exists.
- That clipped-attack candidate was not review-ready: two different drum holdouts
  collapse to nearly the same envelope, and tonal/pad holdouts cannot traverse
  the exact renderer's current timing-confirmation setup.

## Source-Policy Candidate And Timing Rejection

The next candidate replaced the collapsed fixed articulation with typed
`source_transient_chop` and `source_texture_bite` policies. Its development
matrix and first holdout set were deterministic, source-diverse, non-silent,
and unclipped while the accepted Beat03 Base remained byte-identical.

Direct listening still rejected the exact Beat03 Base-to-Hard gesture. The
Hard addition arrived outside the perceived groove and made the complete loop
unusable; it was not merely weak. Technical follow-up found that the trigger
mask selected source slots while playback began at their coarse boundaries,
and one separately retained global source attack was repeated at every Hard
trigger. Strongest rises in the rejected artifact landed as much as about
`25 ms` after their intended eighth-note boundary.

The timing-corrected follow-up removes that global attack layer. Analysis now
stores one detected local-onset cursor per eight source slots, and the realtime
callback starts the selected source onset on the committed eighth-note grid.
A focused callback regression proves the audible onset lands within one output
frame of that grid. Beat03 Base remains byte-identical at
`2c93fd593983182c7efc35d6a5a9b182c351b996d644bcb07d332459fe878b3e`.
Development and reserve-holdout renders remain deterministic, source-backed,
unclipped, and source-diverse; the maximum absolute pairwise Hard-envelope
correlation in the checked eight-source matrix is `0.330187`.

This is technical readiness only. The timing-corrected Hard gesture remains
`human_verdict: unverified` until a fresh exact-artifact listening comparison
confirms that the late/foreign layer is gone and the resulting variation is
musically useful.

### Base loop timing reject and correction

The first exact listening comparison after the local-onset correction received
a fresh human `reject` before Hard usefulness could be judged. Base contained a
doubled kick like an echo; the second hit disturbed the beat and made the loop
feel out of time.

Technical inspection found two independent causes in the reviewed artifact:

- the review renderer committed a one-bar capture from zero-based source beat
  `5`, leaving only three beats before EOF; the capture helper silently clamped
  the requested end and produced a `1.389365 s` artifact instead of a complete
  bar;
- generation/grit playback-rate modulation made that truncated Base artifact
  wrap after about `3.05` transport beats, so the source kick restarted shortly
  after a regular grid attack. The strongest repeated kick rise in the rejected
  render landed about `47.85 ms` after the bar boundary.

The corrected renderer commits at zero-based source beat `4`, producing the
complete four-beat `1.842109 s` capture. Source-window materialization now
fails closed if the whole requested quantized window does not fit before EOF.
The Base callback derives a bounded whole-beat cycle from capture duration and
transport tempo, removes the free-running generation/grit rate offset, and
uses an `f64` source cursor. A focused callback regression proves a full-bar
artifact restarts within one output frame of the transport bar. In the new
Beat03 render the kick body rises about `5–15 ms` after each bar boundary, with
no separate late loop restart.

Seven development/holdout sources rerender non-silent and unclipped, while
their missing-source controls remain digital silence. The maximum absolute
pairwise Hard-envelope correlation is `0.550674`: below the existing `0.95`
collapse ceiling, although less diverse than the earlier onset-only matrix.
The corrected exact artifact remains `human_verdict: unverified`; automated
alignment does not replace another structured listen for the doubled-kick
failure.

### Grid-safe Base pass and Hard articulation weak verdict

Structured review of the grid-aligned exact artifact confirms that Base now has
a stable beat and remains harmonic. The doubled/echo-kick failure is therefore
closed. The same listen found that the second half was not genuinely Hard.
The structured verdict is `technically_ok_but_musically_weak`, with the explicit
constraint that later work must preserve Base and must not fake Hard through a
level-only lift, hollow pitch shift, or off-grid retrigger.

Self-analysis explains the perceptual miss: the prior Hard waveform differed
from Base and was about `1.2 LU` louder, but its envelope never decayed. Hard
therefore had only `0.7%` measured silence versus `1.0%` in Base; the trigger
mask changed source cursors without creating the promised chopped articulation
or inactive-slot space.

The next technical candidate keeps Beat03 Base byte-identical at
`13d5943f0d2c222f80c279991dfbf3593b6e155b99795c916661bb3565f64a4c`.
For `source_transient_chop`, a tempo-derived envelope reaches `0.03` after
`0.55` of an eighth-note step, while source-reactive edge/drive and a bounded
`1.12` whole-path gain compensation keep the surviving hits forward. For
continuous `source_texture_bite`, no trigger grid is imposed; the existing
source waveform receives bounded wavefold/quantization instead.

Across Beat03 plus seven development/holdout sources, every Hard render is
non-silent and unclipped and every missing-source control is digital silence.
The seven transient policies measure relative Base-to-Hard RMS deltas from
`1.062192` to `1.248608`; the continuous texture policy measures `0.152667`.
Maximum absolute pairwise 20 ms Hard-envelope correlation is `0.183690`.
Beat03 Hard has `45.75%` silence, crest factor `7.789`, peak `0.709396`, and no
clipping. This is a technically review-ready candidate, not a human pass.

### Gated Hard candidate human reject and mechanism audit

Owner reviewed the exact committed artifact at `edd90406`:

```text
/tmp/riotbox-1422-hard-review-edd90406/
  06_w30_resample_base_to_hard_live_gesture.wav
SHA-256:
  1ca4b7bdf5029ff60bfc8cc2cb2bb7b24639e04c2010c6e2ff56cd14b5c32ee4
```

The structured verdict is `human_verdict: reject`, strongest element `chop`,
source recognition `source_transformed_but_present`, and hook `clear`. The
binding failure statement is: the second half is audibly choppier and emptier,
but it is not harder. The review is stored at:

```text
/tmp/riotbox-1422-hard-listening-review-edd90406/review.json
SHA-256:
  4a9cf47ce50d526b47e5bd5eb4aa468519bc02a526c9220d973f345ed7d5f633
```

The signal supports that verdict. Relative to Base, the Hard onset raises
`0–10 ms` RMS by `77.6%`, then loses `48.3%` at `10–40 ms`, `69.0%` at
`40–120 ms`, and `91.0%` at `120–200 ms`. Attack/body ratio rises from `6.8`
to `39.0`; the kept event has become a brief bright head without enough body.
Hard loses `38.4%` of Hann-windowed band magnitude in `20–80 Hz`, `21.1%` in
`80–250 Hz`, `18.6%` in `250 Hz–2 kHz`, and `12.8%` in `2–6 kHz`. Only the
low-magnitude `6–20 kHz` band rises, by `53.7%`. Its higher peak therefore does
not represent stronger physical or midrange impact.

Corresponding complete WAVs in the development-matrix Beat03 directory and the
committed review directory are byte-identical; only runtime timestamps and
absolute output paths differ in `session.json`. Within the pack, the exact A/B
Base half is sample-identical to the corresponding first two bars of the
standalone Base sibling. The exact A/B Hard half is not sample-identical to the
standalone Hard sibling because the former keeps callback state across the
Base-to-Hard transition and the latter starts fresh. The A/B Hard half measures
`46.38%` near-silence with `|mono| < 1e-4`; the earlier `45.75%` figure belongs
to the four-bar standalone Hard sibling. Neither value is hardness proof.

The onset audit uses the exact A/B halves, arithmetic stereo-to-mono, and all
twelve active eighth-note anchors implied by mask `11010111`. It concatenates
the stated post-anchor windows before computing RMS; attack/body is
`0–10 ms / 40–120 ms`. The band audit removes each half's mean, applies a Hann
window, and compares root-summed real-FFT magnitudes inside each stated band.
FFmpeg `loudnorm` reports `-16.85 / -19.08 LUFS` and
`-4.09 / -2.26 dBTP` for the exact Base/Hard halves.

The code audit found matching failure mechanisms and additional product-path
risks:

- the current immediate-decay gate removes body instead of providing separate
  source-adaptive attack, hold, and release
- source-cursor jumps are fed into the edge detector without priming its
  history from the new slice, so discontinuities can become amplified clicks
- current technical promotion checks accept any non-silent, unclipped Hard
  signal with enough waveform delta, even when impact is worse
- fresh stopped activation, mid-bar/inactive-slot start, and live seek/restore
  lack safe resample-state regressions
- active source replacement can inherit the previous source cursor/edge history
  because the render plan has no separate source/artifact revision
- all `16,384` atomic source samples are copied per callback; the path needs a
  benchmark and bounded immutable handoff before more realtime DSP is added
- arithmetic mono projection and uniform unfiltered decimation can remove
  stereo body or manufacture high-frequency artifacts

The source-corpus audit also found that `oga_illin_robotic` changed output
across five successive v3–v6 tuning iterations while serving as the sole
`holdout-dense` target. Its final `15.3%` texture delta was explicitly used to
raise the shared texture regression threshold from `0.005` to `0.02` and
freeze the rules. It is therefore consumed development material, not a reserve
holdout. The `2026-07-26` manifest update retires it, records the consuming
ticket/date, and replaces it with the independent, unheard CC0
`oga_riintron_fat_groove_drums` case. That acquisition restores the rotation
contract but is not quality proof. `oga_bretbernhoft_beatloops` and
`oga_akikazer_menu` were also rendered under four changing algorithm
generations. Their causal influence was not established, so the manifest
conservatively retires and independently replaces them rather than presenting
them as fresh.

The current constants remain deterministic implementation state but are not a
listening-approved Hard recipe. Another gate/gain/drive float adjustment is
blocked until the causal mechanism changes. The research handoff is
`docs/engineering/perceptual_hardness_and_musical_impact.md`: preserve Base,
separate source-adaptive attack from body, test band-limited nonlinear bite,
compare native and loudness-matched output, and prove the shared policy across
the multi-family development matrix and untouched holdouts before another
human playback.

## Earlier Onset-Only RuntimeMix Evidence

The following table and hashes describe the onset-only candidate that preceded
the Base loop-timing reject. They remain useful historical evidence for why
local-onset alignment alone was insufficient, but they are not the current
review candidate.

Each candidate uses the exact RuntimeMix simulation with silent TR-909,
MC-202, W-30 preview, and generated source-monitor contributors.

| Source | Matrix role | Base LUFS | Base peak | Hard LUFS | Hard peak | Result |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Beat03 130 BPM | development / Golden Path | `-18.1` | `-4.1 dBFS` | `-16.6` | `-4.0 dBFS` | render pass |
| Beat08 128 BPM | development, same narrow loop family | `-14.1` | `-4.1 dBFS` | `-13.8` | `-3.9 dBFS` | render pass |
| Beat20 128 BPM | development / weak source | `-15.1` | `-4.2 dBFS` | `-14.1` | `-4.0 dBFS` | render pass |
| DH BeatC 120 BPM | holdout / drums | `-17.7` | `-4.2 dBFS` | `-16.5` | `-4.0 dBFS` | diversity fail |
| DH BeatC Kick/Snare 120 BPM | holdout / sparse drums | `-17.9` | `-4.1 dBFS` | `-16.5` | `-4.0 dBFS` | diversity fail |
| DH RushArp 120 BPM | holdout / tonal riff | n/a | n/a | n/a | n/a | exact-path setup fail |
| DH Fadapad 120 BPM | holdout / pad/noise | n/a | n/a | n/a | n/a | exact-path setup fail |

All five successfully rendered base/hard pairs are non-silent and unclipped.
Corresponding missing-source controls are digital silence. RushArp and Fadapad
fail before render because the exact review setup requires a Rust-probed primary
grid even when their family contract calls for manual-confirm or degraded timing.
That may ultimately produce a valid unavailable/degraded product decision, but
the current hard-variation matrix cannot claim coverage from a renderer abort.

The DH BeatC and Kick/Snare hard artifacts have different hashes but their
20 ms RMS envelopes correlate at `0.992589` with mean absolute delta
`0.006449`. This fails the existing cross-source gate of correlation at most
`0.95` and mean absolute delta at least `0.01`. Normalized hard clipping has
collapsed different drum inputs toward the same articulation.

Earlier onset-only Beat03 artifact hashes:

- base SHA-256:
  `2c93fd593983182c7efc35d6a5a9b182c351b996d644bcb07d332459fe878b3e`
- hard SHA-256:
  `d567749a3a13dc06aec50fab4a7cdd6f50bf95785aa06bf3911450844026fa87`
- missing-source SHA-256:
  `77b5e35b2b31b93cb6d497449ebfb6e60c5f6bf089c63ab52d0be72e46335a2d`
- continuous two-bar base to two-bar hard gesture SHA-256:
  `ae81a633c655fe0b25353d6e98faaebdc7405d0e9fc7cece61dd4ad8a019c4ff`

At that stage, the stable Base hash proved hard-mode iteration had not rewritten
the earlier human-accepted anchor. The subsequent exact listen exposed the
separate Base loop-phase defect, so that hash is now a historical control rather
than an accepted current anchor. Distinct Hard hashes alone were also
insufficient because the holdout envelope comparison detected musical collapse.

## Product-Spine Proof

- Queue/commit owner: existing `w30.apply_damage_profile`, quantized to the
  next bar.
- Session/replay: variation derives from the committed action-log position
  after the active resample and its typed lineage target.
- Runtime: coherent callback state carries variation, activation revision,
  intensity, the source-derived trigger mask and local-onset cursors, and
  bounded source PCM without callback I/O or allocation.
- User/observer: Jam/Capture/Log summaries label `base` or `hard_damage`; the
  observer exposes variation, revision, intensity, hard policy, trigger mask,
  local-onset cursors, and transient contrast.
- QA: full-duration projection, distinct base/hard trigger roles, focused
  callback delta, post-resample state retention, snapshot replay, exact
  RuntimeMix, and missing-source silence are covered. Cross-family
  generalization is explicitly not covered: the latest matrix rejects it.

## Listening Gate

The doubled/echo-kick defect is closed and grid-safe Base has a human timing /
harmonic pass. The gated Hard replacement has now received a structured human
`reject`, so no candidate is waiting for another listen.

Before the next listening request:

1. preserve the accepted Base hash
   `13d5943f0d2c222f80c279991dfbf3593b6e155b99795c916661bb3565f64a4c`
2. state one causal Hard hypothesis and the intended domain: source-local
   attack, midrange bite, retained body, roughness, or arrangement impact
3. pass the required development matrix and only the fresh, causally untouched
   active holdout replacements
4. analyze the exact artifact at native and loudness-matched level, including
   onset spectrum, transient/body relation, body retention, timing, source
   identity, and safety
5. assign the exact WAV and tell the listener what should be heard; bass
   pressure is not required unless a typed owner assigns it

## Review Note

Branch review found and fixed one state-retention defect: the first projection
would have kept an old resample active after unrelated later material. The
final projection permits persistence only across focus within that resample's
lineage and has a regression test for unrelated-focus deactivation.

The fresh branch review for the grid-aligned Base fix found one real regression:
the source-transport observer probe called its action a bar capture while
retaining the global four-bar default, then depended on source-EOF clamping.
The probe now sets typed `OneBar` intent explicitly and passes without weakening
the product's fail-closed rule. Focused W-30/capture tests and the complete
`just ci` gate pass after that correction. No unresolved finding remained for
that grid-aligned Base correction. The later Hard-path audit findings recorded
above remain open. They may be tracked in explicit blocker tickets, but ticket
creation does not waive the gate: the findings must be resolved and verified
before this branch can merge.

One known maintainability signal remains:
`render_tr909_w30_preview.rs` is above the soft review-size range. A future
change should consider a real semantic `w30_resample_tap_renderer` module with
explicit visibility and colocated tests. This slice does not add a mechanical
`include!` shard or mix that module move into the audible behavior change.

### H14 pre-freeze branch review

The H14 `code-review` plus Rust/realtime pass found and fixed three additional
issues before fresh holdout rendering:

- `refresh_view` cloned the complete bounded W-30 PCM state before every
  projection. The next projection now borrows the previous state for cache
  comparison and assigns only after the borrow ends.
- exact callback failures had no evaluated state, so a matching source that
  could not satisfy all calibration constraints would repeat the expensive
  offline render search on every refresh. The typed calibration plan now
  distinguishes `exact_callback_evaluated` from
  `exact_callback_calibrated`, exposes both through runtime/observer evidence,
  and caches matching positive or negative outcomes.
- the full-unity pre-hit context began with an instantaneous gain step. On the
  Bertsz stress case that transition measured `0.03934` absolute and `11.31x`
  its local jump RMS. A 2.5 ms lead-in now precedes the complete 20 ms unity
  context; the same diagnostic falls to `0.01547` and `2.02x`. The exact
  callback calibration includes this ramp, and product-path timing, boundary,
  head, body, level, and H13 gesture gates remain green.

The realtime callback still performs no allocation, I/O, file access, or
analysis. Two callback-local `expect` calls on the typed grit-recipe invariant
were replaced with a non-panicking unavailable return. Focused tests cover
successful and rejected calibration-cache reuse, all four calibration cycles,
the hit/following-body/between-hit responsibilities, and the click-safe
pre-roll. The large accumulated H1–H14 diff and the existing large W-30
projection/renderer/diagnostic files remain a review-cost signal; semantic
module extraction belongs in a separate mechanical ticket rather than this
audible freeze.

Full CI then exposed two stale test assumptions outside the focused H14
filters. The texture callback fixture had added a level-compensation assertion
while still supplying an uncalibrated `1.0` gain; it now derives the fixture's
typed target gain from an uncalibrated preflight and reruns the exact
Base-to-Hard transition. The onset-grid tolerance omitted one frame of
transport/fade rounding at 44.1 kHz; its bounded 0.5 ms allowance now includes
that frame. Both focused regressions pass without changing product output or
release gates.

### H14 frozen holdout result

H14 is rejected before human listening because the fresh holdout did not
exercise its causal mechanism.

The source-timing preflight did not invent manual confirmations. Three sources
reported manual-confirm-only timing, one bad-timing one-shot reported
unavailable timing, and none of those four entered candidate rendering. The
remaining dense-break source had a trusted provider value of 140 BPM plus
stable beat evidence and received exactly one product-path render.

That render selected inherited `source_transient_chop` behavior with
`low_impact_recipe=unavailable`; therefore
`exact_callback_evaluated=false` and
`exact_callback_calibrated=false`. It passed the inherited technical gates:

- Base-to-Hard level ratio: `1.0498`
- H12-to-H13 impact-body ratio: `1.066922`
- reverse-pickup relative delta: `1.073662`
- clipping: none in the Hard output
- missing-source control: digital silence

The valid waveform hashes are recorded in
`docs/benchmarks/w30_resample_h14_characterization_v1.json`. These facts prove
the inherited path did not regress, but they cannot establish cross-source
generalization of H14 exact hit-window calibration. Treating non-selection as a
pass would let a candidate evade its own acceptance mechanism.

Holdout A is consumed. H14 must not be retuned against it, and no listening
request is allowed. The rotating holdout contract was restored with five new
unheard, unrendered CC0 reserve sources and passes both fixture and local-file
validation. H15 therefore freezes the byte-identical H14 implementation and
gates as a coverage retry against untouched Holdout B. It introduces no new
DSP or selection constants. H15 can pass only if a fresh source actually
selects and successfully evaluates the causal `source_hit_shaper_v3` path;
non-selection remains a failure, not a pass.

### H15 frozen holdout result

H15 is also rejected before human listening, without producing a candidate
WAV. Holdout B timing preflight left four sources manual-confirm-only or
unavailable. The remaining dense-break source, `Psychic`, has an explicit
provider tempo of 190 BPM, but the Rust probe selected `141.50945 BPM` with
high drift. That is close to a 3:4 tempo alias (`142.5 BPM`), not a matching
primary hypothesis.

The exact product path rejected the attempted 190 BPM confirmation before
Session persistence:

`explicit source BPM 190.00 does not match Rust timing candidate 141.51 within
1.00 BPM`

This is correct under the current source-timing contract. BPM-only input may
confirm a matching analyzer grid; a conflicting or unavailable grid requires
the typed musician-manual form with both BPM and downbeat phase. The holdout
contains no independently trusted downbeat declaration, so phase zero was not
invented and the conflicting 141.51 BPM hypothesis was not substituted.

Holdout B is consumed. H15 neither proves nor disproves H14 DSP behavior; it
proves that the current acceptance corpus cannot reach the candidate through a
trusted grid. No second render, gate change, or human listening request is
allowed. The rotation must be restored again before another frozen generation.

The rotation was restored with five further unheard/unrendered CC0 derivatives
and passed fixture plus local-file validation. H16 is the final byte-identical
coverage retry. Its fresh Holdout A includes a provider-declared 140 BPM,
hard-hitting/start-stop loop selected from metadata before freeze. That improves
the chance of reaching the causal path but does not guarantee selection or
acceptance. If H16 still lacks causal coverage, Riotbox must stop consuming
holdouts and address acceptance reachability as an explicit enabler.

### H16 frozen holdout result

H16 is rejected before candidate rendering and ends the byte-identical coverage
retry sequence. Its metadata-selected 140 BPM hard-hitting/start-stop source
produced a Rust primary candidate of `154.63918 BPM`, manual-confirm-only
readiness, weak downbeat, and high drift. BPM-only confirmation therefore could
not satisfy the existing 1 BPM matching contract, and the source had no
independently trusted downbeat phase for a typed musician-manual grid.

The other three musical sources also remained manual-confirm-only with high
drift; the impact one-shot was timing unavailable. No H16 candidate WAV was
created, `source_hit_shaper_v3` was not evaluated, and no human listening was
requested.

Three frozen generations now establish the reachability problem:

- H14 reached the product path but selected an inherited recipe.
- H15 had explicit provider tempo but failed matching confirmation.
- H16 deliberately improved metadata reachability but still failed trusted
  timing before recipe selection.

Another identical holdout retry would consume evidence without testing a new
hypothesis. The next slice must explicitly prequalify acceptance reachability:
trusted grid ownership and causal recipe-selection coverage, while preserving
Source Graph, Session confirmation, and holdout separation.

### H17 reachability-qualified holdout result

RIOTBOX-1424 made the intended path reachable without changing its DSP or
gates. Two fresh Holdout A families passed technical preflight:

- `oga_congusbongus_lasso_lady`, provider-declared 140 BPM, selected
  `source_transient_chop` plus `source_hit_shaper_v3` and completed exact
  callback calibration;
- `oga_yd_oriented`, whose CC0 LMMS project metadata independently declares
  140 BPM, reached source timing but selected no low-impact recipe and was
  therefore ineligible for causal candidate coverage.

The frozen Lasso candidate failed before human listening at a Base-to-Hard
level ratio of `1.31278`, above the unchanged `1.30` ceiling. Oriented produced
no candidate WAV. Both sources are consumed technical evidence; neither has a
human musical verdict.

### H18 runtime-synchronous calibration correction

Diagnosis found three coupled implementation defects:

- exact calibration included H13 while the release validator first gated an
  H12 counterfactual; an H13 level reduction could therefore authorize an
  over-limit H12 gain;
- `hit_window_compensation_gain` was visible in product/observer state but not
  carried into the realtime snapshot, so it could not preserve a
  source-dependent late body;
- the renderer validator placed windows using the requested BPM rather than
  the product-owned callback tempo.

H18 now calibrates H12, verifies the same gain through H13, transports the
typed compensation into the callback, derives a bounded local 120–200 ms body
target, and measures QA at runtime tempo. The selected H13 impact is no longer
attenuated after articulation; its derived body gain is bounded at `1.75`.
The `0.95` late-body floor and `1.30` exact-path level ceiling are unchanged.

The final development matrix passes across eight sources and six behavioral
families. Exact cases:

| Case | H12 late body | H13 impact body | Base-to-Hard level |
| --- | ---: | ---: | ---: |
| Beat03 | `1.03995` | `1.0633` | `1.2690` |
| Bertsz dense-full-mix stress | `1.02299` | `1.0720` | `1.2340` |
| Lasso consumed development | `0.97333` | `1.1410` | `1.1652` |

Cinameng, Marwan, Fupi, and Beard pass their inherited `1.15` level ceiling;
Pauliuw remains explicitly unavailable and emits no fallback Hard output.
Missing-source output remains digital silence. This matrix freezes the
development mechanism only; a fresh multi-family holdout and structured human
listening are still required before RIOTBOX-1422 can close.

The two H17 technical cases were retired and replaced one-for-one before that
fresh gate. `oga_congusbongus_head_in_the_sand` (tonal riff) and
`oga_srg774_sector` (pad/noise) were acquired from provider-declared
CC0 sources and classified from metadata only. At acquisition time, only
format, duration, peak, clipping, and deterministic conversion were inspected;
neither replacement had yet been listened to, preflighted, or rendered.

### H18 fresh-holdout rejection and H19 reachability evidence

The frozen H18 implementation produced no review candidate from three fresh
Holdout A cases:

- `oga_frenchyboy_tech_rave_faster` exposed only two onsets in 16 seconds, so
  timing was unavailable before product projection;
- `oga_congusbongus_head_in_the_sand` matched its provider 150 BPM at
  `149.66743 BPM`, but the product correctly made Hard unavailable because the
  source was too quiet and sparse (`0.016379` RMS, `0.201` active-frame ratio);
- `oga_sudocolon_dance` reached `source_transient_chop`, but did not select
  `source_hit_shaper_v3`.

H19 adds the existing selector evidence to
`riotbox.w30_reachability_preflight.v1`; it changes neither DSP nor selector
gates. Sudocolon's low-band attack share (`0.48562`) and attack-over-source
ratio (`1.22218`) passed their `0.18` and `0.30` floors, while
attack-over-following-body was only `1.01756` against `1.40`. This is evidence
of sustained low body, not a clear attack-owned low impact. Lowering the gate
would make an inappropriate source satisfy the recipe rather than fix
reachability.

All three sources are consumed technical evidence without candidate WAVs or
human verdicts. Holdout A was replenished one-for-one with unheard CC0 sources
`oga_poinl_perces`, `oga_turnovus_simple_drumbeat_body`, and
`oga_extenz_emotional_piano`. Their family labels remain provisional metadata
classifications until the frozen H19 preflight consumes them. The Turnovus
derivative uses fixed `0.75` import headroom because sample-rate conversion of
the full-scale original exposed sample-peak overshoot; this is corpus hygiene,
not candidate normalization or product DSP.

### H19 timing rejection and H20 replenishment

The frozen H19 retry covered dense-break and tonal-riff families but stopped
before product projection:

- Turnovus exposed a stable `105.05837 BPM` beat, but weak/ambiguous downbeat
  and high drift. No independent provider BPM/downbeat pair existed, so using
  the analyzer result as its own manual confirmation was prohibited.
- Emotional Piano declared 138 BPM, but exposed only one onset in 16 seconds;
  timing was unavailable.
- Perces declared 112 BPM, while Rust selected `138.24884 BPM`; downbeat was
  weak and no independent phase existed for the paired manual override.

No product projection, candidate WAV, or human listening occurred. Sector and
the Trez bad-timing control remain untouched.

H20 changes only the rotating corpus. Metadata-targeted CC0 replacements are
Tricks & Traps' 160 BPM Stomper, Some Weirdo's distorted 120 BPM guitar
progression, and whittakerb1987's 120 BPM Dungeon Run variant. A first
metadata search rediscovered Horde War Drums, but the rotation validator
correctly rejected it as duplicate H7 holdout evidence; it was not reinstated.
The actual replacements were inspected only for licensing metadata, format,
duration, peak, clipping, deterministic conversion, and hash. Their family
labels remain provisional; none was listened to, timing-probed, preflighted,
or rendered before H20 freeze.

### H20 timing rejection and H21 tracker-tempo replenishment

H20 again stopped before product projection. Stomper's Rust primary
`158.45071 BPM` differed from provider 160 BPM by `1.54929`, above the
unchanged 1 BPM tolerance; downbeat remained weak and drift high. The held
guitar progression exposed one onset, while Dungeon Run selected
`149.00665 BPM` against provider 120 with weak downbeat. No independent
downbeat phase existed for either mismatch.

All three are consumed timing evidence without candidate WAVs or human
verdicts. The repeated result now justifies metadata stronger than a title or
variant label, but it does not justify loosening timing.

H21 keeps DSP, selectors, and timing contracts frozen. Its replacements are
the provider-declared 150 BPM Mental Corruption plus two CC0 FamiTracker loops
whose project FRAMES blocks independently encode tempo and speed: 8-Bit Battle
Loop and 8-Bit Cave Loop. Only tracker fields, licensing metadata, format,
duration, peak, clipping, conversion, and hash were inspected before freeze;
their audio and product behavior remained unheard and unprobed.

The first H21 timing result exposed a documentation mistake: a FamiTracker
tempo field is not automatically musical BPM. The documented formula is
`tempo * 6 / speed`; Battle's tempo 150 and speed 5 therefore derive 180 BPM,
while Cave's tempo 150 and speed 6 derive 150 BPM. Battle independently
matched Rust and product timing at exactly 180 BPM, then selected
`source_texture_bite`, so the exact hit shaper remained inapplicable.

Mental Corruption independently matched provider 150 BPM at Rust/product
`150.25043 BPM` and reached `source_transient_chop`. Its low-band attack share
(`0.43791`) and attack-over-source (`1.36893`) passed, but
attack-over-following-body was only `1.17281` against `1.40`; no exact recipe
or candidate was produced.

H21 therefore rejects before candidate rendering across dense-break and
tonal-riff families. No human listening occurred. H22 restores the rotation
with unheard Drama and 8-Bit Victory Loop replacements while preserving the
untouched Cave, Sector, and bad-timing control. Further holdout access is
frozen until a bounded source-adaptive hit-shaper reachability enabler lands
on development material.

## H23 Frozen Timing Rejection

RIOTBOX-1425 landed the source-local hit-shaper reachability mechanism without
changing the `0.18`, `1.40`, or `0.30` ownership gates. H23 then consumed
exactly the two different-family replacements frozen by H22 through the
non-listening preflight:

- Drama's provider 130 BPM conflicted with the Rust `173.07709 BPM`
  manual-confirm-only primary grid.
- 8-Bit Victory Loop's independently derived FamiTracker 180 BPM conflicted
  with the rendered WAV's Rust `133.92857 BPM` manual-confirm-only primary
  grid.

Both cases stopped before Source Graph product projection. No Hard policy,
low-impact recipe, exact callback calibration, candidate WAV, or human verdict
exists. The candidate is rejected; Cave, Sector, and the bad-timing control
were not used as opportunistic retries. No downbeat phase was assumed.

The holdout rotation is restored without listening or musical analysis using
two new provider-declared CC0 sources from independent packs: Lucid Trigger
(progressive break, 150 BPM) and the NES rendering of Chopin Op. 25 No. 2
(tonal material, 140 BPM). Only license/provider metadata, archive listing,
format, duration, peak, clipping, conversion, and hash were inspected.

The next blocker is trusted-grid reachability, not W-30 DSP or hit-shaper
selection. Any enabler must work on consumed development material, must not
pass the Rust BPM back as external confirmation, and must not assume source
phase zero. Machine-readable evidence is frozen in
`docs/benchmarks/w30_resample_h23_characterization_v1.json`.

## H24 Frozen Role And Timing Rejection

RIOTBOX-1426 landed the typed source-backed tempo-guided phase route while
leaving W-30 DSP and the hit-shaper ownership gates unchanged. H24 then
consumed exactly the two replacements restored after H23 through the
non-listening preflight:

- Lucid Trigger's provider 150 BPM matched Rust `150.25041 BPM` and projected
  through Source Graph at the same product tempo. Hard selected
  `source_texture_bite`, however, so `source_hit_shaper_v3` and exact callback
  calibration were inapplicable.
- Chopin's provider 140 BPM conflicted with Rust `144.23077 BPM`. The guided
  route detected only six onsets and rejected with `InsufficientOnsets`, before
  product projection.

The first result is role evidence, not a timing failure. A continuous texture
may legitimately receive source-derived nonlinear bite without an invented
trigger grid, but it cannot satisfy harder-beat or exact-hit-shaper coverage.
The second result proves the timing enabler fails closed rather than guessing
phase from sparse evidence. Neither case produced a candidate WAV, listening
artifact, or human verdict. Cave, Sector, and the bad-timing control remained
untouched.

The rotation is restored using two independent CC0 OpenGameArt packs without
listening or musical analysis: Joth's provider-declared 143 BPM drum-and-bass
loop Black Diamond and nene's instrumental Love Song, whose provider later
described its tempo as approximately 95 BPM. Only license/provider metadata,
format, duration, peak, clipping-safe conversion, and hash were inspected.
Both family assignments remain provisional and unheard.

Another blind holdout retry is frozen. The next bounded slice must establish
the performer-owned Hard intent and its role-specific development evidence
before another fresh attempt. It must not relabel texture damage as a harder
beat, weaken the `0.18`, `1.40`, or `0.30` hit-shaper ownership gates, or tune
timing against consumed H24 material. Machine-readable evidence is frozen in
`docs/benchmarks/w30_resample_h24_characterization_v1.json`.

## H25 Frozen Impact-Intent Gate

RIOTBOX-1427 now carries performer-owned `impact` through the existing
`w30.apply_damage_profile` queue, commit, Session/replay, and product projection
path. Before opening the H25 holdouts, the existing exact-path preflight was
tightened to report both `requested_intent` and `intent_outcome` and to require
`impact` plus `realized_impact` in addition to `source_hit_shaper_v3` and exact
callback calibration. Legacy, texture, mismatch, and unavailable outcomes fail
closed and cannot authorize an impact candidate.

The unchanged Beat03 development source passes the tightened preflight with
`impact`, `realized_impact`, `source_transient_chop`,
`source_hit_shaper_v3`, and successful exact callback calibration. The run
stops before candidate WAV generation. No H25 holdout was accessed while
freezing this acceptance gate; W-30 DSP, the `0.18` / `1.40` / `0.30`
ownership gates, and the accepted Base remain unchanged.

## H25 Source-Aligned Impact V5 Technical Result

The earlier frozen V4 body-shaping candidate remains rejected historical
evidence. H25 does not tune its gain, body EQ, or delayed gesture. The new
`source_aligned_impact_v5` mechanism selects a real grid-near source hit,
preserves the dry source body, and adds only a causal source-local nonlinear
presence head. Source-derived density may fall from the full mask to an
anchored alternating or two-hit mask when the complete source rhythm cannot
retain level and crest.

The exact callback contract is:

- whole-window level `0.90..=1.10`;
- causal presence-head ratio at least `1.10`;
- Hard/Base crest ratio at least `0.90`;
- no fallback audio when selection or calibration fails.

Five contrasting development cases covered four typed families:

- Beat03 (`dense_break`) passed at level `1.01452`, causal head `1.21401`,
  crest `0.98566`, and source-derived mask `01000101`;
- Bertsz (`dense_full_mix` stress) passed at level `1.05359`, causal head
  `1.11037`, crest `0.96589`, and mask `01011001`;
- Cinameng (`dense_break`) and Marwan (`sparse_drums`) correctly rejected
  insufficient local attack over body;
- Fupi (`tonal_riff`) reached exact calibration but rejected level `1.30467`
  and crest `0.76649`, then failed closed to unavailable output.

After freeze, Continue was the first exact H25 holdout pass. Its provider
`138.462 BPM` matched Rust `138.46156 BPM`; the product path realized impact
with `source_aligned_impact_v5`, mask `10010101`, level `1.01426`, causal head
`1.14549`, and crest `1.00119`. The resulting Hard WAV hash is
`8b48fad8dc967e904fbf887e388bbdb42a8a49ddf86dc6964f73507518497b0a`.

Three tonal holdouts then failed honestly:

- Trance Boss Battle and Fever Stadium matched provider tempo but resolved the
  committed impact request as source mismatch before V5;
- Better Days used the independently reconstructed 120 BPM/downbeat route and
  reached V5, but exact calibration retained only `0.65606` whole-window
  level despite causal head `1.16835`, so no candidate WAV was generated.

The exact search initially exposed approximately three minutes of debug
control-plane work on the rejected Better Days case. A sample-stable search
optimization reuses the already-rendered unity candidate and stops descending
gain once level is below the acceptance floor. The difficult reject now takes
roughly 30 seconds in debug. All eight WAVs of the accepted Continue case are
byte-identical before and after the optimization, and Better Days retains the
same optimization-comparison preflight hash
`71d8018922bb8aab692d44e726bc9e726282820b17119b1d549d6020ad00ca5c`.
Evaluated V5 rejections also retain their typed selector evidence behind an
`unavailable` Hard policy, allowing unchanged Jam view refreshes to reuse the
negative result without rerunning that search. The callback cannot trigger the
retained rejected recipe. That explicit retained evidence changes the final
report hash to
`29f2d9d77563df97f7f39a7e61504c790c04ecd29b99caf50684540f61fd3b83`.
It now truthfully records `applicable=true`, `evaluated=true`, and
`calibrated=false`; the rejected level, causal-head, body, and crest metrics
remain unchanged.

H25 remains `human_verdict: unverified`. It has one exact fresh holdout pass
but not the required two prequalified sources across two families. No
structured playback is authorized, and the result is not a musical-alpha
claim. The active rotation is replenished with untouched CC0 sources; no
replacement was pre-rendered or used to tune V5.

Machine-readable evidence is frozen in
`docs/benchmarks/w30_resample_h25_characterization_v1.json`.

## Branch Review Note

`w30_projection.rs` now exceeds the project file-size guidance and carries the
source selector, exact-callback calibration, projection assembly, and their
tests. This is a review-cost warning, not justification for an `include!` or
line-number split. A later behavior-neutral maintenance slice should extract
the stable exact-calibration domain as a real Rust module with explicit
visibility and local tests; doing that mechanically inside this audible slice
would mix module migration with DSP behavior and make sample-stability review
less reliable.

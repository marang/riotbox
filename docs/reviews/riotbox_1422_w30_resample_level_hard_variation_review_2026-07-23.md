# RIOTBOX-1422 W-30 Resample Level and Hard-Variation Review

Date: 2026-07-23

Classification: `audible_vertical_slice`

First candidate human verdict: `reject`

Full-phrase base human verdict: `pass` / loopable

Latest heard hard candidate verdict: `reject`

Post-research successor candidate: `not rendered`

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

Markus reviewed the exact committed artifact at `edd90406`:

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

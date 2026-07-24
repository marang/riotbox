# RIOTBOX-1422 W-30 Resample Level and Hard-Variation Review

Date: 2026-07-23

Classification: `audible_vertical_slice`

First candidate human verdict: `reject`

Full-phrase base human verdict: `pass` / loopable

Latest hard candidate matrix verdict: `reject_before_listening`

Latest hard candidate human verdict: `unverified`

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
- The latest candidate removes pitch/rate movement, holes, random ordering, and
  global hard-mode gain. It keeps the base body and crossfades about `40 ms` of
  each beat into a hard-clipped onset retained separately from the same
  committed source.
- Missing or invalid source audio remains typed unavailable and digitally
  silent. No oscillator or replacement music exists.
- The latest hard candidate is not review-ready: two different drum holdouts
  collapse to nearly the same envelope, and tonal/pad holdouts cannot traverse
  the exact renderer's current timing-confirmation setup.

## Exact RuntimeMix Evidence

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

Current revised Beat03 artifact hashes:

- base SHA-256:
  `2c93fd593983182c7efc35d6a5a9b182c351b996d644bcb07d332459fe878b3e`
- hard SHA-256:
  `d567749a3a13dc06aec50fab4a7cdd6f50bf95785aa06bf3911450844026fa87`
- missing-source SHA-256:
  `77b5e35b2b31b93cb6d497449ebfb6e60c5f6bf089c63ab52d0be72e46335a2d`
- continuous two-bar base to two-bar hard gesture SHA-256:
  `ae81a633c655fe0b25353d6e98faaebdc7405d0e9fc7cece61dd4ad8a019c4ff`

The stable Base hash proves hard-mode iteration did not rewrite the
human-accepted anchor. Distinct hard hashes alone are insufficient because the
holdout envelope comparison still detects musical collapse.

## Product-Spine Proof

- Queue/commit owner: existing `w30.apply_damage_profile`, quantized to the
  next bar.
- Session/replay: variation derives from the committed action-log position
  after the active resample and its typed lineage target.
- Runtime: coherent callback state carries variation, activation revision,
  intensity, and bounded source PCM without callback I/O or allocation.
- User/observer: Jam/Capture/Log summaries label `base` or `hard_damage`; the
  observer exposes variation, revision, and intensity.
- QA: full-duration projection, distinct base/hard trigger roles, focused
  callback delta, post-resample state retention, snapshot replay, exact
  RuntimeMix, and missing-source silence are covered. Cross-family
  generalization is explicitly not covered: the latest matrix rejects it.

## Listening Gate

The full-phrase Base has a human pass, but the latest Hard candidate is rejected
before another listening request. The slice remains open. A future candidate
must first pass a fresh matrix of at least five sources across four typed
families, including two different-family holdouts that did not choose its
algorithm or constants. DH BeatC, Kick/Snare, RushArp, and Fadapad now count as
development evidence because their failures are known; fresh holdouts must
replace them.

Only after that gate should structured listening answer:

1. Is the hard gesture immediately obvious and musically preferable as a
   performer-triggered variation?
2. Does the accepted Base remain byte-stable and recognizable?
3. Does the wider Beat03 origin remain meaningfully related after the capture
   and resample lineage, without requiring the tap to reproduce uncaptured
   source sections?

## Review Note

Branch review found and fixed one state-retention defect: the first projection
would have kept an old resample active after unrelated later material. The
final projection permits persistence only across focus within that resample's
lineage and has a regression test for unrelated-focus deactivation.

The revised full-duration behavior still requires a fresh branch review after
validation. One known maintainability signal remains:
`render_tr909_w30_preview.rs` is above the soft review-size range. A future
change should consider a real semantic `w30_resample_tap_renderer` module with
explicit visibility and colocated tests. This slice does not add a mechanical
`include!` shard or mix that module move into the audible behavior change.

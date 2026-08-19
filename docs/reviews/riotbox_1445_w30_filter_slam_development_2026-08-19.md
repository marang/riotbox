# RIOTBOX-1445 W-30 Filter-Slam Development Exploration

Date: 2026-08-19
Partition: Development only
Result: `provisional_keep`
Decision: RBX-304
Holdout access: none
Commercial-reference access: none

## Narrow Question

Can a source-backed W-30 phrase use a clearly audible long-form filter movement
to create a strong build-to-return contrast that remains recognizable and feels
worth triggering live?

This was bounded musical discovery, not product implementation, source-diverse
qualification, hardness evidence, or release evidence.

## Bounded Source Access

A fresh access log existed before the only selected-file read. It admitted the
already registered Development case `dense_beat03_130` at
`data/test_audio/examples/Beat03_130BPM(Full).wav`, declared tempo `130.0` BPM,
and expected SHA-256
`e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`.
The actual hash matched. The access-log SHA-256 is
`124d7a47f96ac8c10b8c130aac39c3b6b0d05511f711f7afb5b494871db0a165`.

No source directory was discovered. Active rotation-v3 metadata contained no
Holdout path or hash match. No Holdout or commercial-reference audio was
opened, hashed, decoded, rendered, classified, or played.

## Bounded Variants

Variant 1 used a four-beat close-and-open arc. The filter was audible, but the
listener found the musical development too short for the movement to mature.
This was not a near-identity failure; it identified duration and phrase shape as
the missing owner.

Variant 2 changed the causal timeline rather than applying a scalar retry. It
used an eight-beat gesture after eight sample-exact context beats:

- four beats of gradual exponential cutoff movement from 14 kHz to 1.8 kHz
- two beats of deeper movement from 1.8 kHz to 280 Hz
- one beat held at 280 Hz
- one final beat whose first 20 ms crossfade from filtered to ordinary W-30

The low-pass used per-channel RBJ biquads with smoothstep-shaped exponential
cutoff movement, Q rising from `0.707` to `1.2`, and no makeup gain.

## Exact Kept Artifact

| Artifact | SHA-256 | Duration | Peak | RMS | LUFS | Clips |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source anchor | `d22c8b7b5cafacebbf14ad89eafa868658ecc6dedae8598b5e26be59be9b7149` | `3.692313` s | `0.977478` | `0.144538` | `-14.8` | `0` |
| A ordinary W-30 control | `09266320813a3d01b1ed67fb9abd3e12ffef389167f9493a17e47d2d6f0d1e2e` | `7.384625` s | `0.399872` | `0.068233` | `-22.1` | `0` |
| B staged filter slam | `a7ad09ee9674ccf5e8e511e717824a23f138c69b2b218f72c515441bf6e57487` | `7.384625` s | `0.399872` | `0.068187` | `-22.2` | `0` |

The exact PCM16/stereo/48 kHz A/B pair has sample-exact first-eight-beat
context, effect-span delta RMS `0.021616`, waveform correlation `0.946336`,
presence-band change `-4.7548` dB, and air-band change `-20.9464` dB. The
post-crossfade return delta is exactly zero. Boundary deltas remain below the
ordinary signal's maximum adjacent-sample movement, both channels are balanced,
and no artifact clips. The preflight-report SHA-256 is
`5d410b77ece21034cf175a88038c5d9ff25bcde664dff72248c6769a5ee9620f`.

Every playback ended at its announced endpoint and `pw-play` silence was
verified. The isolated A/B contributor was only the source-backed W-30 preview.

## Human Result

The project musician heard Variant 1 but found it insufficiently developed and
requested a longer phrase-scale application. Variant 2 was judged good and
live-usable; the longer arc allowed the filter movement to register, although
its eight-beat duration was considered close to the minimum acceptable span.
This is a provisional Development keep for the exact Variant 2 mechanism. It
does not establish source-general quality, product behavior, beat hardness,
Golden Path completion, or release readiness.

The third variant remained unused because the second earned an explicit keep.
Shorter filter arcs are not equivalent to the kept mechanism.

## Promotion Boundary

The exact heard mechanism is frozen in
`docs/benchmarks/w30_filter_slam_development_v1.json` under RBX-304. Local
exploration scripts and WAVs remain ignored artifacts and are not product or
qualification evidence. A separate Linear-first slice may rebuild this contract
source-blind as `w30.filter_slam` through queue/commit, typed Session/replay,
observer/UI, and exact RuntimeMix, then run fresh source-diverse Development
qualification and one formal product review. Qualification may reject v1 but
may not shorten, lengthen, retune, or otherwise change its heard mappings.

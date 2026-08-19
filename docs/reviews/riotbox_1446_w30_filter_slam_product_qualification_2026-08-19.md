# RIOTBOX-1446 W-30 Filter Slam Product Qualification

Date: 2026-08-19
Partition: Development only
Frozen mechanism: `w30_filter_slam_v1`
Decisions: RBX-304, RBX-305
Result: qualified bounded product keep

## Scope

RIOTBOX-1446 rebuilt the unchanged RBX-304 Filter Slam through the established
product spine as `w30.filter_slam`. The performer action targets the current
focused promoted W-30 capture, queues on the next bar, persists the typed
profile, capture identity, and start beat through Session and replay, remains
visible through observer/UI surfaces, and reaches the isolated RuntimeMix W-30
path. Missing or mismatched source-backed material is refused or silent; no
musical fallback exists.

The frozen product qualification contract is
`docs/benchmarks/w30_filter_slam_product_qualification_v1.json`, SHA-256
`05181ea58ffa060d6f0626e5363dbf6f804e5ea9d3d656192b0d513715f67d6e`.
It was fixed before qualification-source access. Source results were not
allowed to alter the eight-beat duration, cutoff/Q curve, 20 ms return,
render order, source set, threshold, or stopping rule.

## Development Access Boundary

A fresh bounded access log existed before the first qualification-source open.
The runner opened only the four exact registered paths and verified their
hashes before and after each case. It did not discover source directories or
open Holdout or commercial-reference audio. The completed local access log is
`artifacts/development/riotbox-1446/access-log-2026-08-19-v1.json`, SHA-256
`d4d0ad5c4228be59cc3e3651cb84b1cda91a1d1c440751be6b27dcbb0df3ce7b`.

The representative review used a separate one-source access log at
`artifacts/development/riotbox-1446/review-access-log-2026-08-19-v1.json`,
SHA-256
`83444643756d6a48f400b8fd5ee3f7c86069dc0a4b5ec7b6e6005fd9c467a2a4`.

## Exact Product Matrix

All four cases passed before human playback.

| Case | Product BPM | Effect-through-return delta RMS | Candidate peak | Clips / limited | Return / callback |
| --- | ---: | ---: | ---: | ---: | --- |
| `dense_beat03_130` | `130.284943` | `0.021836` | `0.400374` | `0 / 0` | pass |
| `freesound_alastair_pursloe_183441` | `135.110321` | `0.047206` | `0.390915` | `0 / 0` | pass |
| `freesound_dr_skitz_353853` | `119.680878` | `0.024209` | `0.391978` | `0 / 0` | pass |
| `tonal_rusharp_120` | `120.000000` | `0.083568` | `0.264047` | `0 / 0` | pass |

For every case, the frozen close, deepening, hold, and open-return window was
materially distinct; ordinary W-30 output resumed sample-exactly after the
20 ms return; 128- and 257-frame callback partitions matched exactly; and
pre-limiter clips, limiter interventions, and post-limiter clips were zero.
Capture lineage, W-30 grit, music-bus level, Source Monitor, MC-202, and TR-909
state remained unchanged. The missing-source control produced zero active
samples. Source-blind automated tests cover the exact coefficient/timeline
boundaries, Session JSON round-trip, replay equivalence, refusal, finite output,
and stereo-channel behavior.

## Formal Product Review

Exactly one representative composite was prepared after the complete matrix
passed: bounded source context repeated four times, one second of silence, A
exact product control repeated twice, one second of silence, and B exact
product Filter Slam repeated twice. The artifact is stereo 48 kHz PCM16,
`33.248417` seconds, `-21.6 LUFS`, and `-4.8 dBFS` true peak, with no PCM clips
and a 50 ms terminal fade. Its SHA-256 is
`54e206aee389a8e37d3f7f6560d56831a684adbabc73cfacee94f1420657a603`.
The exact A and B components are both `4.144771` seconds and have SHA-256
`40fc54981453e2a3ba8a019ed0ec5d8e3be11be261a34f2d74251dcc35edc183`
and `cc18e7cdbc91b12e6a86866ec423a55549d400618301b9cf044f9fe4add0330a`.

After exact-artifact preflight and fresh readiness, playback reached the
announced endpoint and stopped silently. The musician gave an affirmative
product verdict for the full question set: the long Filter Slam arc was clear,
musically useful, worth retaining as a live effect, source-recognizable, and
cleanly returning. No groove, clarity, timing, source-identity, or duration
objection was reported. This records `human_verdict: keep` for the exact
qualified product bytes.

## Claim Boundary

This qualifies one performer-owned long-form W-30 filter gesture and preserves
the exact frozen v1 behavior. It does not claim percussive hardness, universal
source quality, the complete all-lane P023 Golden Path, Holdout evidence, demo
readiness, or release readiness. Any mechanism change requires a new version
and Decision-Log entry rather than post-qualification retuning.

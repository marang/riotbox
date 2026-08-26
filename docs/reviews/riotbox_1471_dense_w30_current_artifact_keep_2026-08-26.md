# RIOTBOX-1471 Current-Tempo Dense W-30 Foundation Review

Date: 2026-08-26
Status: bounded human keep
Owner: RIOTBOX-1471

## Outcome

The current ordinary promoted W-30 artifact remains musically usable as the
single W-30-led Dense foundation before support-lane stacking. This is a
maintenance/regression confirmation of an established transformation, not a
new audible mechanism or new product behavior.

The listener recognized that the transformation was the previously heard
ordinary W-30 control and confirmed that the current exact artifact remains
usable. The structured review records `human_verdict: keep`, with the chop as
the established strongest element and source character transformed but still
present. The hook-after-two-bars dimension remains `inconclusive` because it
was not separately reassessed in this narrow regression review.

## Why A Fresh Review Was Required

RIOTBOX-1470 correctly rejected direct verdict reuse because the current audio
and product-manifest hashes differ from the RIOTBOX-1444 control. Source-blind
reconstruction under RBX-340 then established that the difference is
intentional: RBX-324 makes the explicitly accepted `130.0 BPM` the current
Session, replay, and Runtime truth. Reverting to the earlier analyzer estimate
of `130.28494262695312 BPM` merely to recover an old hash would be incorrect.

The musical mechanism did not change. The old control is 5.986875 seconds and
the current control is 6.0 seconds, a 0.218707% tempo difference and a 13.125 ms
duration difference. RIOTBOX-1471 therefore asked only whether the current
exact artifact remains usable; it did not present the artifact as a new sound.

## Exact Evidence

- Frozen review contract v1:
  `docs/benchmarks/dense_w30_current_artifact_review_v1.json`
- V1 contract SHA-256:
  `e674807ba5c5fdb9572f50d82f2a1a8975bfc92960e655b19605d6780bbb86d3`
- Frozen source-copy correction v2:
  `docs/benchmarks/dense_w30_current_artifact_review_v2.json`
- V2 contract SHA-256:
  `18f531c5fdbf899058d173c8cd839cdf8533775f4f51e1c7259f0933a2293e5a`
- Completed v2 access log:
  `artifacts/development/riotbox-1471/access-log-2026-08-26-b.json`
- Access-log SHA-256:
  `0b2f1f32d3bec5bf752941c867215867f577da6929290f32a531c24faa8d4d14`
- Passed technical preflight:
  `artifacts/development/riotbox-1471/review-v2/preflight.json`
- Preflight SHA-256:
  `f6cd4d7e076ab75066a629010d6d4c6de4f1b243c36d17ee6dc24e2278b743a6`
- Exact registered source SHA-256:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Exact current candidate SHA-256:
  `baccaa2dcff86e2965571ed3ba4dd4443904fa7c8f3e6b3bb9fd02725637627c`
- Structured listening review:
  `artifacts/audio_qa/local/listening-reviews/RIOTBOX-1471/review.json`
- Structured-review SHA-256:
  `ab96615185dd17758dd4742fc7a51d065241d344415809242833e9456aa2f802`

The failed v1 preflight consumed its one registered Development open before its
temporary runner mishandled the safe accessor return shape. The separately
frozen v2 recovery then consumed its one final authorized open of that same
exact source. Neither session performed directory discovery, opened Holdout
audio or a commercial reference, or rerendered the candidate. V2 created
byte-identical source and candidate presentation copies. Preflight confirmed
48 kHz stereo PCM16, 288000 frames, six seconds, non-silence, zero clipped
samples, peak 0.399871826171875, and RMS 0.06916871855342675 for the candidate.
Playback ran source first, then one second of silence, then the exact candidate;
the playback processes ended and post-playback silence was verified.

## Claim Boundary

This keep establishes one bounded `dense_break` positive-family foundation only
for the exact current W-30-led product path and this registered Development
case. It does not by itself establish a complete demo-ready Dense family. It
grants no new-mechanism, source-general, Holdout, percussive-hardness,
support-lane-quality, automatic-arrangement, universal-quality, release-ready,
or P023-completion claim. Further work must target the next genuine Golden Path
product gap rather than replaying this unchanged control as new musical work.

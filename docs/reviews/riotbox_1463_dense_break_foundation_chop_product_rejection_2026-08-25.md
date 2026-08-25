# RIOTBOX-1463 Dense-Break Foundation Chop Product Rejection

Date: 2026-08-25

Work class: `audible_vertical_slice / product_qualification`

Outcome: frozen v1 rejected after complete technical qualification and one
formal product-path listening review

## Frozen Boundary

RIOTBOX-1463 rebuilt only the provisionally kept
`dense_break_foundation_chop_v1` from RIOTBOX-1462. The immutable recipe keeps
beats zero through five, maps source beat four onto beat six, maps the second
half of source beat five onto the first half of beat seven, then returns to the
exact original second half of beat seven. Every replacement uses a 2 ms linear
splice. No mapping, splice, source identity, source order, threshold, or claimed
role changed after source access.

The qualification used exactly three registered Development sources from three
authors and performed no source-directory discovery. No Holdout audio or
commercial reference was opened.

## Fail-Closed Sessions

The audit retains all three sessions:

1. v1 opened only the exact Beat03 source and stopped before rendering because
   the preregistered WAV width said 16-bit while the hash-matched file is
   24-bit PCM.
2. v2 corrected only that metadata, passed the 130 and 135 BPM cases, then
   stopped because the 172 BPM case exposed a one-sample restart divergence
   between rounded 44.1 kHz source phrase time and 48 kHz output time.
3. v3 derived callback start phase from elapsed output frames at the exact
   confirmed BPM and reran all three sources from a fresh exclusive access log.
   All technical gates passed.

These corrections changed neither the frozen musical mapping nor its sources.

## Technical Matrix

| Case | BPM | Result | A/B delta RMS | Clips / limiter | Replay, restart, callback partitions |
| --- | ---: | --- | ---: | ---: | --- |
| `dense_beat03_130` | 130 | pass | 0.016211 | 0 / 0 | exact |
| `freesound_alastair_pursloe_183441` | 135 | pass | 0.033595 | 0 / 0 | exact |
| `freesound_dabromusic_266735` | 172 | pass | 0.016717 | 0 / 0 | exact |

For every case, the product projection matched an independent implementation
of the frozen mapping. Capture lineage and the typed recipe survived Session
round-trip; promotion and trigger were observer-visible; 128- and 257-frame
callback partitions were sample-exact; restart recovered the exact transport
phase; and missing source, fallback music, and all other product lanes emitted
zero active samples.

The qualification also exposed a general timing-truth defect: explicit 130 BPM
confirmation was admitted against a nearby analyzer estimate but restore could
later consume the estimate instead of the accepted value. The retained product
correction persists `confirmed_bpm` in the confirmation action and Session
runtime state, restores it through replay, prefers it for trusted timing, and
clears it with grid reversion. This correction is independent of the rejected
chop.

## Formal Human Review

The exact 24.153875-second review presentation was 48 kHz stereo PCM16 in this
order:

1. bounded registered source context repeated twice;
2. one second of exact digital silence;
3. A, the exact product straight-control render, repeated twice;
4. one second of exact digital silence;
5. B, the exact frozen v1 product render, repeated twice.

One uniform presentation-only gain of -2.3175 dB covered source, A, and B. The
conservative four-times true-peak estimate reached -1.2 dBTP, sample clipping
was zero, and every composite segment matched its separately emitted review
section. Playback completed at the announced endpoint and `pw-play` was absent
afterward.

The musician found the two-bar chop clearly audible, the source still
recognizable, and the underlying idea otherwise usable. B nevertheless lost
clarity and acquired an unintended band-limited, radio-like character. The
structured result is therefore:

- `human_verdict: technically_ok_but_musically_weak`
- `strongest_element: chop`
- `source_recognition: source_transformed_but_present`
- `hook_after_two_bars: clear`
- `demo_readiness: blocked`

## Technical Interpretation Of The Verdict

No radio filter, EQ, grit stage, playback-rate change, or additional lane
caused the result. The product implementation only moved source-native slices.
In the changed last two beats, B carried about 20.20 dB less 20–200 Hz energy,
2.63 dB less 200 Hz–2 kHz energy, and 1.64 dB less 2–12 kHz energy than A. In
the most affected half-beat, B was 13.85 dB quieter and its spectral centroid
moved from approximately 282 Hz to 889 Hz. The final half-beat returned
sample-exactly, and step-size analysis found no new splice discontinuity.

The audible defect is therefore the fixed answer-slice choice: it selects a
locally thin, quiet region and creates a mid-forward loss of weight and
clarity. This violates v1's frozen `groove_or_clarity_loss` stopping rule even
though the larger anchor/deviation/return topology remains promising.

## Consequence

Frozen v1 is rejected and must not be tuned from this consumed result. Its
product recipe, Feral v3 preset exposure, two-bar qualification control,
callback specialization, and temporary qualification runners are removed
before merge. The frozen contracts and negative evidence remain durable so the
same mapping is not reopened as a scalar adjustment.

Any successor must be a new Linear-first slice with a new version and Decision
frozen before fresh source access. It may preserve the sparse
anchor/deviation/return idea, but it must use a materially different,
clarity-preserving source-native slice-selection rule rather than EQ, gain
compensation, threshold fitting, or an exception for Beat03. No Holdout,
source-general, hardness, demo, quality, release, or P023-completion claim is
authorized.

## Evidence Identities

- frozen mechanism contract: `79671dd532459d2c4dd25636a989c3c088e515555eae1936fa7145e5aef3b2a6`
- frozen qualification v3 contract: `d1cc7c08d66f27c38f762e3286c40c6acb4a498cc2263814417ca961337c0af6`
- failed v1 access log: `f637c726ad1aacfeb3d3c5d2a9262628f3dd5b8eb7e20697f19fe9ebe0c7a58a`
- failed v2 access log: `58aeea16df41f7240341e2f264186bb77e05e8bd7123e6e2f25fc6de13933b2b`
- passed v3 access log: `a5371fbe60e987a1b8d38d4e00ad8880058f8b1cdf05dac79806a9fee222eec7`
- representative qualification report: `3bec8f72966072cb7b48228bb548ff7def182aa8856233a12a731234289f1afe`
- representative exact A control: `150e0ab2e3e741448600aff9834e48c45fbdf3ae0e23fc0762c8b48f2dfec58d`
- representative exact B v1: `8693e03ec452bae61b5fc490e0d9412b73e59736fd818b9b9a5ca809c8036739`
- formal review preflight: `5c01baf4f74e62019d58227c9b7917dbca8e34ebb8dfe99b00486a3f62995cc6`
- formal review presentation: `e17cde5b3f92b7cd20c5f56c7965b0ac66600243248de908005265f03d946dd9`
- structured human verdict: `1a21fd70e071a8d821b893a03bff073e58de54bdd595a7ed9ad033d176726aa7`

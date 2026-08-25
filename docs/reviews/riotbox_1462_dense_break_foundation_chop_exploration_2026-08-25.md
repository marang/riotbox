# RIOTBOX-1462 Dense-Break Foundation-Chop Exploration

- Review date: 2026-08-25
- Render/access session: 2026-08-24
- Stage: `development_exploration`
- Variant count: `1 / 3`
- Human result: `provisional_keep`
- Product integration: `false`

## Question And Prior Failure

RIOTBOX-1461 established that the current Dense sectional intent was
understandable but its foundational transformation was not musically usable.
The base used twelve 90 ms micrograin hits per bar, six source offsets, four
reversed hits, and simultaneous W-30, TR-909, break-snap, MC-202, drive, and
slam processing. Its rebuild-only chop section measured high-band ratio
`0.743797` and transient score `1.177140`, compared with the source-derived
policy probe at `0.142889` and `0.290990`. The human rejection therefore
outweighed its automated source-character pass.

RIOTBOX-1462 asked only whether one sparse source-only foundation could retain
recognizable groove and clarity while creating a useful two-bar hook. It did
not test the higher-level modification, pressure, hardness, arrangement, or
demo value.

## Source-Blind Mechanism

Variant v1 was implemented and synthetically checked before source access. It
keeps the first six beats sample-exact, substitutes source beat 4 for beat 6,
uses the second half of source beat 5 for the first half of beat 7, and returns
to the exact original second half of beat 7. The two substitutions use 2 ms
linear boundary crossfades. There is no reverse, granular micro-hit pattern,
support instrument, additive layer, pitch/time change, or bus effect.

The kept recipe is frozen as
`riotbox.dense_break_foundation_chop.v1` in
`docs/benchmarks/dense_break_foundation_chop_v1.json`, SHA-256
`79671dd532459d2c4dd25636a989c3c088e515555eae1936fa7145e5aef3b2a6`.

## Bounded Development Access

Only registered Development case `dense_beat03_130` was opened:

- path: `data/test_audio/examples/Beat03_130BPM(Full).wav`;
- SHA-256:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`;
- format: stereo PCM24, 44.1 kHz, 3.692313 seconds.

The ignored access log is
`artifacts/development/riotbox-1462/access-log-20260824T200824Z.json`,
SHA-256
`975cb5e3f4e3641d1368788dea882b60b026b7a9f53e84dbe13542a3677c41df`.
Both active Holdout registries were checked for identity collision only. No
Holdout audio, commercial reference, source directory, or alternative
Development file was opened.

## Exact Artifact Preflight

The 14.769342-second source and candidate presentations are stereo PCM16 at
44.1 kHz, repeat the exact two-bar phrase four times, and carry one uniform
presentation-only safety gain. Independent FFmpeg measurements found:

| Artifact | SHA-256 | LUFS | LRA | true peak | clipped samples |
| --- | --- | ---: | ---: | ---: | ---: |
| source | `b8243a1678a36b3f5c0a82cb6427eaad81b92a0934c76ab9614027d0d9abdbe0` | -17.54 | 0.60 | -1.38 dBTP | 0 |
| candidate v1 | `4813485ae93fe44f589523ca700a904371f908814a05f1226037371a655294ea` | -17.95 | 0.90 | -1.38 dBTP | 0 |

The candidate changes `18.75%` of phrase frames, has normalized delta RMS
`0.315095` and waveform correlation `0.949441`, and preserves the source's
high-band ratio (`0.083042 -> 0.083300`). All inspected splice boundaries are
continuous with no anomalous sample step. The small `0.41 LU` level difference
does not give the candidate a loudness advantage.

## Human Usefulness Check

After exact-artifact preflight and fresh readiness, the source was played for
14.77 seconds, followed by one second of silence and the candidate for 14.77
seconds. Playback completed and no player remained active.

The listener gave v1 a provisional keep: the foundational transformation was
musically usable, retained groove and clarity, and its second-bar answer formed
a useful hook. No second or third variant was generated because the first
materially distinct topology answered the exploration question.

## Promotion Boundary

This result selects exactly one mechanism for later qualification; it is not a
formal product verdict. RIOTBOX-1463 owns the product-spine/source-diversity
qualification. The exploratory WAVs cannot enter the human-label
corpus, demo bank, release queue, or product-quality evidence. The successor
must rebuild v1 source-blind through the product spine, preregister its
Development diversity set, prove replay/callback/exact-output behavior, and
run formal structured listening without tuning v1 from source results. A
required mapping, splice, timing, or ownership change requires a new version
and Decision.

No source-general, hardness, Holdout, demo, release, or P023-completion claim
is made.

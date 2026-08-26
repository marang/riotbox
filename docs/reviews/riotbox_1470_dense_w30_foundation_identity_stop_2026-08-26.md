# RIOTBOX-1470 Dense W-30 Foundation Identity Stop

Date: 2026-08-26  
Status: failed closed before human playback  
Owner: RIOTBOX-1470

## Outcome

The current ordinary promoted W-30 path passes its runtime, safety, persistence,
callback-partition, restart, isolation, and missing-source gates, but it is not
byte-identical to the positively reviewed RIOTBOX-1444 control. The frozen v1
stopping rule therefore rejects verdict reuse. No audio was played, no new
human verdict was requested, and the temporary qualification seam is removed.

This is an identity-qualification failure, not a human rejection of the current
sound. Dense remains without a qualified positive demo-family foundation.

## Bounded Development Session

- Contract: `docs/benchmarks/dense_w30_foundation_qualification_v1.json`
- Contract SHA-256:
  `fa49cce7617d2445328a98e225e5756110907cdabe4828a151a25129b9d11715`
- Access log:
  `artifacts/development/riotbox-1470/access-log-2026-08-26-a.json`
- Access-log SHA-256:
  `238a19378ade5c1bf9c94e4450f10a3374cf9eedc9b515424fb7fe65cd750eac`
- Qualification result:
  `artifacts/development/riotbox-1470/qualification-v1/dense-w30-foundation-qualification.json`
- Qualification-result SHA-256:
  `a4c3a6d25a900a26748906bebe26434a8820d6ae26817b12bdb74dd8f9a8fb40`
- Current product-manifest SHA-256:
  `3906af09b1bce218baf973cdd2bd3e054eefc12ece7b637f7f95e364e64dca1f`
- Current audio SHA-256:
  `baccaa2dcff86e2965571ed3ba4dd4443904fa7c8f3e6b3bb9fd02725637627c`

The session opened exactly the registered Development file
`dense_beat03_130` with expected SHA-256
`e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`.
It performed no directory discovery and opened no Holdout audio or commercial
reference.

## Gate Result

Passed:

- exact registered source identity and PCM format;
- exact six-action committed W-30 product path;
- one capture lineage and persisted Session/Source Graph;
- W-30-only isolated contributor set, with TR-909, MC-202, and Source Monitor
  staying out;
- sample-exact 128/257-frame callback partitions;
- sample-exact restart;
- missing-source silence;
- non-silent output;
- zero pre-limiter clips, limiter intervention, or post-limiter clips.

Failed:

- required audio SHA-256
  `7140c8f24e383dc6a7cb75bc6183e03727ef8b5f068b28e9d08ead8371a5ebab`;
- required product-manifest SHA-256
  `6b593663a24ae130e2352ea1dcbe09489ba86ac3f32d8efb68b6ac7c4709c69a`.

The persisted manifest diff identifies three exact drifts: the current timing
confirmation action carries `confirmed_bpm: 130.0`; the current W-30 render
uses `130.0 BPM` instead of the prior confirmed `130.28494262695312 BPM`; and
the resulting RMS/audio identity differs. This record does not infer that tempo
is the sole audible cause. It proves that the current and reviewed product paths
are not identical and that the source-timing authority needs a separate
source-blind product correction before another qualification.

## Claim Boundary And Handoff

RIOTBOX-1470 grants no human keep, Dense demo-family success, source-general,
Holdout, hardness, release-ready, universal-quality, automatic-arrangement, or
P023-completion claim. A successor must be Linear-first and reconcile one typed
confirmed source-tempo authority through Session, runtime, replay/restore, and
observer surfaces before any new Development access. It may not retune v1,
tolerance-match its hashes, or reuse this consumed session as fresh evidence.

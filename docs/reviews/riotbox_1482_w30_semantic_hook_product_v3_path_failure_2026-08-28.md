# RIOTBOX-1482 W-30 Semantic Hook Product V3 Path Failure

Date: 2026-08-28
Status: technical fail-closed; V3 closed; no human playback
Owner: RIOTBOX-1482

## Outcome

The frozen V3 product qualification stopped on its first exact Development
case. No second source was opened and no human playback occurred. V3 correctly
selected an undamaged ordinary post-trigger state and rejected active
capture-scoped damage, but its output still did not equal the provisionally
kept hook loop.

The remaining mismatch is now an exact product-path ownership error. The kept
RIOTBOX-1471 artifact is bound to the six-action
`ordinary_promoted_w30_control_v1` path produced by
`w30_live_path_render`: timing confirmation, preset activation, capture length,
capture, promotion, and W-30 trigger. V3 instead prepared its Session through
`dense_break_live_path_render`. Before export that path had additionally
committed raw-capture audition, MC-202 instigator generation, and three Source
Monitor changes, and it ordered preset activation after capture and promotion.
Isolating only W-30 at render time does not make those distinct state histories
the same semantic product path.

## Exact Evidence

- Frozen V3 qualification contract SHA-256:
  `a92fbadc60a8dbde037e0c17dc0dad372733c0988cf70457f4109774ff30f474`
- Failed V3 access-log SHA-256:
  `613aaae86466b595fbcfd24fd78842c101a4b40e7b409c3ebab037319ff5cf0e`
- Exact source SHA-256 observed through the single authorized original open:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Expected kept loop SHA-256:
  `ab3534e4ad2d17f9b73e2c93417660f61f943bddd2ecba36eed30af08e772faa`
- V3 product-written loop SHA-256:
  `f9331f41d0251e96ec8c797d6162eb12403c0852e77f54990286830b6ab3f659`
- V3 product summary SHA-256:
  `ee2457975d2d623224fbc2f87fb7cb575cd9511621ad1beaaeefecf181d84f5f`
- V3 Session SHA-256:
  `297f5015c48242822678e214860eee407ffe3ec703a59eab8aa3703a610a9fe5`

Both loops are 48 kHz stereo PCM16 with 177231 frames. Expected and V3 PCM
SHA-256 values are `da06cb7695fc3d8438722d90d27ee0ec051229360444ffe4d9d0194c185865b9`
and `e12b86859636c89f5289f4c0baa2e2b02383f2a272a406838a5db4564f09e666`.
Their normalized correlation is `0.2429252123`, delta RMS is `0.0847654058`,
and maximum absolute normalized sample delta is `0.5948181152`. The mismatch is
real audio, not container metadata or quantization noise.

## Stopping Decision

V3 is closed and must not be rerun, retuned, or reviewed. Three source sessions
each stopped after the first registered case and before any second source or
playback. Another in-place qualification attempt would repeat the same
course-error pattern.

Any future attempt must be separately versioned and start source-blind from
the exact existing six-action `w30_live_path_render` owner instead of adapting
the Dense journey harness. It must prove synthetic action-path identity before
fresh Development access. This branch does not authorize that V4 expansion.
V1, V2, and V3 grant no musician surface, source-general, Holdout, release, or
Riotbox-completion claim.

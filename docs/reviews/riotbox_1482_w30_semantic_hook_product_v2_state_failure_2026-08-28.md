# RIOTBOX-1482 W-30 Semantic Hook Product V2 State Failure

Date: 2026-08-28
Status: technical fail-closed; no human playback
Owner: RIOTBOX-1482

## Outcome

The frozen V2 product qualification again stopped on its first exact
Development case. No second source was opened and no human playback occurred.
Changing only the RuntimeMix transport position from beat zero to beat eight
did not reproduce the kept loop.

Inspection of the committed V2 Session identifies the actual missing state
gate: the qualification harness invoked the semantic export after completing
the later changed-return journey. That Session contains committed
`W30ApplyDamageProfile` action `16` at intensity `0.82` for `cap-01`, and its
active W-30 grit is `0.82`. The export action admitted this damaged live-recall
state because its V2 validation checked mode, routing, capture, source profile,
pad playback, and hook articulation, but not the capture-scoped committed
damage profile. The established ordinary control is captured earlier,
immediately after the W-30 trigger and before Slam, Fill, Scene, and changed
return.

## Exact Evidence

- Frozen V2 qualification contract SHA-256:
  `e37d911ba678b39149a29d8f69811e942c5bd501b2eb618e3067a048130644a8`
- Failed V2 access-log SHA-256:
  `76e680ddd072a019cc40eef64c268777c7cf6f279ee25fd4f9bf097e36c5c0a8`
- Exact source SHA-256 observed through the single authorized original open:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Expected kept loop SHA-256:
  `ab3534e4ad2d17f9b73e2c93417660f61f943bddd2ecba36eed30af08e772faa`
- V2 product-written loop SHA-256:
  `4c45cc7385977a498edf923ef86830bb74f00f46990259db810cc6df00fb654b`
- V2 product summary SHA-256:
  `fe0b6010941caff75d0048ad34b8b0ee099afef6b2f547b9fe6385016e807356`
- V2 Session SHA-256:
  `2f2846902bf641f997483e8e1de88a9eb1949a6fc721eb1ecac8e06ab65514e2`

V1 and V2 damaged-state outputs correlate at `0.9996796517` and differ by
only `0.0024611513` RMS, while V2 correlates with the kept ordinary loop at
`-0.2368965854`. V2 therefore changed phase within the wrong damaged state; it
did not correct state ownership.

## Access And Claim Boundary

The exclusive V2 log was created before access. Only
`dense_beat03_130` was opened, exactly once; the renderer used a private
hash-identical owner copy. No source directory, Holdout audio, commercial
reference, second source, or human playback was accessed. V2 remains failed
and frozen. A further attempt must use a new product boundary and qualification
version, reject active capture-scoped damage, and invoke the action from the
ordinary post-trigger Session state.

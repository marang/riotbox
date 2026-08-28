# RIOTBOX-1482 W-30 Semantic Hook Product V4 Acceptance

Date: 2026-08-28
Status: accepted at the frozen V4 product boundary
Owner: RIOTBOX-1482

## Outcome

The exact semantic role `w30_hook_loop` is qualified through
`stem_package.w30_hook_loop_v4`. The product action runs only from the
established `w30_live_path_render` owner after exactly six committed commands:
timing confirmation, preset activation, capture-length selection, bar-group
capture, pad promotion, and W-30 trigger. The existing Action, Session, atomic
writer, receipt, replay, observer, and stem-package QA paths own the result.

All four frozen Development cases passed without changing source selection,
audio behavior, algorithms, thresholds, roles, or contributor assignments.
The Dense product WAV is byte-identical to the previously kept Development
hook. Every case produced byte-identical independent owner and product-writer
audio, retained exact source/timing/capture lineage, completed the observer
lifecycle, remained non-silent, and reported zero full-scale samples, clips,
or limiter intervention.

## Reconciliation Boundary

The original V4 wrapper stopped after all four outputs had already been
generated because it incorrectly required the analyzer hypothesis id for the
registered manual-downbeat tonal case. The failed access log remains immutable.
The separately frozen reconciliation checked the unchanged files and accepted
the exact Action and Session manual-grid identity required by V4. It reopened
no original source, discovered no source directory, rendered no audio, and
accessed neither Holdout nor commercial-reference audio.

- V4 contract SHA-256:
  `459567e3990707f19d58f3cf551c4099e5ebbdfb814909517377e56739ccf996`
- immutable V4 access-log SHA-256:
  `9c9ed2f6bda02b645bab98306b85fab7e6737b8e72daf549f1e6bb609daba52c`
- reconciliation contract SHA-256:
  `8e71af16471e26d766cee34d96d1730da5154d17224b69aa9781ed1bf1755966`
- reconciliation report SHA-256:
  `120f305b2ce3c0fd59a4b7fa79896db5d8cc19f1489a1ccabb883f385a059a59`

Qualified product WAV SHA-256 values:

- `dense_beat03_130`:
  `ab3534e4ad2d17f9b73e2c93417660f61f943bddd2ecba36eed30af08e772faa`
- `freesound_alastair_pursloe_183441`:
  `f9c5b7c203fab1055f27f41e2bbf36a3183eeb6ee3e9efcb8dcc74ae15211b8c`
- `freesound_dr_skitz_353853`:
  `ed4723ad8bb86c736f3e4e54d994cafce6a8984207e76eb4c7e796109e5463ea`
- `tonal_rusharp_120`:
  `9ff3ef7e21ebaa769f2b2e85cf9cbc65c7894c49ab25530b6dc40885a6810542`

## Human Review

The exact 21-second artifact presented the tonal source context twice, one
second of silence, and the exact product-written tonal W-30 hook three times.
Its SHA-256 is
`314934c5dcef03a8a1e7f9ca74cd49949f52640d82ef86cc4227bb324c62e98e`.
Technical preflight confirmed 48 kHz stereo PCM16, exact segment layout,
sample-exact product-WAV inclusion, integrated loudness `-13.6 LUFS`, true peak
`-1.6 dBFS`, and no full-scale samples. Playback ended in verified silence.

The structured human verdict is `keep`. The tonal source remained transformed
but recognizable, and the semantic hook role remained honest on the distinct
source family. The listener had already accepted the transformation and its
loop utility during the bound Development review; after the product-path scope
was clarified, no duplicate loopability preference was requested or inferred.

- structured review SHA-256:
  `dc61bcca3953b305fad3f21d8466d8ddf002115bdcb73512e828e6d6f21aaa41`
- structured summary SHA-256:
  `866eb8e8f4b54b135afaa306d40b5569f3b6fe1810136c914a7c2a7f4f1a2bcf`

## Claim Boundary

This acceptance qualifies one source-recognizable W-30 hook role and its
operator product export boundary. It does not claim a new audio mechanism,
complete track, automatic arrangement, DAW placement, TUI or Ghost readiness,
percussive hardness, Holdout authority, release readiness, universal source
quality, or overall Riotbox completion. No further RIOTBOX-1482 source access,
rendering, tuning, or listening is required.

## Branch Review And CI

The final five-lens branch review found one high-confidence fail-closed gap:
the product action initially checked only the six committed command names, so
a Session with different parameters or stale capture ownership could enter the
V4 writer. The product boundary now validates the exact user actions,
quantization, parameters, targets, confirmed timing owner, active preset,
capture-length state, and capture-creation lineage before rendering or writing.
Dedicated negative tests prove that altered trigger parameters and stale
capture lineage leave no package or receipt.

The stricter action boundary passed six focused Rust tests and a source-blind
rhythmic end-to-end V4 render with callback-partition and owner/writer identity
checks. `just ci`, `cargo fmt --all -- --check`, Python syntax checks, all seven
contract JSON parses, frozen V4/reconciliation hashes, and `git diff --check`
pass. The review has no remaining correctness, product-spine, replay, observer,
realtime, Rust, documentation, or evidence blocker.

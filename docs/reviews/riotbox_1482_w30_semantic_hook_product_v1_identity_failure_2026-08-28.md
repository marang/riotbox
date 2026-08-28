# RIOTBOX-1482 W-30 Semantic Hook Product V1 Identity Failure

Date: 2026-08-28
Status: technical fail-closed; no human playback
Owner: RIOTBOX-1482

## Outcome

The frozen V1 product qualification stopped on its first Development case.
The written `w30_hook_loop` package passed its product-path, Session, lineage,
receipt, observer, format, non-silence, clipping, and existing stem-package QA
checks, but it did not reproduce the exact provisionally kept Development
loop. No second Development source was opened and no human playback occurred.

This is not a weak threshold result. The V1 mapping is internally inconsistent:
it renders the ordinary W-30 path from transport beat zero, while the kept
Development loop is the first eight beats of the previously qualified
`ordinary_promoted_w30_control_v1` artifact whose frozen product path starts at
transport beat eight. The two WAVs have the same 48 kHz stereo PCM16 format and
177231-frame duration, but their PCM differs materially.

## Exact Evidence

- Frozen qualification contract SHA-256:
  `c24c8b0a72972955c30a1d205f26e211327feaef09b486925a17b813afe2991b`
- Failed access log SHA-256:
  `aeddddd12c124381a9ed542fb6e49765078ef45d51142f3f41966c6da542a1f1`
- Exact source SHA-256 observed through the single authorized original open:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Expected kept loop SHA-256:
  `ab3534e4ad2d17f9b73e2c93417660f61f943bddd2ecba36eed30af08e772faa`
- V1 product-written loop SHA-256:
  `9e71eb59ac1713915906468cf6676b836364f2fc14e6e80a2f3d18149fa5e446`
- V1 product summary SHA-256:
  `ba8c07c2c40cb24e6ef62a37c38342ef26255353bb68157e9e73a38cbd49eae0`
- V1 Session SHA-256:
  `4c3dcfb172545f380355af556ebe9475786e1399c38fd30c954818db9d8db0ff`

Both loops contain 177231 frames. Their PCM SHA-256 values are respectively
`da06cb7695fc3d8438722d90d27ee0ec051229360444ffe4d9d0194c185865b9`
and `dd2991a5393af86953a2ae9a635f4dbcfbc710a7129d7517af67edb0b2687dc3`.
Normalized sample correlation is `-0.2373043828`; expected-loop RMS is
`0.0693286240`, V1 product-loop RMS is `0.0974290868`, and delta RMS is
`0.1321440119`. This is therefore audible-content drift, not a WAV header or
container-only difference.

## Access And Claim Boundary

The exclusive V1 log was created before access. Only
`dense_beat03_130` was opened, exactly once; the renderer used a private
hash-identical owner copy. No source directory, Holdout audio, commercial
reference, second source, or human playback was accessed. V1 remains failed
and frozen. Any correction requires a new product boundary, qualification
version, and Decision before another Development session.

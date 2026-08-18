# RIOTBOX-1443 W-30 Pitch-Dive Four-Source Transfer Observation

Status: pre-source brief
Partition: Development only
Mechanism changes: prohibited
Holdout access: prohibited
Commercial-reference access: prohibited

## Question

Does unchanged frozen `w30_pitch_dive_v1` remain musically useful and
source-recognizable on four additional registered Development sources before
Riotbox pays for product-spine integration?

This is informative transfer observation, not formal product qualification.

## Exact Source Boundary

Create a fresh access log before opening any audio, then open only:

| Case | Family | Path | SHA-256 | Declared BPM |
| --- | --- | --- | --- | ---: |
| `freesound_alastair_pursloe_183441` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/01_dense_183441.wav` | `b342ee4a9412de14f460c2c295634c53801f2549c71bfc486644a1b02030abc9` | `135.0` |
| `freesound_dabromusic_266735` | dense break | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/05_dense_266735.wav` | `b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73` | `172.0` |
| `freesound_dr_skitz_353853` | sparse drums | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/09_sparse_353853.wav` | `e75e1e6248d07b63ad58b8ee74a35c8cac066db808ef3e5daf256f20a5ba858d` | `120.0` |
| `freesound_aikighost_19059` | electronic drums | `data/test_audio/external/RIOTBOX-1430/freesound-v3-pool/13_electronic_19059.wav` | `b959afcf9654d2654c003fa956485f569b917afec805b57f9816ca22db9d6b4c` | `120.0` |

Do not discover the containing directory. Stop before decode if any exact hash
does not match. Do not substitute a different source inside this session.

## Frozen Render

For each source, use the existing W-30 live product path to produce the ordinary
source-backed control. Apply RBX-299 unchanged: eight sample-exact control beats,
then the exact four-beat continuous rate curve `0.35 ^ progress`, continuous
cursor, linear interpolation, `0.15`-beat terminal fade, and explicit silent
final frame. Do not change curve, timing, fade, gain, source selection, grit, or
ownership after seeing or hearing any result.

Before playback require matching format and duration, sample-exact first eight
beats, a non-silent intended final-four-beat delta, no clipping, a click-safe
terminal fade, and silence after the endpoint. The only contributor is the
named source-backed W-30 render.

## Human Presentation

For each source, prepare one exact composite in this order:

1. source context
2. one second silence
3. A ordinary W-30 control repeated twice
4. one second silence
5. B frozen pitch-dive candidate repeated twice

Preflight each exact composite before its first playback and request fresh
readiness. Ask whether B is clearly transformed, retains recognizable source
identity, creates a useful exit, avoids groove or clarity damage, and would be
triggered live. Record one neutral source-bound verdict. A direct request to
replay the unchanged artifact is immediate and does not consume a new render.

## Stopping Rule

Any access, hash, timing, render, contributor, clipping, endpoint, or playback
contract failure stops fail-closed. Human rejection limits that source's
transfer evidence; it does not authorize per-source tuning. The result grants
no product, universal-source, hardness, Golden Path, Holdout, demo, or release
claim.

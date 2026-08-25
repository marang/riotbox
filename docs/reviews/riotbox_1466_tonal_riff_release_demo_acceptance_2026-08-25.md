# RIOTBOX-1466 Tonal-Riff Release-Demo Acceptance

- Date: 2026-08-25
- Partition: Development only
- Decisions: RBX-329, RBX-330
- Result: one `tonal_riff` release-demo family success

## Scope

RIOTBOX-1466 qualifies the unchanged exact tonal product journey for the
current live demo bank. It adds no effect, tuning, source-selection policy, or
musical fallback. W-30 owns the source-derived hook and `w30.pitch_dive`;
TR-909, MC-202, and Source Monitor remain silent during generated journey
stages by typed live-policy v2.

The v1 qualification contract SHA-256 is
`114dc61683e42b323df77d2783ecdb144e4bb1969015846da66e650590b75207`.
The exact registered Development source is `tonal_rusharp_120`, SHA-256
`ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`,
at confirmed 120 BPM with zero-second downbeat. The completed bounded access
log has SHA-256
`a1e5d995cf5982aeb14d49be3317d7befac392805f8599462f02e09687735d5d`;
it records one exact path, no directory discovery, and no Holdout or commercial
reference access.

## Technical Qualification

The exact product manifest, SHA-256
`28a95aae429361de50b3590e0feabf99f4426e35e1bdb43a818c006a2fe0b27d`,
passes Source Graph/capture lineage, Action Lexicon queue/commit,
Session/replay/restart, and exact RuntimeMix gates. The 128- and 257-frame
callback partitions match sample-exactly. The complete generated journey and
restart recall are sample-exact to W-30-only output; TR-909, MC-202, and Source
Monitor journey RMS values are all zero. Pitch Dive preserves its first eight
beats sample-exactly and produces active-tail delta RMS `0.141857`. No stage
clips or invokes the limiter.

The 29.0-second stereo PCM16 presentation is 48 kHz, `-16.0 LUFS`, and
`-2.7 dBTP`, with exact one-second pauses at 4-5 and 13-14 seconds and exact
terminal silence at 28-29 seconds. Its SHA-256 is
`24eca9572537d81d6ed87c61c13806a0c679092d8f8f73723e2015bfff490e6b`.
The qualification report SHA-256 is
`2b47167912886616683e6eeaf63f9533c4b4a153ec754a0d6613c63716aab5a4`.

The first validator invocation incorrectly routed this 48 kHz product artifact
through a 44.1 kHz helper after the renderer and post-render source check had
already succeeded. Recovery corrected format handling, revalidated the same
files, and reopened no source, repeated no render, and replaced no artifact.

## Human-Evidence Identity

Preflight established that both current product manifest and presentation are
bit-identical to the already accepted RIOTBOX-1454 evidence. The durable prior
review document, SHA-256
`59678762e51b1267f402393d2223006179aeb82566141c44a3739da0afa97ea2`,
binds structured-review SHA-256
`8c67d9a45c21e0e061906e1310c2fc64f790c9590aba4e3f51e687420c5365ea`
and records `keep`, strongest element `chop`, transformed-but-present source
recognition, and a clear two-bar hook.

RBX-330 freezes the narrow identity-reuse contract, SHA-256
`cfdab651ceae05a494ccee5637a5e4fc3fb47bef24901b4ca5e76531a402cfa0`.
The RIOTBOX-1466 playback that occurred before the duplicate was acted upon is
not a fresh verdict and creates no new quality evidence. The reconstructed
structured review preserves the prior dimensions and has SHA-256
`7bd430d972cbf07af88b48cfb04503e4d49d7526c17da16eaa1e7fa809f14f8a`.
No further playback is permitted or useful.

## Promotion And Boundary

The validated live demo bank, SHA-256
`d49891aff8efa244582ed833965c4b16ce8c34a2ebb151fb3bbf6c42e6df28d0`,
contains one new demo-ready entry,
`tonal-rusharp-exact-product-human-pass`, with no fix category. The live
source-family coverage report, SHA-256
`c504eac808d5a1bb3baf4359913b7275f6943f23392d95f341ba999bfeacad5d`,
marks canonical `tonal_riff` as `demo_ready_covered`.

This closes one positive family only. `dense_break` and `sparse_drums` still
lack positive family success, so release readiness and P023 completion remain
blocked. Nothing here grants source-general, Holdout, universal-quality, or
release-ready status.

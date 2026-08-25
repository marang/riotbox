# RIOTBOX-1467 Sparse-Drums Release-Demo Acceptance

- Date: 2026-08-25
- Partition: Development only
- Decisions: RBX-331, RBX-332
- Result: one `sparse_drums` release-demo family success

## Scope

RIOTBOX-1467 qualifies the unchanged exact sparse product journey for the
current live demo bank. It adds no effect, source-selection rule, timing rule,
mix change, or musical fallback. The frozen product policy assigns W-30 the
source transformation, TR-909 the hardest transient, and MC-202 secondary
punctuation; Source Monitor stays out and bass ownership remains explicitly
unassigned. This is a direct `sparse_drums` product claim, not an invented
MC-202 bass-pressure claim.

The v1 qualification contract SHA-256 is
`03e7c6da9a9c28bb70f7a93ff35ef7d9280b0330cd66b95bf8b871e921923acc`.
The evidence-reuse contract SHA-256 is
`d0a658f12e75366d0243a230ddbb28af85746e0c7a5c601d3271b81ee5ed46c5`.
The one registered Development source is `sparse_kicksnr_120`, SHA-256
`8a970e5d7bd9b29771aba85f75e697c7510940d4404714bfb1e55e210c15f46c`,
at confirmed 120 BPM with zero-second downbeat. The completed bounded access
log has SHA-256
`0ac681427e3214677113d925473e004f715eb73467763c8f155efc06f10885e1`;
it records one exact source access, one render, no rerender, no directory
discovery, and no Holdout or commercial-reference access.

## Technical Qualification

The exact product manifest, SHA-256
`0d8359819210acd99cc2f49aeef999e80adca5fd9ef1d41f7994624c83fbc80d`,
passes Source Graph and capture lineage, Action Lexicon queue/commit,
Session/replay/restart, exact RuntimeMix callback partitions, capture-scoped
Transient Bite, ordinary re-entry, limiter, clipping, and no-fallback gates.
All active W-30, TR-909, and MC-202 contributors are sample-exact across the
frozen callback partitions. Source Monitor remains silent, restart recall is
sample-exact, and the lane-role map matches the typed product policy.

The 31.0-second stereo PCM16 presentation is 48 kHz, `-22.5 LUFS`, `9.3 LU`
LRA, and `-2.1 dBTP`, safely below the frozen `-1.0 dBTP` ceiling. Its SHA-256
is `64bb983b5fccdeced71b03c8d07bd031726a52995a60a6a89aeab8cda8f1c69d`.
The local qualification report has SHA-256
`1bae53456f8c85b7d3f4d6005fa028227813d9b90c5a21bb5d663287007ecf9c`.

## Fail-Closed Access Recovery

During branch validation, the legacy `demo-bank-promotion-fixtures` target
unexpectedly began rebuilding a professional source-WAV fixture and opened the
registered `tonal_rusharp_120` and `sparse_kicksnr_120` Development sources.
The process was terminated before the fixture completed. No source directory,
Holdout, or commercial reference was accessed; no musical or numeric output
was inspected or used, and every qualification artifact retained its frozen
hash.

The incident record has SHA-256
`3aef844e84bfcbdf047b50d78f7c69b6d210820c862ade18ca16ada5fc69ee66`.
RBX-332 freezes the recovery contract, SHA-256
`9d68f6d724610296d8a1e85bba9b767c4df8794abe69c36a3f4286a1cca870bd`:
the unrelated outputs are ineligible for RIOTBOX-1467 evidence, no further
source access/rerender/playback is permitted, and any identity mismatch
invalidates the qualification. The responsible promotion fixture now uses
small source-free hash-bound contract artifacts and passes its existing
positive and fail-closed mutation checks.

## Human-Evidence Identity

The current product manifest and presentation are bit-identical to the
accepted RIOTBOX-1455 evidence. Its durable review document, SHA-256
`d4969c33241ad27decde439b9c6ec241fbcbde8641824fee4b74c4457a4b200a`,
binds structured-review SHA-256
`7091d1699500857e5cde043fba0930409ede3848d170f999597507f20bd30184`
and records `keep`, strongest element `chop`, transformed-but-present source
recognition, and a clear hook.

RBX-331 permits reconstruction only for this exact manifest/audio identity.
The reconstructed structured review preserves those dimensions and has
SHA-256
`e637b6ba2b50ef78970c8ce51da67f83e67e2d76c8378053123814ff4b3c4d50`.
No playback occurred, and RIOTBOX-1467 creates no new verdict or musical
evidence.

## Promotion And Boundary

The validated live demo bank, SHA-256
`1a38d5e390924de06795f74736aa384951e67ba44294e53db54e4364a3ecaec1`,
contains the new demo-ready entry
`sparse-kicksnr-exact-product-human-pass`. The source-family coverage report,
SHA-256
`6b048cda5bcd560a5b8e68eb7d2f31d0df75f89fd27e1006be3b47f48a892df0`,
marks canonical `sparse_drums` as `demo_ready_covered`. The live review queue
is empty because every current bank entry already carries eligible human
evidence.

The aggregate readiness report, SHA-256
`c13f959fc23d35f96eee6dd797018149fea06dd2bea607de3779d68131c58324`,
now recognizes both direct `sparse_drums` and historical
`sparse_bass_pressure` aliases from one shared contract. It remains blocked:
`dense_break` is the only missing family success, the current human-rejected
Dense candidate still routes to `chop_policy`, and scripted professional-suite
diagnostics remain non-quality evidence.

This closes one positive family only. Nothing here grants source-general,
MC-202 bass-pressure, Holdout, universal-quality, release-ready, or P023
completion status.

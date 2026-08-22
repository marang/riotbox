# RIOTBOX-1454 Tonal Live Journey v1 Rejection

Date: 2026-08-22
Partition: Development only
Decision: RBX-315
Result: technically valid but musically weak; do not promote

## Scope

The review exercised the registered `tonal_rusharp_120` Development source
through the real capture, raw audition, promotion, Feral Break Alpha v2,
source-character policy, W-30 trigger, Pitch Dive, ordinary re-entry, and
save/restart/recall path. It did not access Holdout or commercial-reference
audio and did not qualify a new effect or universal source-family behavior.

The source was `data/test_audio/examples/DH_RushArp_120_A.wav`, SHA-256
`ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`,
with the registered manual 120 BPM / 0-second downbeat. The bounded access log
SHA-256 is `aa35db554a757cf1fc528aaa922a8f543195e274a587d1964f1b5f2abe36e2ed`;
it records one exact Development file, no directory discovery, and no Holdout
or commercial-reference access.

## Technical Result

The exact 33-second human-review artifact SHA-256 is
`cdaf203ce8ad1ded86632f7bfa25d5791a14ef3ee017ca2c81c22ca4e5c20bf3`.
Its technical manifest and structured review SHA-256 values are respectively
`0476a9b10515ff6ceba92ac909f5b662e3a86b787d2594c6f7675572e7866943`
and `0ad134ce6de1115067a102f4ea3338425fad3e00879d97f5d5c5acccae4b7789`.

The callback path passed 128-versus-257-frame sample equality, exact source and
manual-grid identity, capture/preset survival through restart, clean ordinary
re-entry, and clip-/limiter-free output. Isolated W-30 evidence kept the first
eight Pitch Dive beats sample-exact to the held state and produced a material
later delta. The held W-30-to-TR-909 RMS ratio was approximately `10.37:1`.
Timeline analysis exposed and corrected an earlier proof-only cursor mismatch
before this bound review artifact was rendered.

## Structured Human Verdict

The listener found the source and held W-30 transformation clear and useful.
The hook remained recognizable after two bars. The complete journey was still
`technically_ok_but_musically_weak` because the restrained TR-909 support pulse
added no useful pressure and became incoherently exposed after the W-30 exited.
The long presented tail then weakened the contrast further.

Durable interpretation:

- keep the source-derived W-30 tonal-hook transformation;
- reject the weak isolated support pulse in tonal held state;
- reject the long empty contrast tail in this journey;
- preserve the already-qualified Pitch Dive mechanism unchanged;
- prefer explicit TR-909 `stay_out` plus direct ordinary re-entry after the
  Pitch Dive's musically active window.

## Claim Boundary

This rejection supersedes no historical RBX-156 evidence and does not retune
its bytes. It motivates the new versioned RBX-315 policy contract before fresh
v2 rendering. It grants no demo, release, Holdout, universal tonal-source,
automatic-arrangement, bass-pressure, or overall P023 completion claim.

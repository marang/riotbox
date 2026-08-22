# RIOTBOX-1453 MC-202 Turn-Taking Development Stop

Date: 2026-08-22

Work class: `development_exploration`

Outcome: stopped as a redundant control; no new musical verdict or product
behavior

## Scope

RIOTBOX-1453 tested one deliberately restrained alternative after the
RIOTBOX-1452 over-layering rejection. Instead of processing the support bus,
the exact human-kept RIOTBOX-1451 MC-202 voice remained absent for one ordinary
W-30/TR-909 phrase and entered unchanged in the following phrase.

The exploration used only registered Development case
`freesound_cyclez_493560`, SHA-256
`4c17679b77bc78376282c5db118ef2fa9f87a5c669cf0cc284b91a38ad3fb485`,
through fresh exclusive access log
`artifacts/development/riotbox-1453/access-log-v1.json`, SHA-256
`5d1c49c5a3cb06808b773f319fbc97b898602027994031489c2a6efdd4d079e8`.
No source directory was discovered, and no Holdout or commercial-reference
audio was opened.

## Technical Result

The exact artifact was
`artifacts/development/riotbox-1453/turn-taking-v1/03_source_then_A_then_B_review.wav`,
SHA-256
`8c97f20f9ae072ade2029cbf60e934751038f1a3a434a7fc396707f5f9d847e9`.
It contained 6.4 seconds of source, one second of silence, 12.8 seconds of A,
one second of silence, 12.8 seconds of B, and 0.5 seconds of end silence.

A repeated the exact ordinary W-30/TR-909 control twice. B was sample-exact to
A for its first phrase and then reused the unchanged RIOTBOX-1451 MC-202
pressure voice in its second phrase. No stem was scaled, no support bus was
processed, and no timed W-30 gesture was active. The artifact measured
`-18.2 LUFS`, `-0.7 dBTP`, and zero clipped samples. The A/B delta was confined
to B's second phrase, matched the earlier MC-202 stem within one PCM16 least-
significant bit, and placed approximately `98.69%` of its measured spectral
energy between 35 and 160 Hz.

## Human Boundary

After exact-artifact preflight and fresh readiness, playback completed and the
host returned to silence. The listener immediately recognized the candidate as
the same periodic addition already assessed in the earlier MC-202 example: the
only new property was its entry after four bars. The presentation therefore
did not pose a materially new musical question.

Durable interpretation:

- no new `keep` or `reject` is assigned;
- the bounded RIOTBOX-1451 single-source keep remains unchanged;
- the RIOTBOX-1452 rejection of broad simultaneous processing remains
  unchanged;
- temporal gating of the same reviewed phrase is not sufficient follow-up
  evidence and should not trigger another listening exposure;
- no second RIOTBOX-1453 variant is justified.

## Stop And Follow-up

The temporary exploration script is removed. MC-202 pressure remains
`stay-out` for the current complete gesture journey; this does not remove or
redefine existing product behavior. Work proceeds to an explicit Foundation
Completion slice so remaining Golden Path gaps are inventoried and closed
before another audible effect detail is considered.

No product, source-general, Holdout, hardness, demo, release, or P023
completion claim follows.

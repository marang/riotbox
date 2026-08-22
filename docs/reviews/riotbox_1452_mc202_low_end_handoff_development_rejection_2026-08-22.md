# RIOTBOX-1452 MC-202 Low-End Handoff Development Rejection

Date: 2026-08-22

Work class: `development_exploration`

Outcome: stopped at the three-variant limit; no product behavior retained

## Scope

RIOTBOX-1452 tested whether the useful source-derived MC-202 pressure voice
found by RIOTBOX-1451 could transfer through phrase-local mix ownership rather
than additive stacking. The bounded exploration used only the exact registered
Development source `freesound_cyclez_493560`, SHA-256
`4c17679b77bc78376282c5db118ef2fa9f87a5c669cf0cc284b91a38ad3fb485`,
through fresh exclusive access logs. No source directory was discovered, and
no Holdout or commercial-reference audio was opened.

The MC-202 stem was never scaled. Every variant instead changed how the
existing W-30/TR-909 support yielded during source-derived MC-202 ownership.
The work remained outside the product spine and did not change the frozen
RIOTBOX-1451 contract.

## Variant Results

| Variant | Causal mechanism | Result |
| --- | --- | --- |
| 1 | Complementary energy-law low-band handoff. | Technical reject: it removed the additive baseline's `28` clips but still required `170` limiter interventions. |
| 2 | Transient-protected low-band sustain slot. | Technical reject: clipping was removed, but `54` limiter interventions remained. |
| 3 | Transient-protected center-bus sustain handoff that preserved stereo sides and support attacks. | Technical candidate gate passed with zero clips and zero limiter intervention; human reject because the complete B mix applied too many effects at once and felt over-layered. |

Variant 3 replaced only center sustain during MC-202 ownership; it did not mute
the full supporting bed. Its post-limiter peak was `0.88372`, RMS was
`0.25346`, and the handoff delta RMS was `0.02733`. Ownership was active for
approximately `57.93%` of analyzed frames, making the mechanism clearly
audible rather than a near-identity comparison.

## Exact Human Review

The technically preflighted review artifact was
`artifacts/development/riotbox-1452/low-end-handoff-20260821t1950-v3/03_source_then_A_then_B_review.wav`,
SHA-256
`1ac01d0fd1045d133e11cc50c9c960ad08bc6e4e1a2cdb0f3333c69a58b1c6cc`.
Its 34.535-second assignment was:

1. the exact registered source for 6.4 seconds;
2. one second of digital silence;
3. A: the frozen RIOTBOX-1451 additive-product render, repeated twice;
4. one second of digital silence;
5. B: the same material with the center-bus handoff, repeated twice;
6. 0.5 seconds of digital end silence.

After fresh readiness, playback completed and the host returned to silence.
The musician considered the available effect vocabulary valuable but rejected
this B balance because too many effects operated simultaneously. The verdict
does not reject MC-202 pressure itself; it rejects the broad, persistent
support-bus processing used to make that pressure own the mix.

Durable interpretation:

- `human_verdict: reject` for the tested center-bus handoff;
- the MC-202 voice remains a useful direction from RIOTBOX-1451;
- future work must reduce ownership density rather than add gain, limiter
  activity, or another stacked effect;
- no provisional keep, frozen mechanism, product behavior, source-diversity
  matrix, Holdout access, hardness, demo, release, or P023 completion claim is
  authorized.

## Stop And Follow-up

The temporary exploration script is removed. RIOTBOX-1452 stops at the
three-variant boundary and does not continue by tuning the rejected mechanisms.

Any successor must be a new Linear-first slice with one restrained ownership
change: preserve the MC-202 voice, leave W-30/TR-909 attacks and hook intact,
avoid persistent center-bus processing, gain escalation, limiter dependence,
and simultaneous effect stacking, and listen early. If that materially
different attempt does not earn a keep, MC-202 pressure must fail closed to a
documented `stay-out` role for this journey while foundation gaps take
priority.

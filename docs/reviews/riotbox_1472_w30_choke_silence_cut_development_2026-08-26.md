# RIOTBOX-1472 W-30 Choke / Silence-Cut Development Exploration

Date: 2026-08-26
Partition: Development only
Result: `provisional_keep`
Holdout access: none
Commercial-reference access: none

## Narrow Question

Can one dedicated W-30 silence cut remain immediately obvious, fit the phrase
musically, and return cleanly enough to justify product integration and formal
qualification?

This was one bounded musical exploration on the already kept current-tempo
Dense W-30 foundation. It did not add a product action, Session/replay state,
UI binding, support lane, fallback voice, source matrix, or new validation
framework.

## Source-Blind Duplicate Audit

The existing product has no dedicated W-30 choke action or Session articulation
profile. `W30HookTurnaround` combines a reverse pickup with a short-gated beat;
`W30PitchDive` ends in persistent silence until a later ordinary intent;
TR-909 choke behavior is drum-lane-owned; and `W30LoopFreeze` materializes a
capture rather than articulating silence. The explored automatic W-30 cut and
return is therefore a new causal gesture rather than a replayed control or a
renamed existing effect.

## Bounded Source Access

Before the source was opened, the exact v1 formula, source identity, technical
gates, review budget, stopping rule, and claim boundary were frozen in
`docs/benchmarks/w30_choke_silence_cut_development_v1.json`, SHA-256
`2781cdc38bce556653d8c7584dbc8807f60a20747b4b2fee6f11f8678a90bd3f`.

One fresh exclusive session opened only registered Development case
`dense_beat03_130` through the no-follow exact-path accessor:

- source SHA-256:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- format: stereo PCM24 at 44.1 kHz, `3.692313` seconds
- access-log SHA-256:
  `36f924cde151da2ba7256dd9f0d2c3fcc5920b43df3ff48b11b8652a7f316771`

The completed log records exactly one source open, no source-directory
discovery, and no Holdout or commercial-reference access.

## Exact Kept Mechanism

The candidate leaves the current ordinary W-30 foundation unchanged until a
5 ms quarter-wave cosine amplitude taper immediately before relative beat four.
It emits exact PCM zero from relative beat four through beat five, applies a
5 ms quarter-wave sine amplitude taper on return, and then resumes the advancing
ordinary control sample-exactly. TR-909, MC-202, and Source Monitor stay out.

| Artifact | SHA-256 | Duration | Peak PCM16 | Clips |
| --- | --- | ---: | ---: | ---: |
| A: unchanged W-30 control | `baccaa2dcff86e2965571ed3ba4dd4443904fa7c8f3e6b3bb9fd02725637627c` | `6.000000` s | `13103` | `0` |
| B: silence cut v1 | `bea401a5c71fef8ca5f47970466acb22488fa0375afbd351affdee1f1d733f24` | `6.000000` s | `13103` | `0` |

Both artifacts are stereo PCM16 at 48 kHz. Their only differing window is
`1.841187` through `2.312625` seconds. Exact PCM silence spans `1.846146`
through `2.307688` seconds, or one `130 BPM` beat. The prefix before fade-out
and the ordinary continuation after fade-in are sample-identical. Candidate
peak does not increase, clipped-sample count is zero, and the maximum boundary
delta (`738` PCM16) meets the frozen control-relative limit.

## Human Result

After technical preflight and fresh readiness, the exact verified source,
unchanged control A, and candidate B were played in that order with one-second
pauses. At the listener's direct request, the unchanged sequence was replayed
once without rerendering or reassignment. Both bounded playbacks stopped at
their declared endpoints and silence was verified.

The project musician found the difference clearly recognizable as a silence
cut and judged it musically fitting. This supports one provisional Development
keep for the exact v1 gesture. Source-recognition strength, hook-after-two-bars,
cross-source transfer, and formal live-trigger willingness were not separately
reassessed in this narrow early check.

The local structured review is bound by SHA-256
`6c289a066abe715e63a8a3cc56bbef29cfcf4f1cea6e7dcfcc0a79d112ebd437`.
Its generic demo consequence is inapplicable here: the P023
`development_exploration` boundary forbids demo or product promotion from this
artifact and verdict.

## Promotion Boundary

RBX-342 freezes the exact heard mechanism. RIOTBOX-1473 owns a separate
Linear-first, source-blind rebuild through queue/commit, Session/replay,
observer/UI, exact RuntimeMix, source-diverse Development qualification, and a
new formal human review. Qualification may reject v1 but may not tune its
timing, taper, ownership, gain, or return behavior from source results.

The temporary exploration runner is removed from the final tree so neither its
artifact nor its provisional verdict can be mistaken for product
implementation or qualification evidence. No source-general, Holdout,
percussive-hardness, automatic-arrangement, demo, release, universal-quality,
or P023-completion claim follows.

# Percussive Hardness Contract

Status: active P023 development contract
Owner: RIOTBOX-1428
Scope: source-backed W-30 events that explicitly own `percussive_hard`

## Musician Meaning

`percussive_hard` means the same recognizable hit at the same pitch and timing
feels as if it was struck more violently and arrives more forcefully.

The expected audible sentence is:

> The second event is the same hit, but its front edge and physical body hit
> harder.

It must not be described as harder merely because it is louder, lower-pitched,
darker, dirtier, more distorted, busier, doubled, delayed, or more technically
different.

## Required Properties

- **One aligned onset:** no duplicate hit, flam, echo, or timing displacement.
- **Pitch stability:** source playback stays at `1.0x`; pitch dive, rate damage,
  and transposition are separate performer roles.
- **Recognizable source:** the hit remains attributable to its selected source
  event even when timbre is transformed.
- **Immediate attack:** at loudness-matched comparison, the source-defined
  attack region must move forward perceptually and must not collapse into a
  thin click.
- **Physical body:** the source-defined post-attack body, normally within the
  first `20-120 ms`, remains present. A higher peak cannot hide body loss.
- **Retained bite:** midrange/body and edge energy must not be low-pass masked
  or traded for unrelated low-frequency weight.
- **Controlled tail:** the event may become tighter or denser, but must not
  sound hollow, truncated, or like missing PCM.
- **Human force verdict:** the musician must hear a meaningful increase in
  force. Metrics may reject a candidate but cannot award this verdict.

## Separate Roles

The word `hard` is not a universal DSP preset:

| Typed role | Musician-facing meaning |
| --- | --- |
| `percussive_hard` | stronger attack plus retained physical drum body |
| `hook_hard` | more midrange bite, roughness, and immediate riff presence |
| `bass_hard` | more controlled low-end impact from an explicitly assigned bass owner |
| `destructive_damage` | deliberate breakage, grit, rate movement, reverse, or aliasing |
| `pitch_dive` | explicit tonal/rate descent |

Passing one role never proves another. In particular, pitch descent and damage
do not prove percussive hardness, and absent bass pressure is not a failure when
bass ownership is unassigned.

## Development And QA Gate

Before another human review of `percussive_hard`:

1. Use one source-general algorithm on at least three contrasting legal
   development sources without filename branches or source-specific constants.
2. Prove playback-rate and onset invariants mechanically.
3. Compare raw and loudness-matched source/candidate events.
4. Analyze the exact attack, body, and tail regions rather than only the whole
   event. Reject attack collapse, body collapse, edge masking, clipping,
   duplicate onsets, and near-identity.
5. Reject any candidate whose main audible difference is pitch, loudness,
   darkness, dirt, or duration rather than force.
6. Present a bounded isolated A/B with no support lanes or hidden callbacks.
7. Require the human to confirm both `clearly different` and `actually more
   forceful`. `Different but not harder` is a reject.

Technical floors are collapse screens, not the semantic definition. Do not
lower or retune them against one source to manufacture a pass.

## RIOTBOX-1428 Negative Evidence

H31 Stage A v1 used a source-derived body playback rate near `0.78x`. It passed
timing, loudness-matched attack/body, crest, body-band, edge-band, and
cross-source difference gates, yet structured listening heard only a lower
second half and no increase in force. The recipe is rejected and may not be
promoted into RuntimeMix or TUI. Its lesson is that pitch stability is a
semantic prerequisite, not a spectral metric inferred after rendering.

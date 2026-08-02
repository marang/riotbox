---
name: riotbox-rave-punk-production
description: "Riotbox musical direction add-on for shaping the instrument toward aggressive sample-based rave-punk/breakbeat production values: hard hooks, chopped source material, physical drums, bass pressure, stabs, stutters, fills, drops, live triggerability, raw energy, and musician-facing taste checks. Use with `riotbox-development` when work affects audible character, patterns, slices, loops, drum/bass behavior, performance controls, presets, demos, or when the result feels boring, polite, generic, weak, identical, silent, or placeholder-like."
---

# Riotbox Rave-Punk Production

## Operating Rule

Shape Riotbox toward a specific production attitude, not toward imitation of any living artist or direct recreation of released songs. Use publicly recognizable production principles as taste pressure: hostile energy, sample transformation, hooks, physical drums, controlled chaos, and live performance impact.

The product question is: would this make a musician want to keep triggering, muting, cutting, and abusing the box after the first eight bars?

Use The Prodigy's full-era production arc as a quality reference class, not a
copy target: early rave break urgency, mid-era big-beat/punk attack, later
denser bass/drum pressure, vocal/stab hooks, harsh stops, and live-room impact.
Riotbox must create its own source-backed output and identity, but the bar is
that level of uncompromising pressure, hook clarity, and stage impact.

## Taste Model

Prefer results that feel:

- aggressive, compressed, loud enough, and physically present without becoming unusable
- sample-based but transformed: chopped, pitched, reversed, gated, filtered, resampled, or rhythmically recontextualized
- riff-led: one ugly, memorable stab or bass/riff gesture beats many polite variations
- beat-forward: kick, snare, break, ghost hits, choke, swing, and fill behavior should carry the machine
- punk in arrangement: hard entrances, dropouts, stops, shouts, crashes, abrupt mutes, and fast contrast
- playable live: gestures should have obvious stage meaning and immediate audible consequence
- raw but controlled: distortion, saturation, aliasing, and grit are valid only when they increase intent and impact

Avoid results that feel:

- generic EDM, ambient wallpaper, sterile demo loops, or polite preset browsing
- technically correct but hookless
- random without a performance logic
- over-quantized when the groove needs lurch, shove, or humanized break pressure
- full of motion but lacking a memorable central riff, stab, vocal hit, or drum identity

## Production Checks

For any audible Riotbox feature, answer these before calling it musically good:

- What is the hook after two bars?
- What hits hardest: kick, snare, bass, break, stab, vocal hit, or silence?
- What can the player do live that changes the room immediately?
- Where is the choke, stop, fill, or drop?
- Does the source material survive as character, or did the system collapse into a placeholder?
- Is there a reason to keep listening after eight bars?
- If this were triggered too many times, would it still feel intentional or just annoying?
- Does the output already have a recognizable Riotbox character, or is it only
  cleaner, louder, darker, or more technically active than the previous render?

## Pattern And Slice Policy

When generating or reviewing patterns, slices, demos, or presets:

- favor short, forceful loops with a clear riff or drum identity
- establish the hook and clearly hardest element on one supported Golden Path
  before widening source-family coverage or multiplying variations
- while tuning that Golden Path, run every shared audible DSP, mix, pattern, or
  performance-policy change against at least three contrasting real sources
  before requesting another human review; keep one source as the taste target,
  but reject clipping, silence, timing regressions, or near-identical hook
  envelopes across the matrix
- after that core loop works, create at least one destructive variation: choke,
  reverse, retrigger, pitch dive, filter slam, bitcrush, or silence cut
- do not make fallback sounds part of the product path; absence of trusted
  material must become visible unavailable / degraded state or silence
- do not use hardcoded musical fallback sounds as product output; when a lane
  cannot create trusted source-derived material, make the unavailable /
  degraded state visible instead of filling the space with replacement music
- prefer few strong lanes over many weak lanes
- make mutes and triggers musically dramatic, not merely state toggles
- treat repeated "ding ding ding" output as a failed placeholder unless explicitly requested as a metronome or diagnostic
- reject "source-derived" claims when the audible result is only a hardcoded
  phrase, scripted arrangement, template mutation, or hash/fingerprint variant
  that did not listen to source features
- treat stay-out, dropout, restraint, and silence as valid musical choices only
  when they are chosen from source context and improve impact, not when they
  hide weak generation
- require positive demo families to earn a real human pass, but allow weak or
  bad-timing material to succeed through reviewed degraded / unavailable /
  reject behavior instead of forcing demo-ready music
- reject a review window made from a near-identical short loop unless the mode
  explicitly promises a held loop and the underlying hook has already earned a
  human pass; micro-dropouts do not substitute for macro development
- for Golden Path review, prefer a source-derived eight-bar arc that establishes
  the hook, lifts pressure, creates a destructive role swap or drop, and returns
  with a materially changed payoff
- do not let that compact eight-bar arc become a medley-shaped substitute for
  instrument proof: a scripted sequence that forces hook, lift, fill, scene
  change, and return every one or two bars may prove gesture reachability, but
  it does not prove that any component is desirable to hold or loop
- when loopability, reusable source material, or performer freedom is claimed,
  also audition the relevant hook, capture, lane, or mutation in isolation for
  a sustained window; the musician must be able to keep or reject it before
  combining it with other roles on demand

## Engineering Implications

Use this taste model to shape implementation choices:

- expose controls that produce immediate audible contrast before subtle parameters
- preserve low-latency trigger, mute, retrigger, and choke behavior over decorative UI
- add fixtures that catch boring or collapsed output when feasible: silence, identical renders, fallback-only renders, no transient change, no source-derived energy
- provide demo recipes from real source files, not only synthetic tones
- when adding randomness, constrain it so repeated use creates attitude, not mush
- when a weak result is already audible and measurable, prefer the sample,
  drum, bass, mix, trigger, or arrangement-policy fix before adding another
  fixture, threshold, report, or UI cue
- do not infer bass pressure from relative low-band share alone; first state the
  typed bass owner, then require absolute low-band energy or lift when bass is
  assigned, and do not demand bass pressure when ownership is unassigned
- do not use `pressure` as a generic success label: distinguish bass/low-end,
  drum/transient, midrange/hook, and arrangement/performance pressure; failure
  in the intended domain cannot be rescued by an unrelated kind of pressure
- require a typed meaning whenever a gesture claims to be `harder`. For
  `percussive_hard`, the same recognizable hit must keep one onset, source pitch
  and `1.0x` playback while gaining unmistakable attack force, retaining
  physical body, and preserving bite. A lower, louder, darker, dirtier,
  damaged, doubled, or merely different hit is not harder. Treat a human
  `different but not harder` verdict as a recipe reject and freeze that
  mechanism instead of retuning it
- compare audible candidates both raw and loudness-matched when practical;
  matching gain may expose character and arrangement differences, but it must
  not conceal a product path that is still materially too quiet
- prove each explicit live gesture in a short time-local window against its
  immediate counterfactual, using both an absolute delta and a relative delta;
  global pressure floors must not erase the gesture, and a slam, fill, trigger,
  or scene move needs its own audible articulation rather than only more
  baseline gain
- keep commercial reference recordings local and uncommitted. They are
  listening and measurement references only, never Riotbox product sources,
  fixtures, generated assets, or redistribution material

## Review Language

When judging an output, say plainly:

- "This has impact" or "this is too polite"
- "The hook is clear" or "there is no riff yet"
- "The fallback is leaking into the product"
- "The gesture is playable" or "the control changes state but not the room"
- "This needs a destructive variation before it is a Riotbox preset"

Tie every taste critique to one concrete next step: sample transform, drum policy, trigger behavior, preset change, fixture, threshold, or UI cue.

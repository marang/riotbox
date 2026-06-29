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

## Pattern And Slice Policy

When generating or reviewing patterns, slices, demos, or presets:

- favor short, forceful loops with a clear riff or drum identity
- create at least one destructive variation: choke, reverse, retrigger, pitch dive, filter slam, bitcrush, or silence cut
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

## Engineering Implications

Use this taste model to shape implementation choices:

- expose controls that produce immediate audible contrast before subtle parameters
- preserve low-latency trigger, mute, retrigger, and choke behavior over decorative UI
- add fixtures that catch boring or collapsed output when feasible: silence, identical renders, fallback-only renders, no transient change, no source-derived energy
- provide demo recipes from real source files, not only synthetic tones
- when adding randomness, constrain it so repeated use creates attitude, not mush

## Review Language

When judging an output, say plainly:

- "This has impact" or "this is too polite"
- "The hook is clear" or "there is no riff yet"
- "The fallback is leaking into the product"
- "The gesture is playable" or "the control changes state but not the room"
- "This needs a destructive variation before it is a Riotbox preset"

Tie every taste critique to one concrete next step: sample transform, drum policy, trigger behavior, preset change, fixture, threshold, or UI cue.

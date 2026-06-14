# MC-202 Source Phrase Planning Plan

Status: accepted implementation plan; upgraded by RIOTBOX-1262
Linear: RIOTBOX-1035, RIOTBOX-1262, RIOTBOX-1263
Phase: P013+ / P023 sound-quality follow-up

## Goal

Move MC-202 beyond fixed `PressureCell`, `FollowerDrive`, bounded contour
hints, and source-aware template mutation. The product target is a bass /
answer planner that listens to the source evidence, extracts phrase-level
musical pressure, and commits replayable MC-202 decisions that a musician would
recognize as a composed response to that source.

The lane should derive bass / answer phrases from Source Graph timing,
sections, low-band pressure, transient placement, onset density, spectral
roughness / brightness, hook-restraint context, phrase memory, and arrangement
intent while remaining replayable and bounded.

The current RIOTBOX-1035 implementation is only the architecture scaffold:
`lane_state.mc202.source_phrase_plan` exists, projects through replay/render,
and varies by Source Graph fingerprint. That is useful infrastructure, but it
is not the final intelligence. It must not be treated as proof that Riotbox can
hear and compose MC-202 bass / answer phrases from the source.

The first implementation must not become a second sequencer, hidden callback
state, or Ghost-only composer. It must extend the existing Source Graph,
Session, Action Lexicon, queue / commit, replay, app projection, and audio
render seams.

## Implementation Slices

1. Add a typed `Mc202SourcePhrasePlan` contract.
   - Store it on the existing MC-202 lane/session state.
   - Include source phrase-slot reference, role intent, rhythm cells, interval
     contour, note budget, touch/accent/glide intent, confidence, and fallback
     reason.
   - Keep old sessions valid with `None`.

2. Promote the Source Graph analysis contract so MC-202 receives musical
   evidence, not only hashes and labels.
   - Add or expose per-phrase low-band envelope features: low-band RMS,
     low/mid ratio, pressure rise/fall, centroid of strongest bass movement,
     and whether the source contains a usable low-end gesture.
   - Add or expose transient features: kick/snare/backbeat anchors, offbeat
     onset density, fill / pickup clusters, and transient contrast across bars.
   - Add or expose hook-restraint features: likely hook bars, vocal / lead
     density, whether MC-202 should answer, leave space, double pressure, or
     stay out.
   - Persist feature provenance and confidence in Source Graph / Session so
     replay can explain why a phrase was chosen.

3. Derive source-backed MC-202 phrase candidates from those features.
   - Generate multiple candidate families, not one mutated role template:
     sub-pressure shove, sparse offbeat answer, call-back stab, hook-restraint
     ghost answer, fill-pickup instigator, and silence / stay-out.
   - Place steps from source evidence: strong low-band movement, phrase pickup,
     offbeat gaps, backbeat avoidance, hook-restraint windows, and section
     transition pressure.
   - Derive interval contour from source pressure and section role first; use
     bounded musical transforms only after the source features choose the
     gesture class.
   - Treat static templates as seed material only. A candidate that can be
     reproduced without the source feature vector is a fallback/control, not a
     source-derived phrase.

4. Score and commit one plan through the existing action path.
   - Prefer source-grid lock, low-end impact, answer contrast, hook avoidance,
     phrase memory, destructive contrast, and variation from the last phrase.
   - Reject source-independent candidates, static-collapse patterns, weak
     low-band pressure, source-grid drift, hook-doubling downbeats, and polite
     midrange question/answer loops that do not change the room.
   - Commit the chosen plan through current MC-202 queue / commit semantics.

5. Render source-derived plans on the existing MC-202 audio seam.
   - Project a committed source phrase plan into `Mc202RenderState`.
   - Render a bounded step plan instead of fixed primitive shapes when the plan
     is trusted.
   - Keep primitive MC-202 shapes as explicit fallback and A/B control only.
   - Route weak or untrusted candidates to an explicit `stay_out` /
     `fallback_reason` path instead of filling silence with fake intelligence.

6. Prove the output path.
   - Add primitive-vs-source-derived A/B render evidence.
   - Gate source phrase-slot alignment, signal delta, static distance,
     cross-source uniqueness, low-band pressure, phrase variation, and
     hook-response restraint.
   - Add same-source reproducibility vs cross-source diversity tests: the same
     source/seed must be stable, while different source families must produce
     materially different MC-202 phrase plans and rendered buffers.
   - Add template-collapse tests: if removing low-band / transient / hook
     features leaves the same phrase, the candidate must fail source-derived
     quality proof.
   - Keep scripted or diagnostic artifacts marked `quality_proof: false` and
     `human_verdict: unverified` until structured listening review approves
     them.

## Intelligence Ladder

### Level 0 - Primitive Control

Fixed pressure, follower, instigator, or answer shapes may prove render plumbing
only. They are never source-derived musical intelligence.

### Level 1 - Source-Aware Scaffold

The existing RIOTBOX-1035 path stores a typed source phrase plan and mutates
bounded step material from Source Graph fingerprints. This proves the product
spine: Source Graph -> Session -> replay -> projection -> audio render. It is
not enough for 10/10 sound-product claims.

### Level 2 - Feature-Derived Phrase Planner

MC-202 candidates are built from measured source features: low-band movement,
transient placement, onset density, section energy, and hook-restraint context.
Template material is allowed only as bounded synthesis vocabulary after source
features choose gesture role, placement, density, and contour.

### Level 3 - Musical Selection Engine

Multiple candidates compete. Scoring must prefer physical bass pressure,
source-grid lock, answer/restraint taste, variation from the previous phrase,
and destructive live usefulness. The losing candidates and rejection reasons
must be observable for QA and debugging.

### Level 4 - Producer-Grade MC-202 Behavior

The lane feels composed: it may shove the room with bass, answer a source hook,
leave space, or cut out because the source does not need more synth. It passes
automated source-diversity gates and structured listening review across the
real-source corpus.

### Level 5 - 20/10 Future Track

Future work may add learned phrase ranking, reference-free taste models,
performer-personalized phrase memory, and source-aware destructive resampling.
These ideas cannot weaken the Level 2-4 release gates.

## Product Acceptance

- `a`, `P`, and `G` can produce source-derived MC-202 behavior when source
  timing is trusted.
- Jam / diagnostics distinguish `source-derived`, `primitive fallback`,
  `timing untrusted`, `stay out`, and `static-collapse rejected`.
- Different sources produce materially different MC-202 phrase plans.
- The result sounds like source-specific bass pressure or answer motion, not a
  reused midrange question/answer loop.
- A source-derived quality claim requires Level 2 or higher evidence. Level 1
  scaffold behavior must be labeled as scaffold / transitional infrastructure.
- Professional-output or demo artifacts may not use hardcoded or
  template-mutated MC-202 material as musical-quality proof.

## Boundaries

- No full melody extraction is required for the first release-grade planner, but
  low-band, transient, density, and hook-restraint evidence are required.
- No unbounded phrase editor or callback-local sequencer.
- No quality or demo-readiness claim from scripted diagnostics alone.
- No bypass of Source Graph, Session, Action Lexicon, replay, or existing audio
  QA contracts.

## Immediate Follow-Up Tickets

The roadmap should keep these as separate, bounded slices:

- RIOTBOX-1263: Source Graph low-band / transient / hook-restraint feature
  contract for MC-202 planning.
- MC-202 candidate-family generator that derives placement from those features.
- Candidate scoring and rejection reasons with `stay_out` as a first-class
  musical choice.
- Cross-source diversity and template-collapse QA gates.
- Structured listening review pack for MC-202 source phrase candidates.

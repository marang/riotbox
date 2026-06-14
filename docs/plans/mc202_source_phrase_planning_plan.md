# MC-202 Source Phrase Planning Plan

Status: accepted implementation plan  
Linear: RIOTBOX-1035  
Phase: P013+ / P023 sound-quality follow-up

## Goal

Move MC-202 beyond fixed `PressureCell`, `FollowerDrive`, and bounded contour
hints. The lane should derive bass / answer phrases from Source Graph timing,
sections, low-band pressure, transient placement, and arrangement intent while
remaining replayable and bounded.

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

2. Derive source-backed MC-202 phrase candidates.
   - Use trusted Source Graph phrase slots, confirmed/ready source timing,
     section labels, energy class, low-band envelope, transient density, and
     hook/chorus context.
   - Generate bounded candidate roles: bass shove, sparse pressure, offbeat
     answer, hook-restraint answer, and instigator spike.

3. Score and commit one plan through the existing action path.
   - Prefer source-grid lock, low-end impact, answer contrast, hook avoidance,
     and variation from the last phrase.
   - Reject source-independent candidates, static-collapse patterns, weak
     low-band pressure, source-grid drift, and hook-doubling downbeats.
   - Commit the chosen plan through current MC-202 queue / commit semantics.

4. Render source-derived plans on the existing MC-202 audio seam.
   - Project a committed source phrase plan into `Mc202RenderState`.
   - Render a bounded step plan instead of fixed primitive shapes when the plan
     is trusted.
   - Keep primitive MC-202 shapes as explicit fallback and A/B control only.

5. Prove the output path.
   - Add primitive-vs-source-derived A/B render evidence.
   - Gate source phrase-slot alignment, signal delta, static distance,
     cross-source uniqueness, low-band pressure, phrase variation, and
     hook-response restraint.
   - Keep scripted or diagnostic artifacts marked `quality_proof: false` and
     `human_verdict: unverified` until structured listening review approves
     them.

## Product Acceptance

- `a`, `P`, and `G` can produce source-derived MC-202 behavior when source
  timing is trusted.
- Jam / diagnostics distinguish `source-derived`, `primitive fallback`,
  `timing untrusted`, and `static-collapse rejected`.
- Different sources produce materially different MC-202 phrase plans.
- The result sounds like source-specific bass pressure or answer motion, not a
  reused midrange question/answer loop.

## Boundaries

- No full melody extraction in v1.
- No unbounded phrase editor or callback-local sequencer.
- No quality or demo-readiness claim from scripted diagnostics alone.
- No bypass of Source Graph, Session, Action Lexicon, replay, or existing audio
  QA contracts.

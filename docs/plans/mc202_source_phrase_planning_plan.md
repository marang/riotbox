# MC-202 Source Phrase Planning Plan

Status: accepted implementation plan; upgraded by RIOTBOX-1262 and RIOTBOX-1264
Linear: RIOTBOX-1035, RIOTBOX-1262, RIOTBOX-1263, RIOTBOX-1264
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

RIOTBOX-1264 owns the producer-grade track for this feature. Implementation may
land through multiple PRs, but the feature is not product-complete until Level 4
is met: measured source evidence drives candidate generation and musical
selection, rendered output is audibly source-specific across the real-source
corpus, template-collapse gates pass, and structured listening review can
approve at least one dense-break and one non-dense-break MC-202 example or keep
the track open with concrete fix tickets.

For this track, "foundation landed" is not a valid product-complete state. A
foundation PR may merge when it preserves the product spine and unblocks the
next step, but the parent feature stays open until the musician-facing output
reaches the stated quality gate.

The current RIOTBOX-1035 implementation is only the architecture scaffold:
`lane_state.mc202.source_phrase_plan` exists, projects through replay/render,
and varies by Source Graph fingerprint. That is useful infrastructure, but it
is not the final intelligence. It must not be treated as proof that Riotbox can
hear and compose MC-202 bass / answer phrases from the source.

The first implementation must not become a second sequencer, hidden callback
state, or Ghost-only composer. It must extend the existing Source Graph,
Session, Action Lexicon, queue / commit, replay, app projection, and audio
render seams.

## No Hidden Fallback Product Rule

MC-202 must not silently fall back to a hardcoded question / answer or primitive
pressure pattern in the musician-facing product path.

If a source-backed MC-202 gesture cannot produce a trusted source-derived
`source_phrase_plan`, Riotbox must degrade honestly:

- do not route primitive MC-202 shapes to the music bus as replacement musical
  output
- keep `stay_out` / `fallback_control` as observable non-audible diagnostic
  states only
- surface the state to the musician as unavailable / degraded, not as a normal
  bass or answer part
- show why it degraded when possible: untrusted timing, missing source graph,
  no phrase slot, neutralized / weak source features, static-collapse rejection,
  or explicit `fallback_reason`
- keep source-derived quality proof, demo promotion, and listening-review
  candidate claims blocked unless a source-derived render plan exists and the
  rendered output passes source-vs-silence / source-vs-rejected-state gates

This means silence is acceptable when the alternative would be fake
intelligence. It is better for the musician to know "MC-202 has no source phrase
yet" than to hear a reusable fallback phrase that appears to be product output.

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
   - RIOTBOX-1266 implements the first replayable candidate-family layer:
     generated count, selected family, rejected count, and provenance are stored
     in `lane_state.mc202.source_phrase_plan`. `fallback_control` remains
     labeled as a rejected control and cannot satisfy source-derived proof.
   - RIOTBOX-1274 adds the typed MC-202 source-expression vector to the
     committed Session plan. It preserves low-pressure contour, bass pressure,
     transient / backbeat pressure, offbeat answer space, phrase density,
     hook-restraint pressure, stab bite, stay-out pressure, confidence, and
     provenance as the next composer input contract.
   - RIOTBOX-1275 moves the candidate-family composer onto that Session
     source-expression vector. Candidate construction, rejection, scoring,
     scorecards, groove fallback placement, contour, note-budget, and accent
     behavior now use bass / answer / transient / hook / bite / stay-out axes
     instead of each step reinterpreting raw feature heuristics. Static fallback
     remains an explicit control path, not source-derived musical proof.
   - RIOTBOX-1271 adds source-phrase groove spacing inside the existing
     candidate-family layer: pressure, answer, callback, hook-safe, and pickup
     steps are derived from source timing anchors and phrase evidence before a
     family writes its 16-step cell plan. This keeps MC-202 placement tied to
     Source Graph evidence instead of only fixed offsets plus feature buckets.

4. Score and commit one plan through the existing action path.
   - Prefer source-grid lock, low-end impact, answer contrast, hook avoidance,
     phrase memory, destructive contrast, and variation from the last phrase.
   - Reject source-independent candidates, static-collapse patterns, weak
     low-band pressure, source-grid drift, hook-doubling downbeats, and polite
     midrange question/answer loops that do not change the room.
   - Commit the chosen plan through current MC-202 queue / commit semantics.
   - RIOTBOX-1267 stores candidate scorecards in Session with low-end impact,
     source-grid lock, answer contrast, hook avoidance, phrase-memory distance,
     destructive usefulness, role fit, selected flag, and rejection reason, so
     QA can explain why a source-backed phrase won or why stay-out/control
     material was rejected.
   - RIOTBOX-1272 tightens phrase-memory selection for repeated live triggers:
     candidates that are too close to the previous source-derived plan are
     rejected with explicit phrase-memory reasons, selected memory distance is
     stored in provenance, and repeated commits must produce a changed plan /
     render or an explicit fallback/stay-out reason.

5. Render source-derived plans on the existing MC-202 audio seam.
   - Project a committed source phrase plan into `Mc202RenderState`.
   - Render a bounded step plan instead of fixed primitive shapes when the plan
     is trusted.
   - Keep primitive MC-202 shape labels only as historical compatibility /
     diagnostic state while removing their hardcoded audible pattern output
     from product rendering. The musician-facing app projection must not route
     them as fallback musical output when no source-derived plan exists.
   - Route weak or untrusted candidates to an explicit `stay_out` /
     `fallback_reason` path and visible unavailable / degraded status instead
     of filling silence with fake intelligence.
   - RIOTBOX-1268 projects selected scorecards into the source render plan as
     accent/destructive masks plus pressure and contrast scalars. The existing
     MC-202 render seam uses those values for source-only gain, gate, drive,
     accent, and pitch-dive/cut behavior; primitive shapes remain fallback
     controls.
   - RIOTBOX-1273 adds typed render articulation for source-derived MC-202
     plans: bass weight, stab bite, and gate snap are derived from the selected
     family / scorecard so pressure phrases render with more body while answer,
     callback, hook-restraint, and pickup phrases render with sharper transient
     behavior.
   - RIOTBOX-1276 deepens the existing MC-202 render seam with production
     sound-design shaping from the projected source phrase plan. Bass pressure,
     answer stabs, callback / pickup bite, gate snap, destructive pitch dives,
     drive, envelope length, transient click, and low-body emphasis are rendered
     from the typed render-plan articulation values; the audio callback still
     makes no phrase-composition decisions.

6. Prove the output path.
   - Add rejected-state-vs-source-derived render evidence without generating
     hardcoded fallback music.
   - Add no-hidden-fallback gates: if a queued source-backed MC-202 gesture
     lacks a trusted source-derived render plan, the app projection must be
     silent or explicitly diagnostic, the runtime view must expose unavailable /
     degraded status, and QA must not count it as musical output.
   - Gate source phrase-slot alignment, signal delta, static distance,
     cross-source uniqueness, low-band pressure, phrase variation, and
     hook-response restraint.
   - Add same-source reproducibility vs cross-source diversity tests: the same
     source/seed must be stable, while different source families must produce
     materially different MC-202 phrase plans and rendered buffers.
   - Add template-collapse tests: if removing low-band / transient / hook
     features leaves the same phrase, the candidate must fail source-derived
     quality proof.
   - RIOTBOX-1269 adds the first automated diversity/collapse gates on the
     existing Source Graph -> Session -> projection -> render path: same-source
     source-backed answers must remain deterministic, a small measured feature
     corpus must produce distinct selected families, phrase plans, render masks,
     and rendered buffers, and a neutralized low/transient/hook source must
     downgrade to non-source-derived silence instead of reusing the measured
     phrase template.
   - RIOTBOX-1270 wires MC-202 source-composed review-gate metadata into the
     professional output listening pack and demo-bank promotion path. Dense and
     non-dense MC-202 candidates can enter human listening queues, but
     `human_verdict: unverified` / `quality_proof: false` remains until a
     structured review records a verdict, and primitive/template-only MC-202
     evidence is blocked from demo-bank promotion.
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

## Producer-Grade Implementation Chain

RIOTBOX-1264 remains open until the MC-202 lane reaches Level 4. Individual
implementation tickets may merge when they improve the spine and prove their
bounded output path, but they are not product-completion claims. The current
chain is:

1. RIOTBOX-1274: add a typed MC-202 source-expression vector. This is the
   composer input contract for low-pressure contour, transient anchors,
   offbeat gaps, phrase density, hook-restraint pressure, bite / roughness, and
   confidence / provenance.
2. RIOTBOX-1275: compose bass and answer motifs from that expression vector.
   This replaces source-aware template mutation with source-conditioned
   musical decisions for role, placement, contour, density, accent, glide,
   destructive intent, rests, and stay-out.
3. RIOTBOX-1276: run the production sound-design pass on the existing MC-202
   render seam. Pressure phrases must gain physical low-end body; answer,
   callback, hook-restraint, and pickup phrases must gain transient bite
   without adding callback-local composition logic.
4. RIOTBOX-1277: tighten the automated gates against source-fake output. A
   phrase that survives low-band / transient / hook neutralization unchanged is
   a control or failure, not quality proof.
5. RIOTBOX-1278: generate dense-break and non-dense real-source listening
   packs with expression summary, selected motif, primitive A/B control, MC-202
   stem, mix, metrics, and `human_verdict: unverified` until reviewed.
6. RIOTBOX-1279: close out only after automated gates and structured listening
   review support demo-bank / professional-output promotion. If the review says
   the sound is still weak, create concrete fix tickets and keep RIOTBOX-1264
   open.
   - `just mc202-producer-grade-closeout-smoke` is the explicit closeout gate.
     It passes only as technical reviewability: dense and non-dense MC-202
     candidates must be source-composed and reviewable, while
     `producer_grade_promotion_result` stays `blocked_for_human_promotion`,
     demo-bank promotion stays false, and RIOTBOX-1264 stays open until
     structured human verdicts accept the sound.
   - Primitive/template-only MC-202 candidates remain blockers and production
     fix inputs. They cannot be counted as producer-grade proof, even when the
     render and source-composed pack are otherwise green.
   - RIOTBOX-1342 adds MC-202 producer fix routing to that closeout gate.
     Every weak or still-unverified review candidate must expose a concrete
     next fix category such as `bass_movement`, `answer_bite`,
     `hook_restraint`, `source_selection`, `mix_bus`,
     `destructive_articulation`, or `human_listening`, with the exact WAV
     artifact, software next step, musician payoff, and `quality_proof: false`.
   - RIOTBOX-1343 consumes those fix candidates during structured human-verdict
     promotion. Promotion must match the exact case and rendered WAV hash before
     deriving demo-bank `fix_categories`; `human_listening` is dropped once a
     human verdict exists, while weak/fail verdicts preserve the concrete
     producer fix categories and remain `not_demo_ready`.
   - RIOTBOX-1344 raises sparse MC-202 bass movement from a barely-passing
     10 Hz diagnostic span to a 12 Hz producer floor. The sparse policy still
     ranks bars from source low-band / timing / transient evidence, but expands
     the selected pressure/restore frequencies far enough that the musician
     hears bass pressure movement rather than a polite midrange trace.

This chain is deliberately allowed to take multiple PRs. The quality standard
does not shrink to fit a slice boundary: a merged slice can be useful
infrastructure, but producer-grade MC-202 means real source evidence causes a
musical decision that reaches the audible output and survives cross-source,
neutralized-source, automated, and human listening checks.

## Product Acceptance

- `a`, `P`, and `G` can produce source-derived MC-202 behavior when source
  timing is trusted.
- Jam / diagnostics distinguish `source-derived`, `unavailable / degraded`,
  `timing untrusted`, `stay out`, and `static-collapse rejected`.
- A musician must be able to tell immediately when MC-202 did not produce a
  source-derived phrase. The UI/runtime summary must not present primitive
  fallback as a playable product result.
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

The roadmap should keep these as separate implementation steps under the open
RIOTBOX-1264 quality track. They may be separate branches and PRs, but they are
not separate product-completion claims:

- RIOTBOX-1263: Source Graph low-band / transient / hook-restraint feature
  contract for MC-202 planning.
- RIOTBOX-1265: measured source phrase evidence from real audio, including
  low-band, transient, offbeat, roughness / brightness, and hook-restraint
  features with provenance and confidence.
- RIOTBOX-1266: source-backed candidate families that derive role, placement,
  density, contour, accents, and stay-out from measured evidence instead of a
  fixed phrase template.
- RIOTBOX-1267: musical scoring, rejection reasons, and phrase memory so the
  winner is selected for impact, source-grid lock, hook restraint, variation,
  and live usefulness.
- RIOTBOX-1268: source-composed render pressure and destructive contrast on
  the existing MC-202 audio seam, with primitive shape labels kept only as
  compatibility / diagnostic state and hardcoded primitive audio fallback
  removed from the product render path.
- RIOTBOX-1269: cross-source diversity and template-collapse QA gates that
  reject identical source-derived claims and feature-independent winners.
- RIOTBOX-1270: structured listening review and demo-bank promotion gate. This
  is the closeout gate for demo-ready MC-202 claims, not an optional polish pass.
- RIOTBOX-1274: typed MC-202 source-expression vector for producer-grade
  phrase decisions.
- RIOTBOX-1275: expression-driven MC-202 composer for bass / answer motifs.
- RIOTBOX-1276: production sound-design pass for source-composed MC-202 motifs.
- RIOTBOX-1277: source-fake / hardcoded-output rejection gate.
- RIOTBOX-1278: real-source corpus listening pack for dense and non-dense
  MC-202 proof.
- RIOTBOX-1279: producer-grade closeout review and demo-bank promotion gate.
- RIOTBOX-1285: tonal-hook MC-202 source-composed support pass; tonal review
  candidates now clear `source_composed_evidence` instead of remaining
  primitive/template-only blockers, while promotion still waits for human
  verdicts.

## Global No-Hardcoded-Fallback Follow-ups

The no-hidden-fallback rule applies beyond MC-202. Follow-up slices must remove
or downgrade these remaining musical fallback surfaces:

- RIOTBOX-1280: W-30 synthetic preview fallback: remove musician-facing fallback preview
  output and keep unavailable / degraded state unless a committed capture,
  source window, or artifact-backed pad is available. RIOTBOX-1277 removes the
  renderer fallback; RIOTBOX-1280 should finish UI copy, observer labels, and
  any remaining pack/report language. Keep source-vs-control QA comparisons
  non-product and explicitly labeled.
- RIOTBOX-1281: Feral-grid / lane-recipe `primitive_renderer` packs: stop treating primitive
  renderer cases as positive musical output; keep them as non-product regression
  controls or convert them to source-derived render plans.
- RIOTBOX-1282: TR-909 primitive support cases: audit kick-pressure / support patterns that
  report `primitive_renderer` and require source-derived support evidence before
  they can count as product output.
- RIOTBOX-1283: scripted professional-output generators: replace `fallback-*` selection
  strategies with source-derived unavailable / degraded handling, or keep the
  scripts diagnostic with `quality_proof: false`.
- RIOTBOX-1284: replay / restore source-plan continuity: action-log-only replay currently
  degrades MC-202 to silence when it cannot reconstruct a source phrase plan.
  A later slice must persist or reconstruct source-derived plans through replay
  instead of relying on any replacement audio.

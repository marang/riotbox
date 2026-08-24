# Human Listening Review

Parent: [Audio QA Workflow Spec](../audio_qa_workflow_spec.md)

---

## Review-pack and queue contracts

These rules keep unreviewed artifacts useful for handoff without promoting
them into product-quality evidence.

Professional output listening packs must include compact demo-readiness reasons
for every review case:

- `demo_readiness`: currently `unverified` unless a structured human verdict
  has promoted the artifact
- `demo_worthy_reason`: why the artifact is worth human review, based on
  existing proof such as strongest audible element, source-character survival,
  pressure, restore, or bass/chop target
- `not_demo_worthy_reason`: why the artifact is not demo-ready yet, usually
  because `human_verdict` is still `unverified` and scripted diagnostics cannot
  claim product quality

The same reasons must appear in `review.json` and the human review prompt so a
reviewer hears the candidate with the intended musical target and the current
quality boundary in view.

Dense-break professional listening packs must also carry
`riotbox.audio_presentation_true_peak_safety.v1` in both the case and its
`review.json`. The contract uses a conservative four-times band-limited
true-peak estimate, allows no emitted presentation WAV above `-1.0 dBTP`, and
targets `-1.2 dBTP` before quantization. When attenuation is required, one
uniform presentation-only gain applies to the source window and every candidate
section so source-relative level and section ratios do not change. Core musical
metrics remain explicitly pre-presentation; the safety gain, pre/post estimates,
coverage, and claim boundary must be reported. Missing or failed safety evidence
blocks human playback and demo-bank promotion.

Release-demo human-review queues use
`riotbox.release_demo_human_review_queue.v1`. They are review worklists, not
quality proof. Every queued candidate must remain
`human_verdict: unverified`, `demo_readiness: unverified`, and
`quality_claim: false` until a structured listener verdict is recorded. Each
queue entry must carry enough musician-facing context for an outside reviewer to
listen with intent:

- source family and demo-bank source-family alias
- strongest audible element
- source-character summary
- hook-within-two-bars summary
- destructive contrast, bass/drum pressure, live-triggerability, and eight-bar
  replay-value summaries
- `demo_worthy_reason`
- `not_demo_ready_reason` that explicitly names the unverified human verdict and
  blocked quality claim
- review blockers such as unverified verdict/readiness, blocked quality claim,
  and missing family coverage
- required listening questions covering strongest element, source survival,
  first-two-bar hook, live gesture contrast, demo-worthiness, and concrete
  follow-up
- required verdict path for `pass`, `weak`, and `fail`, with the current state
  fixed at `human_verdict:unverified/demo_readiness:unverified`

Demo-bank consumers have two explicit evidence modes. `live_readiness` is the
default and never resolves an omitted `--demo-bank` to the checked-in fixture.
An omitted real bank is a valid blocked state with zero eligible human verdicts,
zero demo-ready passes, and the dense-break real-review import as the first
action. `fixture_calibration` must be requested explicitly and labels every
derived report `fixture_only: true`; it exists for deterministic schema,
mutation, and negative-control tests only.

In `live_readiness`, pass/weak/fail entries count only when they carry
`riotbox.demo_bank_human_review_evidence.v1` with a non-fixture reviewer,
explicit `reviewer_kind: human`, an existing structured review JSON path, and a
matching SHA-256 identity. Passing a calibration bank explicitly in live mode
rejects the bank rather than making its verdicts or candidates eligible. Human
review queues and derived listening packs must carry the same evidence mode,
fixture flag, demo-bank state, normalized demo-bank path, and demo-bank SHA-256
as the readiness report that consumes them. Positive families require a
demo-ready human pass. `weak_source` and `bad_timing` instead use
`riotbox.demo_bank_degraded_or_reject_review_evidence.v1`: family success
requires a reviewed `degraded`, `unavailable`, or `reject` outcome through the
product path, a musical `weak`/`fail` plus `not_demo_ready` classification, a
useful reason, and `fallback_music_present: false`. This allows
honest refusal to satisfy the negative-family contract without forcing weak or
untrusted material into demo music.

Live negative-family evidence must resolve to a hash-bound
`riotbox.degraded_product_review.v1`, not a generic listening-review JSON. The
review binds the exact source, Source Graph, Session, and observer stream;
requires the real audio callback to have run; verifies the typed performance
state/reason, empty queue/commit history, stopped transport, source-preview-only
monitoring, idle generated lanes, forbidden confident bar-locked policy, and
absent fallback music; and records whether the musician found the risk state
visible, the reason useful, and the safe next action understandable. A fixture
calibration review never satisfies live human coverage. A completed negative
review may omit `rendered_wav`, because manufacturing a candidate WAV would
contradict the reviewed no-generated-output outcome. Validation must rederive
the stored product-path proof from the bound artifacts and reject unsafe state
in any assigned observer snapshot, rather than accepting only the final or
highest-callback snapshot.

Queue validation must reject missing source-character or strongest-element
context, incomplete listening questions, stale verdict state, or any quality
claim. This keeps the queue useful for human review without promoting
unlistened artifacts into release-ready evidence.

P023 sound-quality readiness reports must aggregate the current
release-demo human-review queue when it exists. The readiness report should
surface queue availability, priority counts, source families, review blockers,
and per-candidate review context: strongest audible element, source character,
demo-worthy reason, not-demo-ready reason, required verdict state, and required
listening questions. Queue entries remain claim blockers while they are
`human_verdict: unverified`; a readiness report must reject stale queue verdict
state, missing review blockers, or any queued candidate with
`quality_claim: true`. This makes the readiness report useful as a release
blocker and reviewer worklist without turning unreviewed artifacts into quality
proof.
When current weak-output reconciliation leaves no active production-fix bucket,
the main source-family Next Actions must also link missing demo-ready families
to matching human-review queue candidates when available. The action must carry
the candidate id, priority, demo-worthy reason, not-demo-ready reason, and
required verdict state so the next loop is an actionable listening task rather
than a generic "create/promote candidate" reminder. The readiness queue summary
and matching source-family action must also preserve the queue's rendered WAV,
metrics, and review-prompt artifact refs (`path` plus SHA-256), so a reviewer
can open the exact files without reverse-engineering the queue report. The
Markdown readiness report must render those refs in both the relevant Next
Action and Human Review Queue sections, not only keep them in JSON.
When local release-demo listening-review packs exist, the readiness report must
also aggregate them under `release_demo_review_packs`, cross-check each queued
candidate against a matching pack, and render the pack directory, `review.json`,
and prompt path in the Release-Demo Review Worklist. Pack refs remain handoff
artifacts only: validation must reject missing candidate-pack context, stale
non-`unverified` verdict/readiness state, or `quality_claim: true`.

### 3.3 Fixture-backed golden render review

For stable fixture, seed, action list, and render config:

- the system should render deterministic review artifacts
- those artifacts should be compared against known baselines
- deltas should be visible before they become production drift

### 3.4 Human listening review

Humans must be able to listen to the same deterministic outputs that automation validated.

Manual listening is required because:

- timing can be technically valid but still feel awkward
- variation can exist numerically but still feel trivial or annoying
- support layers can pass signal checks while still sounding cheap

Before every human playback, preflight the exact WAV or A/B artifact that will
be played rather than relying on a sibling render or report. Verify artifact
paths and segment order; use hashes or sample-exact segment identity where
assignment could be ambiguous; inspect format, sample rate, channels, duration,
and frame count with `ffprobe`; and report peak, RMS, LUFS when available,
silence, and clipping. A/B review must also report role-appropriate time-local
absolute and relative deltas, waveform correlation, and frequency-domain deltas
for the expected owner or function. Whole-render aggregates alone do not prove
that a short gesture is audible.

Interpret that evidence before involving the listener: state what changed,
what stayed unchanged, whether the intended role survives the full mix, and
whether the artifact is correctly assigned. Invalid or misassigned artifacts
must be fixed; artifacts demonstrably too similar for their claimed effect must
be regenerated or labeled weak before requesting a taste verdict. This
technical gate screens and explains the sample but does not replace human
listening.

Only then give the listener a compact factual brief:
the playback context (isolated stem, full mix, source, baseline, or comparison),
the selected product role or candidate family, the intended musical function,
the expected audible effect, important properties that are not expected, and
the dimensions to judge. `Pressure` must never stand alone as a review target:
the brief must distinguish bass/low-end pressure, drum/transient pressure,
midrange/hook aggression, and arrangement/performance impact, name the owning
lane or policy when applicable, and state which domains are not expected. If
bass pressure is the intended target but no recognizable bass is audible, that
is a bass-pressure failure even when drums, loudness, or general bus energy feel
forceful. Do not ask for a taste verdict while role assignment is unresolved or
contradictory. Show a conspicuous listening check and wait for explicit
readiness before the first playback of each exact artifact. If the listener
explicitly requests an immediate replay of the unchanged artifact (`again`,
`nochmal`, or equivalent), play it directly without repeating technical
analysis, the brief, or readiness. Readiness does not transfer to a changed
artifact, assignment, contributor set, duration, or route. Playback without the
required first-play confirmation is treated as unheard and cannot support a
recorded human verdict.

An early `development_exploration` playback follows the same preflight,
readiness, bounded-playback, replay, and stop/silence safety rules. Its narrow
question is whether the intended role is obvious, useful, and worth performing.
Treat the answer as a provisional local direction only: do not import it into a
human-label corpus, demo bank, release queue, or product-quality claim. A kept
idea must be rebuilt from its later frozen contract and receive the normal
formal structured review during product qualification.

For a `percussive_hard` claim, apply
`docs/engineering/percussive_force_and_beat_impact.md`. The candidate must
preserve one onset, `1.0x` playback, recognizable hit identity, immediate
attack, physical body, and bite. Global resampling/transposition is prohibited.
A local source-consistent spectral or resonant change is not an automatic
reject, but it cannot itself count as force evidence. Lower pitch, louder
output, darker EQ, distortion, damage, or a passing crest/transient metric does
not establish hardness. `different_but_not_harder` is a human reject and
freezes that recipe; do not route it into another scalar-tuning pass.

An `isolated` playback label requires an exact audible-contributor inventory.
Source monitor, internal resample taps, support lanes, diagnostic voices, and
stopped manual previews must be silent unless the brief names them as part of a
composite. Internal routing does not make a callback voice inaudible.

Exact Golden Path renderers must execute the same documented preparation and
gesture sequence available to the musician. A lane mode, source-evidence
decision, pattern, monitor route, or other prerequisite may not be queued only
inside QA. Required preparation must be visible in the recipe/UI and resolved
through its real committed action. When a preset promotes a captured source
hook, its candidate render uses the preset-declared monitor route; raw Source or
Blend playback is reserved for explicitly labeled A/B evidence and must not
silently double the promoted hook from another source phase.

Human playback is bounded by default. Use at most 10 seconds for a normal
candidate and 2-5 seconds for an isolated capture or stem when that is
sufficient for the requested judgment. A longer window is permitted only for a
named multi-bar development claim after the reviewer is told the exact duration
and why it is necessary. A repeating live instrument state is not itself a
bounded review artifact: the operator must schedule an explicit audible stop at
the announced endpoint, verify that all active lanes are silent, and terminate
the runtime immediately if transport stop does not silence them.

Multi-stage live review timing must be derived from committed observer beats,
not assumed from wall-clock automation. Queue each action with enough lead time
for its intended quantized boundary, then validate the landed stage intervals
and final transport-stop position before asking a human to listen. A missed
beat/bar boundary invalidates that take for the declared arrangement comparison
even if its command order and sound recipe are correct. The Feral Break Alpha
live review contract uses
`scripts/validate_feral_break_live_review_timing.py` to prove the committed
`8 -> 8 -> 4 -> 4 -> 8` beat arc. A verdict from another duration or stage arc
must remain bound to its original artifact and cannot be transferred.

Observer, transport, capture, and quantization corrections that intentionally
leave the sound recipe unchanged are technical QA, not automatically a new
human listening task. Use the mechanical evidence unless exact-artifact
preflight identifies a material audible question. If repeated candidates are
perceptually indistinguishable, stop requesting comparisons; do not convert
listener fatigue or "no opinion" into a negative verdict, and do not overwrite
an existing hash-bound review.

Routine readiness, playback, and unchanged-artifact replay are operational
events. They do not belong in the research Decision Log. Record the resulting
artifact-bound verdict, or a genuinely durable protocol/contract decision, not
each reversible interaction step.

After at most two consecutive review-ready generations remain
`human_verdict: unverified`, stop generating that candidate and perform or
explicitly hand off structured human listening. Do not replace the missing
verdict with another report, fixture, threshold, or validator unless the
failure is genuinely unobservable.

Structured listening review records the human layer as explicit artifact data,
not chat memory and not CI-only truth. For audio-producing slices, the local
workflow is:

```bash
just listening-review-pack RIOTBOX-123
just listening-review-record artifacts/audio_qa/local/listening-reviews/RIOTBOX-123/review.json \
  keep kick source_transformed_but_present clear
```

The pack command writes a local review directory with:

- `prompt.md`: one-question-at-a-time listening prompt
- `review.json`: structured verdict data, initially `human_verdict: unverified`
- `metrics.json`: compact source/candidate file presence and byte metadata
- `README.md`: local artifact ownership notes

The record command updates `review.json` and writes `review-summary.md`.
Required verdict fields include:

- ticket, PR, command, source file, and seed/config when available
- `technical_status`
- `automated_musical_fitness_status`
- `human_verdict`: `keep`, `reject`, `technically_ok_but_musically_weak`, or
  `inconclusive`
- strongest element: `kick`, `snare`, `bass`, `stab`, `chop`, `vocal`,
  `silence`, `restore`, or `none`
- source-recognition verdict
- hook verdict after two bars
- failure reason
- preferred direction
- avoid list
- concrete follow-up

PRs that affect audible behavior must say whether a listening-review pack
exists, whether a human verdict was recorded, or why the change remains
`human_verdict: unverified`. The structured verdict complements automated
musical fitness; it does not replace deterministic metrics, and it must not be
stored only in agent memory.

P023 release-demo review queues can also be materialized into local listening
review packs with `just release-demo-listening-review-packs-fixtures` /
`scripts/generate_release_demo_listening_review_packs.py`. These packs preserve
the queued candidate id, priority, source family, artifact refs, blockers,
verdict state, and required listening questions so a reviewer can execute the
same worklist without hand-copying JSON. They remain handoff artifacts:
`human_verdict` and `demo_readiness` stay `unverified`, `quality_claim` stays
`false`, and no release/demo-ready claim is allowed until a human records a
structured verdict.

Human labels intended for future audio-judge calibration must use
`riotbox.human_listening_label_corpus.v1`. The corpus stores labels by
review-pack identity and SHA-256 artifact hashes so local source audio does not
need to be committed. It distinguishes `pass`, `weak`, `fail`, and
`inconclusive` human labels from technical or agent-promising status.

Structured `riotbox.listening_review.v1` reviews can be imported into the label
corpus only when they carry explicit `audio_judge_label` metadata with source
family/id, review pack identity, artifact identity hashes, created date, and
reason tags. The importer maps structured listening verdicts to label verdicts:
`keep` to `pass`, `technically_ok_but_musically_weak` to `weak`, `reject` to
`fail`, and `inconclusive` to `inconclusive`. It must reject `unverified`
reviews and reviews missing the audio-judge metadata so human labels cannot be
created from chat memory or vague listening notes.

Run:

```bash
just listening-review-label-import-fixtures
```

---


### 4.2 Local listening mode

Operator-facing manual review:

- render deterministic WAV outputs for a known fixture pack
- write metrics beside those renders
- compare candidate output to baseline output
- let the operator listen before approving a change

Local listening mode is for:

- musical judgment
- product taste
- identifying weak but technically legal outputs

---


### 5.3 Listening pack harness

Riotbox should support named listening packs such as:

- `tr909-smoke`
- `capture-smoke`
- `w30-preview-smoke`
- `feral-review`
- `dense-break-performance-pack`
- `agent-musical-review-pack`

Each listening pack should render a small fixed set of review cases to one output directory.

---


## 8. First Listening Rubric

Every manual listening case should be scored against a short fixed rubric.

Recommended fields:

- rhythmic clarity
- energy appropriateness
- transition quality
- variation usefulness
- support-layer tastefulness
- rebuild-only usefulness
- source-layer dependency
- anchor-preservation honesty
- capture-worthiness
- artifact severity

Recommended scale:

- `1` unacceptable
- `2` weak
- `3` acceptable
- `4` strong
- `5` excellent

Short comments should also note concrete failure classes such as:

- too empty
- too busy
- cheap-sounding support
- awkward phrasing
- weak impact
- over-repetitive
- only works with original source underneath
- generated layers drift against the source grid
- promised anchor was lost
- capture not worth keeping

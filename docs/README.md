# Riotbox Docs

Status: active implementation documentation

This directory holds implementation-facing specifications, plans, reviews, benchmarks, and workflow notes derived from the strategy documents in `plan/` and from shipped Riotbox slices.

## Source of Truth

- `plan/riotbox_masterplan.md`
  Source of truth for product structure, MVP, phases, and system architecture.
- `plan/riotbox_liam_howlett_feral_addendum.md`
  Source of truth for the `feral_rebuild` profile and its backlog deltas.

## Current Active Direction

`P023 | Sound Excellence / Production Quality` remains the single active
product priority. Its first bounded exit, the `RIOTBOX-1396` Usable Musical
Alpha, closed on 2026-07-21: one trusted dense-break source reached a strong
hook through the exact live `riotbox-app` path, exposed playable contrast,
survived capture / recall / replay, and earned a structured human `pass`.
`RIOTBOX-1404` has now carried that policy into human-kept tonal-hook and
sparse-pressure held/destructive states without freezing the reviewed
eight-bar QA choreography as a composition. `RIOTBOX-1403` has also separated
fixture calibration from live readiness: live mode now starts with no eligible
demo-bank evidence unless a real, hash-matched human review is supplied
explicitly. `RIOTBOX-1405` has now proven honest live
degraded/unavailable handling for weak and bad-timing sources without generated
replacement music; both negative families have a bounded human product pass,
not a sound-quality pass. `RIOTBOX-1457` now adds the exact registered
pad/noise outcome: untrusted soft-attack timing stays visibly unavailable, all
generated lanes stay silent, and no fallback music is invented. That handling
earned a bounded human product pass and satisfies the pad/noise dual-path
coverage contract without becoming a demo-ready sound claim. `RIOTBOX-1458`
now makes aggregate readiness honor that result: `pad_noise` no longer returns
as a generic weak/fail or unresolved edge blocker, while the still-unresolved
`bad_timing` family and every unrelated quality/release gate remain blocked.
`RIOTBOX-1459` then attempted to recreate the expired Beat20 negative-family
evidence without inferring a verdict from old Markdown. The exact live path
again failed safely to `degraded / needs_user_confirmation`, but fresh listening
placed the musical downbeat at the file boundary while the probe selected beat
3. Both new structured reviews are therefore `needs_fix`, not reconstructed
passes; `weak_source` and `bad_timing` remain unresolved until the existing
RIOTBOX-1033 detector-quality work supplies stronger musical phase evidence.
`RIOTBOX-1033` now supplies that bounded evidence without inventing a lock:
repeated complete-bar onset structure may reorder only an already-ambiguous
phase toward the file boundary. Beat20 consequently suggests phase zero while
retaining three alternatives and explicit musician confirmation; four varied
registered Development cases show no cue misactivation. This corrects the
current explanation but does not retroactively promote the rejected
RIOTBOX-1459 family records.
`RIOTBOX-1460` now rebuilds those two records from the corrected current
product state. Exact live and restart evidence keeps phase zero as the
suggestion, all three alternatives, explicit confirmation, stopped transport,
source-only monitoring, idle generated lanes, and no fallback. That handling
earned a fresh bounded human product pass without replaying unchanged audio.
`weak_source` and `bad_timing` now satisfy their reviewed-negative contracts;
only `dense_break`, `sparse_drums`, and `tonal_riff` remain without family
success, while release readiness and every quality claim remain blocked.
`RIOTBOX-1461` then rebuilds a fresh current-state professional Dense candidate
because the historical temporary live-review WAV no longer exists. A newly
frozen uniform true-peak presentation gate rejects the first `+6.0 dBTP`
candidate before playback and makes the replacement safe to review. The human
review rejects the foundational transformation as musically unusable despite
understandable sectional intent; Dense remains non-demo-ready and now routes
to one bounded `chop_policy` foundation correction rather than another source
search. Sparse and tonal positive-family candidates remain separately open.
`RIOTBOX-1462` completes that Development exploration with one source-only
long-slice answer: the first six beats remain exact, only the final 1.5 beats
form a bounded deviation/return, and no generated support or bus treatment is
present. Exact Beat03 review provisionally keeps v1 as musically usable with
preserved groove/clarity and a useful hook. RBX-320 freezes the recipe for a
separate product-spine/source-diversity qualification in RIOTBOX-1463; Dense
is not yet demo-ready and no quality or release claim is added.
`RIOTBOX-1463` subsequently passes the complete frozen three-source technical
matrix but rejects v1 in formal product listening: its answer loses local
low-frequency weight and clarity and sounds unintentionally radio-like. The
unqualified behavior is removed. `RIOTBOX-1464` then freezes and tests a
materially different clarity-preserving v2 selector. Its single bounded
Development session finds no coherent two-beat source cell that satisfies both
the local-weight and material-contrast gates, so it stops before rendering or
human listening. Dense remains non-demo-ready; neither result authorizes
threshold tuning, Holdout access, or a quality/release claim.
`RIOTBOX-1408` has now replaced the synthetic W-30
resample-tap proxy with hydrated capture audio through the exact live path.
Determinism, three-source diversity, replay identity, and missing-source silence
pass, but direct source-to-tap listening rated the result
`technically_ok_but_musically_weak`: the tap is very timid, its hook is weak,
and Beat03 is no longer perceptually recognizable. Repeated scalar tuning then
failed to establish a defensible `percussive_hard` mechanism. `RIOTBOX-1429`
therefore records the research prerequisite: it distinguishes force, punch,
hardness, aggression, bass pressure, groove, and arrangement impact; defines a
multi-scale beat-analysis rubric; and hands falsifiable source-adaptive
hypotheses directly to `RIOTBOX-1428` Stage A. `RIOTBOX-1429` changes no
renderer and does not itself prove or unblock Stage B product promotion.
`RIOTBOX-1428` and `RIOTBOX-1430` then established a bounded, legally usable
Development source pool and froze the Stage-A force contracts before opening
registered audio. The final three-family, four-source, two-event matrix ran
without holdout access, but the F2 comparison was human-rejected as
near-identical. `RIOTBOX-1428` subsequently qualified F4 mechanically and again
received a human near-identity rejection. `RIOTBOX-1434` tested natural
velocity as a control: the snare difference was audible, while the whip pair
was not. It therefore failed the declared cross-source control and
`RIOTBOX-1435` was canceled before implementation. These results remain frozen
negative evidence; they do not justify more threshold tuning.

`RIOTBOX-1433` audited the audible algorithm portfolio and selected W-30 hook
choice as the next smaller product gap. `RIOTBOX-1432` then froze two
source-blind policies, persisted their evidence and decisions through Source
Graph and capture lineage, and ran the exact three-source RuntimeMix comparison.
Dense and sparse failed the frozen downbeat-confidence eligibility gate; both
policies selected the same tonal bar and produced byte-identical candidate
output. No policy won, so `FeralBreakAlphaV2` retains the transport-selected
baseline and no unnecessary human playback occurred. The holdout remains
unopened.

`RIOTBOX-1436` and `RIOTBOX-1437` then closed two further bounded TR-909
experiments without product behavior: a technically clean collision-local
impact pocket was inaudible, while a source-backed counter-rhythm passed the
dense case but failed the frozen source-diversity gate. `RIOTBOX-1438` reused
the already valuable Fill/Slam cut-hit-return vocabulary as one new live
gesture. Its dense exact RuntimeMix qualification passed mechanically and the
listener judged both source transformations musically useful, but the full A/B
arcs were substantially similar apart from B's opening. The new one-key gesture
therefore closes fail-closed and is removed; existing Fill, Slam, cut, hit,
return, and source transformation behavior remains. The next P023 work must
restore an early audible-discovery step before expensive promotion proof and
must not reopen scalar force tuning.

`P016`, `P021`, and `P022` remain subordinate to the active P023 product path.
Offline renderers, scripted packs, fixture verdicts, reports, and validators
remain diagnostic evidence; they do not count as instrument progress until the
behavior lands in the live product path. Broader positive source-family
coverage and source-backed W-30 resampling now follow from the accepted
dense-break baseline and honest negative-source handling.

`RIOTBOX-1439` owns the accepted delivery-order correction: a new audible
mechanism first receives at most three bounded Development variants and an
early usefulness check, then only a provisional human keep permits a frozen
rebuild, product-spine implementation, source-diversity qualification, and
formal product verdict. This does not weaken any release, replay, realtime,
source, or Holdout gate. `RIOTBOX-1417` applied the correction and stopped
fail-closed after three MC-202 instigator variants produced no provisional
keep; no product behavior or qualification claim remains. A successor must use
a materially different musical owner or grammar rather than tuning the
rejected retrigger or full-bed-cut mechanisms. `RIOTBOX-1399` remains the
measured validation-cost optimization when it blocks an audible path.

`RIOTBOX-1440` subsequently shipped the first kept mechanism under that order:
the source-backed W-30 Hook Turnaround earned a formal product pass on its
bounded qualification source. `RIOTBOX-1441` then found the unchanged gesture
usable on four of five additional Development sources and clearly beneficial
on three, while one source lost groove and musical usefulness. Its longer
source-first comparisons were more reliable than the initial short presentation
but exceeded the frozen review duration and lacked one completed five-source
access log. The transfer result therefore remains informative human evidence,
not a formal qualification pass; no Holdout or commercial reference was used.

`RIOTBOX-1442` provisionally kept one continuous W-30 Pitch Dive after the
first bounded Development source produced a clearly transformed, musically
useful exit with recognizable source identity. The exact curve and terminal
fade are frozen before the requested four-source transfer observation. This is
not yet product behavior, source-general qualification, or a hardness claim.

`RIOTBOX-1443` then observed that unchanged frozen Pitch Dive on four additional
registered Development sources. A was useful in all four cases and B was
preferred in all four, spanning dense breaks, sparse drums, and a tonal riff.
This supports the next source-blind product-integration slice without per-source
tuning; formal product qualification and Holdout remain unearned.

`RIOTBOX-1444` rebuilt that exact frozen Pitch Dive as the explicit
`w30.pitch_dive` performer action through queue/commit, Session/replay,
observer/UI, and exact RuntimeMix. All four registered Development cases passed
the frozen source, boundary, callback-partition, limiter, lineage, replay, and
missing-source gates. The single representative formal product review retained
recognizable source identity and a clear hook, judged the control useful, and
preferred the Pitch Dive's musical payoff. This is a bounded W-30 product pass;
Holdout, hardness, universal-source, Golden-Path, demo, and release claims
remain unearned.

`RIOTBOX-1445` then provisionally kept a materially different W-30 Filter Slam.
A four-beat filter arc was audible but underdeveloped; the accepted eight-beat
version gradually closes for four beats, deepens for two, holds dark for one,
and opens back to ordinary W-30 on the final beat. The project musician judged
that longer version good and live-usable, with eight beats close to the minimum
duration needed for the movement to register. RBX-304 freezes the exact heard
mechanism before any source-blind product rebuild. It is not yet a product
action, source-general qualification, or release claim.

`RIOTBOX-1446` rebuilt that unchanged eight-beat Filter Slam as the explicit
`w30.filter_slam` performer action through queue/commit, typed Session/replay,
observer/UI, and exact RuntimeMix. Its four-source Development matrix and one
representative formal product review passed without Holdout or commercial
references. This is a bounded product gesture, not automatic arrangement or a
universal source-quality claim.

`RIOTBOX-1447` then found and fixed a shared W-30 transition defect rather than
adding another effect: a completed destructive Pitch Dive could leave ordinary
pad playback trapped behind persisted terminal silence. Explicit ordinary hit,
recall, audition, and damage intents now supersede a prior timed articulation
identically in live Session state and replay. One exact Development journey
passed technical state/output/replay gates and the musician accepted the clean
re-entry. Musical combinations and preferred gesture ordering were explicitly
not assessed and remain future work only when a concrete use case warrants it.

`RIOTBOX-1448` subsequently reviewed that exact fixed sequence for the concrete
purpose of a complete performer arc. The musician kept the sequence as
harmonically coherent and well executed, found all three gestures effective in
different roles without a meaningful ranking, and confirmed that source and
hook remained clear. This closes only the reviewed sequence; automatic order,
simultaneous stacking, source-general, Holdout, demo, and release claims remain
excluded.

`RIOTBOX-1449` then tested the missing MC-202 owner through a materially
different source-derived sparse offbeat answer. The controlled exact RuntimeMix
diagnostic passed, but the full-mix A/B was perceptually unchanged: the new lane
was too temporally sparse and masked in its intended low-frequency bands. The
temporary Development selection seam is removed and no mechanism is promoted.
`RIOTBOX-1450` owns the separate fail-closed bug in which an explicit MC-202
intent could otherwise commit a mismatched candidate family.

`RIOTBOX-1450` now closes that defect as a contract enabler. Explicit pressure,
instigator, answer, leader, and follower intents gate the compatible typed
candidate families before selection. If no compatible source-derived family
survives, the decision is persisted and shown as degraded silence; restore and
replay cannot turn a historical mismatch into audible output. This changes no
MC-202 synthesis algorithm and is not a musical-quality pass.

`RIOTBOX-1451` then found a genuinely useful single-source MC-202 bass-pressure
pedal in bounded Development exploration: the musician clearly heard and kept
the added low-end owner. Its unchanged frozen v1 product rebuild nevertheless
failed the first transfer case because a different source-derived anchor layout
made the lane active for `48.92%` of frames and forced limiter intervention.
Qualification stopped before later sources or formal playback. The unqualified
product behavior was removed and no v2 tuning was admitted; the result is
durable positive discovery plus negative transfer evidence, not shipped MC-202
bass pressure or P023 completion.

`RIOTBOX-1452` then tested three materially different non-additive handoffs for
that MC-202 voice. Two retained limiter intervention; the third preserved
support attacks and stereo sides with zero clipping or limiter activity and was
clearly audible. The musician nevertheless rejected the complete B mix because
the broad center-bus processing made too many effects operate at once. The
temporary behavior is removed at the exploration limit. This narrows the next
attempt to one restrained ownership change and adds no product, source-general,
Holdout, hardness, demo, release, or P023 completion evidence.

`RIOTBOX-1453` applies that restrained check without new DSP: the unchanged
RIOTBOX-1451 MC-202 voice enters only after one ordinary W-30/TR-909 phrase.
Although technically safe and exactly assigned, the listener recognizes it as
the same periodic addition already reviewed, with only a later entry point.
No new verdict or second variant follows; the temporary seam is removed and
MC-202 pressure stays out of the current combined gesture journey while the
remaining Foundation Completion gaps take priority.

`RIOTBOX-1410` is the first bounded Foundation Completion closure. It replaces
the source monitor's construction-only PCM ownership with one coherent atomic
snapshot for source, mode/gains, and anchors. Control refreshes cannot silently
replace PCM; explicit replacement changes the exact RuntimeMix from source A to
source B without stale playback, while absent replacement remains degraded /
silent with no fallback. The callback reads without locks or allocation, and
old PCM is reclaimed from the control side. This is a regression/contract
enabler, not a new effect or human sound-quality pass.

`RIOTBOX-1407` is the second bounded Foundation Completion closure. The live
Jam/TUI Play/Pause path now enqueues and immediately commits the existing
`transport.play` / `transport.pause` action at the current clock boundary.
Session/runtime state, the audio-driver request, observer history, and replay
therefore share one product-spine action instead of a direct UI mutation. This
preserves immediate transport response and adds no TUI polish or audible
effect.

`RIOTBOX-1334` closes the third and final named Foundation Completion gap. The
sidecar now negotiates protocol compatibility before graph acceptance, stamps
truthful clock-injected provenance, preserves request-less provider errors,
uses separate bounded control/analysis deadlines, and resolves its bundled
script independently of process CWD. Static failure fixtures and an
outside-repository-CWD launch regression cover those boundaries. Stub transport
graphs remain explicitly non-source-derived scaffold and cannot support
release/demo or quality claims. The named Foundation inventory is therefore
closed; the next slice may return to the highest-value unblocked audible Golden
Path gap.

## Documentation Rules

- Stable core contracts live in `docs/`.
- Exploratory thinking and generative planning stay in `plan/`.
- Accepted implementation plans live in `docs/plans/` and should be anchored from the roadmap, phase definition of done, README, and decision log when they freeze a durable direction.
- [P023 Audible Delivery Course Correction](./plans/p023_audible_delivery_course_correction.md)
  owns the Development-exploration versus product-qualification order for new
  audible mechanisms.
- Profile behavior must be expressed as policy, preset, or scoring extensions, not as a parallel product architecture.
- Incoming refinements to the feral addendum should update profile-oriented specs, not the core contracts unless they truly change the core.

## Recommended Reading / Build Context Order

For ticket, branch, PR/CI, Linear, archive, or cleanup work, start at the
[Workflow Conventions](./workflow_conventions.md) router.

1. [PRD v1](./prd_v1.md)
2. [Execution Roadmap](./execution_roadmap.md)
3. [Architecture And Phase Map](./architecture_phase_map.md)
4. [Technology Stack Spec](./specs/technology_stack_spec.md)
5. [Rust Engineering Guidelines](./specs/rust_engineering_guidelines.md)
6. [Module Policy](./engineering/module_policy.md)
7. [Riotbox Improvement Tracks Plan](./plans/riotbox_improvement_tracks_plan.md)
8. [Source Graph Spec](./specs/source_graph_spec.md)
9. [Session File Spec](./specs/session_file_spec.md)
10. [Action Lexicon Spec](./specs/action_lexicon_spec.md)
11. [Replay Model Spec](./specs/replay_model_spec.md)
12. [Audio Core Spec](./specs/audio_core_spec.md)
13. [TUI Screen Spec](./specs/tui_screen_spec.md)
14. [Ghost API Spec](./specs/ghost_api_spec.md)
15. [Preset & Style Spec](./specs/preset_style_spec.md)
16. [Validation & Benchmark Spec](./specs/validation_benchmark_spec.md)
17. [Fixture Corpus Spec](./specs/fixture_corpus_spec.md)
18. [Audio QA Workflow Spec](./specs/audio_qa_workflow_spec.md)
19. [Percussive Force And Beat Impact](./engineering/percussive_force_and_beat_impact.md)
20. [Audio Numeric Values Guide](./engineering/audio_numeric_values.md)
21. [Sound Product Readiness Rubric Spec](./specs/sound_product_readiness_rubric_spec.md)
22. [Release-Grade Musician Demo Bank Spec](./specs/release_grade_musician_demo_bank_spec.md)
23. [20/10 Sound-Product Future Ideas Spec](./specs/sound_product_2010_future_ideas_spec.md)
24. [Source Timing Intelligence Spec](./specs/source_timing_intelligence_spec.md)
25. [Arrangement / Scene System Spec](./specs/arrangement_scene_system_spec.md)
26. [Recovery Notes](./recovery_notes.md)
27. [Phase Definition of Done](./phase_definition_of_done.md)
28. [Research / Decision Log](./research_decision_log.md)
29. [Source Timing Intelligence Plan](./plans/source_timing_intelligence_plan.md)
30. [Source Transport Map Capture Plan](./plans/source_transport_map_capture_plan.md)
31. [MC-202 Source Phrase Planning Plan](./plans/mc202_source_phrase_planning_plan.md)
32. [MC-202 Real-Source Listening Pack Benchmark](./benchmarks/mc202_real_source_listening_pack_v1_2026-06-18.md)
33. [MC-202 Producer-Grade Closeout Benchmark](./benchmarks/mc202_producer_grade_closeout_v1_2026-06-18.md)

This is an orientation list, not a mandatory context bundle. Start with the
entry document relevant to the task and follow only its module routes.

## Why This Order

- The PRD fixes scope and acceptance criteria for the product spine.
- Source graph, session file, and action schema are the main contracts the rest of the system depends on.
- TUI and Ghost API become much easier once actions, state, and persistence are explicit.
- The feral profile can then evolve as a style layer without destabilizing the core.
- Accepted plans such as Source Timing Intelligence are linked here after the stable contracts they extend.

## User Learning Path

If you are trying to learn the current shell rather than read specs first, use this path:

1. [Repo README](../README.md)
   Musician-facing overview, quickstart, limitations, and current product promise.
2. [Jam Recipes](./jam_recipes.md)
   Concrete practice flows for first gestures, capture/reuse, undo, and source comparison.
3. [Local Test Audio Notes](../data/test_audio/README.md)
   Where the current example sources came from and how to fetch them locally.
4. [Example Source Notes](../data/test_audio/examples/README.md)
   Which local example files are good for which kind of learning run.

## Suggested File Layout

```text
docs/
  README.md
  jam_recipes.md
  prd_v1.md
  architecture_phase_map.md
  execution_roadmap.md
  workflow_conventions.md
  workflow/
    github_pr_ci.md
    linear_lifecycle.md
    archive_cleanup.md
    context_hygiene.md
  dev_environment.md
  recovery_notes.md
  phase_definition_of_done.md
  research_decision_log.md
  engineering/
    audio_numeric_values.md
    module_policy.md
    percussive_force_and_beat_impact.md
    percussive_force/
      research_evidence.md
      stage_a_design_history.md
    textual_include_allowlist.txt
    textual_include_inventory_2026-06-29.md
  archive/
    linear_issues/
      README.md
      TEMPLATE.md
      index.md
  assets/
    brand/
      README.md
  benchmarks/
    README.md
    jam_workflow_baseline_2026-04-17.md
    percussive_force_development_matrix_v1.json
    percussive_force_development_matrix_v2.json
    percussive_force_development_matrix_v3.json
    percussive_force_stage_a_protocol_v1.json
    percussive_force_stage_a_protocol_v2.json
    source_holdout_rotation_v2.json
    source_holdout_rotation_v3.json
  plans/
    riotbox_improvement_tracks_plan.md
    source_timing_intelligence_plan.md
    source_transport_map_capture_plan.md
  reviews/
    README.md
    whole_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-17.md
    periodic_codebase_review_2026-04-17_w30_followup.md
    scene_launch_audio_coupling_2026-04-25.md
    external_review_refresh_2026-05-22.md
    tui_include_shell_audit_2026-05-22.md
    riotbox_1431_agent_context_modularization_2026-08-11.md
  spikes/
    cpal_audio_latency_spike.md
    mempalace_evaluation.md
    rust_python_sidecar_transport_spike.md
  screenshots/
    jam_shell_baseline.txt
    jam_shell_trust_action_baseline.txt
    jam_log_screen_baseline.txt
    jam_perform_first_baseline.txt
    jam_inspect_mode_baseline.txt
    jam_first_30_seconds_baseline.txt
    jam_gesture_language_baseline.txt
    jam_tr909_takeover_baseline.txt
    source_screen_baseline.txt
    capture_w30_live_recall_baseline.txt
    w30_audible_preview_baseline.txt
  specs/
    source_graph_spec.md
    source_timing_intelligence_spec.md
    session_file_spec.md
    action_lexicon_spec.md
    replay_model_spec.md
    technology_stack_spec.md
    rust_engineering_guidelines.md
    audio_core_spec.md
    tui_screen_spec.md
    ghost_api_spec.md
    preset_style_spec.md
    validation_benchmark_spec.md
    fixture_corpus_spec.md
    audio_qa_workflow_spec.md
    audio_qa/
      automated_qa.md
      listening_review.md
      manifests_and_artifacts.md
      status_history_and_future.md
    arrangement_scene_system_spec.md
```

## Current Status

- `prd_v1.md`: product spine and MVP framing captured
- `architecture_phase_map.md`: component and P000-P020 phase map captured
- `execution_roadmap.md`: active roadmap with Source Timing Intelligence anchored
- `workflow_conventions.md`: compact normative workflow core and task router
- `workflow/`: focused GitHub/PR/CI, Linear, archive/cleanup, and context-hygiene procedures
- `dev_environment.md`: sandbox, host, search, and environment notes captured
- `jam_recipes.md`: learning-path guide captured
- `recovery_notes.md`: current manual recovery and snapshot-payload label guidance captured
- `specs/technology_stack_spec.md`: Stack Freeze v1 captured with current timing-contract clarification
- `specs/rust_engineering_guidelines.md`: Rust engineering guidelines captured
- `engineering/module_policy.md`: semantic Rust module and textual include policy captured
- `engineering/audio_numeric_values.md`: guide to measurements, runtime
  boundaries, QA thresholds, controls, DSP coefficients, recipe parameters, and
  fixture values, including the RIOTBOX-1402 `0.9161` / `0.92` example
- `engineering/percussive_force_and_beat_impact.md`: compact active semantic
  contract and router for percussive force and beat impact
- `engineering/percussive_force/`: supporting research evidence and explicitly
  historical Stage-A design material; frozen benchmark JSON remains execution authority
- `benchmarks/percussive_force_development_matrix_v1.json`: unexecuted
  RIOTBOX-1428 Stage-A preregistration draft with source-admission, holdout,
  control, cross-product, and promotion boundaries
- `benchmarks/percussive_force_stage_a_protocol_v1.json`: immutable historical
  RIOTBOX-1428 source-blind execution freeze for event qualification, F1--F3
  mechanisms, false controls, matching, reject-only screens, and bounded
  listening; superseded for any retry by the RBX-254 Protocol-v2 boundary
- `benchmarks/percussive_force_development_matrix_v2.json`: RIOTBOX-1428
  Stage-A execution snapshot binding the admitted source registry and exact
  protocol before source qualification or candidate rendering
- `benchmarks/percussive_force_stage_a_protocol_v2.json` and
  `benchmarks/percussive_force_development_matrix_v3.json`: immutable
  RIOTBOX-1430 retry contracts; both are historical after the fail-closed v2
  qualification and must not be retuned from its source results
- `benchmarks/percussive_force_stage_a_protocol_v7.json` and
  `benchmarks/percussive_force_development_matrix_v7.json`: late immutable
  RIOTBOX-1428 F4 qualification snapshots binding the preregistration and
  mechanical-result identities before the subsequent near-identity human
  rejection
- `reviews/riotbox_1428_stage_a_development_qualification_rejection_2026-08-10.md`:
  first fresh frozen Stage-A development qualification, exact access boundary,
  two-source mechanical rejection, evidence hashes, post-execution audit, and
  no-render stop rule
- `reviews/riotbox_1430_stage_a_v2_development_qualification_rejection_2026-08-11.md`:
  second frozen development-only qualification, exact v2 access boundary,
  four-source event counts, fail-closed stop, and no-render/no-listening result
- `reviews/riotbox_1430_stage_a_v5_qualification_pass_matrix_v6_freeze_2026-08-11.md`:
  qualified Development pool, executed three-family/four-source/two-event
  matrix, and frozen F2 near-identity human rejection
- `reviews/riotbox_1434_natural_velocity_control_qualification_2026-08-11.md`:
  natural-velocity snare/whip control, exact human result, and fail-closed
  rejection before any follow-up implementation
- `reviews/riotbox_1433_audible_algorithm_value_audit_2026-08-12.md`:
  bounded retain/replace/retire audit and frozen RIOTBOX-1432 W-30 hook-choice
  handoff
- `reviews/riotbox_1432_w30_source_hook_selection_2026-08-14.md`:
  exact three-source W-30 comparison and fail-closed no-winner closeout
- `engineering/textual_include_inventory_2026-06-29.md`: RIOTBOX-1321 Rust textual include inventory captured
- `engineering/textual_include_allowlist.txt`: current manual guardrail allowlist for textual include owners/counts captured
- `specs/source_graph_spec.md`: Source Graph v1 contract captured
- `specs/source_timing_intelligence_spec.md`: Rust-first all-lane timing contract captured
- `specs/session_file_spec.md`: Session file and recovery boundary captured
- `specs/action_lexicon_spec.md`: action vocabulary and queue/commit semantics captured
- `specs/replay_model_spec.md`: replay model and current allowlist captured
- `specs/audio_core_spec.md`: audio core contract captured
- `specs/tui_screen_spec.md`: TUI screen contract captured
- `specs/ghost_api_spec.md`: Ghost Watch / Assist contract captured
- `specs/preset_style_spec.md`: preset/style contract captured
- `specs/validation_benchmark_spec.md`: validation and benchmark contract captured
- `specs/fixture_corpus_spec.md`: fixture corpus contract captured
- `specs/audio_qa_workflow_spec.md`: compact normative audio-QA core and task router
- `specs/audio_qa/`: automated gates, listening review, manifest/artifact, and
  implementation-status detail loaded only when relevant
- `specs/sound_product_readiness_rubric_spec.md`: 10/10 sound-product readiness rubric captured
- `specs/release_grade_musician_demo_bank_spec.md`: musician demo-bank contract captured
- `specs/sound_product_2010_future_ideas_spec.md`: post-10/10 sound-product future ideas captured
- `specs/arrangement_scene_system_spec.md`: P014 Arrangement / Scene System contract captured
- `phase_definition_of_done.md`: phase DoD with current phase status captured
- `research_decision_log.md`: architecture decisions captured
- `plans/source_timing_intelligence_plan.md`: all-lane Rust-first timing intelligence plan captured
- `plans/source_transport_map_capture_plan.md`: Ingenious First source transport,
  adaptive Source Map, monitor, and capture workflow plan captured
- `plans/mc202_source_phrase_planning_plan.md`: RIOTBOX-1035 MC-202
  source-derived bass / answer phrase planning plan captured
- `plans/riotbox_improvement_tracks_plan.md`: RIOTBOX-1320 improvement-track split captured for semantic modules, runtime audio quality, source-backed instrument work, sidecar/provenance, and QA/UX
- `reviews/riotbox_1431_agent_context_modularization_2026-08-11.md`: reproducible mandatory-context size report, ownership routing audit, and no-drift boundary
- `archive/linear_issues/README.md`: archive policy started
- `archive/linear_issues/TEMPLATE.md`: archive template started
- `archive/linear_issues/index.md`: archive index started
- `assets/brand/README.md`: brand asset notes captured
- `benchmarks/README.md`: benchmark archive policy started
- `benchmarks/audio_qa_artifact_convention_2026-04-26.md`: audio QA baseline-vs-candidate artifact convention captured
- `benchmarks/audio_qa_listening_review_template_2026-04-26.md`: local audio QA listening-review template captured
- `benchmarks/jam_workflow_baseline_2026-04-17.md`: workflow benchmark baseline captured
- `benchmarks/scene_jump_restore_workflow_baseline_2026-04-18.md`: Scene Brain workflow benchmark baseline captured
- `benchmarks/scene_timing_readability_baseline_2026-04-18.md`: Scene Brain timing-readability baseline refreshed for energy-aware live/restore cues
- `benchmarks/scene_guidance_stack_baseline_2026-04-18.md`: Scene Brain queued-guidance stack baseline captured
- `benchmarks/scene_restore_ready_readability_baseline_2026-04-18.md`: Scene Brain restore-ready `scene/energy` readability baseline captured
- `benchmarks/scene_restore_state_contrast_baseline_2026-04-18.md`: Scene Brain restore `waits` vs ready contrast baseline captured
- `benchmarks/scene_post_landed_energy_cue_baseline_2026-04-25.md`: Scene Brain post-landed `scene/energy` and `909 lift` cue baseline refreshed
- `benchmarks/scene_cue_ladder_baseline_2026-04-25.md`: Scene Brain full cue-ladder readability baseline captured
- `benchmarks/scene_footer_tick_readability_baseline_2026-04-25.md`: Scene Brain footer timing tick readability baseline captured
- `benchmarks/scene_contrast_launch_baseline_2026-04-25.md`: Scene Brain contrast launch target readability baseline captured
- `benchmarks/scene_tr909_support_context_baseline_2026-04-26.md`: Scene Brain TR-909 support-context readability baseline captured
- `benchmarks/scene_tr909_support_accent_audio_baseline_2026-04-26.md`: Scene Brain TR-909 support-accent audio-buffer baseline captured
- `benchmarks/lane_recipe_listening_pack_2026-04-26.md`: lane-level recipe listening-pack harness captured
- `benchmarks/w30_preview_smoke_listening_pack_2026-04-26.md`: W-30 preview local listening-pack convention captured
- `benchmarks/listening_manifest_schema_policy_2026-04-29.md`: audio QA manifest schema policy captured
- `benchmarks/listening_manifest_v1_json_contract_2026-04-29.md`: audio QA manifest v1 field-level JSON contract captured
- `benchmarks/observer_audio_correlation_template_2026-04-29.md`: observer/audio correlation template captured
- `benchmarks/observer_audio_summary_json_contract_2026-04-29.md`: observer/audio summary JSON contract captured
- `benchmarks/automated_musical_fitness_v1_2026-06-03.md`: automated musical fitness report contract captured
- `benchmarks/dense_break_performance_pack_v1_2026-06-04.md`: dense-break 8-bar sound-quality Golden Path captured
- `benchmarks/agent_musical_review_pack_v1_2026-06-04.md`: agent-facing dense-break audio review pack captured
- `benchmarks/human_listening_label_corpus_v1_2026-06-04.md`: human listening label corpus contract captured
- `benchmarks/p023_human_listening_label_corpus_2026-07-11.json`: RIOTBOX-1398 real dense, tonal, and sparse human labels captured by artifact hash
- `benchmarks/audio_judge_spike_v1_2026-06-04.md`: CLAP/MERT-style audio judge spike boundary captured
- `benchmarks/musical_pass_gate_policy_v1_2026-06-04.md`: agent/human musical-pass verdict policy captured
- `benchmarks/sound_excellence_source_corpus_v1_2026-06-05.md`: P023 real-source coverage contract captured
- `benchmarks/source_holdout_rotation_v1.json`: RIOTBOX-1423 legal CC0
  historical predecessor for the legal CC0 corpus and rotating holdouts
- `benchmarks/source_holdout_rotation_v2.json`: active RIOTBOX-1428 legal CC0
  corpus snapshot with native-rate development admissions and unchanged active
  holdout identities
- `benchmarks/weak_output_fix_routing_v1_2026-06-05.md`: P023 weak-output failure-to-production-fix routing contract captured
- `benchmarks/source_family_release_demo_coverage_v1_2026-06-12.md`: P023 source-family release-demo coverage gate captured
- `benchmarks/sound_quality_readiness_report_v1_2026-06-12.md`: P023 sound-quality readiness status report captured
- `benchmarks/mc202_real_source_listening_pack_v1_2026-06-18.md`: RIOTBOX-1278 MC-202 dense/non-dense real-source listening-review scaffold captured
- `benchmarks/p011_exit_evidence_manifest.json`: machine-checkable P011 exit evidence index captured
- `reviews/external_review_refresh_2026-05-22.md`: external review freshness, audio-QA, and runtime module-cut refresh captured
- `reviews/tui_include_shell_audit_2026-05-22.md`: TUI include-shell ownership audit and leaf-first conversion recommendation captured
- `reviews/p014_exit_candidate_review_2026-05-30.md`: P014 Arrangement / Scene
  exit-candidate evidence and PR/merge blocker captured
- `reviews/p015_exit_evidence_checklist_2026-05-31.md`: P015 Productization
  Alpha exit evidence checklist captured
- `reviews/p015_exit_review_2026-05-31.md`: P015 bounded Productization Alpha
  exit review captured
- `reviews/p016_export_action_boundary_2026-05-31.md`: P016 export action
  boundary before file-writing workflow captured
- `reviews/riotbox_1398_human_listening_review_2026-07-11.md`: RIOTBOX-1398 real human verdicts and the selected bass-pressure follow-up captured
- `benchmarks/product_export_reproducibility_boundary_2026-05-07.md`: normalized product-export reproducibility boundary captured
- `benchmarks/stage_style_stability_proof_2026-05-07.md`: normalized stage-style repeated-run stability proof captured
- `benchmarks/jam_footer_color_hierarchy_baseline_2026-04-25.md`: Jam footer color hierarchy readability baseline captured
- `benchmarks/capture_do_next_readability_baseline_2026-04-25.md`: Capture `Do Next` readability baseline captured
- `benchmarks/capture_pending_do_next_readability_baseline_2026-04-25.md`: Capture pending `Do Next` readability baseline captured
- `reviews/README.md`: review artifact handling and follow-up freshness rule captured
- `reviews/whole_codebase_review_2026-04-13.md`: review captured
- `reviews/periodic_codebase_review_2026-04-13.md`: review captured
- `reviews/periodic_codebase_review_2026-04-17.md`: review captured
- `reviews/periodic_codebase_review_2026-04-17_w30_followup.md`: review captured
- `reviews/periodic_codebase_review_2026-04-18.md`: review captured
- `reviews/jam_first_use_feedback_2026-04-18.md`: first-use UX feedback captured
- `reviews/feral_policy_entry_audit_2026-04-26.md`: Feral policy entry audit captured
- `reviews/mc202_mvp_exit_review_2026-04-26.md`: MC-202 MVP exit review captured
- `reviews/periodic_scene_brain_tui_seam_review_2026-04-25.md`: Scene Brain TUI seam review captured
- `reviews/scene_launch_audio_coupling_2026-04-25.md`: Scene launch to TR-909 audio-coupling audit captured
- `reviews/periodic_jam_hierarchy_seam_review_2026-04-26.md`: Jam hierarchy seam review captured
- `reviews/periodic_w30_capture_seam_review_2026-04-26.md`: W-30 capture seam review captured
- `reviews/routine_audio_output_audit_2026-04-26.md`: README and Jam recipe control/audio proof audit captured
- `reviews/w30_mvp_gap_review_2026-04-26.md`: W-30 MVP gap review captured
- `reviews/w30_mvp_exit_review_2026-04-26.md`: W-30 MVP exit review captured
- `reviews/scene_brain_mvp_gap_review_2026-04-26.md`: Scene Brain MVP gap review captured
- `reviews/rust_hotspot_semantic_review_2026-04-29.md`: Rust hotspot semantic review captured
- `reviews/p009_feral_policy_gap_review_2026-04-29.md`: Feral policy gap review captured
- `reviews/p009_feral_policy_exit_review_2026-04-29.md`: Feral policy exit review captured
- `reviews/p010_ghost_watch_assist_exit_review_2026-04-29.md`: Ghost Watch / Assist exit review captured
- `reviews/p011_replay_hardening_checkpoint_2026-04-29.md`: P011 replay hardening checkpoint captured
- `reviews/p011_qa_gate_periodic_review_2026-04-30.md`: P011 QA gate review captured
- `reviews/p011_replay_recovery_codebase_review_2026-04-30.md`: P011 replay/recovery codebase review captured
- `reviews/p011_replay_recovery_exit_checklist_2026-04-30.md`: P011 replay/recovery exit checklist captured
- `reviews/p011_evidence_gate_periodic_review_2026-05-07.md`: P011 evidence-gate periodic review captured
- `reviews/p011_evidence_gate_codebase_review_2026-05-07.md`: P011 executable evidence-gate codebase review captured
- `reviews/snapshot_payload_hydration_boundary_2026-04-30.md`: snapshot payload hydration boundary review captured
- `reviews/docs_consistency_review_2026-05-03.md`: docs consistency review captured
- `reviews/source_showcase_false_positive_review_2026-05-03.md`: source-showcase audio QA false-positive review captured
- `reviews/representative_showcase_musical_quality_2026-05-14.md`: representative showcase musical-quality review captured
- `reviews/w30_chop_articulation_showcase_review_2026-05-14.md`: W-30 chop articulation showcase review captured
- `reviews/mc202_bass_phrase_variation_showcase_review_2026-05-20.md`: MC-202 bass phrase-variation showcase review captured
- `reviews/p013_representative_showcase_seam_review_2026-05-20.md`: P013 representative showcase seam review captured
- `reviews/p013_mc202_representative_quality_gate_review_2026-05-29.md`: P013 MC-202 representative quality-gate review captured
- `reviews/p013_w30_source_accent_dynamics_review_2026-05-29.md`: P013 W-30 source accent-dynamics review captured
- `reviews/p013_tr909_source_accent_dynamics_review_2026-05-29.md`: P013 TR-909 source accent-dynamics review captured
- `reviews/p013_all_lane_mix_movement_review_2026-05-29.md`: P013 all-lane mix movement review captured
- `reviews/riotbox_1408_source_backed_w30_resample_review_2026-07-21.md`: source-backed W-30 resample implementation and human weak verdict captured
- `reviews/p013_mc202_source_contour_review_2026-05-29.md`: P013 MC-202 source contour review captured
- `reviews/p013_exit_review_2026-05-29.md`: P013 bounded all-lane musical-depth exit review captured
- `reviews/p012_source_timing_qa_contract_review_2026-05-20.md`: P012 Source Timing QA contract review captured
- `reviews/p012_current_source_timing_spine_review_2026-05-21.md`: P012 current Source Timing spine review captured
- `reviews/p012_source_timing_report_expectation_gate_review_2026-05-21.md`: P012 Source Timing report expectation gate review captured
- `reviews/p012_source_timing_validator_surface_review_2026-05-22.md`: P012 Source Timing validator surface review captured
- `reviews/p012_exit_review_2026-05-28.md`: P012 bounded Source Timing foundation exit review captured
- `spikes/cpal_audio_latency_spike.md`: draft started
- `spikes/mempalace_evaluation.md`: draft started
- `spikes/rust_python_sidecar_transport_spike.md`: draft started
- `screenshots/jam_shell_baseline.txt`: baseline captured
- `screenshots/jam_shell_trust_action_baseline.txt`: baseline captured
- `screenshots/jam_log_screen_baseline.txt`: baseline captured
- `screenshots/jam_perform_first_baseline.txt`: baseline captured
- `screenshots/jam_inspect_mode_baseline.txt`: baseline captured
- `screenshots/jam_taste_proof_glossary.md`: P015 Jam taste/proof glossary captured
- `screenshots/jam_first_30_seconds_baseline.txt`: baseline captured
- `screenshots/jam_gesture_language_baseline.txt`: baseline captured
- `screenshots/jam_tr909_takeover_baseline.txt`: baseline captured
- `screenshots/jam_tr909_render_seam_baseline.txt`: baseline captured
- `screenshots/jam_tr909_render_diagnostics_baseline.txt`: baseline captured
- `screenshots/jam_tr909_pattern_adoption_baseline.txt`: baseline captured
- `screenshots/jam_tr909_phrase_variation_baseline.txt`: baseline captured
- `screenshots/source_screen_baseline.txt`: baseline captured
- `screenshots/capture_screen_baseline.txt`: baseline captured
- `screenshots/capture_w30_live_recall_baseline.txt`: baseline captured
- `screenshots/w30_audible_preview_baseline.txt`: baseline captured
- `screenshots/w30_resample_tap_baseline.txt`: baseline captured
- `screenshots/w30_resample_lab_diagnostics_baseline.txt`: baseline captured
- `screenshots/w30_diagnostics_baseline.txt`: baseline captured
- `screenshots/w30_bank_forge_diagnostics_baseline.txt`: baseline captured
- other review artifacts and screenshots are historical baselines unless referenced by the active roadmap, DoD, or workflow conventions

# Riotbox Docs

Status: active implementation documentation

This directory holds implementation-facing specifications, plans, reviews, benchmarks, and workflow notes derived from the strategy documents in `plan/` and from shipped Riotbox slices.

## Source of Truth

- `plan/riotbox_masterplan.md`
  Source of truth for product structure, MVP, phases, and system architecture.
- `plan/riotbox_liam_howlett_feral_addendum.md`
  Source of truth for the `feral_rebuild` profile and its backlog deltas.

## Documentation Rules

- Stable core contracts live in `docs/`.
- Exploratory thinking and generative planning stay in `plan/`.
- Accepted implementation plans live in `docs/plans/` and should be anchored from the roadmap, phase definition of done, README, and decision log when they freeze a durable direction.
- Profile behavior must be expressed as policy, preset, or scoring extensions, not as a parallel product architecture.
- Incoming refinements to the feral addendum should update profile-oriented specs, not the core contracts unless they truly change the core.

## Recommended Reading / Build Context Order

1. [PRD v1](./prd_v1.md)
2. [Execution Roadmap](./execution_roadmap.md)
3. [Architecture And Phase Map](./architecture_phase_map.md)
4. [Technology Stack Spec](./specs/technology_stack_spec.md)
5. [Rust Engineering Guidelines](./specs/rust_engineering_guidelines.md)
6. [Source Graph Spec](./specs/source_graph_spec.md)
7. [Session File Spec](./specs/session_file_spec.md)
8. [Action Lexicon Spec](./specs/action_lexicon_spec.md)
9. [Replay Model Spec](./specs/replay_model_spec.md)
10. [Audio Core Spec](./specs/audio_core_spec.md)
11. [TUI Screen Spec](./specs/tui_screen_spec.md)
12. [Ghost API Spec](./specs/ghost_api_spec.md)
13. [Preset & Style Spec](./specs/preset_style_spec.md)
14. [Validation & Benchmark Spec](./specs/validation_benchmark_spec.md)
15. [Fixture Corpus Spec](./specs/fixture_corpus_spec.md)
16. [Audio QA Workflow Spec](./specs/audio_qa_workflow_spec.md)
17. [Sound Product Readiness Rubric Spec](./specs/sound_product_readiness_rubric_spec.md)
18. [Release-Grade Musician Demo Bank Spec](./specs/release_grade_musician_demo_bank_spec.md)
19. [20/10 Sound-Product Future Ideas Spec](./specs/sound_product_2010_future_ideas_spec.md)
20. [Source Timing Intelligence Spec](./specs/source_timing_intelligence_spec.md)
21. [Arrangement / Scene System Spec](./specs/arrangement_scene_system_spec.md)
22. [Recovery Notes](./recovery_notes.md)
23. [Phase Definition of Done](./phase_definition_of_done.md)
24. [Research / Decision Log](./research_decision_log.md)
25. [Source Timing Intelligence Plan](./plans/source_timing_intelligence_plan.md)
26. [Source Transport Map Capture Plan](./plans/source_transport_map_capture_plan.md)

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
  dev_environment.md
  recovery_notes.md
  phase_definition_of_done.md
  research_decision_log.md
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
  plans/
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
    arrangement_scene_system_spec.md
```

## Current Status

- `prd_v1.md`: product spine and MVP framing captured
- `architecture_phase_map.md`: component and P000-P020 phase map captured
- `execution_roadmap.md`: active roadmap with Source Timing Intelligence anchored
- `workflow_conventions.md`: active contributor / agent workflow conventions captured
- `dev_environment.md`: sandbox, host, search, and environment notes captured
- `jam_recipes.md`: learning-path guide captured
- `recovery_notes.md`: current manual recovery and snapshot-payload label guidance captured
- `specs/technology_stack_spec.md`: Stack Freeze v1 captured with current timing-contract clarification
- `specs/rust_engineering_guidelines.md`: Rust engineering guidelines captured
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
- `specs/audio_qa_workflow_spec.md`: audio QA workflow plan captured
- `specs/sound_product_readiness_rubric_spec.md`: 10/10 sound-product readiness rubric captured
- `specs/release_grade_musician_demo_bank_spec.md`: musician demo-bank contract captured
- `specs/sound_product_2010_future_ideas_spec.md`: post-10/10 sound-product future ideas captured
- `specs/arrangement_scene_system_spec.md`: P014 Arrangement / Scene System contract captured
- `phase_definition_of_done.md`: phase DoD with current phase status captured
- `research_decision_log.md`: architecture decisions captured
- `plans/source_timing_intelligence_plan.md`: all-lane Rust-first timing intelligence plan captured
- `plans/source_transport_map_capture_plan.md`: Ingenious First source transport,
  adaptive Source Map, monitor, and capture workflow plan captured
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
- `benchmarks/audio_judge_spike_v1_2026-06-04.md`: CLAP/MERT-style audio judge spike boundary captured
- `benchmarks/musical_pass_gate_policy_v1_2026-06-04.md`: agent/human musical-pass verdict policy captured
- `benchmarks/sound_excellence_source_corpus_v1_2026-06-05.md`: P023 real-source coverage contract captured
- `benchmarks/weak_output_fix_routing_v1_2026-06-05.md`: P023 weak-output failure-to-production-fix routing contract captured
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

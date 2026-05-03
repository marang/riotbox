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
3. [Technology Stack Spec](./specs/technology_stack_spec.md)
4. [Rust Engineering Guidelines](./specs/rust_engineering_guidelines.md)
5. [Source Graph Spec](./specs/source_graph_spec.md)
6. [Session File Spec](./specs/session_file_spec.md)
7. [Action Lexicon Spec](./specs/action_lexicon_spec.md)
8. [Replay Model Spec](./specs/replay_model_spec.md)
9. [Audio Core Spec](./specs/audio_core_spec.md)
10. [TUI Screen Spec](./specs/tui_screen_spec.md)
11. [Ghost API Spec](./specs/ghost_api_spec.md)
12. [Preset & Style Spec](./specs/preset_style_spec.md)
13. [Validation & Benchmark Spec](./specs/validation_benchmark_spec.md)
14. [Fixture Corpus Spec](./specs/fixture_corpus_spec.md)
15. [Audio QA Workflow Spec](./specs/audio_qa_workflow_spec.md)
16. [Recovery Notes](./recovery_notes.md)
17. [Phase Definition of Done](./phase_definition_of_done.md)
18. [Research / Decision Log](./research_decision_log.md)
19. [Source Timing Intelligence Plan](./plans/source_timing_intelligence_plan.md)

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
  execution_roadmap.md
  workflow_conventions.md
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
  reviews/
    whole_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-17.md
    periodic_codebase_review_2026-04-17_w30_followup.md
    scene_launch_audio_coupling_2026-04-25.md
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
```

## Current Status

- `prd_v1.md`: product spine and MVP framing captured
- `execution_roadmap.md`: active roadmap with Source Timing Intelligence anchored
- `workflow_conventions.md`: active contributor / agent workflow conventions captured
- `jam_recipes.md`: learning-path guide captured
- `recovery_notes.md`: current manual recovery and snapshot-payload label guidance captured
- `specs/technology_stack_spec.md`: Stack Freeze v1 captured with current timing-contract clarification
- `specs/rust_engineering_guidelines.md`: Rust engineering guidelines captured
- `specs/source_graph_spec.md`: Source Graph v1 contract captured
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
- `phase_definition_of_done.md`: phase DoD with current phase status captured
- `research_decision_log.md`: architecture decisions captured
- `plans/source_timing_intelligence_plan.md`: all-lane Rust-first timing intelligence plan captured
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
- `benchmarks/jam_footer_color_hierarchy_baseline_2026-04-25.md`: Jam footer color hierarchy readability baseline captured
- `benchmarks/capture_do_next_readability_baseline_2026-04-25.md`: Capture `Do Next` readability baseline captured
- `benchmarks/capture_pending_do_next_readability_baseline_2026-04-25.md`: Capture pending `Do Next` readability baseline captured
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
- `reviews/snapshot_payload_hydration_boundary_2026-04-30.md`: snapshot payload hydration boundary review captured
- `reviews/docs_consistency_review_2026-05-03.md`: docs consistency review captured
- `spikes/cpal_audio_latency_spike.md`: draft started
- `spikes/mempalace_evaluation.md`: draft started
- `spikes/rust_python_sidecar_transport_spike.md`: draft started
- `screenshots/jam_shell_baseline.txt`: baseline captured
- `screenshots/jam_shell_trust_action_baseline.txt`: baseline captured
- `screenshots/jam_log_screen_baseline.txt`: baseline captured
- `screenshots/jam_perform_first_baseline.txt`: baseline captured
- `screenshots/jam_inspect_mode_baseline.txt`: baseline captured
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

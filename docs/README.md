# Riotbox Docs

Status: initial spec scaffold

This directory holds implementation-facing specifications derived from the strategy documents in `plan/`.

## Source of Truth

- `plan/riotbox_masterplan.md`
  Source of truth for product structure, MVP, phases, and system architecture.
- `plan/riotbox_liam_howlett_feral_addendum.md`
  Source of truth for the `feral_rebuild` profile and its backlog deltas.

## Documentation Rules

- Stable core contracts live in `docs/`.
- Exploratory thinking and generative planning stay in `plan/`.
- Profile behavior must be expressed as policy, preset, or scoring extensions, not as a parallel product architecture.
- Incoming refinements to the feral addendum should update profile-oriented specs, not the core contracts unless they truly change the core.

## Recommended Build Order

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
16. [Phase Definition of Done](./phase_definition_of_done.md)
17. [Research / Decision Log](./research_decision_log.md)

## Why This Order

- The PRD fixes scope and acceptance criteria for the MVP.
- Source graph, session file, and action schema are the main contracts the rest of the system depends on.
- TUI and Ghost API become much easier once actions, state, and persistence are explicit.
- The feral profile can then evolve as a style layer without destabilizing the core.

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
  phase_definition_of_done.md
  research_decision_log.md
  archive/
    linear_issues/
      README.md
      TEMPLATE.md
      index.md
  benchmarks/
    README.md
    jam_workflow_baseline_2026-04-17.md
  reviews/
    whole_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-17.md
    periodic_codebase_review_2026-04-17_w30_followup.md
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

- `prd_v1.md`: draft started
- `execution_roadmap.md`: draft started
- `workflow_conventions.md`: draft started
- `jam_recipes.md`: learning-path guide captured
- `specs/technology_stack_spec.md`: draft started
- `specs/rust_engineering_guidelines.md`: draft started
- `specs/source_graph_spec.md`: draft started
- `specs/session_file_spec.md`: draft started
- `specs/action_lexicon_spec.md`: draft started
- `specs/replay_model_spec.md`: draft started
- `specs/audio_core_spec.md`: draft started
- `specs/tui_screen_spec.md`: draft started
- `specs/ghost_api_spec.md`: draft started
- `specs/preset_style_spec.md`: draft started
- `specs/validation_benchmark_spec.md`: draft started
- `specs/fixture_corpus_spec.md`: draft started
- `specs/audio_qa_workflow_spec.md`: audio QA workflow plan captured
- `phase_definition_of_done.md`: draft started
- `research_decision_log.md`: draft started
- `archive/linear_issues/README.md`: archive policy started
- `archive/linear_issues/TEMPLATE.md`: archive template started
- `archive/linear_issues/index.md`: archive index started
- `benchmarks/README.md`: benchmark archive policy started
- `benchmarks/jam_workflow_baseline_2026-04-17.md`: workflow benchmark baseline captured
- `benchmarks/scene_jump_restore_workflow_baseline_2026-04-18.md`: Scene Brain workflow benchmark baseline captured
- `benchmarks/scene_timing_readability_baseline_2026-04-18.md`: Scene Brain timing-readability baseline refreshed for energy-aware live/restore cues
- `benchmarks/scene_guidance_stack_baseline_2026-04-18.md`: Scene Brain queued-guidance stack baseline captured
- `benchmarks/scene_restore_ready_readability_baseline_2026-04-18.md`: Scene Brain restore-ready readability baseline captured
- `reviews/whole_codebase_review_2026-04-13.md`: review captured
- `reviews/periodic_codebase_review_2026-04-13.md`: review captured
- `reviews/periodic_codebase_review_2026-04-17.md`: review captured
- `reviews/periodic_codebase_review_2026-04-17_w30_followup.md`: review captured
- `reviews/periodic_codebase_review_2026-04-18.md`: review captured
- `reviews/jam_first_use_feedback_2026-04-18.md`: first-use UX feedback captured
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
- all other specs: not started

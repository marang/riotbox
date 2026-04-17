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
15. [Phase Definition of Done](./phase_definition_of_done.md)
16. [Research / Decision Log](./research_decision_log.md)

## Why This Order

- The PRD fixes scope and acceptance criteria for the MVP.
- Source graph, session file, and action schema are the main contracts the rest of the system depends on.
- TUI and Ghost API become much easier once actions, state, and persistence are explicit.
- The feral profile can then evolve as a style layer without destabilizing the core.

## Suggested File Layout

```text
docs/
  README.md
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
  reviews/
    whole_codebase_review_2026-04-13.md
    periodic_codebase_review_2026-04-13.md
  spikes/
    cpal_audio_latency_spike.md
    mempalace_evaluation.md
    rust_python_sidecar_transport_spike.md
  screenshots/
    jam_shell_baseline.txt
    jam_shell_trust_action_baseline.txt
    jam_log_screen_baseline.txt
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
```

## Current Status

- `prd_v1.md`: draft started
- `execution_roadmap.md`: draft started
- `workflow_conventions.md`: draft started
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
- `phase_definition_of_done.md`: draft started
- `research_decision_log.md`: draft started
- `archive/linear_issues/README.md`: archive policy started
- `archive/linear_issues/TEMPLATE.md`: archive template started
- `archive/linear_issues/index.md`: archive index started
- `reviews/whole_codebase_review_2026-04-13.md`: review captured
- `reviews/periodic_codebase_review_2026-04-13.md`: review captured
- `spikes/cpal_audio_latency_spike.md`: draft started
- `spikes/mempalace_evaluation.md`: draft started
- `spikes/rust_python_sidecar_transport_spike.md`: draft started
- `screenshots/jam_shell_baseline.txt`: baseline captured
- `screenshots/jam_shell_trust_action_baseline.txt`: baseline captured
- `screenshots/jam_log_screen_baseline.txt`: baseline captured
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
- `screenshots/w30_diagnostics_baseline.txt`: baseline captured
- all other specs: not started

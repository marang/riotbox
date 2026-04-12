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
2. `Source Graph Spec`
3. `Session File Spec`
4. `Action Lexicon Spec`
5. `Audio Core Spec`
6. `TUI Screen Spec`
7. `Ghost API Spec`
8. `Preset & Style Spec`
9. `Validation & Golden Render Spec`

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
  specs/
    source_graph_spec.md
    session_file_spec.md
    action_lexicon_spec.md
    audio_core_spec.md
    tui_screen_spec.md
    ghost_api_spec.md
    preset_style_spec.md
    validation_spec.md
```

## Current Status

- `prd_v1.md`: draft started
- all other specs: not started

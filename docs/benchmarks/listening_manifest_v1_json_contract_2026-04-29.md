# Listening Manifest v1 JSON Contract

Date: 2026-04-29

## Purpose

Riotbox audio QA pack runners write `manifest.json` files beside generated WAVs,
metrics, Markdown reports, and local listening notes.

This document defines the stable JSON contract for schema version `1`. It is the
contract that automation may rely on when it needs to prove an output path exists
without parsing pack-specific Markdown.

## Stable Envelope

Every schema version `1` listening manifest must be a JSON object with these
top-level fields:

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema_version` | integer | yes | Listening manifest schema version. Current value: `1`. |
| `pack_id` | non-empty string | yes | Stable id for the pack runner or convention. |
| `result` | string | yes | Overall pack verdict. Current values: `pass` or `fail`. |
| `artifacts` | array | yes | Generated artifacts that make the run inspectable. Must not be empty. |

Readers should treat additional top-level fields as pack-specific data unless a
future schema version explicitly promotes them into the shared contract. The
repo validator may still validate named optional QA contracts when a producer
emits them.

## Artifact Records

Every entry in `artifacts` must be a JSON object with these stable fields:

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `role` | non-empty string | yes | Stable role inside the pack, such as `baseline`, `candidate`, `full_mix`, or `comparison`. |
| `kind` | non-empty string | yes | Artifact kind, such as `audio_wav`, `markdown_report`, or `markdown_readme`. |
| `path` | non-empty string | yes | Local path to the generated artifact. |
| `metrics_path` | string or null | no | Sibling metrics path when the artifact has one. Missing should be treated like `null`. |
| `case_id` | string or null | no | Stable case id for multi-case packs. Missing should be treated like `null`. |

Schema version `1` readers must not assume artifact files are committed. Local
audio QA normally writes ignored artifacts under `artifacts/audio_qa/` or a temp
directory.

## Pack-Specific Fields

Schema version `1` intentionally keeps these fields pack-specific:

- `source`, `source_window`, source start, and source duration metadata
- tempo, grid, bar, frame, sample-rate, and channel-count metadata
- `thresholds`
- `metrics`
- `cases`
- verification commands or pack notes

Pack-specific fields should stay deterministic and machine-readable, but adding
or extending them does not require a schema version bump.

## Named Optional QA Contracts

These fields are not part of the required stable envelope, but the repo-local
validator checks them when present so automation does not accept malformed known
QA evidence.

### `feral_scorecard`

The Feral grid pack may emit a top-level `feral_scorecard` object. When present,
it must include:

- non-empty string fields: `readiness`, `break_rebuild_potential`, `top_reason`
- non-negative integer fields: `hook_fragment_count`, `break_support_count`,
  `quote_risk_count`, `capture_candidate_count`
- boolean fields: `source_backed`, `generated`, `fallback_like`
- non-empty string arrays: `lane_gestures`, `material_sources`, `warnings`

This keeps the Feral QA explanation machine-readable without making every
schema version `1` listening pack emit Feral-specific fields.

## Current Producers

Current schema version `1` producer ids:

| Pack id | Producer | Current role in QA |
| --- | --- | --- |
| `w30-preview-smoke` | `w30_preview_render` / `w30_preview_compare` | Baseline-vs-candidate W-30 preview smoke output. |
| `lane-recipe-listening-pack` | `lane_recipe_pack` | TR-909, Scene-coupled TR-909, and MC-202 lane recipe comparisons. |
| `feral-before-after` | `feral_before_after_pack` | Source excerpt, Riotbox-transformed after render, stems, and before-then-after listening file. |
| `feral-grid-demo` | `feral_grid_pack` | Grid-locked source-aware TR-909/W-30 source-first and generated-support Feral demo output. |

## Compatibility Rules

Do not bump `schema_version` for:

- adding a new pack-specific metric
- adding a new pack-specific threshold
- adding a new artifact role
- adding a new optional top-level field
- adding a new producer id

Bump `schema_version` only when one of the stable envelope or artifact fields is
renamed, removed, changes type, or changes meaning in a way that would break
existing readers.

When bumping the version:

- update `LISTENING_MANIFEST_SCHEMA_VERSION` in `riotbox_audio::listening_manifest`
- update this contract or add a successor document
- keep readers tolerant of older versions where practical
- update producer tests and validator fixtures

## Non-Goals

This contract does not define perceptual quality, musical usefulness, waveform
similarity, or human listening pass/fail policy. Those remain part of the audio
QA workflow and local listening-review layer.

This contract also does not freeze the shape of `metrics` or `thresholds`.
Automation that needs pack-specific metrics must document that dependency
separately.

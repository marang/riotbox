# Listening Manifest Schema Policy

Date: 2026-04-29

## Purpose

Riotbox local audio QA packs write `manifest.json` files beside generated WAV, metrics, and report artifacts. The manifest exists so agents and humans can connect output evidence to a concrete pack run instead of relying on logs or filenames alone.

This policy freezes the small shared contract for schema version `1` while leaving pack-specific metrics and thresholds flexible enough for the current local-first QA phase.

## Schema Version 1 Stable Fields

Every generated audio QA manifest should include these top-level fields:

- `schema_version`: integer schema version. Current value: `1`.
- `pack_id`: stable string identifying the pack runner or convention.
- `artifacts`: array of generated artifact records.
- `result`: stable string result, currently `pass` or `fail`.

Every artifact record should include:

- `role`: stable role within the pack, such as `baseline`, `candidate`, `comparison`, or `full_mix`.
- `kind`: stable artifact kind, such as `audio_wav`, `markdown_report`, or `markdown_readme`.
- `path`: local path to the generated artifact.
- `metrics_path`: local path to sibling metrics when the artifact has one, otherwise `null`.

Artifact records may include:

- `case_id`: stable case id when a pack contains multiple cases.

## Pack-Specific Fields

Schema version `1` allows packs to define their own additional top-level fields for:

- source paths and source-window metadata
- sample rate, channel count, tempo, grid, bar, and duration metadata
- thresholds
- metrics
- case lists
- verification commands

Those fields should be deterministic and descriptive, but they are not yet a public compatibility contract across every pack.

## Compatibility Rule

Do not bump `schema_version` for additive pack-specific fields.

Bump `schema_version` only when one of the stable fields above is renamed, removed, changes type, or changes meaning in a way that would break manifest readers.

When bumping the version:

- keep readers tolerant of older versions where practical
- document the migration in this file or its successor
- update `LISTENING_MANIFEST_SCHEMA_VERSION` in `riotbox_audio::listening_manifest`
- update manifest tests for every pack runner that writes `manifest.json`

## Current Producers

Current schema version `1` producers:

- W-30 preview smoke comparison
- lane recipe listening pack
- Feral before/after pack
- Feral grid demo pack

## Current Non-Goals

This policy does not freeze a complete JSON Schema document yet.

This policy does not define perceptual audio quality thresholds, waveform comparison semantics, or release gates. It only keeps the machine-readable manifest envelope stable enough for local QA automation and observer/audio correlation.

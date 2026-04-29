# Observer / Audio Summary JSON Contract

Date: 2026-04-29

## Purpose

`observer_audio_correlate --json` emits the machine-readable summary that pairs:

- control-path evidence from `riotbox-app --observer <events.ndjson>`
- output-path evidence from an audio QA `manifest.json`

This contract documents the stable summary fields that automation may depend on for schema version `1`.

## Schema Version 1 Stable Fields

Every observer/audio JSON summary should include:

- `schema`: stable string marker. Current value: `riotbox.observer_audio_summary.v1`.
- `schema_version`: integer schema version. Current value: `1`.
- `control_path`: object describing whether committed user/action evidence is present.
- `output_path`: object describing whether manifest-backed output evidence is present.
- `needs_human_listening`: boolean reminder that the summary is not a complete musical-quality approval.

The `control_path` object should include:

- `present`: boolean verdict for committed control-path evidence.
- `observer_schema`: observer event schema string when present.
- `launch_mode`: launch mode such as `ingest` or `load`.
- `audio_runtime_status`: latest observed audio runtime status.
- `key_outcomes`: array of compact key outcome strings.
- `first_commit`: compact description of the first committed transport action, or `none`.

The `output_path` object should include:

- `present`: boolean verdict for passing non-collapsed output evidence.
- `issues`: array of missing or collapsed output-evidence issue strings.
- `pack_id`: source audio QA pack id.
- `manifest_result`: source manifest result.
- `artifact_count`: number of manifest artifacts.
- `metrics`: object containing the currently required output metric fields; values may be numbers or `null` when evidence is missing.

## Compatibility Rule

Do not bump `schema_version` for additive fields that do not change the meaning of existing fields.

Bump `schema_version` when a stable field above is renamed, removed, changes type, or changes meaning in a way that would break automation.

When bumping the version:

- update `observer_audio_correlate` tests
- update `just observer-audio-correlate-json-fixture`
- update the GitHub Actions audio QA smoke assertion
- update this contract or add a successor document

## Current CI Smoke

The committed fixture JSON smoke currently requires:

- `schema == "riotbox.observer_audio_summary.v1"`
- `schema_version == 1`
- `control_path.present == true`
- `output_path.present == true`
- `output_path.issues` is empty
- `scripts/validate_observer_audio_summary_json.py` accepts the generated summary shape

## Current Non-Goals

This contract is not a full JSON Schema file yet.

It does not define pack-specific manifest metrics, perceptual audio quality thresholds, waveform comparison semantics, or human listening approval.

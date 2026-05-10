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
- `commit_count`: integer count of committed actions seen in the observer stream.
- `commit_boundaries`: array of unique commit boundary labels seen in committed actions.
- `observer_source_timing`: `null` or compact Source Timing Intelligence readiness
  copied from the observer snapshot when a Source Graph was attached.

When non-null, `control_path.observer_source_timing` should include:

- `source_id`
- `cue`: musician-facing Source Timing trust cue such as `grid locked`,
  `needs confirm`, `listen first`, `fallback grid`, or `not available`.
- `bpm_estimate`
- `bpm_confidence`
- `quality`
- `degraded_policy`
- `primary_hypothesis_id`
- `hypothesis_count`
- `anchor_evidence`: `null` or compact primary anchor counts copied from the
  observer snapshot.
- `groove_evidence`: `null` or compact primary groove residual evidence copied
  from the observer snapshot.
- `primary_warning_code`
- `warning_codes`

The observer should source `cue`, `quality`, `degraded_policy`,
`primary_warning_code`, compact `anchor_evidence`, and compact
`groove_evidence` from the shared Jam source timing summary. Raw
beat/downbeat/phrase detail, hypothesis ids, counts, and full warning-code lists
may remain direct Source Graph diagnostics.

`primary_warning_code` is the summary's focused warning, selected by the shared
musician-facing warning priority. `warning_codes` remains the full diagnostic
list from Source Graph timing state when present.

The `cue` must match the shared musician-facing label for `degraded_policy`; a
summary that says `degraded_policy=manual_confirm` but `cue=listen first` is
malformed because the control path would no longer explain timing trust
consistently.

The `output_path` object should include:

- `present`: boolean verdict for passing non-collapsed output evidence.
- `issues`: array of missing or collapsed output-evidence issue strings.
- `pack_id`: source audio QA pack id.
- `manifest_result`: source manifest result.
- `artifact_count`: number of manifest artifacts.
- `grid_bpm_source`: manifest grid BPM source, one of `unknown`,
  `user_override`, `source_timing`, or `static_default`. Older/non-grid
  manifests may report `unknown`.
- `grid_bpm_decision_reason`: manifest grid BPM decision reason, one of
  `unknown`, `user_override`, `source_timing_ready`,
  `source_timing_needs_review_manual_confirm`,
  `source_timing_requires_manual_confirm`, `source_timing_not_ready`,
  `source_timing_missing_bpm`, or `source_timing_invalid_bpm`.
  Older/non-grid manifests may report `unknown`.
- `source_timing_bpm_delta`: manifest source/grid BPM delta when present, or
  `null`.
- `source_timing`: `null` or a compact object copied from manifest
  source-timing readiness evidence.
- `source_timing_alignment`: `null` or compact evidence comparing observer-side
  Source Timing readiness with manifest-side Source Timing evidence.
- `source_timing_anchor_alignment`: `null` or compact evidence comparing
  observer-side primary anchor counts with manifest-side primary anchor counts.
- `source_timing_groove_alignment`: `null` or compact evidence comparing
  observer-side primary groove residual evidence with manifest-side primary
  groove residual evidence.
- `metrics`: object containing every currently required output metric field; values may be numbers or `null` when evidence is missing.

When non-null, `source_timing` should include:

- `cue`: musician-facing readiness cue derived from the manifest timing
  readiness and manual-confirm flag.
- `readiness`
- `requires_manual_confirm`
- `beat_status`
- `downbeat_status`
- `primary_downbeat_offset_beats`
- `confidence_result`
- `drift_status`
- `phrase_status`
- `alternate_evidence_count`
- `anchor_evidence`: `null` or compact primary anchor counts copied from
  manifest source-timing readiness evidence.
- `groove_evidence`: `null` or compact primary groove residual evidence copied
  from manifest source-timing readiness evidence.

When non-null, `source_timing_alignment` should include:

- `status`: one of `aligned`, `partial`, or `mismatch`.
- `bpm_delta`: absolute BPM difference between observer and manifest timing, or
  `null` when either side does not expose a comparable BPM.
- `bpm_tolerance`: the tolerance used for this comparison. Current value:
  `1.0` BPM.
- `warning_overlap`: normalized warning codes present on both sides.
- `issues`: mismatch reasons. Strict evidence treats any issue as an output-path
  failure.

`source_timing_alignment` compares musical timing evidence, not source identity.
Observer source ids and manifest artifact/source ids may intentionally differ in
generated probes, so source ids are reported separately and are not an alignment
criterion.

When non-null, `source_timing_anchor_alignment` should include:

- `status`: one of `aligned`, `partial`, or `mismatch`.
- `observer`: `null` or the observer-side `anchor_evidence` object.
- `manifest`: `null` or the manifest-side `anchor_evidence` object.
- `issues`: mismatch reasons. Strict evidence treats any issue as an output-path
  failure.

Anchor alignment intentionally does not require exact count equality. Different
current probes may expose different evidence density on the control and manifest
paths. The strict gate rejects only clear contradictions, such as an observer
that reports primary kick/backbeat/transient anchor evidence while the manifest
reports zero comparable anchors for that class.

When non-null, `source_timing_groove_alignment` should include:

- `status`: one of `aligned`, `partial`, or `mismatch`.
- `observer`: `null` or the observer-side `groove_evidence` object.
- `manifest`: `null` or the manifest-side `groove_evidence` object.
- `issues`: mismatch reasons. Strict evidence treats any issue as an output-path
  failure.

Groove alignment intentionally does not require exact residual-offset equality.
The current observer and manifest paths may expose different evidence density.
The strict gate rejects only clear contradictions, such as an observer that
reports primary groove residuals while the manifest reports zero comparable
residuals.

The current stable metric keys are:

- `full_mix_rms`
- `full_mix_low_band_rms`
- `mc202_question_answer_delta_rms`
- `source_grid_output_drift`: `null` or an object with `hit_ratio`, `max_peak_offset_ms`, and `max_allowed_peak_offset_ms`
- `w30_candidate_rms`
- `w30_candidate_active_sample_ratio`
- `w30_rms_delta`

Pack-specific output evidence may still be validated from the source
`manifest.json` outside these stable summary metric keys. For example, the
current lane recipe correlation accepts `lane-recipe-listening-pack` output only
when its required MC-202 Recipe 2 cases are present, passing, non-collapsed,
above their signal-delta thresholds, aligned to their internal phrase grid, and
attached to a selected Source Graph phrase slot.

For `feral-grid-demo` manifests that include source-grid output drift evidence,
strict correlation also treats that metric as an output smoke gate. A low hit
ratio or a peak offset beyond the reported allowed window means the generated
support output no longer proves it landed near the selected source grid.

For manifests that include source timing evidence, strict correlation treats a
malformed `source_timing` object as an output-path issue. Missing `source_timing`
remains non-fatal for older and non-Feral manifests.

For Feral-grid summaries, `source_timing_bpm_delta` must agree with the grid BPM
decision. `source_timing` decisions require a numeric `0.0` delta and
`source_timing.bpm_agrees_with_grid: true`. `static_default` and `user_override`
decisions with usable source BPM evidence require a numeric delta, and
`source_timing.bpm_agrees_with_grid` must match the current `1.0` BPM tolerance.
Missing or invalid source BPM fallback reasons require a `null` delta and
`source_timing.bpm_agrees_with_grid: null`.

When both observer and manifest source timing evidence are present and well
formed, strict correlation also evaluates `source_timing_alignment`. A BPM delta
above tolerance or non-overlapping warning evidence when both sides emit warnings
sets `status: mismatch` and adds an output-path issue. Comparable BPM evidence
or shared warning evidence with no issues sets `status: aligned`; well-formed but
not directly comparable evidence sets `status: partial` and remains reviewable
instead of becoming a false failure.

When both observer and manifest primary groove evidence are present and well
formed, strict correlation also evaluates `source_timing_groove_alignment`.
Comparable residual presence with no issues sets `status: aligned`; missing or
non-comparable evidence sets `status: partial`; clear contradictions set
`status: mismatch` and add output-path issues.

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
- `control_path.commit_count` is an integer
- `control_path.commit_boundaries` is an array of strings
- `control_path.observer_source_timing` is present as an object or `null`
- `output_path.present == true`
- `output_path.issues` is empty
- `output_path.grid_bpm_source` is present as one of the stable source labels
- `output_path.grid_bpm_decision_reason` is present as one of the stable
  decision-reason labels
- Feral-grid output rejects `unknown` grid BPM decisions; older/non-grid output
  may still use `unknown/unknown` until those paths expose equivalent evidence
- `source_timing` grid BPM decisions require matching Source Timing readiness
  evidence, so source timing, user override, and static fallback paths cannot be
  confused in machine-readable QA
- `output_path.source_timing_bpm_delta` is present as a number or `null`, and
  for Feral-grid output is consistent with `grid_bpm_source`,
  `grid_bpm_decision_reason`, and `source_timing.bpm_agrees_with_grid`
- `output_path.source_timing` is present as an object or `null`
- `output_path.source_timing_alignment` is present as an object or `null`
- `output_path.source_timing_anchor_alignment` is present as an object or `null`
- `output_path.source_timing_groove_alignment` is present as an object or `null`
- every stable metric key is present, with a number or `null` value
- `source_grid_output_drift`, when non-null, has the three numeric fields listed above
- `scripts/validate_observer_audio_summary_json.py` accepts the generated summary shape
- validator fixtures cover a valid failure summary with `null` metrics, a rejected invalid schema marker, a rejected missing metric key, rejected grid BPM decision mismatches, rejected BPM-delta contradictions, and rejected Source Timing shape/cue mismatches
- `just first-playable-jam-probe` also exercises the W-30 source-diff metric fields against generated artifacts
- `just observer-audio-correlate-generated-feral-grid` requires generated Feral
  Grid observer evidence and output manifest evidence to report aligned source
  timing, including BPM tolerance, empty alignment issues, and shared
  `phrase_uncertain` warning evidence

## Current Non-Goals

This contract is not a full JSON Schema file yet.

It does not define pack-specific manifest metrics, perceptual audio quality thresholds, waveform comparison semantics, or human listening approval.

# P012 Source Timing Actionability Surface Review - 2026-05-21

Scope: cadence review after RIOTBOX-875 through RIOTBOX-879, focused on the
Source Timing actionability chain across Jam Trust, observer/audio correlation,
generated Feral-grid manifests, P012 all-lane proof summaries, and the standalone
`source_timing_probe` CLI.

## Summary

The current product direction is coherent. The musician-facing phrase now travels
from the shared Jam summary into Jam Trust, Source / Help surfaces, observer
snapshots, observer/audio summaries, generated Feral-grid manifests, compact P012
proof tables, and probe CLI text / JSON. That materially improves the review path:
a musician or reviewer no longer has to infer from `readiness` and
`requires_manual_confirm` whether the next move is to trust, confirm, listen, or
treat timing as unavailable.

No blocking architecture issue was found, and the reviewed work did not add a new
action system, replay truth, arrangement model, or audio callback risk. Two
follow-up risks should be handled before actionability vocabulary grows again.

## Findings

- **Location**: `crates/riotbox-audio/src/bin/feral_grid_pack/timing_readiness_manifest.rs:140`,
  `crates/riotbox-audio/src/bin/source_timing_probe.rs:281`,
  `crates/riotbox-app/src/bin/observer_audio_correlate/summary_render.rs:261`,
  `crates/riotbox-app/src/source_timing_cues.rs:23`
- **Category**: scope
- **Severity**: minor
- **Title**: Readiness-to-actionability labels are still locally duplicated across runtime surfaces
- **Description**: `grid_use` already has a shared core helper, and policy-level
  Jam cues live behind shared summary/app helpers. The readiness-derived
  `cue/actionability` pair is still reconstructed in the Feral-grid manifest
  builder, the standalone probe CLI, and the observer/audio summary fallback.
  The strings currently match, but this repeats the same branch table at each
  surface that emits or recovers manifest/probe actionability.
- **Suggestion**: Add a shared readiness-label helper, preferably near the typed
  readiness/grid-use policy in core for Rust producers, and use it from the
  Feral-grid manifest builder and probe CLI. Keep downstream Python validators as
  external compatibility checks, but avoid adding more Rust producer-side copies.

- **Location**: `scripts/validate_listening_manifest_json.py:195`,
  `scripts/validate_listening_manifest_json.py:203`,
  `scripts/validate_listening_manifest_json.py:210`,
  `scripts/validate_auto_feral_grid_source_timing_pack.sh:140`,
  `docs/specs/source_timing_intelligence_spec.md:668`
- **Category**: scope
- **Severity**: minor
- **Title**: Generic listening-manifest validation does not yet enforce required generated Feral-grid actionability fields
- **Description**: The Source Timing spec now says generated Feral-grid manifests
  must preserve `source_timing.cue` and `source_timing.actionability`. The
  generated Recipe 15 proof script verifies those fields for the current local
  P012 profiles, but the generic listening-manifest validator still treats both
  fields as optional whenever `source_timing` exists. A future generated
  `feral-grid-demo` manifest outside that Recipe 15 path could drop the
  musician-facing fields and still pass the generic contract.
- **Suggestion**: Tighten the generic validator for Feral-grid grid-BPM manifests
  after checking legacy fixture compatibility: when `pack_id` is
  `feral-grid-demo` and `grid_bpm_source` is present, require
  `source_timing.cue`, `source_timing.actionability`, and their
  readiness/manual-confirm matches.

## Dimensions

- Architecture / boundaries: Source Graph remains the durable timing truth; the
  reviewed surfaces consume compact summary or readiness evidence instead of
  introducing a second timing model.
- Design consistency: strong improvement since RIOTBOX-875; the visible phrase is
  now present across Jam, observer, proof, manifest, and probe surfaces.
- Maintainability: current risk is repeated vocabulary mapping, not a broken
  runtime path.
- Coupling: no new `JamAppState` truth, queue side path, or lane-local timing
  policy was found.
- Safety / performance: no realtime audio callback work, blocking I/O, or model
  call was added to the reviewed path.
- Recommendation: next implementation slices should first centralize
  readiness-derived actionability labels for Rust producers, then tighten the
  generic Feral-grid manifest validator to match the spec's required field
  contract.


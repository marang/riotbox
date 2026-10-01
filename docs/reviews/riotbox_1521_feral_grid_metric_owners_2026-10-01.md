# RIOTBOX-1521 — Feral grid numerical-evidence ownership

Date: 2026-10-01. Classification: maintenance/regression. Original code baseline
`c40bd19a76b0c164c10c9f46703128e76729f471`; predecessor RIOTBOX-1520 merge
`e8a7db660c060060b3835993c6a086cca9175076` integrated before implementation,
with no Feral grid baseline changes. This is not an all-at-once root migration.

## Scope and boundaries

Three existing numerical metric families become ordinary private modules:
bar variation, spectral energy and source-grid drift. Existing shared QA config,
Grid/complete impl/frame rounding and one-pole filtering become dependency leaves
so metrics do not depend back on pack orchestration or render stems. Pure helpers
for correlation, energy and peak search stay private. Shared data/functions are
binary-bound, not library API or Core/Session/arrangement/replay truth.

Explicit root compatibility imports preserve unconverted lexical consumers and
the existing manifest/product-stem modules. Drift regressions use an ordinary
directory-module child with actual-owner imports. Test-only root imports remain
conditional. The dependency DAG is acyclic: config has no internal dependency;
grid/filter consume config; bar/drift consume config/grid; spectral consumes
filter. No numerical owner depends back on CLI, orchestration or presentation.
Seven new/converted compiled owner/test files total 451 lines; no numbered shards.

RBX-395, canonical module policy/inventory and allowlist record the boundary:
**30 include sites/two owners -> 26/two**. Twenty-five root includes and one
legacy test-helper include remain explicitly counted, not relabeled migrated.
Same Cargo binary/root identity; no Cargo manifests, dependencies, library
runtime/DSP or frozen Stage-A contract changes.

## Exact behavior proof

- All **73 complete definitions** in changed owners are identical before/after
  Rustfmt after only visibility/trailing-comma normalization. Includes all
  **37** moved production/helper definitions, five complete drift test/fixture
  definitions and every retained definition in the two reduced legacy shards.
  Full multiset equality, not just names/counts; whole impls/literals preserved.
- All **44** executed synthetic test leaf/status entries are unchanged and pass
  in Debug and actual Release after exactly four documented ownership-path
  substitutions: `source_grid_output_drift_tests::` becomes
  `source_grid_output_drift::tests::`. Bodies/assertions and the pulse fixture
  remain exact; other full test paths/statuses unchanged. No ignored/test loss.
- All **31 actual CLI cases** preserve exact stdout/stderr/status bytes: help,
  missing/unknown parameters, NaN/infinity/negative/text values, bar validation,
  missing WAV, and successful auto-timing/explicit-140-BPM two-bar packs.
- At identical fresh paths under `/tmp/riotbox-1521-byteproof.UXi0Jt`, both
  packs preserve all **38 artifact hashes**, including **16 WAVs**, stem/product
  stem metrics, grid reports, README and manifests. The exact generated input
  hash remains unchanged too (**39 total**). No tolerance/path normalization.
  Input exactly `synthetic-control.wav`, newly generated for four seconds by
  the existing `scripts/write_synthetic_break_wav.py`, never real source audio.
  Only those named fresh pack outputs were enumerated for complete file coverage.
- Both baseline and final manifests independently validate with
  `validate_listening_manifest_json.py --require-existing-artifacts`. This
  metadata/existence check is not waveform identity; the separate hash parity
  checks bytes. Neither numeric pass nor synthetic reconstruction is a verdict.
- Final check/build/Debug/Release and baseline logs explicitly warning/error-free:
  `/tmp/riotbox-1521-check-final.log`, baseline-tests, baseline-build, tests,
  release and build logs with the same `/tmp/riotbox-1521-` prefix.
- Full source-free `just ci` passes in `/tmp/riotbox-1521-ci.log`, explicitly
  warning/error-free: workspace Rust/Python/contracts, synthetic audio gates,
  formatting/tracked JSON and strict all-target/all-feature Clippy. Native
  exact-head CI/merge/archive/cleanup remain separate completion obligations.

## Review and limits

Solo sequential correctness, Rust, architecture, workflow/docs and source-safety
review followed by short self-review, not an independent panel. Reviewed the
selected original metric/Grid/filter definitions, complete changed-owner mapping,
actual imports/visibility/DAG, root compatibility/cfg and Cargo discovery,
regression path/body preservation, retained thresholds and exact CLI/file proof.
**Zero additional changed-diff findings** after correcting the initial module
wiring. No entire 8,000-line Feral root audit or complete root migration claimed.

Initial all-target check failed because a path-overridden external module's
`mod tests` resolved the existing sibling legacy `tests.rs`, not the intended
nested drift test file. The drift owner now uses conventional directory
`source_grid_output_drift/mod.rs` + `tests.rs`. Three test-only compatibility
imports were also made conditional. Final all-target/test/build logs verify
correction; original failed `/tmp/riotbox-1521-check.log` retained, not treated
as successful or suppressed. Compile errors were traced to module wiring,
not fixed by dropping regressions or widening arbitrary imports.

No algorithm/cutoff/floor/threshold/schema/frame-rounding/source-timing-trust
change, resource-policy tuning or behavior fix bundled. Legacy helper behavior
and bounds remain unchanged. No real Development/Holdout/commercial audio,
source-directory discovery, device/DAW/playback or new/transferred human verdict.
Synthetic timing, metrics and product-stem reconstruction prove technical
regressions only, not source-general/product/musical/hardness/release/live-device
qualification. Architecture cadence advances from three to four since
RIOTBOX-1516 only after merge; RIOTBOX-1509 stays open.

# Product Export Reproducibility Boundary

Date: 2026-05-07

## Boundary

The current closest product-facing export seam is the Feral grid pack's
generated-support full mix:

- command: `just product-export-reproducibility-smoke`
- pack: `feral-grid-demo`
- export artifact role: `full_grid_mix`
- export artifact file: `05_riotbox_generated_support_mix.wav`
- proof validator: `scripts/validate_product_export_reproducibility.py`

This is the first normalized export reproducibility proof for the MVP spine. It
is still bounded: it is not full arrangement export, stem package export, live
recording export, or DAW-style delivery.

## What The Proof Checks

The validator renders the same deterministic generated source into two separate
Feral grid packs, validates both listening manifests, and then compares a
normalized product-export proof.

The normalized proof includes:

- source WAV SHA-256 instead of source file path
- stable grid and render settings
- stable listening-manifest metrics and thresholds
- stable Feral scorecard fields
- stable audio artifact SHA-256 values
- the `full_grid_mix` export SHA-256

The proof intentionally excludes temp directory paths, absolute source paths, and
other host-local path noise. If the generated-support export changes, the export
hash or normalized manifest hash changes.

## Product Scope

This boundary promotes the existing helper smoke into a product-oriented export
contract without introducing a second export architecture. It uses the existing
listening manifest and current Feral grid pack output directory as the export
surface under test.

Future export work should grow from this seam toward explicit product export
commands and packages rather than adding a parallel export format.

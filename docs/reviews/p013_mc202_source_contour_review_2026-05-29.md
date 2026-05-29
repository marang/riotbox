# P013 MC-202 Source Contour Review - 2026-05-29

## Scope

RIOTBOX-1028 adds a bounded source-section contour proof for Feral-grid MC-202
support. This keeps the MC-202 phrase labeled as primitive support, but lets the
source window shape contour, note budget, touch, and support level. The manifest
records `metrics.mc202_source_contour` and compares the rendered bass against a
primitive support control.

## Review Findings

- No blocking correctness findings in the branch diff.
- The proof does not claim source-derived phrase planning or question/answer
  placement; it is source-derived contour support only.
- The output-path gate rejects contours whose rendered bass does not differ from
  the primitive control by at least the bounded RMS delta.
- Rust file-size budget remains under control after moving MC-202-specific
  manifest checks into `manifest_mc202_assertions.rs`.

## Output Proof

- `cargo test -p riotbox-audio --bin feral_grid_pack`
- `just representative-source-showcase-musical-quality-fixtures`
- Representative showcase selected `tonal_hook_chop/head` with no issues:
  - MC-202 source contour hint `drop`
  - MC-202 source contour delta RMS `0.0006838905`
  - MC-202 contour low-band energy ratio `0.7577535`
  - MC-202 contour event density per bar `44.516125`
  - MC-202 bass-pressure RMS `0.005162664`
  - MC-202 bass-pressure low-band RMS `0.003352935`
  - all-lane MC-202 contribution ratio `0.05555886`

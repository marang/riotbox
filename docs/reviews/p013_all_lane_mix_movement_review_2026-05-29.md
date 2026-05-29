# P013 All-Lane Mix Movement Review - 2026-05-29

## Scope

RIOTBOX-1027 adds representative-showcase proof that the generated-support mix
is not only non-silent or balanced in aggregate. The manifest now records
`metrics.all_lane_mix_movement` with source-first/support mix delta,
correlation, per-lane contribution ratios, and generated/W-30 contribution
ratio.

## Review Findings

- No blocking correctness findings in the branch diff.
- The proof intentionally allows high waveform correlation up to `0.999`
  because source-backed W-30 can still dominate both listening mixes; the RMS
  delta and per-lane contribution checks catch the collapsed same-mix case.
- `manifest_assertions.rs` reached the soft 500-line budget during the slice, so
  mix-specific manifest checks were split into
  `manifest_mix_assertions.rs`.

## Output Proof

- `cargo test -p riotbox-audio --bin feral_grid_pack`
- `just representative-source-showcase-musical-quality-fixtures`
- Representative showcase selected `break_low_drive/head` with no issues:
  - all-lane mix RMS delta `0.0131248515`
  - all-lane mix correlation `0.96764785`
  - TR-909 contribution ratio `0.14561467`
  - MC-202 contribution ratio `0.07285235`
  - W-30 contribution ratio `0.44838515`
  - generated/W-30 contribution ratio `0.4872307`
  - support generated/source RMS ratio `0.3528312`

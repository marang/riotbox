# RIOTBOX-1517 — Semantic lane recipe diagnostic ownership

Date: 2026-10-01. Classification: maintenance/regression. Baseline:
`942dfe60fdecf0f6b7b9b6bc0d045475ead55899`; same existing Cargo binary,
`lane_recipe_pack`. This protects the Recipe-2 QA regression boundary without
claiming audible product progress or bypassing user/DAW/listening-dependent work.

## Scope and authority

Replace exactly six textual includes with Cargo-standard directory `main.rs`
and ordinary private owners. Arguments/config and existing typed report data
are separate from case construction, pack orchestration, per-case rendering,
grid/source-slot measurements, signal deltas, WAV I/O, manifest serialization
and Markdown presentation. Data/config/signal-measurement leaves have no
dependency back to builders or renderers. Shared fields/helpers are bounded
to the binary; serialization details and one-owner helpers remain private.
Two ordinary regression families import their actual owners explicitly.

The former `tr909_support_case` shard mixed MC-202 fixtures, rendering and
presentation; keeping it intact as a module would preserve misleading ownership.
No library API, Cargo target/dependency, Core/Session/replay/action owner or
audio runtime/DSP is changed. These ephemeral models are not another instrument,
renderer, policy, Source Graph or arrangement system. RBX-392 and the module
policy record the boundary. Include inventory/guardrail: **44 sites/six owners
-> 38/five**, no allowance increase. All 15 compiled files are coherent owners;
largest production owner is the 347-line case catalog, not a size-driven shard.

## Complete-definition and output proof

- All **75** complete definitions remain: **65 production** and **10 regressions**.
  Before formatting, all match after visibility/trailing-comma normalization.
  After formatting, **74** match strictly; the source-slot `.find` predicate
  only gains expression braces. Individually inspected: same expression,
  comparisons, order and return value. No algorithm/literal/threshold change.
- All **10** executed test leaf-name/status entries are identical and pass;
  complete bodies, assertions and synthetic fixture identities are retained.
- All **13** bounded CLI cases preserve exact stdout/stderr bytes and status:
  long/short help, unknown flag, missing values, zero/negative/NaN/infinity/text
  duration, and default/custom successful packs.
- At the same exact fresh temporary output paths, the 2.0-second/default and
  1.5-second/custom packs each retain all **42** files byte-for-byte: **84 total,
  including 32 WAVs**, JSON manifests and Markdown reports/metrics. No path
  normalization or numeric tolerance hides changes. Sorted `sha256sum` listing
  digest: `ffa009c1e8220b775993e6b3a000f7a8ed2c41aa65688f8c3b2f70a342b7e88e`.
  Scope: `/tmp/riotbox-1517-bytepack.qAOnzG/{default,custom}`.
- Cargo metadata retains exactly one binary of the same name with the standard
  directory root. Cargo manifests/lock are unchanged.
- All-target check, final build and target test logs explicitly scanned
  warning/error-free. Initial root-scope/format-capture imports and two unused
  field-name imports were corrected, not suppressed or called warning-free.
  Logs: `/tmp/riotbox-1517-check-final.log`,
  `/tmp/riotbox-1517-baseline-tests.log`, `/tmp/riotbox-1517-final-tests.log`,
  `/tmp/riotbox-1517-final-build.log`.
- Full source-free `just ci` passes in `/tmp/riotbox-1517-ci.log`, explicitly
  warning/error-free: workspace Rust/Python/contracts, Recipe-2/audio-QA
  synthetic gates, formatting/tracked JSON and strict all-target/all-feature
  Clippy. Native exact-head PR CI remains a separate merge gate.

## Review and limits

Solo sequential correctness, Rust, architecture, workflow/docs and audio-boundary
review followed by short self-review; not an independent panel. Reviewed complete
original source, retained definitions, actual imports/visibility, Cargo target,
test equivalence and full generated-byte comparisons. No remaining finding in
this migration. The single formatting difference is documented rather than
treated as an exact-token match. Initial compile findings were fixed and final
logs verify them; no warning allowances or missing tests were introduced.

Every case retains `primitive_renderer` / `non_product_diagnostic_control`,
`product_output_allowed=false`, `quality_proof=false`,
`demo_readiness=unverified`, `promotion_blocked=true`. Generated PCM timing and
fixed typed render plans are controls, not fresh source evidence or intelligence.
The listening-review skill keeps technical-only reruns separate from human
approval: no playback, readiness request or new/transferred taste verdict.

No real Development/Holdout/commercial audio, source-directory discovery,
device or DAW access. No schema/sample-rate/threshold/frozen Stage-A changes,
musical/hardness/source-general/demo/release qualification or realtime/device
proof. RIOTBOX-1509 remains an unresolved Windows startup diagnosis. This is
the first substantive ownership slice after RIOTBOX-1516's verified checkpoint
closeout only when merged; tests/PR preparation do not advance that counter.

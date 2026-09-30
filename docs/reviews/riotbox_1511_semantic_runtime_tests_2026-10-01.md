# RIOTBOX-1511 — Semantic synthetic Runtime regression ownership

Date: 2026-10-01. Baseline: `a5306fe4d5985fc5cb3ecd92ea2745c1525f4135`;
the synchronized `0ff4d5e3` adds only RIOTBOX-1510 archive documentation.
Classification: test-only maintenance/regression, not audible instrument progress.

## Ownership and boundaries

Replace all twelve Runtime-test `include!` sites with ordinary regression
modules. Lifecycle, shared state, mixer/lane behavior, transport stop,
Source Monitor routing/motion/lifetime, fills, gestures and metrics own their
existing tests. Separate shared owners contain fixture models, synthetic PCM,
signal measurements, mix plans, fill recipes and gesture fixtures. Cross-family
helpers use `pub(super)` within the Runtime test subtree; helpers needed only
by their own regression family remain private. All 32 selected test children
are below 500 lines; the largest is 435 lines.

Remove only the two `#[cfg(test)]` import aggregations from the Runtime root.
The existing telemetry health regression imports its actual production owners
instead of those removed aliases. Its assertion/body logic remains unchanged.
No production definition, export, callback, DSP body, state model or dependency
changes. There are no wildcard imports, relocated includes, numbered shards,
suppressed checks or new ignored tests.

## Retention and verification

- All **174** selected definitions are retained: 162 functions, eight structs
  and four impl blocks. **173** match token-for-token after only visibility and
  trailing-comma normalization, retaining literals and qualified paths. The
  remaining callback-scratch regression changes exactly two references from
  `super::shared_transport_tr909::callback_scratch_sample_count` to the same
  production function via `crate::runtime::shared_transport_tr909`. With that
  specific path substitution, all 174 match; no broad path-stripping comparison
  is used.
- All **129** selected tests preserve attributes, complete assertions, sample
  values and bodies under that bounded comparison. Before/after Audio libraries
  retain the identical **280** executed leaf-name/status entries: **279 pass**,
  and the same pre-existing manual cache benchmark remains ignored. Logs:
  `/tmp/riotbox-1511-baseline-audio.log`, `/tmp/riotbox-1511-audio.log`.
- The three `include_str!` JSON fixture literals and their physical parent
  directories remain identical, resolving to the same TR-909, W-30 preview and
  W-30 resample fixtures. No WAV/source fixture is opened for this inspection.
- The first compile exposed missing direct imports for fill modules/helpers and
  the telemetry test's root-alias dependency, plus three unnecessary imports.
  All were corrected at their actual consumers, without restoring the hidden
  aggregator, using globs or weakening tests. The replacement all-target Audio
  check passes without warnings: `/tmp/riotbox-1511-check-2.log`.
- The include guard passes at **58 sites / 9 owners**, down from 70 / 10,
  without increasing any allowance. Formatting and whitespace checks pass.
- Full source-free `just ci` passes, including App 770, Audio 279, Core 470
  and Sidecar 24 library cases, binary/subprocess tests, Python/contracts,
  synthetic audio/observer smokes and strict all-target/all-feature Clippy.
  Log: `/tmp/riotbox-1511-ci.log`. Exact-head native PR checks separately
  gate merge.

## Sequential solo review

Applied code-review and Rust lenses to compatibility, explicit import ownership,
synthetic fixture identity, cfg/test visibility, source-access boundaries,
callback/DSP isolation and assertion retention. The normalized definition
comparison constrains every moved body, rather than relying on green tests
alone. The Runtime-root diff contains only removed test imports; telemetry's
diff contains only test imports. Existing public interfaces and production
runtime code are unchanged. No new product state, queue/replay consequence,
live audio behavior or alternative persistence model is introduced.

The compile findings are fixed narrowly and validated by the warning-free
all-target check and complete Audio regression run. No unresolved correctness,
architecture, missing-test or scope finding was identified. Follow-up self-review
found zero additional findings. This is sequential solo review, not an
independent reviewer panel or human listening verdict.

RBX-388 and module policy record actual test ownership only. Frozen Stage-A
algorithms/contracts remain untouched. No real source, holdout, commercial
reference, device, DAW or playback is accessed or claimed. This test-only slice
does not advance the substantive production-architecture cadence: two slices
remain counted since RIOTBOX-1507.

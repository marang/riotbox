# RIOTBOX-1340 — Explicit production Runtime dependencies

Date: 2026-09-30
Production baseline: `e54b644e546bfcada10c9a5c6897fd4125c7bd20`;
integrated parent `354762f7` adds only archive/checkpoint documentation.
Classification: maintenance/regression, governed by module_policy.md and the
original `p023_runtime_audio_review_2026-06-29.md` follow-up.

## Scope

Replace all nine broad production parent imports in these existing owners:
`tr909_tail_telemetry`, `shared_transport_tr909`, `shared_w30_resample_callback`,
`shared_mc202`, `public_api_shell`, `render_tr909_w30_preview`, `source_monitor`,
`w30_filter_slam` and `w30_tr909_signal_helpers`. The last is a mixed wildcard/
named `fill_step` import, not omitted from the count.

Dependencies now name actual semantic owners or external crates directly:
render state and callback state, snapshot publication, lane signal helpers,
control-side runtime lifecycle, telemetry, source cache, CPAL traits and atomics.
The Runtime parent removes obsolete private aggregation; imports still needed
only by its existing synthetic test fixtures are confined to `cfg(test)`.
The small production bridges consumed by already-explicit children remain.

No module split, helper-body change, visibility change, new dependency,
ActionCommand, Session/replay field, callback operation, lock/atomic ordering,
audio algorithm, gain or musical threshold change. Existing public wildcard
re-exports are compatibility API, not production dependency globs, and remain
unchanged. Existing test-only globs/includes are outside this ticket; the include
inventory remains 94 sites / 13 owners. No cosmetic file-budget split is added.

## Compatibility and verification

- Before and after `cargo test --locked -p riotbox-audio`: Audio library
  **279 passed / one unchanged ignored**, plus all binary/doc tests. The full
  executed/ignored result multiset is identical (**369 entries**). Logs:
  `/tmp/riotbox-1340-baseline-audio.log`, `/tmp/riotbox-1340-audio-tests.log`.
- Strip only private top-level import statements and comments/whitespace:
  every other token matches exactly in all ten changed Rust files, including
  the parent Runtime. **212 top-level items** (functions/types/constants/impl
  blocks) remain accounted for. No normalization of literal values, qualified
  body paths, braces, operators or visibility was needed. This supports the
  import-only boundary; it is not proof of real-device timing or musical quality.
- `cargo check --locked -p riotbox-audio --all-targets` is warning-free.
  Log: `/tmp/riotbox-1340-check-3.log`. Formatting, diff check and the unchanged
  include guard pass. Migration-time unused aggregation/trait imports were
  removed, not hidden behind a blanket warning allowance.
- Full source-free `just ci` passes: App 770, Audio 279, Core 470 and Sidecar
  24 library tests, binary/subprocess tests, synthetic audio smokes, Python/
  contract fixtures and strict all-target/all-feature Clippy. Log:
  `/tmp/riotbox-1340-ci.log`. Exact native PR-head checks still gate merge.

## Solo branch review and self-review

Code-review and Rust-specific lenses were applied sequentially by the same
agent, not an independent panel. Scope: definition-owner resolution, trait
method availability, public and private test compatibility, cfg boundaries,
unchanged complete bodies/atomic ordering, audio callback/control separation,
test retention and scope/workflow evidence. All nine wildcard sites are removed;
remaining Runtime `use super::*` occurrences are test-only. Existing exported
API remains available through the same facade.

No unresolved correctness, architecture drift, missing-test, Rust-safety,
dependency or workflow finding was identified. Follow-up self-review checks the
complete non-import token comparison and identical test results. This does not
claim native Windows Audio/CPAL qualification from the Sidecar Windows CI job.

No new architectural or product contract decision arose, so routine import
cleanup does not get a Decision Log entry. No real source, holdout, commercial
reference, device, DAW or human playback was accessed. Synthetic renderer/mixer
regressions are retained engineering evidence, not a new musical/human pass.

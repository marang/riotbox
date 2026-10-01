# Rust Textual Include Inventory

Date: 2026-06-29
Ticket: RIOTBOX-1321
Policy: `docs/engineering/module_policy.md`

---

## Summary

Current scan:

```bash
rg -n 'include!' crates --glob '*.rs'
```

Result:

- original RIOTBOX-1325 baseline: 236 textual include sites in 17 owning Rust files
- refreshed 2026-08-29 baseline: 248 textual include sites in 18 owning Rust files
- RIOTBOX-1337 migration, 2026-09-30: 203 remaining sites in 16 owners;
  all 45 library CLI and CLI-test includes replaced by real modules
- RIOTBOX-1411 migration, 2026-09-30: 94 remaining sites in 13 owners;
  all 109 UI production, UI-test, and JamApp-test includes replaced by real modules
- RIOTBOX-1508 migration, 2026-09-30: 82 remaining sites in 11 owners;
  all 12 Core Jam projection and projection-test includes replaced by real modules
- RIOTBOX-1510 migration, 2026-09-30: 70 remaining sites in 10 owners;
  all 12 Source Graph timing-candidate includes replaced by real semantic modules
- RIOTBOX-1511 migration, 2026-10-01: 58 remaining sites in 9 owners;
  all 12 Audio runtime-test includes replaced by ordinary regression modules
- RIOTBOX-1512 migration, 2026-10-01: 56 remaining sites in 8 owners;
  both JamApp lane-projection includes replaced by semantic render owners
- RIOTBOX-1513 migration, 2026-10-01: 54 remaining sites in 7 owners;
  both Core TR-909 policy/test includes replaced by ordinary semantic modules
- RIOTBOX-1514 migration, 2026-10-01: 44 remaining sites in 6 owners;
  all ten Observer/audio correlation includes replaced by diagnostic modules
- RIOTBOX-1517 migration, 2026-10-01: 38 remaining sites in 5 owners;
  all six lane recipe includes replaced by ordinary diagnostic modules
- RIOTBOX-1519 migration, 2026-10-01: 32 remaining sites in 3 owners;
  all six W-30 renderer/comparison includes replaced by semantic QA modules
- no generated-code include site identified in this inventory
- every current include is treated as legacy/mechanical until proven otherwise

This inventory is a migration map, not a quality verdict on the original work.
The goal is to stop new textual include shells from appearing and then reduce
the current allowlist through behavior-preserving module migrations.

## Guardrail

The manual guardrail is:

```bash
scripts/check_no_textual_includes.sh
```

It compares the current owning-file include counts against
`docs/engineering/textual_include_allowlist.txt`. During migration, a PR should
update the allowlist only when it intentionally removes or converts include
sites. New include owners or changed counts fail the guardrail until reviewed.

### 2026-08-29 Baseline Refresh

The tracked allowlist had fallen behind the already-merged `HEAD` inventory.
RIOTBOX-1485 converts its live-master production CLI, CLI tests, and JamApp
tests to real Rust modules rather than adding three more textual includes. The
allowlist is refreshed to the exact remaining `HEAD` counts after that
conversion so the guardrail can reject future drift again. This refresh records
legacy inventory; it does not approve new textual include sites.

## Inventory

| Owner | Count | Included files / families | Purpose | Classification | Migration risk | Follow-up |
| --- | ---: | --- | --- | --- | --- | --- |
| `crates/riotbox-audio/src/bin/feral_grid_pack.rs` | 29 | pack builder, metrics, TR-909, MC-202, W-30, mix, timing, manifest, render, tests | Feral grid QA/pack CLI | mechanical QA-bin split | medium: large QA surface | future QA-bin module slice |
| `crates/riotbox-audio/src/bin/feral_grid_pack/tests.rs` | 1 | shared test helpers | Feral grid QA tests | mechanical test compatibility split | low: test-only helper scope | future QA-bin module slice |
| `crates/riotbox-audio/src/bin/feral_before_after_pack.rs` | 2 | pack builder, metrics manifest | Feral before/after QA CLI | mechanical QA-bin split | low | future QA-bin module slice |

## Migration Order

Recommended first wave:

1. Source Graph root and timing candidate subtree.
2. Session root.
3. Audio MC-202 and source audio roots.
4. App binary CLI shell.
5. UI/Jam view and JamApp projection/test shells.
6. QA-bin shells, grouped by binary family.

Keep each migration behavior-preserving. If a migration exposes naming,
visibility, or ownership problems, capture those as follow-up changes instead
of mixing them into the module move.

## Migrated Owners

| Owner | Former count | Migrated by | Notes |
| --- | ---: | --- | --- |
| `crates/riotbox-core/src/source_graph.rs` | 10 | RIOTBOX-1322 | Replaced by `crates/riotbox-core/src/source_graph/mod.rs` with real child modules and `pub use` compatibility exports. |
| `crates/riotbox-core/src/session.rs` | 3 | RIOTBOX-1323 | Replaced by `crates/riotbox-core/src/session/mod.rs` with real child modules and `pub use` compatibility exports. |
| `crates/riotbox-audio/src/mc202.rs` | 5 | RIOTBOX-1324 | Replaced by `crates/riotbox-audio/src/mc202/mod.rs` with render-type exports, internal sound-design module, and real test modules. |
| `crates/riotbox-audio/src/source_audio.rs` | 2 | RIOTBOX-1324 | Replaced by `crates/riotbox-audio/src/source_audio/mod.rs` with cache exports and real test module. |
| `crates/riotbox-app/src/bin/riotbox-app.rs` | 20 | RIOTBOX-1325 | Replaced by a thin binary entrypoint that calls `riotbox_app::cli::run()`. The existing mechanical CLI include shell now lives at `crates/riotbox-app/src/cli.rs` pending a semantic module split. |
| `crates/riotbox-app/src/cli.rs` | 22 | RIOTBOX-1337 | Private semantic modules own configuration, argument parsing, launch, terminal lifecycle, observer serialization, event routing, performer controls, and offline export/report modes. Only `run()` remains public. |
| `crates/riotbox-app/src/bin/riotbox-app/tests.rs` | 23 | RIOTBOX-1337 | Real `cli::tests` children own regression families; shared synthetic fixtures have a separate test-only owner. No test scenario removed. |
| `crates/riotbox-app/src/ui.rs` | 10 | RIOTBOX-1411 | Public shell/render compatibility exports over private semantic presentation, input-state, cue, and diagnostic owners. Existing public risk-cue contract stays in place. No keys, text, layout, or audio behavior changed. |
| `crates/riotbox-app/src/ui/tests.rs` | 25 | RIOTBOX-1411 | Real regression-family children with explicit imports and separate synthetic fixture owners; no scenario removed. |
| `crates/riotbox-app/src/jam_app/tests.rs` | 74 | RIOTBOX-1411 | Real integration-family children and explicit synthetic fixture owners. Four pre-existing crate-internal fixture entrypoints retain compatibility re-exports. |
| `crates/riotbox-core/src/view/jam.rs` | 8 | RIOTBOX-1508 | Public projection compatibility facade over private model/builder, source summary/timing/map, scene/arrangement, capture and performer-state owners. No product state or projection behavior changed. |
| `crates/riotbox-core/src/view/jam/tests.rs` | 4 | RIOTBOX-1508 | Real projection regression families with one shared synthetic fixture owner; source-map rows/tests and timing tests are ordinary child modules. No test scenario removed. |
| `crates/riotbox-core/src/source_graph/timing_probe_candidates.rs` | 12 | RIOTBOX-1510 | Explicit public compatibility exports over private onset-evidence, period-scoring, downbeat, hypothesis, grid, drift/groove, model, report and grid-use-policy owners. Hybrid candidate tests become ordinary regression-family children with separate synthetic fixtures; comparator tests live under period scoring. Algorithms and thresholds unchanged. Historical RIOTBOX-1330 covered live-ingest/confirmation wiring, not this include migration. |
| `crates/riotbox-audio/src/runtime/tests.rs` | 12 | RIOTBOX-1511 | Ordinary lifecycle, shared-state, mixer/lane, transport-stop, Source Monitor, fill, gesture and metrics regressions with explicit imports. Shared fixture models, synthetic PCM, signal helpers, mix plans, fill recipes and gesture fixtures are test-only owners. The existing telemetry test imports its actual runtime owners; production runtime and DSP are unchanged. |
| `crates/riotbox-app/src/jam_app/projection.rs` | 2 | RIOTBOX-1512 | Explicit JamApp-only compatibility exports over private TR-909, MC-202, scene-context and W-30 preview/material/resample owners, retaining the existing source-phrase child. All render policy, preparation, timing/identity and fail-closed behavior unchanged. |
| `crates/riotbox-core/src/tr909_policy.rs` | 2 | RIOTBOX-1513 | Explicit public compatibility facade over private typed model, render-policy composition, source-support and pattern/phrase owners; ordinary synthetic regressions with explicit imports. All vocabulary, source/scene/transport precedence, algorithms and public APIs unchanged. Historical RIOTBOX-1331 shipped source-derived lane differentiation, not this include migration. |
| `crates/riotbox-app/src/bin/observer_audio_correlate.rs` | 10 | RIOTBOX-1514 | Cargo-standard directory main and ordinary private diagnostic owners for metadata I/O, report types, observer/manifest parsing, alignment, evidence and JSON/Markdown presentation; explicit regression imports. Binary identity, schemas, thresholds, CLI/report bytes and fail-closed evidence unchanged. |
| `crates/riotbox-audio/src/bin/lane_recipe_pack.rs` | 6 | RIOTBOX-1517 | Cargo-standard directory main and ordinary private CLI/config, report data, case catalog, rendering, grid/source-slot measurements, signal delta, WAV I/O, manifest and Markdown owners; explicit regression imports. Pure data/config/measurement leaves prevent builder/rendering cycles. Primitive diagnostic boundary, complete algorithms, CLI/report/WAV bytes and thresholds unchanged. |
| `crates/riotbox-audio/src/bin/w30_preview_compare.rs` | 4 | RIOTBOX-1519 | Same Cargo binary through directory main; private arguments/config, metrics ingestion, comparison policy, Markdown and manifest owners with explicit ordinary regression imports. All finite checks, thresholds, schemas and CLI/artifact bytes retained. |
| `crates/riotbox-audio/src/bin/w30_preview_render.rs` | 2 | RIOTBOX-1519 | Same Cargo binary through directory main; private arguments/config, source-window adaptation and artifact I/O owners with explicit regressions. RIOTBOX-1518 guard and all rendering/source-window behavior retained; only freshly generated synthetic controls used for parity. |

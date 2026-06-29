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

- 243 textual include sites after RIOTBOX-1323
- 19 owning Rust files after RIOTBOX-1323
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

## Inventory

| Owner | Count | Included files / families | Purpose | Classification | Migration risk | Follow-up |
| --- | ---: | --- | --- | --- | --- | --- |
| `crates/riotbox-core/src/source_graph/timing_probe_candidates.rs` | 12 | `timing_probe_candidates/types`, confidence, period scoring, drift, groove, phrase, model, hypothesis, downbeat, grid, readiness, policy | Source timing candidate internals | mechanical product split | medium/high: timing confidence contract | RIOTBOX-1330 or Source Graph follow-up |
| `crates/riotbox-core/src/tr909_policy.rs` | 2 | `tr909_policy/render_policy`, tests | TR-909 policy and tests | mechanical product split | low/medium: policy API | RIOTBOX-1331 |
| `crates/riotbox-core/src/view/jam.rs` | 8 | `jam/view_model_types`, source timing, source map, arrangement, builder, capture, scene, tests | Jam view model and projections | mechanical view split | medium: app/core UI contract | future UI/view module slice |
| `crates/riotbox-core/src/view/jam/tests.rs` | 4 | jam test fixture and scenario shards | Jam view tests | mechanical test split | low/medium: test imports | future UI/view module slice |
| `crates/riotbox-audio/src/mc202.rs` | 5 | `mc202/render_types`, sound design, tests | MC-202 audio rendering and tests | mechanical product split | high: audible output stability | RIOTBOX-1324 / RIOTBOX-1332 |
| `crates/riotbox-audio/src/source_audio.rs` | 2 | `source_audio/cache`, tests | Source audio cache and tests | mechanical product split | medium: callback/cache boundary | RIOTBOX-1324 |
| `crates/riotbox-audio/src/runtime/tests.rs` | 7 | runtime fixture, W-30, mix, metrics, monitor test shards | Audio runtime tests | mechanical test split | low/medium: test-only but broad | future runtime test module slice |
| `crates/riotbox-audio/src/bin/feral_grid_pack.rs` | 29 | pack builder, metrics, TR-909, MC-202, W-30, mix, timing, manifest, render, tests | Feral grid QA/pack CLI | mechanical QA-bin split | medium: large QA surface | future QA-bin module slice |
| `crates/riotbox-audio/src/bin/feral_before_after_pack.rs` | 2 | pack builder, metrics manifest | Feral before/after QA CLI | mechanical QA-bin split | low | future QA-bin module slice |
| `crates/riotbox-audio/src/bin/lane_recipe_pack.rs` | 6 | pack builder, lane cases, manifest, tests | Lane recipe QA CLI | mechanical QA-bin split | low/medium | future QA-bin module slice |
| `crates/riotbox-audio/src/bin/w30_preview_compare.rs` | 4 | compare CLI, metrics, manifest, tests | W-30 preview comparison CLI | mechanical QA-bin split | low/medium | future W-30 QA-bin slice |
| `crates/riotbox-audio/src/bin/w30_preview_render.rs` | 2 | render CLI, tests | W-30 preview render CLI | mechanical QA-bin split | low | future W-30 QA-bin slice |
| `crates/riotbox-app/src/bin/riotbox-app.rs` | 20 | launch, export/report modes, event loop, controls, args, tests | Main app binary and CLI modes | mechanical app split | high: CLI compatibility and app launch | RIOTBOX-1325 |
| `crates/riotbox-app/src/bin/riotbox-app/tests.rs` | 21 | CLI and observer test shards | Main app binary tests | mechanical test split | medium: CLI regression coverage | RIOTBOX-1325 |
| `crates/riotbox-app/src/bin/observer_audio_correlate.rs` | 10 | args, source timing evidence, summary build/render/evidence | Observer/audio correlation CLI | mechanical QA-bin split | medium: QA contract | future observer QA-bin slice |
| `crates/riotbox-app/src/ui.rs` | 10 | UI state, shell render, perform layout, footer/help, source/log/capture helpers | TUI surface | mechanical app split | medium/high: musician UI contract | future UI module slice |
| `crates/riotbox-app/src/ui/tests.rs` | 25 | UI fixture and screen tests | TUI tests | mechanical test split | medium: broad fixture surface | future UI module slice |
| `crates/riotbox-app/src/jam_app/projection.rs` | 2 | TR-909 and W-30 projections | Jam app projection helpers | mechanical app split | medium: app facade boundary | future JamApp projection slice |
| `crates/riotbox-app/src/jam_app/tests.rs` | 72 | JamApp fixtures, recovery, export, source timing, W-30, MC-202, TR-909, scene, replay test shards | JamApp integration tests | mechanical test split | high: very broad test root | future JamApp test module slice |

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

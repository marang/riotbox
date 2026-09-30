# RIOTBOX-1411 — Semantic UI and Jam regression modules

Date: 2026-09-30
Baseline: `a58add32` (archived RIOTBOX-1337 closeout)
Owning contract: `docs/engineering/module_policy.md`; decision: RBX-385.

## Scope and ownership

This is a mechanical maintenance slice, not new TUI functionality. The public
`ui` facade retains `JamShellState`, `ShellScreen`, `ShellLaunchMode`,
`JamViewMode`, `ShellKeyOutcome`, `render_jam_shell` and
`render_jam_shell_snapshot`. The public risk-cue contract remains in place.
Private owners distinguish input state and gesture vocabulary from rendering,
screen/layout, footer/help, source/export inspection, lane diagnostics, warnings
and performer/capture/scene cues. Eight hybrid path-attributed UI children are
rehomed as ordinary modules; existing semantic onramp/diagnostic children remain.

UI and Jam tests have real regression-family modules and explicit imports.
Shared synthetic fixture models, shell/session/source builders, WAV I/O, restore
parity and lane recipe/replay helpers have test-only owners. The four existing
crate-internal fixture entrypoints (`sample_graph`, `sample_session`,
`w30_hook_export_state`, `write_source_matched_handoff_fixture`) retain their
paths through compatibility re-exports. The separate root-attached Feral runtime
projection test module is unchanged.

All 109 selected textual includes disappear: production UI 10, UI tests 25,
Jam tests 74. The reviewed allowlist and inventory decrease from 203 sites in
16 owners to 94 in 13; there are no relocated includes or wildcard imports in
the selected trees. No Core/Session/action, queue, observer, DSP or audible
policy changes are bundled into this move.

## Compatibility evidence

- Before and after `cargo test --locked -p riotbox-app --lib`: 770 passed,
  zero failed and one unchanged ignored test. All 770 executed test leaf names
  match as a multiset; fully qualified names naturally gain semantic owners.
  Logs: `/tmp/riotbox-1411-baseline-app.log`, `/tmp/riotbox-1411-app-tests.log`.
- The selected former includes contain the same 489 tests. Existing real
  modules also remain; counting only the former include tests is not the full
  App regression count.
- Inventory of the 143 selected original files accounts for the same 1,126
  top-level items after migration: 1,020 functions, 18 enums, 11 impl blocks,
  48 constants and 29 structs. Normalizing visibility, qualified ownership
  paths, whitespace and optional trailing commas leaves 1,116 matches. All ten
  remaining item diffs were inspected: five preserve the same resolved JSON
  fixture path; four contain only rustfmt match/closure expression braces; one
  reorders existing function-local imports. No assertion, literal value,
  operator, call or control flow changes were identified. This is supporting
  mechanical evidence, not a general semantic-equivalence proof.
- Migration-time compiler failures caught an incomplete helper extraction,
  missing explicit imports and wrong relative module paths. They were fixed
  before accepting the body comparison and green regression evidence; merely
  preserving test attributes was not treated as sufficient proof.
- `cargo check --locked -p riotbox-app --lib --tests` is warning-free; formatting
  and the textual-include guard pass. Log: `/tmp/riotbox-1411-module-check-4.log`.
- Full `just ci` passes: App 770, Audio 279, Core 470 and Sidecar 24 library
  tests, subprocess regressions, source-free synthetic smokes, Python/contract
  fixtures and strict all-target/all-feature Clippy. Log:
  `/tmp/riotbox-1411-ci.log`. Native GitHub CI must also pass at the exact PR head.

## Deliberate boundaries

All production UI owners are below 700 lines. Seven cohesive, pre-existing Jam
integration/fixture families remain above that review signal; the largest are
product-export regression coverage and shared MC-202 recipe fixtures. Their
assertions/helpers are not subdivided into numbered shards for a cosmetic limit.
Family-specific reusable helpers may retain their existing semantic family
owner with explicit sibling imports; common fixture contracts are separated.

No real source, holdout, commercial reference, audio device, DAW or human playback
was accessed. Synthetic tests are regression proof, not a new musical, hardness
or human-listening pass. This structural work removes a maintenance/review
obstacle for P023 without bypassing its blocked human/host gates.

## Solo branch review and self-review

Code-review, Rust and UI review lenses were applied sequentially by the same
agent, not an independent panel. The review covers all selected definition
bodies, import/relative-path resolution, public and crate-internal compatibility,
cfg/serde attributes, key routing, rendering text/layout, fixture ownership,
unchanged assertions, scope boundaries, include guard and workflow evidence.
Duplicate-named receipt helpers retain their distinct original semantic owners;
the test-only onramp re-export and Unix-only file-admission module keep their cfg
boundaries. Function-local trait imports remain available after relocation.

No unresolved correctness, architecture, missing-test, unsafe/concurrency,
UI-regression or workflow finding was identified. The follow-up self-review
confirms the complete unchanged App case set and no out-of-scope file changes.
No general platform, real-terminal interaction or musical equivalence is claimed
from synthetic snapshot/regression evidence. The broader architecture checkpoint
is due after this fifth substantive slice following RIOTBOX-1503; it is the next
autonomous workflow step, not silently replaced by this branch review.

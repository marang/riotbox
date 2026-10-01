# Riotbox Module Policy

Version: 0.1
Status: Draft
Audience: Rust contributors, reviewers, coding agents

---

## Purpose

This policy keeps Riotbox reviewable without turning file-size limits into
mechanical source sharding.

Small files are useful, but semantic Rust modules are more important than
line-count compliance.

## Core Rule

Prefer real Rust modules over textual `include!` splits.

`include!` is allowed only for:

- generated code with a documented generator
- deliberately embedded static artifacts
- narrow macro or compatibility cases that cannot be represented cleanly as a
  normal module

Do not use `include!` as a normal way to make a large Rust file appear smaller.

## File Size Guidance

- Aim for files that are easy to review, typically under roughly 400-700 lines.
- Treat line count as a warning signal, not as the design goal.
- A file may exceed that range when further splitting would make ownership less
  clear; record the reason in a short comment, review note, or decision log.
- Do not split large type or enum files by line number. Split by domain concept,
  visibility boundary, test ownership, or stable API boundary.

## Migration Rule

When replacing textual includes, the first slice must be mechanical:

- no behavior change
- no feature work in the same PR
- preserve public API compatibility with `pub use` where appropriate
- make imports explicit enough that child module dependencies are reviewable
- keep tests green before and after the move

After migration, follow-up slices may improve naming, visibility, and ownership.

### App Shell And Regression Ownership

The migrated App `cli` and `ui` roots are compatibility facades over private
semantic owners, not new product models (RBX-384 / RBX-385). UI input state and
presentation consume existing App/Core projections; they do not own persistent
Session, replay, queue or audio truth. Regression families are real test modules
with explicit imports and separately owned shared synthetic fixtures. Preserve
existing crate-internal fixture entrypoints through narrow re-exports when other
test families already consume them. A cohesive integration or lane-recipe fixture
owner may remain long when splitting by line count would obscure its contract.

### Core Jam Projection Ownership

`view::jam` is a compatibility facade over private Core projection owners
(RBX-386 / RIOTBOX-1508). The model and builder assemble existing Session,
Source Graph and Action Queue truth. Source summary/timing/map, scene selection
and arrangement readiness, capture summaries and performer-state projections
own their existing derived types and helpers. They do not add a parallel
Session, scheduler, replay state or UI-owned product model. Keep public view
paths stable with explicit re-exports and limit sibling helper visibility to
the Jam subtree. Source-map rows and projection tests are ordinary children;
shared synthetic graph/session/queue fixtures belong only to the test tree.

### Source Graph Timing Candidate Ownership

`source_graph::timing_probe_candidates` preserves its public interface through
explicit compatibility exports (RBX-387 / RIOTBOX-1510). Private modules own
onset normalization, period scoring, downbeat selection, hypothesis construction,
beat/bar/phrase grids, drift/groove, final model assembly, evidence reports and
grid-use policy. Shared normalized onset evidence has its own owner instead of
making scoring depend on hypothesis construction. Warning messages stay with
model assembly; phrase construction stays with grid construction. Keep helpers
private or bounded to the candidate subtree. Ordinary test children own the
existing synthetic regression families; comparator tests belong to period
scoring so its ordering helper and report-only score fields remain private.
This migration changes no algorithm, threshold, readiness/trust policy or
Source Graph/Session schema and grants no fresh source qualification.

### Runtime Regression Ownership

`runtime::tests` contains ordinary synthetic regression modules, not a shared
lexical include namespace (RBX-388 / RIOTBOX-1511). Lifecycle, shared state,
mix/lane behavior, transport stop, Source Monitor, fills, gestures and metrics
have explicit dependencies on their existing production owners. Shared fixture
models, synthetic PCM, signal measurements, mix plans, fill recipes and gesture
fixtures stay in the test subtree; cross-family helper visibility is bounded to
that subtree and one-family helpers remain private. Other runtime test children
import actual production owners instead of depending on root test-only aliases.
Preserve complete regression bodies, fixture identities and existing ignored
status. This ownership change does not alter callbacks, DSP, runtime state,
public APIs or any audio/source policy.

### JamApp Audio Projection Ownership

`jam_app::projection` is a JamApp-only compatibility facade, not another
runtime or persistent state model (RBX-389 / RIOTBOX-1512). Private TR-909 and
MC-202 owners map existing Core/Session policy into Audio render state; shared
scene context is independent of either lane. The existing source-phrase child
retains its mapping contract. W-30 preview/action state, cached material/sample
preparation and capture resample state have separate owners. Keep transform
data with material preparation to avoid a dependency back from material into
preview state. Re-export only the six existing entrypoints at their existing
JamApp visibility; sibling helpers remain projection-scoped and all others
private. Preserve timing trust, source/section/capture identity, availability,
sample-selection/chop logic and committed-action semantics. This boundary
neither loads audio nor changes sound policy.

### Core TR-909 Policy Ownership

`tr909_policy` remains the Core-owned public policy facade
(RBX-390 / RIOTBOX-1513). Explicit exports preserve its nine public types and
three public functions. Private modules own typed vocabulary/projection,
render-policy composition, source-support section/profile/reason and
pattern/phrase adaptation. Source Graph and Session remain inputs, not App or
Audio dependencies. Shared internal support data and the three cross-owner
helpers are visible only within the policy subtree; remaining helpers stay
private. Ordinary synthetic regressions consume the existing facade explicitly.
Preserve complete enum labels/methods, public attributes/fields/signatures,
source/scene/transport precedence, algorithm bodies and fixture identities.
This migration creates no second policy or product model and changes no sound.

### Observer / Audio Correlation Ownership

`observer_audio_correlate/main.rs` is the Cargo-standard root of the same
existing binary (RBX-391 / RIOTBOX-1514), not a new target or library API.
Ordinary private owners separate metadata I/O, existing report types, observer
commit/scene/timing parsing, manifest timing/metrics parsing, anchor/groove
metadata, alignment and evidence policy from JSON and Markdown presentation.
Report types remain ephemeral diagnostic views, never Session, replay, source
identity or human-listening authority. Pure typed report data and scalar readers
are dependency leaves; composition consumes parser/alignment outputs, evidence
consumes the report, and presentation consumes report/evidence/labels. Keep
cross-owner visibility bounded to this binary and imports explicit, including
ordinary regressions. Preserve complete algorithms/literals, schemas, numeric
floors/tolerances, fixture paths and CLI/report/error bytes. Metadata paths may
be read; referenced source/artifact audio paths are not hydrated by this tool.
Existing cohesive CLI/drift regression families remain together despite their
roughly 500-line size; splitting their fixture contracts only for line count
would obscure the first migration's behavior-preservation proof.

### Lane Recipe Diagnostic Ownership

`lane_recipe_pack/main.rs` retains the single existing Cargo binary
(RBX-392 / RIOTBOX-1517). Ordinary private owners separate arguments/config,
typed ephemeral report data, the existing case catalog, pack orchestration,
per-case rendering, MC-202 grid/source-slot measurements, signal deltas,
WAV publication, manifest serialization and Markdown presentation. Shared data,
config and signal measurements are dependency leaves; rendering and reporting
consume them, not the reverse. Cross-owner helpers/fields are bounded to the
binary; serialization internals and one-owner helpers stay private. Ordinary
regressions name actual owners and retain complete bodies/fixture identities.
The existing generated PCM timing evidence and fixed render plans are
diagnostic controls, not product source intelligence or a second arrangement
model. Preserve every algorithm/literal/threshold, CLI/report/WAV byte and
the existing primitive-renderer promotion boundary. Technical-only reruns do
not supply or transfer a human verdict.

### W-30 Preview QA Ownership

`w30_preview_render/main.rs` and `w30_preview_compare/main.rs` retain the two
existing standalone Cargo binaries (RBX-393 / RIOTBOX-1519). Renderer arguments
retain their complete parsing/input-adaptation impl; source-window projection
and synthetic control construction are independent of CLI and artifact I/O.
The existing WAV/metrics publication belongs to one artifact owner. Comparison
configuration is a leaf; metrics ingestion owns its existing typed values,
comparison owns the ephemeral report and numeric policy, and Markdown and
manifest presentation consume those owners. Manifest-only serialization types,
numeric defaults and one-owner helpers stay private; shared items are bounded
to each binary. Ordinary tests use explicit actual-owner imports. Preserve
complete algorithms/impls, the finite-duration rejection, thresholds, schemas,
CLI/error/report bytes and existing side-effect order. Synthetic controls are
QA inputs, never missing-source product fallback or fresh source intelligence.
This migration changes no library API, runtime/DSP or product-state ownership.

### Feral Before/After Diagnostic Ownership

`feral_before_after_pack/main.rs` retains the same standalone Cargo binary
(RBX-394 / RIOTBOX-1520), not a product fork. Config is a leaf; pack orchestration
consumes argument/input metadata, pure source-window adaptation, existing fixed
lane render-state plans, mix/sequence/delta measurements, WAV/metrics publication,
Markdown and manifest serialization. No dependency points back to orchestration.
One-owner serialization types/helpers and synthetic test PCM stay private;
shared items are bounded to the binary and ordinary regressions import actual
owners. Preserve complete algorithms/impls/literals, numeric gates, CLI/artifact
bytes and side-effect/error ordering. Fixed plans and numerical distinctness
are diagnostic controls, not source intelligence, human approval or live-mixer
proof. This migration changes no library runtime/DSP/API or product-state model.

### Feral Grid Numerical Evidence Ownership

The bounded numerical-evidence family inside `feral_grid_pack` has ordinary
private owners (RBX-395 / RIOTBOX-1521), not a fully migrated root. Existing
config, QA `Grid`/frame rounding and one-pole filtering are dependency leaves;
bar variation, spectral energy and source-grid drift import their actual
owners without depending back on orchestration or render stems. Drift
regressions are a directory-module child with explicit owner imports.
Correlation/energy/peak helpers stay private and shared items stay binary-bound.
Explicit root compatibility imports preserve remaining lexical consumers,
including existing manifest/product-stem owners; unconverted includes remain
counted legacy. Preserve complete algorithms/impls, thresholds, schemas,
fixture bodies and CLI/artifact bytes. These ephemeral QA metrics/Grid are not
Session, arrangement/replay truth or fresh source/human authority.

### Feral Grid Timing Policy And Evidence Ownership

The bounded timing family inside `feral_grid_pack` uses ordinary private
owners (RBX-397 / RIOTBOX-1523). Existing complete CLI arguments/parsers are
a config consumer; conservative BPM selection imports those arguments and the
Core readiness interface. Pure existing anchor/groove evidence adapts Core
timing data without depending on serialization presentation. Probe adaptation
consumes those data, the unchanged policy profile and QA config. The bounded
groove consumer imports BPM/evidence/config; readiness serialization and status
labels consume the same evidence, BPM decision and profile. None depends back
on pack orchestration, root compatibility imports or another product truth.

Shared fields/functions remain binary-bound; single-owner parsers, helpers,
constants and serialization internals remain private. Ordinary BPM regressions
import actual owners and preserve their fixtures and test identities. Root
explicit/cfg-test compatibility imports preserve untouched legacy consumers;
remaining includes stay counted. Preserve complete algorithms/impls, trust
conditions, thresholds, labels, schemas, source-access/error ordering and every
CLI/artifact byte, including RIOTBOX-1522's literal verification command.
Core owns timing truth; these are ephemeral QA consumers, not source-general,
Session, arrangement/replay, human or live-device authority.

### Feral Grid W-30 QA Policy And Presentation Ownership

The bounded W-30 family inside `feral_grid_pack` uses ordinary binary-private
owners (RBX-398 / RIOTBOX-1527). Shared scalar sample measurements are a
dependency leaf also consumed by legacy source-window selection. Window
preparation owns selection, gain, articulation and loop-closure evidence;
slice choice owns its candidate scoring and private selected offsets. Existing
trigger-event data form a shared leaf, so accent evidence and trigger planning
do not depend on each other cyclically. Trigger policy consumes Grid, slice
choice and accent features; playback profile consumes articulation/spectral
metrics. JSON presentation consumes their evidence, never the reverse.

Single-owner helpers, constants, candidate/offset internals and serializer
fields remain private. Cross-owner items stay `pub(super)`, not library APIs.
Ordinary W-30 regressions import actual owners and preserve every test/fixture
body and identity; only the still-legacy stem renderer uses root compatibility.
Root explicit imports preserve untouched consumers. Remaining lexical includes
stay counted legacy, not an approved completed root migration. Preserve full
algorithms/impls, literals, thresholds, labels, schemas, render/access/error
ordering and CLI/artifact bytes, including prior capacity and literal-command
fixes. These policies are offline QA controls, not another product source,
Session, arrangement/replay or human/musical/hardness authority.

### Feral Grid TR-909 QA Profile And Pressure Ownership

The bounded TR-909 source-profile and kick/accent-pressure family uses ordinary
binary-private owners (RBX-399 / RIOTBOX-1528). Profile policy consumes Grid,
config, filtering and spectral/runtime measurements. Pressure owns the existing
primitive-support render seam, accent/policy helpers and evidence; it consumes
the profile and existing library renderer, not root orchestration. The unchanged
fractional beat-to-frame helper lives beside the existing QA Grid calculations;
retain the complete Grid impl, mechanical capacity guard and all regressions.
Separate JSON presentation consumes profile/pressure evidence, never the reverse.

Private policy/metric helpers and serializer fields stay private. Cross-owner
evidence/functions and constants needed by retained assertions are binary-bound;
root test-only compatibility imports remain conditional. Ordinary grid-consumer
regressions import their actual profile/pressure/timing/evidence/metric owners.
Preserve all complete definitions, fixtures/test identities, frozen literals,
thresholds, schemas, provenance and CLI/artifact bytes. Existing `allow(dead_code)`
attributes on retained QA control wrappers are carried unchanged, not new
warning suppressions. The rendered-mix-pressure and broader mix families remain
explicitly legacy, outside this slice. No new renderer, fallback, Session/replay
truth, public API or musical/source-general authority is introduced.

### Feral Grid Source-Window QA Ownership

The bounded source-character search/selection family is an ordinary binary-private
owner (RBX-400 / RIOTBOX-1529). It consumes the existing Args/Grid, decoded
in-memory SourceAudioCache/typed windows and actual scalar-measurement owner.
Search preparation and selection belong together, not in pack orchestration.
The root retains file hydration, format validation and publication ordering;
the new policy owner opens no source file and depends on no root compatibility
import or second source model. Existing directly serializable evidence retains
its shape and labels, not new qualification authority.

Keep ranking/RMS helpers private. Evidence/functions and constants used by
retained ordinary regressions remain binary-bound. Tests import actual owners
and retain both identities and full synthetic in-memory bodies. Remove obsolete
root scalar/window aliases rather than suppress warnings. Preserve complete
algorithms, frozen thresholds/labels/schema, error/access/publication order,
CLI/artifact bytes and all prior guards. Remaining includes are still counted
legacy; this is neither a completed root migration nor source-general,
Core/Session/replay, human/music/hardness or live-device proof.

## Target Shape

Prefer this:

```rust
pub mod defaults;
pub mod export;
pub mod mc202;
pub mod model;
pub mod validation;

pub use defaults::*;
pub use export::*;
pub use mc202::*;
pub use model::*;
pub use validation::*;
```

Over this:

```rust
include!("session/version_types.rs");
include!("session/mc202_types.rs");
include!("session/defaults.rs");
```

The `pub use` layer can preserve compatibility, but the true ownership tree
should be visible as modules.

### Feral Grid MC-202 QA Ownership

The bounded MC-202 family inside `feral_grid_pack` uses ordinary binary-private
owners (RBX-401 / RIOTBOX-1530). Contour measurement/classification and low-band
dominance own the existing profile. Phrase/state policy consumes that profile
and the existing TR-909 support profile. Pressure rendering/evidence consumes
those policies, low-body DSP and the existing library renderer; JSON
presentation consumes the typed proofs and keeps serializer fields private.
Shared render measurements own the unchanged `RenderMetrics`, `render_metrics`
and `rms_delta`, rather than making pressure depend backwards on stem or mix
orchestration. All dependencies are in-process, not new adapters or product
truth. Shared data/functions and retained assertion constants are binary-bound;
single-owner helpers/thresholds remain private. Explicit root compatibility
imports preserve untouched legacy consumers, including manifest metric types.

Preserve complete definitions, typed origin, fixed phrase vocabulary, all
thresholds/literals/schemas, allocation and access/error/publication order, and
existing regression/fixture identities and CLI/artifact bytes. No algorithm
tuning, library runtime/DSP/API, Core/Session/action/replay or frozen Stage-A
change. These fixed QA plans and synthetic metrics stay diagnostic, not
source-general/product/music/hardness/human/live-device authority. Other root
families remain visibly legacy; this is not a completed root migration.

### Feral Grid Mix Policy And Evidence Ownership

The bounded mix family inside `feral_grid_pack` uses ordinary binary-private
owners (RBX-402 / RIOTBOX-1531). Mix components own existing scalar `MixPolicy`
data, pre/master-bus rendering and weighted contribution/ratio measurements.
Source/contour-selected balance policy consumes those computations and the
actual MC-202 contour, retaining its caps/floors and call-facing wrappers.
Movement evidence consumes components, policy and shared RMS measurements;
correlation/evidence helpers stay private. No dependency points backwards to
root orchestration or presentation. Existing directly serializable movement
evidence keeps all fields/labels; retained legacy fixture construction requires
binary-bound fields/constants, not a new public product model.

Product-stem production imports consume actual component/filter owners.
Existing root test compatibility paths/bodies stay intact, not newly claimed as
a fully migrated regression family. Local helpers/policy constants stay private;
shared data/functions and retained assertion constants stay binary-bound.
Preserve complete definitions, values/thresholds/literals/schemas, allocations,
limiter and access/error/publication order, regression identities and every
CLI/artifact byte. No library runtime/DSP/API, Core/Session/replay or Stage-A
change. These mixes remain QA controls, not source-general/music/hardness/human
authority. After this fifth ownership slice since RIOTBOX-1524, perform a
risk-directed architecture checkpoint before another family migration; this
bounded checkpoint is not a new global numeric audit cadence.

### Rendered TR-909 QA Pressure Ownership

The rendered TR-909 mix-pressure proof is an ordinary binary-private pure
computation owner (RBX-403 / RIOTBOX-1533), separate from root orchestration and
JSON presentation. It consumes existing source profile, render measurements,
kick/accent, mix movement and source-grid drift evidence through explicit
actual-owner imports. Keep the scalar input and directly serializable proof;
the three serializer-only fields and threshold-selector helpers stay private.
Shared inputs/read fields and constants needed by existing callers/assertions
remain binary-bound. Presentation imports the actual proof owner; ordinary
explicitly imported regressions retain all four names and every test/fixture
body. Root aliases serve only untouched legacy consumers.

Preserve all algorithms, thresholds/literals/schemas, allocation and source
access/error/publication order, test identities and CLI/artifact bytes. No new
adapter, public/library DSP/runtime interface, Core/Session/replay or frozen
Stage-A change. This proof stays diagnostic, never hardness or musical/source-
general/human/release authority. The RIOTBOX-1532 checkpoint is complete before
this next ownership slice; no global numeric audit cadence is invented.

### Feral Manifest Assertion Ownership

The three Feral manifest assertion families are ordinary `cfg(test)` owners
(RBX-404 / RIOTBOX-1534), not textual includes sharing root wildcard or constant
aliases. They consume the actual config, Core timing, listening schema,
TR-909/MC-202 and product-stem owners. Keep all nine assertion/helper bodies,
visibility, sibling/consumer call paths, test identities and failure ordering.
Small scalar/path/spectral/timing helpers stay private; the main complete
manifest witness deliberately remains cohesive and long rather than being
sharded by line count. Explicit dependencies and real namespaces reduce review
cost without a replacement validator, new product model or runtime interface.

Preserve every schema, threshold/literal, artifact-existence check, fixture,
production/orchestration body and CLI/artifact byte. Remove only obsolete
compatibility imports; retain root imports still used by untouched test families.
These assertions constrain diagnostics, never award music/hardness/source-
general/human/release or live-device authority. No source/DAW/device/playback,
Core/Session/replay/library DSP/runtime or frozen Stage-A change.

## Include Inventory And Guardrail

The initial RIOTBOX-1321 inventory lives in
`docs/engineering/textual_include_inventory_2026-06-29.md`.

To refresh the raw scan:

```bash
rg 'include!' crates --glob '*.rs'
```

For each include site record:

- owning file
- included files
- purpose
- generated/static/compatibility vs mechanical split
- migration risk

A manual guardrail rejects unexpected new textual include owners or changed
include counts while the allowlist is still being reduced:

```bash
scripts/check_no_textual_includes.sh
```

The current allowlist is
`docs/engineering/textual_include_allowlist.txt`. Update it only in PRs that
intentionally remove or convert include sites, or after a reviewed exception.
The guardrail should remain a developer check until the migration is far enough
to make it a hard CI gate.

## Review Checklist

Before merging a module-policy slice, answer:

- Did this remove or avoid a textual include used only for line-count pressure?
- Is the new boundary semantic rather than numbered or arbitrary?
- Are public exports preserved or intentionally changed?
- Are imports explicit enough for reviewers to see dependencies?
- Did the PR avoid unrelated feature or behavior changes?
- Are tests and formatting green?

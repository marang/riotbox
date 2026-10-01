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

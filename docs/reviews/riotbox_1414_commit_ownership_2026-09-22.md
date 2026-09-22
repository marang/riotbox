# RIOTBOX-1414 — Commit ownership and Core layering

Classification: maintenance/regression. Integrated base `341de137`. This makes
the existing performer-action commit path easier to audit; it does not add a
gesture, change source policy, retune sound or grant a new musical claim.

## Ownership changes

App `jam_app/side_effects.rs` now owns one exhaustive match over the existing
64 `ActionCommand` variants. It invokes only the appropriate lane/control
handler. Commands handled by earlier capture materialization, external export
transactions or other structural paths have explicit no-effect arms for this
stage. `W30LoopFreeze` remains a two-stage case: materialize its capture first,
then apply its W-30 effect. A new enum variant cannot silently fall through a
wildcard; no parallel registry or persisted dispatch table was introduced.

The commit pipeline still snapshots Undo before mutation, materializes capture
audio before lane effects, discards rejected undo state, records replay
artifacts, and mirrors transport afterward. The individual handler bodies and
their parameter/error/result behavior are unchanged. MC-202 restore retains
its existing dedicated hydration use of the handler rather than re-executing
the whole live dispatcher during replay.

Core `session/source_timing.rs` now owns the two timing-confirmation state types
and the source-ID/optional-hypothesis comparison. Their JSON shapes and public
Session exports are unchanged. Policy and App observer import from Session;
the old Jam-view function path remains a compatibility re-export. This removes
the production policy-to-view dependency without creating new trust semantics.

`CommitBoundary::rank` is the one internal musical ordering rule. Quantization
maps to a required boundary; replay ordering consumes the same rank when
transport positions tie. Immediate/Beat/HalfBar/Bar/Phrase/Scene behavior and
the remaining replay sort keys are unchanged. These are reversible ownership
corrections to existing contracts, not a new algorithm/schema decision.

## Verification

A test-only frozen copy of the prior broadcast sequence compares complete
Session state and action results against the dispatcher for all 64 commands,
15 valid/missing/mismatched parameter cases, two target shapes and both present
and absent Graph context: **3,840 comparisons**. Preset variants and a seeded
confirmed grid ensure preset application and grid reversion are exercised,
not only no-op paths. The oracle is test-only; production owns one dispatcher.
This supplements existing end-to-end action/queue/capture/replay tests rather
than claiming exhaustive proof for arbitrary inputs.

Additional tests cover every quantization/boundary pair, all six replay boundary
kinds at one transport position, confirmation source/hypothesis mismatches,
legacy no-hypothesis confirmation, missing confirmation and unchanged serde
shape/defaults. Initial Core, dispatcher differential and workspace Library
runs pass. Full integrated, normal-parallel source-free `just ci` passes,
including 763 App library tests, the remaining Rust/Python suites, synthetic
audio/observer gates and strict Clippy. The textual-include guard also passes.
Local logs: `/tmp/riotbox-1414-ci.log` and `/tmp/riotbox-1414-includes.log`.

No real source file, Holdout, commercial reference, DAW or human playback was
used. New timing fixtures are metadata-only; existing audio checks generate
their own synthetic inputs. Rave-punk guidance is applied as a preservation
constraint: performer gestures keep their existing effect and timing; no new
taste verdict is claimed.

## Branch review

Solo code-review/Rust lenses cover exhaustive mapping against each handler's
command guard, commit sequencing, source-timing compatibility, replay order,
module exports, tests and workflow/spec drift. No handler body or musical
constant changed. New source-timing ownership is a real module containing its
state and rule, not a textual include or a pass-through layer. Existing public
imports remain valid. No new `ActionCommand` requires additional product
surfaces; the change hardens ownership of the current ones.

No retained additional finding; follow-up self-review finds no further issue.
This records maintainability/regression evidence, not live-device, host-import,
source-general, hardness or human-listening qualification.

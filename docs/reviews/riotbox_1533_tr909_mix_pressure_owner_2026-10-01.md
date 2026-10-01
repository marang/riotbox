# RIOTBOX-1533 — Rendered TR-909 QA pressure ownership

Date: 2026-10-01
Baseline: `782cacfe7c9b0f45f0f541b92abf9a8927d218a4`
Integrated predecessor archive: `2d91e8cdce423c6bf927a28295d7b2fe2a8d149f`
Decision: RBX-403
Classification: maintenance/regression; behavior-preserving semantic migration

## Outcome and scope

Rendered TR-909 mix-pressure computation now has one cohesive ordinary
binary-private owner. It consumes actual source profile, render measurements,
kick/accent, mix movement and source-grid drift owners through explicit imports.
The existing scalar input and directly serializable proof stay intact. Threshold
selectors, the primitive-only evidence label and three serializer-only proof
fields remain private; shared input/read fields and constants required by
existing callers/assertions stay binary-bound.

An ordinary explicit regression owner retains the four existing names and every
test/fixture body. JSON presentation imports the actual proof owner. Narrow root
compatibility aliases serve untouched legacy consumers, not a reverse dependency
from computation or regression into root orchestration. No new adapter, getter
layer, public/product model or renderer. The production and regression owners
are 139 and 227 lines; responsibility and dependency direction, not file length,
justify the split. Other root render/orchestration/assertion families stay legacy.

This Track A slice follows explicit autonomous non-DAW authorization. It changes
no musician action, display or expected sound and grants no audible Golden Path
closure. Codebase-design informs the cohesive in-process interface; the repo
migration contract overrides generic replacement-test advice. Domain-modeling
records the ownership trade-off in the canonical EOF Decision Log. Development,
Rave-Punk and Listening-Review preserve the diagnostic/non-musical claim boundary.

RIOTBOX-1532's required risk-directed checkpoint is reviewed and merged before
this family starts. The issue is Todo then In Progress before dedicated branch
creation and any code edits. Baseline tests/build/CLI verification use the
archive-only predecessor candidate with an explicit empty code diff against
merged main. The predecessor archive is subsequently merged and fully closed;
the bounded ownership-series counter resets there. Its disjoint three-file
archive update is fast-forwarded without discarding this slice's changes.

## Exact preservation

All 38 complete definitions remain equal after Rustfmt with only visibility and
trailing-comma normalization: ten computation/data/constant definitions, ten
regression/fixture definitions, 14 retained serializer definitions and four
retained orchestration definitions. Complete bodies, derives, strings and
thresholds are compared, not merely names or fingerprints. No algorithm,
value/literal/schema, allocation, limiter or source access/error/publication
ordering changes. Prior capacity/numeric/timing/literal-command and migrated
W-30/TR-909/window/MC-202/mix contracts remain unchanged.

Both actual Cargo profiles pass before and after edits with identical 52 unit
and two CLI integration names/statuses. No test is removed or ignored. Fresh
standalone builds precede all 31 exact CLI status/stdout/stderr comparisons.
All 39 hashes remain identical: 38 outputs including 16 WAVs, complete reports,
README and manifests, plus input. Both manifests validate existing paths. No
parameter is adjusted against these synthetic results.

The sole file input is this exact fresh four-second synthetic control:
`/tmp/riotbox-1533-byteproof.JESAO6/synthetic-control.wav`.
Retained unit fixtures are synthetic/in-memory. Only this run's bounded outputs
are overwritten for comparison. No real Development/Holdout/commercial audio,
source-directory search, DAW/device or playback. Technical-only reruns need no
new listening verdict and do not inherit any historical one.

## Review and diagnostics

Initial compilation succeeds but its warning scan rejects obsolete root
origin/Serde/profile/drift/mix assertion imports. Remove unneeded imports and
make retained source-origin/profile compatibility test-only. The initial
`check-final` log stays diagnostic; only `check-reviewed` and the final reviewed
test/build logs count as warning/error-free proof, without suppression.

Solo sequential code-review/Rust correctness, architecture, tests/spec and
workflow lenses, followed by short self-review: zero remaining actionable
findings, no independent panel. Actual callers, serialization, dependency
direction and field/helper visibility are inspected. Normalized bodies and
actual baseline/final CLI/hash evidence independently constrain behavior drift.
No callback/library DSP/API or Core/Session/replay/Stage-A path changes.

Include inventory shrinks nine to eight sites in the same two owners, seven root
plus one legacy shared test helper. Guard, policy, inventory and EOF RBX-403 agree;
no allowance expansion or fabricated numeric checkpoint cadence.

## Verification and remaining gates

- Baseline Debug/Release/build: `/tmp/riotbox-1533-baseline-*.log`
- Final check: `/tmp/riotbox-1533-check-reviewed.log`
- Final tests: `/tmp/riotbox-1533-final-{debug,release}-reviewed.log`
- Final build: `/tmp/riotbox-1533-final-build-reviewed.log`
- Exact synthetic CLI/hash comparison and both manifest validators; Rustfmt,
  diff/include guards; final focused logs scanned warning/error-free
- Full source-free `just ci` passed with actual exit zero and warning/error-free
  `/tmp/riotbox-1533-ci-final.log`; exact-head native Rust/Windows Sidecar jobs
  and review remain required before merge, archive and exact branch/Linear cleanup

Numerical rendered-pressure evidence stays diagnostic, not product/source-
general, musical, hardness, human, release or live-device proof. Native Windows
transport does not establish Windows audio/filesystem/device behavior.
RIOTBOX-1509 remains Todo awaiting causal startup evidence. No new hardening,
calibration, source qualification or P023 closure is implied by this migration.

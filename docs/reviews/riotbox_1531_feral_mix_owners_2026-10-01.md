# RIOTBOX-1531 — Feral mix computation and evidence owners

Date: 2026-10-01
Baseline: `050e4f46bc6cc9376c7980dd681fadd9cd2ee064`
Integrated predecessor archive: `dd3d510e013993c88c238723676843052de7fb31`
Decision: RBX-402
Classification: maintenance/regression; behavior-preserving semantic migration

## Outcome and scope

Existing mix component math, source/contour-selected balance policy and movement
evidence now belong to ordinary binary-private owners. Components own scalar
MixPolicy data, pre/master-bus rendering and weighted contribution/ratio
measurements. Balance policy consumes those computations and the actual MC-202
contour, retaining caps/floors and existing call-facing wrappers. Movement
evidence consumes components, policy and shared RMS measurements, never root
orchestration. Correlation and evidence helpers remain private.

Placing shared scalar data in the policy owner would create a backwards
measurement dependency. Components provide the existing in-process data and
computation instead, also consumed directly by product-stem reconstruction.
No adapter, mock, new renderer, product source model, Session/arrangement/replay
truth or public library interface. The original directly serializable proof
keeps every field, label and threshold. Existing legacy fixture construction
requires binary-bound proof fields/constants; no getter or constructor layer.

Root explicit/cfg-test imports retain untouched callers and existing test paths.
Product-stem production imports now use actual component/filter owners, but its
existing regression compatibility path/body is intentionally unchanged. This
does not claim a fully migrated regression family or root. Other root families
remain visibly legacy. Owners are 132, 213 and 177 lines, grouped by dependency
direction and responsibility, not arbitrary file-size shards.

This improvement-track A slice follows the user's autonomous non-DAW scope.
It changes no musician action or expected sound and claims no audible Golden
Path closure. Codebase-design informs the one-way in-process seams; the repo
migration contract overrides generic advice to replace existing tests.
Domain-modeling records the ownership trade-off in the canonical Decision Log.

## Exact preservation

All 44 complete definitions remain equal after Rustfmt with only visibility
and trailing-comma normalization: 28 mix definitions, 14 retained production/
helper/type/constant definitions and two retained product-stem regression/
fixture bodies. Equality covers complete policies, render/measurement helpers,
mix gates and retained orchestration. No algorithm, gain/drive value, threshold,
literal, schema, allocation, limiter or source-access/error/publication order
change. Prior capacity/literal-command and W-30/TR-909/window/MC-202 fixes stay.

Baseline preparation used the reviewed predecessor candidate
`fb91eaf8451e505b3055b6127f2dc8f3c05fbfbb`, before its native merge. Its entire
tree was verified identical to merged main before the RIOTBOX-1531 branch was
created from the baseline above. Linear was Todo then In Progress before branch
creation and all implementation changes; no stacked implementation branch.

Both actual Cargo profiles pass before and after edits with the same 52 unit
and two integration test names/statuses. All test/fixture bodies and ignored
status remain unchanged. Standalone builds precede baseline/final reviewed CLI
comparisons. All 31 status/stdout/stderr results remain exact. Bounded auto and
explicit-140-BPM two-bar packs retain all 39 hashes: 38 outputs including 16
WAVs, full reports/README/manifests, plus input. Both manifests validate existing
paths. Algorithms and values are not tuned against these synthetic results.

The sole file input is the exact fresh four-second generated control:
`/tmp/riotbox-1531-byteproof.MRxbva/synthetic-control.wav`.
Existing unit fixtures are synthetic/in-memory. Only this run's bounded
technical outputs are overwritten for comparison. No real Development/Holdout/
commercial audio, source-directory discovery, device, DAW or playback.

## Review and diagnostics

Initial compilation passes but the explicit warning scan rejects stale root
Mc202ContourHint/signal_metrics_with_grid runtime aliases. Remove the obsolete
signal-metric alias and keep contour/note-budget compatibility test-only. The
already-obsolete master-bus alias and root RMS-delta alias are also removed;
root MixPolicy/filter aliases are test-only. Final reviewed check/build/tests
are warning/error-free, with no new suppression. Initial warning-bearing logs
remain diagnostic, not accepted verification.

Solo sequential code-review/Rust/correctness, architecture, tests/spec and
workflow review, then short self-review: zero remaining actionable findings,
no independent panel. Actual imports, dependency directions, visibility and
limiter/product-stem consumers inspected. No speculative finding is added for
tool-detected import cleanup. No callbacks or library sound-design paths change.
The one converted include shrinks inventory 10 → 9 in the same two owners
(eight root, one legacy shared test helper), without allowance expansion.
Policy, inventory and EOF RBX-402 agree.

## Verification and remaining gates

- Baseline Debug/Release/build logs: `/tmp/riotbox-1531-baseline-*.log`
- Final check: `/tmp/riotbox-1531-check-reviewed.log`
- Final tests: `/tmp/riotbox-1531-final-{debug,release}-reviewed.log`
- Final build: `/tmp/riotbox-1531-final-build-reviewed.log`
- Both exact synthetic manifest validators; Rustfmt, diff/include guards;
  final focused logs explicitly scanned warning/error-free
- Full source-free `just ci` before PR; exact-head native Rust/Windows Sidecar
  jobs/review before merge, then archive and exact cleanup

Numerical mix movement/ratios remain diagnostic, not source-general/product,
music, hardness, human, release or live-device proof. No library runtime/DSP/API,
Core/Session/action/replay or frozen Stage-A change. Windows transport CI is not
Windows audio/filesystem/device proof. RIOTBOX-1509 remains Todo awaiting causal
startup evidence. After this fifth ownership slice since RIOTBOX-1524, perform
the bounded risk-directed architecture checkpoint before another family
migration; this introduces no universal numeric audit cadence.

# RIOTBOX-1534 — Ordinary Feral manifest assertion owners

Date: 2026-10-01
Baseline: `1451cf28159e1c5c2eac9c4a580c90a33ccf055e`
Integrated predecessor archive: `e879dea8eeee2f73803d731e5b54649cdf210e80`
Decision: RBX-404
Classification: maintenance/regression; behavior-preserving semantic migration

## Outcome and scope

Three existing manifest assertion families are ordinary `cfg(test)` owners at
their existing module/call paths. Main schema, threshold, artifact, timing and
spectral checks consume actual config, Core timing, listening schema, TR-909 and
product-stem owners. MC-202 checks consume actual pressure, origin-presentation
and source-phrase owners; mix checks retain their scalar-report interface.
No wildcard root dependency, replacement validator, adapter or product model.

All nine complete assertion/helper bodies and existing visibility stay intact.
Scalar/path/spectral/timing helpers remain private. Existing sibling calls and
the consumer `tests::renders_grid_pack_files_and_noncollapsed_audio` are unchanged.
The main complete manifest witness intentionally remains cohesive and long:
677 lines including explicit imports and Rustfmt expansion. Splitting that
stable assertion body by field count would obscure its failure/order contract.
The two other semantic owners are 86 and 44 lines. This is an explicit soft-
budget review, not an invented file-size compliance claim.

Remove only now-obsolete root compatibility imports. Retained root imports
still serve untouched regression families; shared helper/test and production
render/orchestration includes remain visibly legacy. The guard shrinks eight to
five sites in the same two owners, four root plus one shared legacy test helper.

This Track A slice follows autonomous non-DAW authorization and grants no
musician-facing behavior or audible Golden Path advance. Codebase-design favors
existing cohesive interfaces and actual dependencies; repo migration rules
override generic advice to replace old tests. Domain-modeling records the
ownership choice at canonical EOF RBX-404. Development, Rave-Punk and Listening-
Review keep diagnostics separate from musical/source/hardness authority.

## Baseline and preservation

The required RIOTBOX-1532 checkpoint is already complete. Source-free baseline
verification uses reviewed predecessor candidate
`5d4e716af10b9e4001320ef2918a27e9c14c8ae1`; its entire tree is verified equal
to merged main before the issue's implementation branch starts. Linear is Todo
then In Progress before branch creation and any code edit. The predecessor
archive later closes normally, and its disjoint three-file update is fast-
forwarded without discarding this slice's changes. No stacked feature branch.

All 25 complete definitions remain identical after Rustfmt with only visibility
and trailing-comma normalization: nine moved assertions/helpers, 12 retained
regression bodies and four retained orchestration definitions. This compares
complete bodies and literals, not symbol counts or fingerprints. No schema,
threshold/literal, assertion order, artifact-existence check, fixture,
production/orchestration algorithm, allocation, limiter or source
trust/access/error/publication order change. Previous capacity/numeric/timing/
literal-command and ownership guards remain unchanged.

Debug and actual Release both preserve all 52 unit and two CLI integration
names/statuses, without removed/ignored tests. Fresh standalone builds precede
31 exact CLI status/stdout/stderr comparisons. All 39 hashes are identical:
38 outputs including 16 WAVs, full reports/README/manifests, plus input. Both
manifests validate existing paths. No values are tuned against synthetic results.

The sole file input is this exact fresh four-second generated control:
`/tmp/riotbox-1534-byteproof.oO0Rb8/synthetic-control.wav`.
Existing unit fixtures remain synthetic/in-memory. Only this run's bounded
technical outputs are overwritten for comparison. No real Development/Holdout/
commercial audio, source-directory search, DAW/device or playback. Technical-
only reruns do not need or inherit a human verdict.

## Review and diagnostics

Initial compilation succeeds but the warning scan rejects eight obsolete
root alias/import groups. Remove those imports rather than suppress warnings.
The initial check log stays diagnostic, never green proof; final reviewed
check/test/build logs are warning/error-free.

Solo sequential code-review/Rust correctness, dependency/visibility/call-path,
test/schema/threshold/assertion/failure-order, workflow/docs and file-growth
lenses followed by short self-review: zero remaining actionable findings,
no independent panel. The main witness's size is intentional and bounded;
no arbitrary sharding or unreviewed whole-root migration. Policy, inventory,
allowlist and EOF RBX-404 agree, with no allowance expansion.

## Verification and remaining gates

- Baseline Debug/Release/build: `/tmp/riotbox-1534-baseline-*.log`
- Final check: `/tmp/riotbox-1534-check-reviewed.log`
- Final tests: `/tmp/riotbox-1534-final-{debug,release}-reviewed.log`
- Final standalone build: `/tmp/riotbox-1534-final-build-reviewed.log`
- Exact synthetic CLI/hash comparison, both manifests; Rustfmt and diff/include
  guards, final focused logs explicitly scanned warning/error-free
- Full source-free `just ci`: actual exit zero and warning/error-free
  `/tmp/riotbox-1534-ci-final.log`
- Exact-head native Rust/Windows Sidecar and PR review gates remain before
  merge, followed by archive and exact branch/Linear cleanup

No Core/Session/replay/library DSP/runtime/public API or frozen Stage-A change.
Assertions constrain diagnostics, never grant music/hardness/source-general/
human/release/live-device authority. Windows transport does not prove Windows
audio/filesystem/device behavior. RIOTBOX-1509 remains Todo awaiting causal
startup evidence. No new source qualification, calibration or P023 closure.

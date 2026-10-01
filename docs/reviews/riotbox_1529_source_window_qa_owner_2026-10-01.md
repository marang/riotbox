# RIOTBOX-1529 — Bounded Feral QA source-window owner

Date: 2026-10-01
Baseline: `7dc792487c36c4dd33d5f7a98ed317ded4a1af0f`
Integrated predecessor archive: `f5e020c72f54225e05d67ab7ba5a50d51799d65e`
Decision: RBX-400
Classification: maintenance/regression; behavior-preserving semantic migration

## Outcome and scope

Existing search-window preparation and character selection now belong to one
ordinary binary-private owner. It consumes actual Args/Grid, decoded in-memory
SourceAudioCache/typed windows and scalar measurements, not root lexical aliases.
The root retains hydration, format validation and publication. This owner opens
no source file and creates no new product source/timing/Session/replay truth.

This is a bounded improvement-track ownership slice after the RIOTBOX-1524
checkpoint and merged W-30/TR-909 policies. It makes QA selection/provenance
reviewable beside audible work; it does not claim an audible Golden Path blocker
was resolved or a new mechanism qualified. The existing directly serializable
evidence keeps every field, label and value. Ordinary regressions import actual
owners without retaining wildcard root dependencies or changing test identities.

Selection/ranking/RMS helpers remain private. The evidence type and cross-owner
fields/functions/test thresholds are binary-bound. Four fields consumed only
within the selector/derived serializer remain private: requested/search starts
and selected frame start/count. No new getters, adapter, mock or dependency.
Root orchestration and existing manifest/text consumers use explicit imports;
all other root families remain legacy.

## Exact preservation

All 15 complete definitions remain equal after Rustfmt with only visibility and
trailing-comma normalization: eight moved production/helper/type/constant
definitions, two moved regressions and five retained orchestration definitions.
The search helper moves whole. Ranking, hop/final-candidate logic, RMS retention,
window bounds, source-format validation, numeric/string literals and thresholds
remain unchanged. No algorithm is tuned against the synthetic results.

Both actual Cargo profiles passed before edits. After final review corrections,
the same 52 unit plus two integration leaf names/statuses remain exact and pass
in Debug and Release. Both synthetic in-memory source-selection regression
bodies are retained, without removed/ignored tests or ownership path substitutions.
The migration contract takes precedence over generic test-replacement guidance.

Standalone builds precede the baseline and final reviewed CLI comparisons. All
31 actual CLI status/stdout/stderr cases remain byte-identical. The fresh bounded
auto/explicit-140-BPM two-bar packs keep all 39 hashes: 38 outputs, including 16
WAVs, numerical reports, README and manifests, plus the exact input. Both
manifests validate existing artifact paths. Privacy changes preserve the complete
serialized evidence shape and bytes, not only audio fingerprints.

The sole file input is the exact newly generated four-second synthetic control:
`/tmp/riotbox-1529-byteproof.vYeHmk/synthetic-control.wav`.
The two unit regressions construct synthetic caches in memory; their filename
labels do not open audio files. Only this run's bounded technical output packs
are intentionally overwritten for comparison. No real Development/Holdout/
commercial audio or source directory is accessed.

All prior capacity, literal verification-command, W-30/TR-909 ownership and
Core timing-trust contracts are retained. Access/error/directory/artifact/gate
ordering remains the existing behavior, not a newly transactional QA pipeline.

## Findings and disposition

**P3 — stale root window alias, fixed.** Moving the search helper leaves the
`SourceAudioWindow` import in `pack_builder.rs` unused. Remove that alias rather
than suppress the warning; final check/build and both reviewed test profiles are
warning/error-free. Initial warning-bearing logs remain diagnostic, not green
evidence. The now-obsolete four root scalar-measurement aliases were also removed.

**P3 — unnecessary evidence-field exposure, fixed.** The initial migration
made four selector/serializer-only fields binary-visible. Keep those fields
private; final compilation proves consumers need no getter or changed API,
whole-definition normalization and the reviewed 39 hashes prove preservation.

Solo sequential correctness, Rust/architecture, tests/spec and workflow review,
then self-review: zero remaining actionable findings, no independent panel.
Actual-owner dependencies and absence of I/O/root cycles inspected. The ordinary
policy owner is 236 lines; the regression owner is 97 lines, not arbitrary
numbered shards. Include inventory shrinks 13 → 12 sites in the same two owners
(11 at the root, one legacy shared test helper), without allowance expansion.
Module policy, inventory and EOF RBX-400 agree.

An older restore-history scaling suggestion was checked against current indexed
`history_validation.rs` and rejected as already fixed by `cef503d6` / PR #1525.
It is not reopened from stale review evidence. RIOTBOX-1509 remains Todo awaiting
a fresh causal Windows startup reproduction.

## Verification and remaining gates

- Final check: `/tmp/riotbox-1529-check-final.log`
- Baseline tests: `/tmp/riotbox-1529-baseline-{debug,release}.log`
- Final reviewed tests:
  `/tmp/riotbox-1529-final-{debug,release}-reviewed.log`
- Baseline/final reviewed standalone builds:
  `/tmp/riotbox-1529-baseline-build.log`,
  `/tmp/riotbox-1529-final-reviewed-build.log`
- Both exact synthetic manifest validators, Rustfmt, diff/include guards;
  final focused logs explicitly scanned warning/error-free
- Full source-free `just ci` before PR, then exact-head native Rust/Windows
  Sidecar jobs/review before merge; focused proof does not replace those gates

No real device, DAW, playback, human verdict or fresh source qualification.
QA selection/metrics stay diagnostic, never source-general/product/music/
hardness/human/release/live-device proof. Core/Session/action/replay, library
runtime/DSP/API and frozen Stage-A are unchanged. Windows transport CI is not
Windows audio/filesystem/device proof.

After native gates: merge the reviewed full head, sync main, archive/validate/
merge the issue archive and perform only exact branch/Linear cleanup. Structural
cadence advances from two since RIOTBOX-1524 to three only on implementation merge.

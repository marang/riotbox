# RIOTBOX-1520 — Semantic Feral before/after diagnostic ownership

Date: 2026-10-01. Classification: maintenance/regression. Code baseline:
`fa93d2233fd49a8d734274bd4afab54455d5e482`. Independent implementation began
while RIOTBOX-1519's W-30 PR #1582 awaited native CI; its merged predecessor
`7f3b0cb4a8d978b2d080e5df93e1ebd584128e8e` is now integrated, with no changes
to the original Feral baseline. RBX-394 and canonical inventory/policy retain
RBX-393 and reduce includes from **32/three owners to 30/two**.

## Scope and authority

The same `feral_before_after_pack` Cargo binary now has a standard directory
`main.rs` and ordinary private modules instead of two textual includes.
Config is a leaf; CLI retains the complete Args impl. Pack orchestration consumes
source-window metadata/averaging, existing fixed lane render-state plans, mix/
sequence/delta measurements, WAV/metrics publication, Markdown and manifest.
No dependency cycle. Main never exports a library API; one-owner serialization
DTOs/manifest construction and the test PCM fixture remain private. Shared
fields/functions are bounded to this binary and tests name actual owners.
Eleven compiled files, largest production owner 134 lines; not numbered shards.

Fixed TR-909/MC-202 plans and the offline combined output remain diagnostic
scaffolding, not a second Feral instrument or source intelligence. The existing
manifest's `pass` is only its numeric gate result, never human approval. No
Core/Session/action/replay, runtime/DSP, algorithm/literal/threshold/schema or
source-access contract changes. The first migration deliberately preserves
side effects and errors rather than bundling behavioral corrections.

## Verification

- All **41 complete definitions** match before and after Rustfmt after only
  visibility/trailing-comma normalization: 37 production definitions, three
  complete unit regressions and their synthetic PCM fixture function.
- The same **three executed test leaf/status entries** pass in original/final
  Debug and final actual Release, including generated PCM render/manifest and
  signal-distinctness checks. No test/assertion/fixture body removed.
- All **29 actual CLI cases** retain exact stdout/stderr/status bytes: help,
  missing/unknown arguments, non-finite/negative/zero/text values, missing WAV,
  too-short source, default two-second and custom one-second successful packs.
  Existing `--help` without `--source` still fails; no hidden parser fix.
- Two fresh packs at identical paths retain all **28** file hashes, including
  **12 WAVs**, metrics, comparison, README and manifest; the exact input WAV's
  hash also remains unchanged (**29 total**). No tolerance/path normalization.
  Scope: `/tmp/riotbox-1520-byteproof.KtJorn/{default,custom}`; input exactly
  `/tmp/riotbox-1520-byteproof.KtJorn/synthetic-control.wav`, newly generated
  by the existing `scripts/write_synthetic_break_wav.py` for 2.5 seconds.
  No source directory or real audio corpus was opened or searched.
- Both baseline and final generated manifests independently validate with
  `validate_listening_manifest_json.py --require-existing-artifacts`; this
  checks metadata/artifact existence, not waveform identity or a human verdict.
- Cargo metadata retains exactly one unchanged binary identity, with only its
  standard directory entrypoint changed; no Cargo manifests/dependencies edited.
- All-target check, baseline/final build and Debug/Release logs explicitly
  scanned warning/error-free: `/tmp/riotbox-1520-check.log`, baseline-tests,
  baseline-build, tests, release and build logs with the same `/tmp/riotbox-1520-`
  prefix. All imports explicit; no warning suppression.
- Full source-free `just ci` passes in `/tmp/riotbox-1520-ci.log`, explicitly
  warning/error-free: Rust/Python/contracts, synthetic audio gates, formatting/
  tracked JSON and strict all-target/all-feature Clippy. Native exact-head PR
  checks and merge/archive/cleanup remain separate completion obligations.

## Review and limits

Solo sequential correctness, Rust, architecture, workflow/docs and audio-access
review followed by self-review; not an independent panel. Inspected complete
original source, 41-definition mapping, dependencies/visibility, Cargo target,
source-before-render checks, publication/numeric gate order, regressions and exact
CLI/artifact parity. **Zero additional changed-diff findings**. Extraction's
module-name convention and the manifest validator's one-path invocation were
corrected in verification orchestration before recording successful evidence;
neither was a production change or suppressed failure.

The original tool creates the output/stems directory before source loading and
writes WAV/report/README before numeric gate failure; that order is preserved,
not described as transactional or fail-before-all-I/O. Existing unlimited finite
resource and tiny-positive rounding behavior are outside this ownership slice.
The synthetic source fixture and fixed render states do not establish source
intelligence, source diversity, hardness, release or live-mixer/device behavior.

No real Development/Holdout/commercial audio, source-directory discovery, device,
DAW, playback or new/transferred human/musical verdict. Frozen Stage-A contracts
untouched. Architecture cadence remains two after RIOTBOX-1519 merge and advances
to three only after this ownership slice merges; RIOTBOX-1509 stays open.

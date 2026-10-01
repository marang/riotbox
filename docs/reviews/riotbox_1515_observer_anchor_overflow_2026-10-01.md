# RIOTBOX-1515 — Reject unrepresentable Observer anchor totals

Date: 2026-10-01. Classification: maintenance/regression, not audible product
progress. Scope: the existing Observer/audio metadata parser and source-free
CLI regressions. No thresholds, schema, Stage-A protocol, product-state owner,
audio behavior or dependency changes.

## Defect and disposition

**P2, fixed:** `SourceTimingAnchorEvidence::typed_anchor_count()` previously added
three untrusted `u64` values without checking either addition. The caller's
comparison with the declared primary total did not make those additions safe.
Two new tests fail against the original implementation in both profiles:

- Kick `u64::MAX`, backbeat one, transient zero overflows the first addition.
- Kick `u64::MAX - 1`, backbeat one, transient one overflows the second addition.

Both use declared total `u64::MAX`. Debug panics during addition; actual Release
tests return `Ok(Some(...))` instead of rejecting the contradictory counts.
The Release consequence is now executed evidence, not just a code-derived claim.
Logs: `/tmp/riotbox-1515-red-debug.log`,
`/tmp/riotbox-1515-red-release.log`.

The existing private sum now returns `Option<u64>` using two `checked_add`
operations. The shared collector turns `None` into its existing `Err(())` before
comparing a representable total. Both manifest and observer collectors already
translate this failure into malformed source-timing evidence. No saturation,
wrapping or silent conversion hides contradictory counts; ordinary JSON and
strict validation retain their established handling of malformed metadata.

## Verification

- Minimal two-test red -> green verified in Debug and Release before adding
  boundary/CLI coverage.
- All **67** Observer binary tests pass in each profile (61 retained, six new).
  Seven valid cases cover zero, ordinary exact/slack totals, each single typed
  count at `u64::MAX`, and a three-part sum exactly reaching `u64::MAX`.
  Round-trip anchor JSON equals the input; no arbitrary maximum is introduced.
- Three non-overflowing totals above their declared count still reject. Absent/
  null optional evidence remains absent. Negative, fractional, string, boolean
  and null values reject for each of the four count fields without lossy casts.
- **Two CLI integration tests** pass in Debug and Release. Each runs both
  overflow positions through actual manifest/observer inputs. Local JSON marks
  invalid timing unavailable/malformed; strict mode returns status one with
  no stdout, neither creates an absent report nor overwrites an existing report.
  Existing observer/manifest metadata fixtures are embedded; only fresh temporary
  NDJSON/JSON/report files are read/written. Referenced audio paths are never opened.
- All **21** previously captured valid/error/help CLI stdout/stderr/exit cases
  remain byte-identical after the fix.
- Logs `/tmp/riotbox-1515-tests-debug.log` and
  `/tmp/riotbox-1515-tests-release.log` explicitly scanned warning/error-free.
- Full source-free `just ci` passes in `/tmp/riotbox-1515-ci.log`, explicitly
  warning/error-free: workspace Rust/Python/contracts, synthetic audio smokes,
  formatting/tracked JSON and strict all-target/all-feature Clippy. Native
  exact-head PR CI remains a separate gate.

## Review and trust-boundary audit

Solo sequential correctness, Rust, architecture and workflow review followed by
short self-review; not an independent reviewer panel. The mathematical boundary,
both caller paths, serialization, actual strict CLI publication and regression
coverage were inspected. **No additional finding in the changed diff.** The
demonstrated P2 is fixed locally, not claimed merged before its native gate.

Adjacent count readers use `Value::as_u64`; source-timing alignment compares
counts rather than aggregating them. The adjacent groove preview cap still casts
a `u64` count to `usize` before capping at four. On this tested 64-bit target it
does not truncate; a 32-bit portability limitation remains code-derived and
unexecuted here. This slice does not claim a 32-bit qualification or hide it as
a corrected behavior. No new architectural/product contract or Decision Log
entry is required to restore the existing malformed-evidence semantics.

Core/Session/replay truth and realtime/audio code are untouched. Full CI's
synthetic audio controls are not real-source or human evidence. No Development/
Holdout audio, commercial reference, device, DAW or playback access. Historical
RIOTBOX-1509 Windows startup flakiness remains a separate unresolved diagnosis.

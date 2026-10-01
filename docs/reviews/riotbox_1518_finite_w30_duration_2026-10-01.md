# RIOTBOX-1518 — Finite W-30 QA render duration

Date: 2026-10-01. Classification: maintenance/regression. Scope: standalone
`w30_preview_render` argument validation; no ownership migration or audio change.

## Defect and disposition

**P2, fixed locally:** the render-duration branch parsed `f32` but rejected
only `<= 0`, unlike the helper's already-finite source-window arguments.
`NaN` passed, cast to zero frames and produced a header-only WAV with success.
Fresh bounded source-free repro wrote
`/tmp/riotbox-w30-duration-repro.1JectL/nan.wav`: status zero, 44 bytes,
duration `NaN`, zero samples/RMS. No source argument, device or playback.

Separate parser regressions fail for NaN and positive infinity in Debug and
actual Release. Positive infinity's capacity-overflow-sized allocation is
code-derived; it was not rendered unconstrained. Logs:
`/tmp/riotbox-1518-red-debug.log`, `/tmp/riotbox-1518-red-release.log`.

One production-line change adds `!duration_seconds.is_finite()` to the existing
invalid-duration guard. It rejects before source hydration, rendering or writes;
preserves existing zero/negative/nonnumeric errors and all finite-positive
evaluation. No duration clamp, arbitrary maximum, schema or threshold is added.
W-30 comparison already rejects non-finite metrics/limits; that protection is
not treated as a new defect or reopened by this slice.

## Verification

- Minimal two-test red -> green in both profiles before CLI coverage.
- **Ten unit tests and two actual CLI integration tests** pass in Debug and
  Release; eight existing unit regressions retained, two added.
- **40 rejection paths:** ten non-finite spellings, with/without a source
  argument and absent/existing output artifacts. Each returns status one,
  empty success stdout and the duration error; no panic, source hydration,
  new output directory/artifact or overwrite of prior WAV/metrics bytes.
  The optional source path is a named nonexistent temporary path, never audio.
- Finite 0.05-second explicit synthetic control still writes the expected
  **8864-byte** PCM16 WAV and **4410** interleaved samples plus metrics.
- All **13** previous help/error/finite-render CLI stdout/stderr/status cases
  are byte-identical. The default, 0.5-second and 1.5-second finite controls at
  `/tmp/riotbox-1518-finite-bytes.vupEEl` retain all **six** file hashes:
  three WAVs and their metrics. No tolerance/path normalization used.
- Target logs `/tmp/riotbox-1518-tests-debug.log` and
  `/tmp/riotbox-1518-tests-release.log` explicitly scanned warning/error-free.
- Full source-free `just ci` passes in `/tmp/riotbox-1518-ci.log`, explicitly
  warning/error-free: Rust/Python/contracts, synthetic audio gates, formatting/
  tracked JSON and strict all-target/all-feature Clippy. Native exact-head PR CI
  and normal merge/archive/cleanup remain separate gates.

## Review and limits

Solo sequential correctness, Rust, architecture and workflow review followed
by self-review, not an independent panel. Inspected the original parser and
actual main ordering, both source-window guards, frame/buffer construction,
one-line correction, fixture boundaries and CLI side effects. Demonstrated
P2 fixed and regression-proven; **no additional changed-diff finding**.
No warning suppression, dependency or tests removed.

This is finite-value validation, not comprehensive resource hardening: extremely
large finite durations and tiny positive durations rounding to zero frames
retain existing behavior. No allocation bound, general malformed-input safety,
device proof or comprehensive waveform qualification is claimed. A future
resource policy needs its own bounded contract, not a hidden cap in this fix.

Core/Session/action/replay, library DSP, thresholds, schemas, source-access and
frozen Stage-A contracts are untouched. Existing text includes are not migrated
inside this bug fix. All rendered audio is the helper's explicit synthetic QA
control; no real Development/Holdout/commercial audio, source-directory search,
device, DAW, playback or new/transferred human/musical verdict. Technical reruns
do not qualify product sources. RIOTBOX-1509 stays a separate open Windows
startup diagnosis; architecture cadence stays one after merged RIOTBOX-1517.

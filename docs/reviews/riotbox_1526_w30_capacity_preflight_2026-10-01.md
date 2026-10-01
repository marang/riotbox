# RIOTBOX-1526 — W-30 render and PCM16 capacity preflight

Date: 2026-10-01. Classification: maintenance/regression. Integrated baseline:
`e242fe596905636275e97704e3ad06b70ccfd0b3` (merged RIOTBOX-1525). Preparation
began on `522d8e19ccc5dd864b2248ac4caeefb428fbc402`; exact commit comparison
confirms the W-30 helper was unchanged between those baselines. Main was
fast-forwarded after verifying incoming paths did not overlap local changes.

## Defect and scope

The finite-duration parser fix in RIOTBOX-1518 deliberately left large finite
values open. `3.4028235e38` remains finite/positive but its existing `f32`
sample-rate multiplication reaches infinity and casts to `usize::MAX`. The
offline API's saturating channel multiplication cannot produce a legal
`Vec<f32>` layout. A source-free scalar probe imports the actual config and
mirrors the main/API expressions; Debug and optimized runs both produce
`Layout::array::<f32>(usize::MAX) == Err(LayoutError)`. The normal 0.05-second
control retains 2205 frames/4410 samples. Logs and probe:
`/tmp/riotbox-w30-capacity-probe.gDTGlL`.

This is no-allocation layout evidence plus complete parser/main/API code
inspection, not an executed huge renderer, source hydration or panic repro.
The PCM16 writer also checks its `u32` data/RIFF size, but only after rendering.
An otherwise representable buffer that cannot be written as PCM16 therefore
needs early format rejection, not an invented duration cap.

## Correction and ownership

`Args::render_frame_count()` retains the exact original `f32` multiplication,
rounding and cast, then calls a private mechanical capacity check. Main uses
that Result after help and before source hydration, renderer allocation or
writes. Interleaved sample count and `f32` bytes use checked multiplication;
the byte layout cannot exceed `isize::MAX`. PCM16 data and RIFF overhead must
fit their existing `u32` fields. Saturated counts fail this check; errors are
explicit, never clamps or panic-catching admission.

Preflight and writer share the existing physical PCM16 bytes-per-sample and
RIFF overhead through config constants. The writer's calculations and byte
format remain unchanged; normal hash proof covers that necessary small
extraction. No new product model, renderer or dependency. Input/capacity
ownership stays in the existing binary-private argument owner (315 lines
including its adjacent regressions); the CLI integration is 86 lines.

Help keeps its prior precedence over finite capacity and source hydration.
Tiny positive durations rounding to zero retain header-only output. A large
representable, writable buffer can still exhaust memory. The library offline
API itself is unchanged and is not claimed generally hardened by this CLI fix.

## Regression and exact output evidence

- A behavior-preserving extraction first made the existing unchecked frame
  acceptance testable. Against that unguarded seam, four pure rejection tests
  fail and one ordinary-rounding control passes in Debug and actual Release:
  `/tmp/riotbox-1526-red-{debug,release}.log`. These are not misrepresented as
  tests of a previously existing preflight function.
- All five pass after validation, before huge-control CLI tests are introduced:
  `/tmp/riotbox-1526-green-{debug,release}.log`. No large buffer is allocated.
  Pure cases cover saturated frames, sample/byte overflow, `isize` layout,
  exact PCM16 data and RIFF addition boundaries, ordinary counts and zero-frame
  compatibility. The exact RIFF boundary case is explicitly 64-bit-only; a
  32-bit Vec layout binds earlier. No 32-bit execution was performed.
- **15 unit tests plus four CLI integration tests** pass in Debug and actual
  Release, `/tmp/riotbox-1526-tests-{debug,release}.log`. Ten previous unit
  cases and RIOTBOX-1518's two integrations remain unchanged.
- New integration covers **20 guarded rejection paths**: five finite controls
  with/without a named nonexistent source and absent/existing output files.
  Each returns status one, empty success stdout and the exact buffer/format
  error; no source error, panic, new output directory/WAV/metrics or overwrite
  of prior sentinel bytes. A separate help integration returns the original
  help bytes despite huge finite duration and a nonexistent optional source.
- All **42 previous render/comparison CLI stdout/stderr/status cases** are
  byte-identical. At the same exact fresh paths under
  `/tmp/riotbox-1526-byteproof.ukWtH7`, all **24 hashes** remain identical:
  nine WAVs plus metrics, comparison reports and manifests. The source-backed
  preview control opens only the explicitly generated default synthetic WAV.
  No real source or source directory is accessed or discovered.
- Final targeted/build logs were explicitly scanned warning/error-free.
  Original red logs remain defect evidence, not green validation. Initial
  source-free `just ci` passed in `/tmp/riotbox-1526-ci.log` before the final
  conditional test-constant correction. Final targeted Debug/Release tests
  pass in `/tmp/riotbox-1526-tests-{debug,release}-final.log`, and the final
  source-free CI passes with verified exit zero in
  `/tmp/riotbox-1526-ci-final.log`. All three final logs were explicitly scanned
  warning/error-free. No production body changed after the CLI/hash proof.

## Review and remaining limits

Solo sequential correctness, Rust, compatibility, architecture and workflow
review traced the complete parser, actual main ordering, allocation consumer,
writer, shared physical constants and tests. A P3 Rust portability finding was
fixed: the RIFF test's error constant now has the same 64-bit condition as its
only consumer, avoiding a 32-bit unused-constant warning. That disposition is
verified by matching source conditions, not a local 32-bit build; the i686
standard-library directory is absent. The P2 is fixed locally and
regression-proven; no additional changed-diff finding survives. Self-review
checks exact scope, output identities, help/error ordering and coverage limits
with zero remaining findings.
Native exact-head CI, merge/main/archive/cleanup remain separate gates.

The repair enforces existing Vec and PCM16 representability requirements, not
a new domain algorithm or resource policy; no Decision Log entry is needed.
No Core/Session/action/replay, library DSP/runtime/API, musical threshold,
schema, source-access authorization or frozen Stage-A change. No structural
ownership migration. No real Development/Holdout/commercial audio, device,
DAW, playback, or new/transferred human, musical, source-general, hardness or
release qualification. Technical reruns grant no listening verdict.
RIOTBOX-1509 remains unresolved; structural cadence stays zero after the fully
closed RIOTBOX-1524 checkpoint.

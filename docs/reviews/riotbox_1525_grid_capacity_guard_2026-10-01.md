# RIOTBOX-1525 — Reject unrepresentable Feral render buffers

Date: 2026-10-01. Classification: maintenance/regression. Baseline:
`a8071df525ec43f20d6969cf705769c374ec0564`, including the merged RIOTBOX-1523
implementation. This fixes the pre-existing P2 demonstrated in the separate
RIOTBOX-1524 architecture checkpoint; it is not a structural migration.

## Defect and bounded correction

Finite positive `--bpm 1e-38 --bars 2` passes the existing parser and saturates
cumulative frame calculation to `usize::MAX`. Stereo W-30 and MC-202 consumers
then multiply by the channel count without checking, before Vec allocation.
The checkpoint reproduced only actual Args/Grid and scalar multiplication:
caught Debug overflow and optimized wrapping, not a huge renderer or allocation.

The existing Grid constructor now checks the configured interleaved sample
count and `f32` byte count with `checked_mul`, then rejects a byte layout above
`isize::MAX` through its existing Result. A saturated frame count fails this
same check. Existing frame rounding, constructor fields, beat-count overflow
check, BPM selection, source trust and normal rendering remain unchanged.
No new state owner, renderer, buffer allocation, dependency or musical threshold
is introduced. The owning source-timing spec records the mechanical boundary.

This is not a memory-budget policy: a representable but impractically large
grid still passes and may exhaust memory if rendered. Source loading/analysis
and directory creation still precede Grid construction; rejection prevents
lane rendering and artifact publication, not every preliminary side effect.
No power-loss, transaction or general hostile-input safety claim follows.

## Regression and output evidence

Six pure Grid regressions are added adjacent to the owner, without allocating
their large frame counts. Four reject saturated frames, sample multiplication
overflow, byte multiplication overflow and representable `usize` byte counts
above `isize::MAX`. Two controls retain large-but-representable admission,
ordinary cumulative rounding/bar boundaries and existing invalid-BPM, zero
meter/bar and beat-overflow errors. Boundary controls derive from architecture
size, not a new hardcoded BPM ceiling.

Both original Debug and actual Release runs fail the four rejection tests and
pass the two controls. Logs: `/tmp/riotbox-1525-red-{debug,release}.log`.
After the guard, all six pass in both profiles before invalid-input CLI tests
are introduced: `/tmp/riotbox-1525-green-{debug,release}.log`.

The complete **52 unit tests and two CLI integration tests** pass in Debug and
actual Release, `/tmp/riotbox-1525-tests-{debug,release}.log`. The new integration
uses only its freshly generated 0.1-second synthetic WAV and performs eight
guarded invocations: four unrepresentable controls with absent and existing
artifacts. Every invocation returns status one, empty success stdout and the
capacity error, without panic, new WAV/README/manifest publication or changing
prior manifest/artifact sentinel bytes. Existing empty-directory creation is
not misrepresented as no side effect. The other integration preserves literal
verification argv for the normal auto/explicit render paths.

All **31 previous CLI stdout/stderr/status cases** remain byte-identical.
Technical reruns at the exact named generated control paths under
`/tmp/riotbox-1523-byteproof.uYJSWo` preserve every **39 hashes**, including 16
output WAVs, the generated four-second input, reports, README and both safely
quoted verification manifests. No tolerances or path normalization are used.
These are technical QA controls, not real-source or human listening evidence.
Final targeted test/build logs were explicitly scanned warning/error-free;
initial red logs remain defect evidence and are excluded from that claim.
Full source-free `just ci` passes with verified exit status zero in
`/tmp/riotbox-1525-ci.log`, explicitly scanned warning/error-free. Native
exact-head checks remain a separate merge gate.

## Review and limits

Solo sequential correctness, Rust, compatibility, architecture and workflow
review inspected the full constructor, all Grid construction sites, actual
allocation consumers, pack ordering and regression bodies. Saturating frame
conversion is not used to accept a wrong representable Grid: its saturated
value necessarily fails the stereo byte-layout check. No extra production
struct literal or frame mutation bypasses the constructor in this binary.
The fix enforces an existing Vec representability requirement; it does not
choose a new domain algorithm or resource policy, so no Decision Log entry is
needed for this reversible arithmetic validation repair.

The confirmed P2 is fixed locally and regression-proven. No additional
changed-diff finding survives this review; short self-review checks exact
scope, documentation and fixture boundaries. Native exact-head CI, merge,
main synchronization, archive and cleanup remain separate completion gates.

No Core/Session/action/replay, library runtime/DSP/API, timing algorithm or
threshold, JSON schema, frozen Stage-A contract or textual-include ownership
change. No real Development/Holdout/commercial audio, source-directory search,
device, DAW or playback was accessed. No new or transferred human, musical,
source-general, hardness, live-device or release verdict. RIOTBOX-1509 remains
open. This behavior fix does not advance the structural ownership cadence.

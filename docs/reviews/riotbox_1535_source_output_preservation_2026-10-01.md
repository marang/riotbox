# RIOTBOX-1535 — Preserve input sources across Feral-grid output publication

Date: 2026-10-01
Baseline: `50befb256ebcee56eb2f7492b4de9774d6debd25`
Integrated predecessor archive: `4ba79c767d99f362b20375374bac5b2fbf9be3df`
Decision: RBX-405
Classification: pre-existing offline QA integrity defect; intentional failure-path change

## Finding and causal proof

The actual CLI accepts a source path that is also its first output WAV.
On the reviewed predecessor tree, two synthetic-only runs report success
while the input changes from 705644 to 659500 bytes and its SHA-256 changes.
A separate-output run and a help-only run preserve the regenerated source.
This isolates output replacement rather than decoding, source analysis or the
test wrapper. No real source is sacrificed to reproduce the failure.

The first actual CLI integration regression fails on changed input bytes,
then passes after the correction. The original feedback-loop script
`/tmp/riotbox-source-collision.YUZ0ec/repro.sh` now reports exit 1, an explicit
`output aliases input source` error and unchanged 705644-byte source.
Its generated input is exactly
`/tmp/riotbox-source-collision.YUZ0ec/stems/01_tr909_beat_fill.wav`.

## Implementation and ownership

The binary-private ordinary `output_paths` owner holds a typed plan for
eight WAVs, eight metrics files and report/manifest/README. Actual WAV and
metadata writes consume those named paths. The existing metrics filename
function moves unchanged to that owner; both the writer and manifest
projection use it. No independent safety-only output catalog.

After existing loading/format validation and before analysis/rendering or
the first artifact write, read-only physical identity handles compare the
input against every existing planned destination. `same-file` 1.0.6 was
already locked transitively; the Audio manifest now declares its direct use,
without a second version, unsafe custom syscall or realtime dependency use.
Both compared handles remain alive through the equality check.

Only an absent directory entry skips identity work. Existing dangling links,
non-regular destinations and every metadata/identity/open failure reject.
Same path, hardlink, symlink, source alias and directory alias are protected.
Existing output files must be readable/identifiable. Unsupported identity
backends fail closed. Equal-content distinct files are not aliases; unrelated
input inside output remains allowed.

The new production owner is 96 lines and CLI regression owner 195 lines.
The cohesive plan/preflight boundary is required by this bug fix, not an
unrelated migration, framework or shadow source/persistence model.

## Regression and compatibility evidence

All six new actual CLI regressions pass in Debug and Release:

- Independent literal catalog: each of 19 direct output names can contain
  valid WAV input, even metrics/metadata suffixes; every prior artifact is intact.
- Hardlinks for WAV, metrics and late README destinations.
- Unix output symlinks, an input symlink and an output directory symlink.
- Dangling late-output link rejects before replacing prior audio or creating
  its missing target.
- Late README collision with otherwise absent outputs publishes no earlier WAV.
- Distinct equal-content output can be replaced normally while the independent
  source inside output remains unchanged.

The existing 52 binary unit cases and two capacity/verification CLI
integrations pass in both profiles. The independent final six-case runs
include the additional absent-output regression. Test byte assertions avoid
dumping raw PCM on failure; the original verbose red log is diagnostic only.

Fresh standalone build precedes normal-path parity. All 31 existing CLI
status/stdout/stderr records and all 39 SHA-256 values remain identical:
38 output files including 16 WAVs across auto/explicit packs, plus input.
Both listening manifests validate their actual artifact paths. Exact bounded
input: `/tmp/riotbox-1535-byteproof.9gGOqt/synthetic-control.wav`.
Values are not tuned to obtain parity.

## Solo review and boundaries

Code-review and Rust lenses cover the complete changed functions, every write
consumer, error propagation, visibility, dependency/platform behavior,
regression sensitivity, spec and workflow evidence, then short self-review.
No remaining actionable finding in this diff; no independent reviewer panel.
An obsolete production PathBuf import is removed rather than suppressing its
warning. Initial warning-bearing test/build logs are diagnostic, not final proof.

This is a stable-namespace offline preflight, not adversarial concurrent
namespace locking, atomic whole-pack publication or power-loss durability.
Directory creation retains its original position and may precede rejection.
Windows identity source is inspected; local alias regressions run on Linux.
The native Windows Sidecar gate does not prove Windows Audio/filesystem behavior.

No Core/Session/replay/App/runtime/public library API, DSP, threshold, musical
policy, source timing or frozen Stage-A contract changes. No source diversity,
product intelligence, hardness, human taste, live-device, DAW or release claim.
Only fresh generated synthetic controls and existing in-memory fixtures;
no Development/Holdout/commercial audio, source-directory search or playback.
Necessary source protection takes priority over optional namespace cleanup.

## Validation and remaining gates

- Red feedback: `/tmp/riotbox-1535-source-safety-red.log`
- Final build: `/tmp/riotbox-1535-build-reviewed.log`
- Existing and initial guard families: `/tmp/riotbox-1535-focused-{debug,release}.log`
- Final six guard cases: `/tmp/riotbox-1535-guard-{debug,release}-reviewed.log`
- Rustfmt, diff/include guards, RBX-405 targeted readback and both manifests pass.
- Full source-free `just ci`: actual exit zero, warning/error-free
  `/tmp/riotbox-1535-ci-final.log`. The initial five guard cases run there;
  the additional sixth case passes the separate final Debug/Release runs.
- Exact-head native PR gates, merge, archive and cleanup remain required.

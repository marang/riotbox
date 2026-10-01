# RIOTBOX-1536 — W-30 source preservation and one offline identity owner

Date: 2026-10-01
Baseline after normal predecessor merge: `6d6de8d3efa32246b8c6b21156a2588e09958d93`
Integrated predecessor archive: `5ad7c1db7f557d06c7dde8b2ad255f3d011a1d43`
Decision: RBX-406
Classification: pre-existing W-30 offline QA source-loss correction

## Causal evidence

Two actual pre-fix W-30 CLI invocations report success while a fresh generated
source changes from 705644 to 17684 bytes and its SHA-256 changes. Separate
output preserves the regenerated input. The first public CLI regression is
red on changed source bytes, not a private-helper expectation.

The initial branch starts from clean main before the unrelated Feral PR is
merged; only the independent red W-30 test is written there. After #1614's
actual native-gated merge, its entire candidate tree equals main. Its update is
fast-forwarded without discarding the untracked W-30 test. No stacked feature
implementation or Linear backfill.

## Ownership and deliberate behavior change

One ordinary `qa_source_safety` binary-only module owns the existing read-only
physical-identity/error loop. The loop's full token stream, including literals,
remains identical to the extracted predecessor loop; inspection also confirms
source/output handle lifetimes, absence-only skip, nonregular rejection and
filesystem errors. Feral keeps its typed 19-destination plan and all six
complete CLI regression bodies/names. Both ordinary roots import the same
physical source file. Cargo metadata confirms no new executable target, public
library API, dependency or version.

W-30 sends its actual WAV and sibling metrics path to this helper when input is
explicit. Existing duration/capacity validation and source loading remain first;
preflight then precedes rendering and both writes. W-30's metrics naming
function is unchanged. Explicit no-source synthetic QA mode remains unchanged:
it is a diagnostic control, not missing-source musical fallback. No generalized
pack abstraction, duplicate destination list or parallel source-identity truth.

The shared production owner is 38 lines; W-30 CLI regression owner is 171.
This necessary deepening avoids two safety implementations while preserving
renderer layout ownership. It does not migrate unrelated textual includes.

## Actual CLI and regression proof

Six new W-30 CLI cases cover direct source at WAV/metrics names, hardlinks at
either destination, Unix output/source/directory symlinks, dangling late metrics
and nonregular late metrics rejection before prior WAV replacement, plus
distinct equal-content output with independent input inside output.
Input bytes and prior artifacts remain intact on rejection.

All 85 focused names/statuses pass in Debug and Release: 67 existing binary
unit tests, six capacity/duration/verification integration cases, six preserved
Feral safety cases and six new W-30 safety cases. No test is removed or ignored.

Fresh standalone builds precede compatibility comparison:

- W-30: 32 exact CLI status/stdout/stderr records and nine hashes (four WAVs,
  four metrics files and input), including explicit no-source modes.
- Feral: 31 exact CLI records and 39 hashes (38 outputs including 16 WAVs,
  reports/README/manifests and input); both manifests validate actual paths.

The sole normal-parity input is exactly this fresh four-second generated WAV:
`/tmp/riotbox-1536-byteproof.Pq7VhX/synthetic-control.wav`.
CLI fixtures use their own bounded generated controls. No values are tuned.

The original feedback script initially still targets the unchanged secondary
worktree binary and stays red; this is an excluded old-build diagnostic, not
new artifact evidence or reason to alter production code. With an explicit
fresh root binary argument it now returns exit 1 and collision error while
the original 705644-byte source remains unchanged:
`bash /tmp/riotbox-w30-source-collision.r9po8K/repro.sh collision /home/markus/Dev/riotbox/target/debug/w30_preview_render`.
Its exact synthetic input is `/tmp/riotbox-w30-source-collision.r9po8K/control.wav`.

## Solo review and limits

Sequential code-review/Rust correctness, complete callers/error paths,
layout/identity separation, lifetimes/visibility, Cargo/platform behavior,
regression sensitivity, spec/workflow and soft module-size lenses, then short
self-review: no remaining actionable finding in this diff, no independent panel.
Existing destination readability and stable-namespace limitations remain
explicit. The shared owner adds no callback I/O or runtime authority.

No concurrent namespace lock, atomic pack/power-loss guarantee, Windows Audio/
filesystem matrix, real device/DAW or musical/source/hardness/release claim.
No Core/Session/replay/App/runtime/schema/DSP/threshold/limiter/Stage-A changes.
No real Development/Holdout/commercial audio, source-directory search, playback
or subagents. Codebase-design guides the shared I/O boundary; Diagnosis/TDD
require actual CLI feedback, while project skills retain honest audio scope.

## Validation and remaining gates

- Red: `/tmp/riotbox-1536-source-safety-red.log`
- Green original: `/tmp/riotbox-1536-source-safety-green.log`
- Final build: `/tmp/riotbox-1536-build-final-reviewed.log`
- Final tests: `/tmp/riotbox-1536-focused-{debug,release}.log`
- Full source-free CI: `/tmp/riotbox-1536-ci-final.log`, actual exit zero
  and explicit final warning/error scan clear.
- Targeted RBX-406 readback, both manifests, Cargo metadata, Rustfmt,
  diff/include gates and focused final warning/error scan pass.
- Exact-head native PR gates, merge, archive and cleanup remain required.

# RIOTBOX-1551 — bounded recording windows

Date: 2026-10-02
Issue: RIOTBOX-1551, child of RIOTBOX-1036 / P016
Branch: `feature/riotbox-1551-bounded-recording-windows`
Base: `92f4f9b954f405e4949b83a230b6eb93a66dfac6`
Decision: RBX-422

## Outcome and scope

The existing non-interactive master recorder gains explicit eight/sixteen-bar
V3 windows. Omission or `2` retains two-bar V2. This is an engineering extension
of the accepted P023 recording path, not a new musical mechanism or fresh
instrument qualification. It removes RIOTBOX-1036's fixed two-bar operator limit;
the umbrella remains open for the broader export workflow and separately
authorized real-host extended-window evidence.

Use `--live-master-recording-execute --live-recording-bars 8` with the existing
Session/destination arguments, or `just live-master-recording SESSION DESTINATION
"" "" 8`. The trailing Just parameter preserves old graph/observer positions.
The current Scene remains fixed; this is not an interactive performance
recorder, input recorder, DAW host, new TUI control or arbitrary-duration take.

## Contract and acceptance mapping

- Core owns `LiveRecordingDuration`, the optional Action/receipt identities,
  canonical runtime-f32 frame geometry and the bounded V3 position-arithmetic
  contract. V1/V2 omit new fields; V3 admits only eight/sixteen bars.
- App selects the version before queueing, binds the planned duration to the
  pending Action, publishes the exact float32 WAV/proof through the existing
  no-clobber transaction and validates full receipt readiness before commit.
  Failed Session save restores state and removes only hash-owned outputs.
- Indexed Core validation cross-checks explicit-duration Action/receipt pairs
  at App restore and Core replay entrypoints, including snapshot hydration.
  It neither opens audio nor makes external recording replayable.
- CLI rejects unsupported, duplicate, orphan or missing duration arguments.
  Launch/result/receipt observers and read-only reports expose actual duration.
  Existing UI changes are exhaustive boundary labels only, not a new control.
- The callback, DSP, limiter and allocation ceiling are unchanged. Tests use
  generated PCM, complete-frame partition checks and the actual production
  transport-clock function. V2-only DAW admission rejects V3 receipts and typed
  V3 duration injected into a hash-updated V2 proof.
- No real source/capture/Holdout/commercial audio, source-directory discovery,
  host output, human playback or DAW import was used. Generated test WAVs are
  controls only. No release, hardness, source-general or musical-quality claim.

## Review findings and disposition

1. **P2 — inconsistent timing authority / arithmetic order. Fixed.**
   Independently found by Adversarial Implementation Reviewer / API consumer
   compatibility and Spec and Evidence Auditor; the Core implementation lane
   independently reproduced the first case. Integer micro-BPM planning and
   original-f32 callback geometry could select different frames: at 44.1 kHz,
   eight bars, `120.1_f32`, a perfect capture exceeded the half-frame check.
   A follow-up found different floating division order rounded a 121.5 BPM,
   48 kHz frame span to 42187 versus 42188 nanobeats. V3 now reconstructs and
   checks the exact runtime f32 before allocation and uses identical arithmetic
   order. V2 formulas remain frozen. Generated Core sweeps and App publication
   cases cover 80.0082, 90.049, 120.1 and 121.5 BPM, plus unrepresentable-tempo
   rejection without allocation/publication.
2. **P2 — actual accumulated clock rejected a clean sixteen-bar take. Fixed.**
   Found by Adversarial Implementation Reviewer / Performance and operations.
   Actual `advance_transport_timing` plus `SharedLiveMasterCapture`, at
   96 kHz / 137.25 BPM / 128-frame callbacks, produced exactly 2,685,902 frames
   and zero faults but an endpoint residual of `1.267925e-6` frames, exceeding
   the old `1e-6`-frame comparison. The preserved RED log is
   `/tmp/riotbox-1551-runtime-clock-red.log`. RBX-422 explicitly defines a
   V3-only bounded representation guard, capped at both 1/1024 frame and one
   microbeat; nominal frame rounding retains the half-frame physical gate.
   Actual endpoints stay in evidence, V2 stays unchanged and runtime code is
   untouched. Production-clock 64/128/1024/varied-partition regressions turn
   green; App accepts the reproduced values but rejects a quarter-frame
   displaced end without publishing files.

No findings were deferred or rejected. Both independent follow-up reviews
retain zero open findings: Spec and Evidence Auditor, and Adversarial/API/
Performance lenses. Each reran the Core/App/actual-clock checks relevant to
its findings. Coordinator compatibility/maintainer, product-pragmatist and
risk-assessor self-review also retains zero additional findings; the deliberate
boundary is engineering implementation without new host/human qualification.

## Verification

- Core recording contracts: 25 passed, including canonical tempo, 20,000
  generated geometry combinations, serialization, identity mutation, snapshot/
  replay rejection and roundoff boundary cases.
- Core replay family: 121 passed before the final isolated roundoff helper;
  full final CI re-exercises it.
- CLI library tests: 144 passed; Just dry-runs preserve existing argument order.
- App V3 publication/restore/rollback/precision family: six passed, including
  exact WAV/proof identity and V2 DAW refusal.
- Audio capture family: 18 passed; actual-clock RED-to-GREEN and strict Audio
  library/tests Clippy passed. Final source-free `just ci` passed; full log:
  `/tmp/riotbox-1551-ci-final.log`. This includes workspace tests, format,
  strict all-target/all-feature Clippy, generated audio smokes and contract
  fixtures; it does not supply host-device or human-listening evidence.
- Initial full CI crossed an intermediate missing-helper compilation while the
  reproduced clock issue was being fixed; that run is superseded, not reported
  as final proof. No frozen gate or failing assertion was removed.

The new Core Session contract and new test families have explicit semantic
owners. Existing Action vocabulary and artifact transaction/codec modules keep
their established public ownership; no line-count-only split, textual include,
shadow recorder, Session model or new dependency was introduced. No persistent
state was added to `JamAppState` and no new `ActionCommand` was needed.

## Remaining evidence boundary

### Current-state cadence checkpoint

The post-RIOTBOX-1549 sequence added limiter-calibration access/runner contracts
and recording duration identity, so this closeout also samples current-state
architecture with `review-codebase`, separately from the branch review.
Coverage: workspace/four crate manifests; Audio runtime/calibration/parity/public
shell/transport/capture; App calibration admission/preparation, recording CLI,
artifact transaction, shared export commit, restore and V2 DAW admission; the
calibration runner's resource/admission controls and generated regression owners.
The Core implementation author audited these cross-subsystem consumers, not
their own new Core code as independent evidence; the other reviewers cover it.

No retained P0–P3 finding. Dependency direction remains App to Audio/Core/Sidecar
and Audio/Sidecar to Core. Default-off calibration feeds ordinary admitted-byte
preparation and existing offline RuntimeMix, not production fallback or a second
Session/recorder. The production callback remains fixed limiter, atomic capture,
device conversion; runtime stop precedes file publication. No mandatory refactor
was found. Relocating the existing six-consumer export-commit helper out of the
stem-named module is an optional locality improvement, not a functional defect
or reason to add another transaction abstraction. This is sampled coverage,
not an exhaustive repository audit or filesystem-race/device qualification.

### Follow-up authorization

All recorded pass counts are source-free engineering evidence. A new bounded
real-host eight/sixteen-bar take needs its own exact authorization and access
contract; any human playback additionally needs artifact preflight, purpose,
fresh readiness and bounded stop/silence verification. The existing qualified
two-bar recording and its human judgment do not transfer automatically.

Keep RIOTBOX-1551 in recent Linear history after merge; archive its final context
before any later deletion. PR/native CI/merge and main synchronization remain
mandatory closeout gates, not implied by this report.

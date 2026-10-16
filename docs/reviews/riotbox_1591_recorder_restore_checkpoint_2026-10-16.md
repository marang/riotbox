# Integrated recorder and restore checkpoint — RIOTBOX-1591

Date: 2026-10-16. Reviewed main: `b42d99a84f8c4490d136a8cb905bf52e0c34a236`,
tree `6b8c0724e1ddc6000e1aa4cc698aa5e6769a55bf`. Read-only investigation
started on the RIOTBOX-1590 candidate `51d341407ce3a40fd73c68d4e976e26db7e604c6`;
the coordinator verified the merged main tree is identical before creating this
report branch. This is risk-directed sampling of Core history/replay, App
recording/restore, Audio transport/capture and their generated QA boundaries,
not an exhaustive repository, DSP, device or Sidecar audit. No production code
or test is changed by this report.

## Findings and dispositions

### [P2] Delayed observed transport feedback becomes an unintended seek

Location: `crates/riotbox-audio/src/runtime/shared_w30_resample_callback.rs:494–503`;
App consumer `crates/riotbox-app/src/cli/event_loop.rs:49–67` and the recorder's
`sync_live_master_runtime()` in `cli/live_master_recording/capture_lifecycle.rs`.

Reviewers: Adversarial Implementation Reviewer, correctness/data-contract lens.

The App reads completed callback position, processes that observation, then
publishes its resulting position back to Audio control. Meanwhile callbacks can
continue. Audio treats a changed control position more than `0.125` beats from
its current position as a seek, although this update contains no new performer
position-change intent. A delayed ordinary observation can therefore rewind
playback. Coherent publication alone cannot distinguish observation from intent.

A generated in-memory ramp drives the production Audio library's public exact
RuntimeMix sequence seam. It observes beat `0.2`, permits callbacks to advance
to `0.4`, then publishes that observation without a seek, tempo change or toggle.
Expected final sample is `0.018478166`; feedback produces `0.009678166`.
No-delay and 50-ms-delay controls pass; the 100-ms-delay case fails deterministically.
This establishes a valid synthetic interleaving and output consequence, not its
frequency, complete App scheduling behavior or a historical host incident.

The coordinator inspected the source and real App feedback chain, rebuilt Audio
with repository-locked offline dependencies, independently compiled the diagnostic
and reran all three controls. Source `/tmp/riotbox_1591_observed_feedback_rewind.rs`,
SHA256 `1f004df279bef52972c65f66cb90a605dd9cb95b18840b9b623f234326f1e264`;
Audio rlib SHA256 `881f9c555bc15d004a0dcdd02639a90aeaeece40148d83a0ba25fbd58acadf32`.
Logs: `/tmp/riotbox-1591-root-audio-build.log`,
`/tmp/riotbox-1591-root-feedback-build.log` and
`/tmp/riotbox-1591-root-feedback-{0,2400,4800}.log`.

Disposition: **repair next in
[RIOTBOX-1592](https://linear.app/riotbox/issue/RIOTBOX-1592/separate-observed-transport-progress-from-explicit-seek)**,
not waived. Separate observed progress from explicit position-change intent
through existing product/runtime owners. Preserve intentional forward/backward
and repeated seeks, transport transitions, tempo, restore, queue boundaries and
realtime constraints. Increasing the tolerance merely moves the failure boundary.

### [P2] Backward navigation reorders durable actions during replay

Location: `crates/riotbox-core/src/replay/plan.rs:169,455–469`;
ordinary navigation producer `jam_app/source_map_navigation.rs:82` and App
restore entrypoint `jam_app/restore_replay.rs:31`.

Reviewers: Spec And Evidence Auditor, Adversarial replay/data-integrity lens.

The replay builder sorts by musical beat before the remaining commit fields.
After backward navigation, transport coordinates no longer represent chronological
commit order. This violates [Replay Model §7.2](../specs/replay_model_spec.md#72-action-ordering):
durable committed order must remain authoritative.

A reference-free Session uses ordinary App queue/commit APIs: Source Monitor
Blend at beat 12 (action 1), TransportSeek(0) committed at beat 12 (action 2),
then Source Monitor Source at beat 0 (action 3). Durable commit records are
`[(1,12,1), (2,12,2), (3,0,1)]`, but replay order becomes `[3,1,2]`.
Live state is Source; replay yields Blend. Public save/load accepts and preserves
the history, then `apply_restore_target_suffix(3)` also changes Source to Blend.
A second check uses the normal public PreviousBar navigator at beat 12→8 with
an in-memory metadata-only Source Graph: the same order/state mismatch occurs
without a custom seek draft. No named audio path is hydrated and no runtime starts.

The coordinator inspected both checks, independently built App/Core with the
repository lock, compiled a separate executable and reran it. Exit 0 means its
assertions demonstrate the defect, not correctness or on-disk save loss.
Copied source `/tmp/riotbox-1591-root-replay-repro.rs`, SHA256
`d51b79238c9936022e0e23cdb049d23bef8fe997ad36f527f056fa632b97fe44`;
logs `/tmp/riotbox-1591-root-app-build-fixed.log` and
`/tmp/riotbox-1591-root-replay-{build,run}.log`. An initial duplicate `--lib`
command-line argument failed before compilation and is not test evidence.

Disposition: **repair separately in
[RIOTBOX-1593](https://linear.app/riotbox/issue/RIOTBOX-1593/preserve-durable-replay-order-across-backward-transport)**,
not waived. Audit committed/target/snapshot suffix ordering, structural Undo,
same-boundary sequence validation and capture/export reconstruction together.
Neither request-time nor ActionId sorting automatically establishes commit
chronology. Preserve one Core replay authority and explicitly decide compatibility;
do not silently renumber or reinterpret old histories.

No additional P0–P3 finding survives the completed sampled Audio or coordinator
QA checks. The Core/App auditor reached its usage quota after delivering the
replay diagnostic and reachability evidence, before final review synthesis.
Its unfinished broader assessment is **not** counted as an independent zero-findings
verdict. The coordinator verified the retained defect and inspected the relevant
producer/restore chain; coverage remains explicitly bounded.

## System model and inspected boundaries

- Core owns durable actions, commit boundaries/sequences, Session, replay and
  capture/export identity. Runtime beat coordinates can move backwards; a
  boundary-local sequence is not a global chronology by itself.
- App consumes observed timing, commits ready actions and projects state back
  to Audio. Recorder supervision owns stop/abort/finalize/publication ordering;
  a capacity refusal must not skip device shutdown or publish success.
- Audio separates coherent control publication from observed callback timing.
  Its unique readers borrow prepared payloads; control retains reclamation.
  Capture admission closes before quiescence/finalization. RBX-452's callback
  predecessor fixes preparation accounting, not feedback/seek intent.
- The loaded QA workload admits exact generated fixture/run copies, correlates
  marker/stdout/Session/action/receipt/proof/WAV and calls Core readiness.
  A milestone alone never grants success. Typed group/stream cleanup precedes
  artifact analysis; clocks, route identity and first-failure no-retry retain
  their existing owners. No real host or prior ignored attempt was inspected.

## Newly executed source-free checks

Existing passing tests do not disprove the two uncovered counterexamples.
Coordinator selections:

| Focus | Passed | Log under `/tmp/` |
| --- | ---: | --- |
| Core queue/capacity | 26 | `riotbox-1591-core-queue.log` |
| Core replay planning/cursors | 44 | `riotbox-1591-core-replay-plan.log` |
| App history exhaustion | 34 | `riotbox-1591-app-history-exhaustion.log` |
| Shared recording lifecycle | 29 | `riotbox-1591-app-recording-lifecycle.log` |
| QA cross-owner/containment controls | 16 | `riotbox-1591-qa-cross-owner.log` |

The replay selection retains one existing ignored benchmark; nonmatching Cargo
targets with zero tests are not added to counts. The 16 QA selections exercise
Core-authoritative readiness, prelaunch run-copy admission, no-retry failure,
premature/overlapping absence, bounded markers, unsafe routes, typed cleanup,
raw proof hashing, action/receipt/history correlation, source-redirection refusal
and generated child supervision. They do not execute audio or query audio services.

The Audio reviewer independently ran 85 passing selections: capture 33,
production callback/heap 8, runtime lifecycle 10, in-memory Source Monitor 17,
source publication 6 and actual-source Loom models 11. Coordinator inspected
their result logs, not reran those 85 checks. Logs:
`/tmp/riotbox-1591-{capture-lifecycle-memory-tests,output-callback-tests,lifecycle-tests,source-monitor-memory-tests,source-publication-tests,runtime-models}.log`.
No failures/ignored tests in those selections. Counts overlap previous suites
and are not unique repository-wide coverage. The two new diagnostics and their
controls are separate from these passing regression selections.

## Cadence, limits and next priority

RIOTBOX-1583 was the preceding current-state checkpoint. Subsequent substantive
repairs cover control coherence (1584), ActionId admission (1587), commit capacity
(1588) and capture-gap continuity (1590). The bounded 1589 observation supplies
an integration checkpoint, not a fifth feature or an endurance pass. Operational
and report-only branches do not inflate the five-feature cadence.

Repair 1592 first because ordinary active callback feedback is affected, then
1593's durable restore ordering. Keep both production repairs outside this
report-only slice. No generic redesign or immediate extra host attempt follows.
RIOTBOX-1590's exact candidate local CI and native run38930065517 attempt1 passed
both configured jobs; that is supporting inherited evidence. This report branch's
own local/native CI, independent report review and closeout are recorded as they
finish in Linear and the project journal, not inferred from the earlier run.

Excluded: actual host/device shutdown, physical timing, complete App scheduling,
Sidecar cancellation/process-tree stalls, hostile concurrent writers/power-loss,
unsampled DSP/UI/source behavior, endurance, latency, DAW and human taste.
These are verification limits, not demonstrated additional defects. No actual
host/audio invocation or metadata, real source/capture/Holdout/commercial audio,
old ignored attempts, playback or DAW access occurred. Frozen contracts and
consumed RIOTBOX-1589 evidence remain unchanged. RIOTBOX-1041/1036 and the original ALSA
cause remain open; no musical, source-general, hardness or release claim is made.

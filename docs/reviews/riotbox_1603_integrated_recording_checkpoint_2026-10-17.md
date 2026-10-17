# Integrated replay, transport and recording checkpoint — RIOTBOX-1603

Date: 2026-10-17. Reviewed main:
`dd89ec9ba4250312ad2f5265ca240c6240cc0909`, tree
`addd3c3f54d91db7742d65c15b6655a03fcecfa0`. Origin was fetched and its main
identity checked before report publication. The implementation checkout remained
clean throughout inspection and witness execution.

This current-state checkpoint follows five substantive branches since1596:
1597,1599,1598,1601,1602. Mechanical dependency maintenance1600 does not advance
the cadence. Coverage is the integrated replay/recall and bounded recording path,
not an exhaustive repository, DSP, device, filesystem or Sidecar audit. This
branch changes only this report; no production or test code is shipped here.

## Findings and dispositions

### [P1] Stateful articulation can bypass the bounded W-30 stop envelope

Location: `crates/riotbox-audio/src/runtime/render_tr909_w30_preview.rs:264–288`.
Originating reviewer: independent Rust/Audio Correctness. Coordinator inspected
the full render/callback interaction and reran the actual-callback witness.

`stop_gain` attenuates input before PitchDive history resampling and FilterSlam.
PitchDive can subsequently emit older nonzero history after the gain reaches
zero. The exhausted-stop latch clears history only at the next callback ingress.
Audio Core §7.2 requires every active lane silent within5ms of running→stopped,
not merely by the following callback.

Executed counterexample: generated looping pad, mono1000Hz/120BPM, PitchDive
active at beat9.25, then Pause/Preserve and a50-frame stopped block. The dry
control has0 nonzero samples after frame5; PitchDive has45/45, including final
sample0.04258656. The next callback is silent. Material identity, stationary
acknowledged transport and zero callback heap checks pass. This is output from
the exact prepared processor used by CPAL, not a physical-device latency or
human-audibility measurement. FilterSlam shares the ordering exposure by
inspection; its failure was not separately executed in this checkpoint.

Disposition: first repair in
[RIOTBOX-1604](https://linear.app/riotbox/issue/RIOTBOX-1604/bound-stopped-w-30-final-output-after-stateful-articulation).
Enforce the existing fade on final per-channel output after stateful processing,
without double-fading the dry path or retuning frozen gestures. Preserve running
bytes, stopped retrigger/resume and historical compatibility; prove articulated,
dry, mono/stereo and zeroheap controls at the actual callback seam.

### [P1] Preset writes are absent from typed Undo conflict domains

Locations: Core `src/action.rs:755–766`, `src/style.rs:130–131`; consumers
`src/replay/plan/undo_validation.rs` and App `jam_app/commit/typed_undo.rs`.
Originating reviewer: independent Spec And Evidence Auditor, Data And Contracts.
Coordinator inspected the shared predicate and executed the public-API witness.

`PresetActivate` overwrites monitor mode and the macro state, including MC-202
touch, but the shared conflict predicate declares neither write. An older typed
snapshot therefore remains eligible across that newer non-undoable writer.
Action Lexicon typed-Undo rules, Session typed-Undo domains and Replay chronology
require newer overlapping mutations to be conflict barriers.

Executed ordinary public sequence: monitor Blend → FeralBreakAlphaV2 preset →
Undo. Undo succeeds and restores live monitor Source. Core accepts the marker
history; replay omits the undone monitor action, reapplies the preset and ends
at Riotbox. The same monitor Undo without a newer preset has live/replay parity.
This is not a request to make presets undoable. MC-202 touch overlap is established
by the writer/predicate, not a second executed musical/source counterexample.

Disposition: repair after1604 in
[RIOTBOX-1605](https://linear.app/riotbox/issue/RIOTBOX-1605/treat-preset-writes-as-typed-undo-conflict-barriers).
Cover actual preset writes in SourceMonitorMode/Mc202Lane using the shared Core
predicate. Do not add a false Tr909FillWindow conflict: presets do not write
`last_fill_bar`. Require public live/Core replay parity controls and an explicit
compatibility decision for previously persisted cross-preset markers; do not
silently rewrite history or upgrade replay policy.

### [P2] Silent scratch overflow can leave drums behind the transport clock

Locations: Audio `src/runtime/shared_transport_tr909/output_callback.rs:118–132`
and `src/runtime/render_tr909_w30_preview.rs:79`.
Originating reviewer: independent Performance And Operations. Coordinator traced
clock admission/overflow/render and independently reran the callback witness.

Overflow advances and acknowledges the shared clock but invalidates only W-30
phase. TR-909 retains its unrendered cursor; its ordinary≤0.125beat tolerance
can ignore a short skipped interval. Following equal-sized blocks advance both
cursors equally, retaining lag until a separate realignment event. Audio Core
§7 makes the shared transport the playback-timing authority.

Executed counterexample: isolated Takeover drums, mono1000Hz/120BPM, prepared
50-frame scratch, warm to0.4beats, then51 silent overflow frames. Timing reaches
0.502, and recovery completes at0.602. Nevertheless, all50 recovered samples
equal a no-overflow control's old0.4→0.5 interval, delaying the authoritative
half-beat trigger. Overflow count1 and zeroheap checks pass; every other mixer
contributor is idle. Fill has analogous tolerance-based exposure by inspection,
not a separately executed failure here.

Disposition:
[RIOTBOX-1606](https://linear.app/riotbox/issue/RIOTBOX-1606/realign-tr-909-rendered-phase-after-silent-scratch-overflow).
Invalidate rendered drum synchronization on overflow and consume the next
renderable callback cursor without the numeric deadband. Preserve clock/ack
identity, ordinary continuity, voice semantics, recording fault refusal and
frozen offline behavior. Prove short/repeated overflow and Takeover/Fill at the
actual callback; do not create a second clock or silently retrigger all lanes.

### [P2] Snapshot hydration accepts detectably after-cursor runtime provenance

Location: Core `src/replay/target_execution.rs:105–106`; upstream App
`jam_app/persistence/history_validation.rs` checks payload headers only.
Originating reviewer: independent Spec And Evidence Auditor, Data And Contracts.
Coordinator inspected the owning boundary and executed the public App witness.

Hydration checks payload identity/cursor headers and then installs its
`RuntimeState`. A generated cursor0 snapshot containing timing confirmation
`confirmed_by_action = ActionId(1)` from committed cursor1 is admitted at target0.
Its suffix is empty, yet future `confirmed_grid` survives. A valid cursor0
payload without future provenance hydrates successfully.

Replay §9.3 explicitly requires rejecting payload state after the anchor cursor.
The demonstrated reference is checkable from existing metadata without analysis
or source access. This is malformed-input/recovery integrity, not evidence that
ordinary save fabricates the input. The Session load contract explicitly mandates
header checks; hydration is the primary violated boundary, not a separately
asserted load-contract defect. Arbitrary semantic payload equivalence is not
claimed to be cheaply provable.

Disposition:
[RIOTBOX-1607](https://linear.app/riotbox/issue/RIOTBOX-1607/reject-snapshot-runtime-provenance-after-the-selected-replay-cursor).
Validate explicit replay-relevant action provenance and same-Session references
in Core before installation. Require typed refusal/nonmutation and valid-prefix
controls through Core and public App hydration, with no silent anchor fallback,
automatic repair, source read or hidden App-local validation truth.

## System model and healthy boundaries

Core/Session owns actions/records, declared chronology, typed Undo domains,
snapshot cursors, replay traversal and capture/export lineage. App admits live
mutations, supplies only the optional legacy missing-plan selection, owns external
side effects and rebuilds projections. The1599 traversal seam preserves Core
invalidation and transactional installation; no new defect survived there.

Committed Seek/Stop/restore intent flows through the App pending target and CLI
transport handoff into a checked position identity. One callback-owned clock
publishes coherent timing/identity; shell observation remains unavailable until
the newest admitted identity is acknowledged. This handoff, supersession,
exhaustion and ordinary Preserve/tempo path had no new retained defect. Correct
clock acknowledgment alone does not prove each stateful renderer consumed it.

The recorder owns one borrowed App facade and consumed fixed-deadline wait.
Pending supervision synchronizes/validates faults and completion before one
bounded input attempt. Verified stop precedes observer I/O, capture finish and
artifact publication. Queue/Core status, not input counters, owns Fill commits;
late commits do not claim recorded delivery. Writer admission binds active
capture/lineage and Session timing identity; exact WAV/proof read-back and Core
receipt gates precede Session publication, with owned rollback on failure.
Independent Adversarial Implementation / API And Consumer Compatibility review
retained0 new recording/input findings after tracing these consumers.

V2 Linux input deliberately has fixed storage, one nonblocking one-byte read per
attempt, ownership-aware independent raw/flag restoration and failure latching.
Linux-only support, shared-open-file-description prerequisites,50ms Escape
ambiguity and indistinguishable unmarked paste/autorepeat remain explicit limits,
not newly discovered defects. Default observer V1 and opt-in V2 remain distinct.

## Executed evidence and coverage limits

Replay witness: `/tmp/riotbox1603_replay_spec_witness.rs`, source SHA-256
`38d85cca259e157cb43b3c5f781e89bd12ce7dc111de8956405653f654d3b687`.
Coordinator freshly built App/Core from the pinned unchanged checkout, linked
with `rustc --test`, and executed4 tests:2 intended regression assertions failed,
2 positive controls passed,0 ignored. Final log
`/tmp/riotbox-1603-replay-witness-executed.log`, SHA-256
`f3a6a6e05556920fdb0da6081680a783b7615cb1686220e9d099f9e6fdea149b`.
An intermediate control-source borrow error was corrected before compilation;
that non-execution is not counted as a failing product test.

Audio witness: `/tmp/riotbox_1603_audio_checkpoint_witnesses.rs`, source SHA-256
`3baa77bc637f9e732495cf4e689dc4811f001314a54616a96d7a87eb6d655075`.
Only3 test-module wiring lines were temporarily added in the initially clean
`/tmp/riotbox-1599-ci.r6GXNI` checkout at1602 reviewed head `d7cfe0ca`; its full
tree equals pinned main. No production source was changed. Command
`cargo test -p riotbox-audio --lib riotbox_1603_review_witnesses -- --nocapture --test-threads=1`
actually ran2 tests, both failing at their intended output assertions; coordinator
reran both with identical observations. Original log
`/tmp/riotbox-1603-audio-review-red.log`, SHA-256
`7e238b3fbf8c0709efdc2a0434ded5162da39e8772d41b96e9572d7989899543`.
The temporary wiring was removed before report-branch integration validation.
Witnesses/logs are local diagnostic artifacts, not a new shipped test suite;
each follow-up must add durable owning-module regressions and obtain actual RED
then GREEN evidence. The concrete setup and outcomes above preserve the repro.

Six existing audio controls passed: offline compatibility2, dry stopped1,
W-30 overflow2 and uninterrupted running/tempo1, in
`/tmp/riotbox-1603-audio-{offline,stop,overflow,running}-controls.log`.
All callback witness/control processing checks zero heap operations. Existing
1602 main-push CI38983999503 has both jobs green; green existing CI does not
invalidate these new counterexamples. This report's branch review/CI and merge
remain independently recorded in PR/Linear.

Jev1.13.0 supplied a bounded wording check, not correctness, severity or permission.
Four claims selected bounded; Undo was low-confidence (.43,confidence.14), so it
was not treated as validation. Snapshot/Stop/overflow probabilities were
.94/.94/.97; a deliberately unsupported host/endurance claim selected overclaim
at1.00. Usage1689input/191output tokens, estimated$0.00007094. A first request
with an incomplete model ID returned HTTP400 and produced no judgment. Code,
owning contracts and executed witnesses remain authoritative.

No production fix or new operating/algorithm contract is frozen by this report.
The sequence1604→1605→1606→1607 repairs existing P023 Golden Path output and
recall blockers rather than claiming new musical progress. No immediate host
attempt, DAW setup or new musical mechanism follows from the checkpoint.

No actual user Session migration, source/capture/Holdout/commercial/ignored-attempt
access or directory discovery, host/device/service/TTY observation/runtime,
playback, DAW or Windows/TUI expansion occurred. No new musical/source-general/
human/hardness/physical-latency/endurance/release claim. RIOTBOX-1589 remains a
passed consumed one-off; original ALSA cause and broader1041/1036 remain open.
Hostile filesystem writers, power-loss, unsampled DSP and Sidecar are outside
coverage. No speculative file-size/refactoring ticket was created.

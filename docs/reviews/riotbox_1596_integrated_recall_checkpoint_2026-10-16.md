# Integrated transport and recall checkpoint — RIOTBOX-1596

Date: 2026-10-16. Reviewed main:
`c5614a74b9d3ce990b2d63dae9426a658f666afb`, tree
`daf8454fd26544b33b5e8ca74cc3ea39c0aba4ec`. Read-only inspection began on
1595 candidate `1d19ea065c2a4a696ae5a4d9d6cfdca5f37b187a`; coordinator
verified exact candidate/merged tree identity before creating this report branch.

This is an active-phase checkpoint after four substantive repairs since1591:
live intent/acknowledgment, declared chronology/Undo, restore joins and explicit
upgrade copy. It is not a fifth-feature claim or an exhaustive repository,
DSP/device/Sidecar audit. No production code or test changes in this branch.

## Findings and dispositions

### [P2] Live admission can publish history incompatible with its legacy policy

Locations: App `jam_app/commit.rs:27–137`, `commit_ready_actions`; Core
`replay/plan/undo_validation.rs:142–156` and
`replay/plan/committed_order.rs:91–105`. Ordinary Undo caller:
`cli/event_loop.rs:585`.

Reviewer: independent Spec And Evidence Auditor, Data And Migration lens.
Coordinator inspected and independently relinked/reran all six controls.

A loaded `DeterministicPreferred` Session commits Source Monitor Blend at1000,
saves, and reloads successfully. `undo_last_action(500)` then reports success,
marks the target Undone, removes its restoration snapshot and restores the
monitor. Ordinary `app.save()` succeeds and replaces that previously loadable
generated checkpoint. Replay, public App reload and conservative upgrade now
reject `InvalidUndoTargetRelation`: historical policy still requires target
timestamp not later than marker timestamp. V2 rollback and legacy forward-clock
controls pass. This is successful publication of invalid declared history, not
an observed filesystem-corruption or actual user-data-loss incident.

A related public Queue→commit path accepts Legacy TransportStop at beat12.
Subsequent save succeeds but load rejects unsupported legacy navigation. Stop
alone remains explicitly upgradeable. A normal typed monitor commit at reset
beat0 makes stored `[Stop12, Monitor0]` disagree with canonical order, so upgrade
also refuses ambiguity. V2 accepts the same sequence. No existing keyboard or
Ghost Stop binding was established; this part is public product-API reachability.
The SourceMap-specific backward-seek guard does not cover those producers.

Disposition: repair next in
[RIOTBOX-1597](https://linear.app/riotbox/issue/RIOTBOX-1597/refuse-legacy-live-mutations-before-they-create-an-unreplayable-saved).
Admit against the declared Core policy before destructive live/history mutation;
preserve valid legacy/V2 behavior and nonmutation on refusal. Never auto-upgrade,
retime, reorder or relax old semantics. A save check alone cannot undo a falsely
successful live Undo. The explicit copy correctly refuses this invalid input;
no new defect was found in1595's copy boundary.

### [P2] App restore revives an invalidated MC-202 phrase plan

Locations: `jam_app/restore_replay.rs:97–119,171–210`;
`jam_app/side_effects/source_timing.rs:34–68` and the Core source-timing executor.

Reviewer: independent Spec And Evidence Auditor, Data And Contracts lens.
Coordinator inspected and independently relinked/reran all three controls.

A valid fresh V2 generated history confirms a Grid and carries a typed persisted
phrase plan. Public matching Grid Revert clears the plan; reconfirming that same
Grid does not regenerate it. Live state, ordinary save/load and Core replay
correctly retain None. `apply_restore_target_suffix(4)` then returns the old
plan as Some. Without Revert the plan remains present everywhere; with Revert
but no reconfirmation it stays absent everywhere.

The App postprocessor collects only phrase actions, not intervening invalidation,
and considers current Source identity sufficient to reuse the last plan. It
therefore overrides correct Core state after the original Grid returns. This is
a current-state ownership/parity defect, not a1594 indexing regression: its
retained old-scan oracle deliberately preserved that earlier behavior.

Disposition: repair after1597 in
[RIOTBOX-1599](https://linear.app/riotbox/issue/RIOTBOX-1599/preserve-mc-202-plan-invalidation-through-app-target-and-snapshot).
Respect chronological invalidation across target and payload/snapshot paths;
do not infer validity solely from current Source/Grid or silently replan.
No source selector, phrase recipe, threshold or qualification contract may change.

### [P2] Active W-30 timing ignores transport reposition

Locations: `runtime/render_tr909_w30_preview.rs:167–168,788–789`;
`runtime/shared_transport_tr909/output_callback.rs:99–153`;
App `jam_app/projection/w30_preview.rs`.

Reviewer: independent Rust/Audio implementation auditor. Coordinator inspected
the shared renderer, ordinary projection and actual callback call chain, then
independently relinked/reran the generated public offline diagnostic.

Active preview and resample states seed private beat position only while
inactive, then schedule from their independently advanced cursor. The actual
callback supplies the newly acknowledged transport position but invalidates only
the drum cursor. An ordinary seek leaves focused W-30 material/trigger unchanged,
so that lane can retain old scheduling phase while transport/drums/source monitor
jump. Audio Core §7 makes transport the playback timing authority.

An in-memory ramp proves shared-renderer phase ignorance through the public
historical numeric offline ingress: after750frames at1000Hz/120BPM, forward
actual1.5→target4 and backwardactual9.5→target4 each produce the second W-30 buffer
bit-identical to their respective no-seek control. Forward-vs-cold target phase:
all1000 samples differ, maximum delta0.0054650363. That cold control only
distinguishes phase; it does not prescribe retrigger/envelope semantics.

Executed proof is not a typed live callback/host run. Live reachability is
source-traced callback→shared mixer→same renderer. Historical numeric offline
compatibility was intentionally preserved by1592; no1592 regression is claimed.
Reviewer suggestedP1; coordinator retainsP2 for bounded single-lane alignment,
without measured live frequency, physical-device or endurance impact.

Disposition: repair separately in
[RIOTBOX-1598](https://linear.app/riotbox/issue/RIOTBOX-1598/apply-live-transport-reposition-to-active-w-30-timing-without-changing).
First define existing-clock live timing synchronization without blindly
cold-retriggering one-shots or changing frozen gestures. Require exact generated
production-callback phase/output and heap controls; preserve offline compatibility.

## System model and coverage

Core/Session owns declared replay policy, actions/records, typed Undo, cursors,
snapshot hydration and capture/export identity. App produces live mutations and
projects/restores that truth; it cannot admit one policy and save another meaning
or overwrite Core replay with a filtered competing history. Audio owns callback
timing and per-lane rendering. Control delivers explicit intent before reading
acknowledged observations; observation is neither a new command nor proof that
every stateful lane consumed the new phase.

Sampled entrypoints include public load/save, normal Undo and commit, target
restore, snapshot/payload reconstruction, pending intent and callback observation,
recorder shutdown/abort, and isolated upgrade-copy publication. QA inspection
covered generated fixture admission, immutable prefix/policy correlation,
action/receipt/marker/proof identity and Core-authoritative readiness in
`loaded_recorder_{fixture,evidence,artifacts,workload}.py`. No QA defect was
retained from that sample. Capture/export inspection was bounded, not an audit
of every schema or artifact-producing action.

## Reproducible generated evidence

All standalone witnesses were compiled separately against current existing
libraries, with no Cargo or checkout mutation during1595's clean-head CI.
Coordinator read each source and independently relinked/reran it. Exit0 means
the counterexample/control assertions hold, not that product behavior is correct.

| Witness under `/tmp/` | Controls | Source SHA-256 |
| --- | ---: | --- |
| `riotbox-1596-legacy-admission-witness.rs` | 6 | `52a2bde8ae2bed72fb7c50b09d12197c01ad685487a0d8d1a1b8e962aac67758` |
| `riotbox-1596-plan-invalidation-witness.rs` | 3 | `e02e0b46258d934f96ca43bcaeb43528b2f12b6979e37de90d56b96a88b3741f` |
| `riotbox-1596-w30-seek-probe.rs` | forward/backward/no-seek/target | `47b4a7bc4c63cbb59d1523f0c4cdc1eaac1e4d67452639fbbecf2954c53f5838` |

Coordinator logs `/tmp/riotbox-1596-root-{legacy,plan-invalidation,w30}-{link,run}.log`.
Linked App49e8f3876be05298 SHA-256
`445c6172fd79431317db14b749cd5c57a0f1af8c4218f2b5ebb6cea3aa0e121a`;
Corebe900b0c2308e721
`8970a3d2645c8a43524b25fc263d90ea64bfa368b329aa74d4a7bd21421c93c5`;
Audio882ae0a40bbd50d5
`7949b7150f0becf608e9546a614bd946a3695802bd047213069ad28f10431d6f`.
Only fresh owned temporary Session metadata was saved/read by the first two;
no referenced material was hydrated. W-30 renders only generated in-memory PCM,
not an audio file. Observations preserve saved metadata bytes.

Inherited integration evidence is explicit:1595 exact-head local `just ci` and
native38953927698 attempt1 both jobs passed; its tree matches reviewed main.
These green existing checks do not disprove the new counterexamples. This report
branch's own review/CI and closeout remain separately recorded in PR/Linear.

## Next priority and limits

All three findings are retained for repair, not waived. Order:1597 legacy saved
history admission,1599 invalidation parity, then1598 W-30 alignment. No immediate
new host attempt, DAW setup or musical exploration is justified by this checkpoint.
No new algorithm or operating-contract decision is frozen by this report.

No actual user Session migration, source/capture/Holdout/commercial/ignored-attempt
access or directory discovery, host/device/service observation/runtime, playback,
DAW or Windows/TUI expansion occurred. No new musical/source-general/human/
hardness/latency/endurance/release claim. Complete scheduler/device behavior,
hostile directory writers, power-loss, unsampled DSP and Sidecar remain outside
coverage. RIOTBOX-1589 remains a passed consumed one-off, not an approval to
repeat; original ALSA cause and broader RIOTBOX-1041/1036 remain open.

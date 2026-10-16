# RIOTBOX-1601 — Cooperative master-capture supervision

Date: 2026-10-16. Review base: merged main
`21e22ae99e1402b8697b1a620fa283646d1349ad` (RIOTBOX-1598).
P016 maintenance / necessary controller deepening under RIOTBOX-1036,
protecting P023 recording. Linear was In Progress before branch creation.
The branch joined its tree-identical squash base through ordinary merge
`31d8fde146739df635036ea97670a15d2d6963f0`; all seven then-dirty file hashes
were preserved. No reset, rebase, stash or unrelated changes.

## Purpose and boundary

The existing blocking capture-wait loop monopolizes the control thread during
a take. This slice exposes one complete supervisory iteration so a later
performer-input consumer can regain control between pending observations.
It does not add interactive recording, performer input or a new musical action.
That later slice needs its own bounded input/delivery/proof contract: Core
commit order alone does not acknowledge an action's first rendered frame.

RBX-462, Audio Core and Module Policy were frozen before implementation.
The private, non-Clone `CaptureWait` owns only borrowed immutable launch/plan/
output context, started-transport policy and the existing fixed deadline.
Each consumed poll borrows App/runtime and either returns the same owner or
terminates. No Session copies, shadow progress, recorder, audio buffers, new
clock, persistent App state, scheduler or public Audio test constructor.

The operation order remains sync → runtime health → progress faults →
completion → deadline. Faults beat simultaneous completion; valid completion
beats deadline. A pending poll never waits, finalizes, emits observations or
publishes artifacts. Only the unchanged blocking wrapper waits its existing
10ms between pending polls; no numeric budget changed.

Completion still performs final synchronization, retains pre-stop health,
stops the actual runtime and verifies Stopped before yielding a typed fact.
`StoppedCapture` means this boundary only, not clean capture, successful finish,
receipt or publication. Optional `WindowStopped` evidence is constructed
internally from the original plan; the parent emits it after verified stop.
Disabled observation performs no clock/evidence work. Finish, late-fault
rejection, WAV/proof/read-back and Session-save gates remain downstream.
Failure cleanup preserves stop → abort → rejection, including the historical
blocked-summary helper's second stop.

Initial transport confirmation and terminal device/artifact operations remain
synchronous. This is cooperative capture-wait supervision, not a realtime or
blanket nonblocking API, cancellation-on-drop guard or interactive frontend.

## Test-first evidence

Three sequential new-interface RED→GREEN verticals compiled and executed
exactly one test apiece. These prove a new API contract, not a newly discovered
pre-existing product defect; the original blocking wrapper remained unchanged
until the cooperative policy passed, then adopted that same implementation.

1. Completed poll must stop before returning, without wait or publication.
   The initial waiting-only implementation returned Continue incorrectly.
   RED: `/tmp/riotbox-1601-completed-poll-red.log`, SHA-256
   `2ce8c937099a14b38887fd864698a5c6b4b20b2fd71c9feeb6db287321f93acd`.
   GREEN: `/tmp/riotbox-1601-completed-poll-green.log`.
2. Repeated pending polls must retain the original deadline. Both missing and
   incomplete progress yield at explicit virtual times0,0,7990ms, then terminate
   at8000ms without sleeping or publishing. The first implementation's early
   Continue bypassed timeout. RED: `/tmp/riotbox-1601-fixed-deadline-red.log`,
   SHA-256 `64ffad192b17cc7665fb961c0d36a814eb8998a01377d56b7e3045fc5c6176c2`.
   GREEN: `/tmp/riotbox-1601-fixed-deadline-green.log`.
   The final control additionally enables observation to detect clock work
   during pending/timeout, not merely absence of an observation sink.
3. Runtime/stream/scratch/progress faults must beat simultaneous completion.
   The new API initially lacked those guards and returned a stopped fact.
   RED: `/tmp/riotbox-1601-fault-precedence-red.log`, SHA-256
   `48708227320db49c52add99ab7fb072bc5aef52a60384f11eb35b80b4cb3ed34`.
   GREEN: `/tmp/riotbox-1601-fault-precedence-green.log`.

Additional acceptance controls are green-on-arrival, not semantic RED witnesses:

- Completion at deadline, unconfirmed stop, finish failure and late fault compare
  direct stopped-only results against the existing blocking wrapper. A verified
  stop cannot conceal downstream rejection or manufacture a receipt.
- An acquired synchronization refusal beats otherwise healthy complete progress,
  preserves the unsafe pending action, stops/aborts and leaves Session bytes and
  artifact targets unchanged.
- Enabled observation proves completion-clock → Stop → stopped-clock before
  return. The returned milestone matches the original plan identity, hash,
  geometry, references and exact generated clocks. No sink call, Finish,
  publication or Session save occurs until the caller explicitly emits evidence.

Fixtures use existing generated `RuntimeControl`, owned temporary metadata/
artifacts and explicitly advanced virtual time. They do not instantiate a
device stream, open source audio or prove actual callback delivery.

## Review and verification

Independent read-only Rust / Adversarial / Maintainer review and independent
Spec / Evidence / Product / Risk review retained zero P0–P3 findings. Both
reviewed the complete production modules, original/new wrapper ordering,
contracts and six direct tests; neither claimed to run tests or CI.
Coordinator self-review likewise found no retained defect or necessary further
semantic refactor. The production lifecycle shrinks from593 to472 lines and
the new200-line owner encapsulates complete supervision policy, not a sleep
helper or mechanical include shard. The pre-existing cohesive generated test
parent gains child-module declarations only; new tests have semantic owners.

The final integrated controller run passes37 tests with0 failures or ignores:
`/tmp/riotbox-1601-controller-extra-acceptance.log`. Each additional direct
ordering control also passes when selected alone. `cargo fmt --all --check`,
strict workspace all-target/all-feature Clippy and `git diff --check` pass;
Clippy log: `/tmp/riotbox-1601-clippy-preflight.log`.
Final full source-free local/native CI results are recorded in the linked PR
and Linear closeout. They are merge gates, not implied passes from this
report's code-review snapshot. Existing wrapper tests retain2/8/16
bars, already-playing transport, unavailable timing, admission/capacity,
late faults, optional observer errors and save/rollback coverage.

No new ActionCommand or queue/replay/receipt schema, sound recipe, DSP,
threshold or audio-producing behavior. No TUI, DAW or Windows-product scope.
No real Session migration, source/capture/Holdout/commercial/ignored-attempt
access or discovery, host/device/service observation, runtime or playback.
No human, musical, source-general, hardness, physical latency, endurance or
release claim. RIOTBOX-1589 remains passed with its one-time approval consumed;
RIOTBOX-1041/1036 and the original ALSA cause remain open.

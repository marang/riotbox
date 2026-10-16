# Checked ActionId exhaustion — RIOTBOX-1587

Date: 2026-10-16. Classification: maintenance/regression, not an audible
mechanism or new instrument-quality claim. Base: merged main
`961dceffa3f40946fa42acfba80caa5df629cb12`. Scope: the one existing Core
allocator, App producers and their CLI/observer/generated-render/test consumers.
The current-state checkpoint's second retained P2 is the starting evidence;
RBX-450 and Action Lexicon §3 own the repaired contract.

## Defect and correction

The earlier public-loader reproduction accepts coherent small histories ending
at `u64::MAX` or `MAX−1`, then ordinary transport toggles reuse MAX, save
successfully and make the next restore/replay reject duplicate IDs. This does
not require naturally executing 2^64 actions. The original evidence is retained
in the RIOTBOX-1583 checkpoint; it is not a new host/audio test.

Core now holds a private optional next ID. MAX can be issued once; the next
allocation returns typed `ActionIdExhausted`. Reserving MAX exhausts the same
allocator, and later empty/older reservations cannot reopen it. Preserve the
existing distinct constructor behavior: `new()` starts at1, `Default` at0.
There is no cursor schema, renumbering, sentinel ID, second allocator or panic.
AlreadyPending and eligibility/no-target outcomes keep their old precedence
when no allocation is needed. Existing queued IDs may still commit/reject.

The Rust producer APIs intentionally become fallible. App propagates the Core
error; writer APIs wrap it in the existing `JamAppError`. Undo reserves its
marker before even normalizing old policies or removing/restoring a snapshot.
Ghost validates acceptance on a staged copy and only publishes it after a
successful enqueue. Export autoqueue refuses before any artifact work and no
longer assumes a pending action through `expect`.

CLI control status shows exhaustion rather than successful queue/commit.
The observer probe also replaces its initial requested-status text with the
error and emits no successful action event. Render producers propagate the
error; existing generated fixtures explicitly unwrap expected available IDs
before their unchanged behavior assertions.

Recorder shutdown is a separate safety operation: a failed initial Play stops
the device; export/final-Pause exhaustion also stops it. An acquired capture is
aborted, failure context is retained and no WAV/proof/receipt is published.
When Pause cannot receive an ID, no successful Pause or stopped Session
transport is fabricated. Generated controls prove device stop separately from
that unchanged action truth. A complete final three-ID Play/export/Pause path
still saves and reloads uniquely.

## Executed focused evidence

- Genuine pre-fix Core RED: `/tmp/riotbox-1587-core-exhaustion-red.log`.
  Two enqueues after reserving the last ID produced two pending entries where
  only one unique allocation was valid; no panic-catching substitute.
- Core package:508 library and8 binary tests passed; one existing manual
  replay-scaling benchmark remains ignored. Queue controls are a subset, not
  extra coverage. `/tmp/riotbox-1587-core-final-tests.log`.
- Eight new App boundary tests passed:
  `/tmp/riotbox-1587-app-action-id-tests.log`. They use the normal public
  RuntimeFull loader on generated reference-free Sessions, exercise MAX/MAX−1,
  save/reload/executable replay, refusal without state/projection mutation,
  pending/eligibility precedence, Undo snapshot/normalization order, final
  Undo marker, Ghost state and export writers before side effects.
- Recorder lifecycle23 passed, including three new ID-boundary tests and
  existing generated2/8/16-bar/fault/publication/rollback controls.
  `/tmp/riotbox-1587-recorder-tests-green.log`.
- Two new CLI controls passed:
  `/tmp/riotbox-1587-controls-tests.log`; five control operations surface errors
  without state/projection mutation and final-ID AlreadyPending stays honest.
- Observer probe16 and UI143 passed:
  `/tmp/riotbox-1587-observer-tests-2.log` and
  `/tmp/riotbox-1587-ui-tests.log`. The new observer regression covers five
  transport/capture/fill/bool/control keys. `cargo check --bins --tests` passed
  in `/tmp/riotbox-1587-adapters-check-3.log`.

The first full `just ci` failed at the optional `limiter-calibration` feature
because its BPM producer and generated fixtures still used the former return
shape. `/tmp/riotbox-1587-full-ci.log` retains the compiler failure; it is not
a passing full run. That producer now propagates typed exhaustion before commit;
its old assertions unwrap successful IDs, and a new generated valid-Graph/MAX
control requires unchanged Session/runtime/queue/Graph and no created files.
The exact feature gate now passes (Audio9, App9, render binary18, Python47,
plus rotation metadata fixtures) in
`/tmp/riotbox-1587-limiter-calibration-fixtures.log`.
`cargo check --workspace --all-targets --all-features` passes in
`/tmp/riotbox-1587-all-features-check.log`. Both independent reviewers rechecked
the complete delta with zero findings and reran the optional App suite9 and
new boundary control1 respectively, PASS:
`/tmp/riotbox-1587-independent-limiter-delta.log` and
`/tmp/riotbox-1587-spec-review-limiter-boundary.log`.
The coordinator rechecked that only typed propagation and successful-test
unwrapping change the old feature path. Full local source-free `just ci` then
passed in `/tmp/riotbox-1587-full-ci-final.log`, including normal Rust/generated
audio/contracts/JSON/format checks and strict all-target/all-feature Clippy.
Its implementation candidate is `b1622d2c9bdaae629aedf689607e005b41e17c87`;
subsequent edits to this report only record executed evidence/follow-up status.

The first App/UI builds still reported four main-owned unused-Result warnings
while recorder propagation was in progress; those calls were subsequently
handled, not suppressed. The final recorder/control builds are warning-free.
An initial recorder test incorrectly treated rejected queue history as
committed Session history; it was corrected to check both owning collections,
their uniqueness and unchanged disk bytes separately. No product assertion was
weakened to accept duplicates. The first observer regression exposed stale
requested-status feedback; the error boundary was corrected and all16 tests
rerun. These intermediate failures are not hidden as green runs.

Selections overlap; counts must not be added as unique repository coverage.
The generated recording tests write only their own temporary synthetic
artifacts. No actual AudioRuntime/device is constructed or launched.

## Review and merge gates

An independent Torvalds-Inspired Maintainer / Adversarial Implementation / API
compatibility review inspected the complete code boundary and found no P0–P3
defect. It reran Core9, App8, recorder3 and observer1 controls, all PASS:
`/tmp/riotbox-1587-independent-{core-exhaustion,app-exhaustion,recorder-exhaustion,probe-exhaustion}.log`.
A separately independent Spec And Evidence Auditor / Product Pragmatist /
Devil's Advocate review found no P0–P3 defect, verified the cited intermediate
and green logs, preserved existing assertions and scope, and reran Core9,
App/CLI/recorder13 and observer1 controls, all PASS:
`/tmp/riotbox-1587-spec-review-{core,action-id,observer}.log`.
The coordinator's follow-up self-review found no additional ActionId-specific
P0–P3 issue in the checked cursor, mutation order, failure propagation,
unconditional device stop, public API compatibility, contracts or assertion
preservation. The later adjacent capacity finding is separately retained below.
`ui-expert` additionally checked that the existing error feedback
does not imply a successful action; no layout, keymap or new TUI surface was
introduced. Full local source-free CI passes as recorded above. Exact-head
native CI remains a later PR gate; no pending native/merge result is claimed.

Size review includes existing cohesive large queue, event-loop and recorder
owners. Semantic exhaustion test children separate this boundary from ordinary
queue, App and recorder behavior; no textual include shards or generic error
framework were added. The public API propagation necessarily touches many
existing tests; ordinary expected values/assertions are retained.

## Separate adjacent finding and disposition

### [P2] Per-boundary commit sequence exhaustion still panics after mutation

Reviewers: Adversarial Implementation Reviewer (coordinator's subsequent
adjacent-boundary diagnostic, not a finding attributed to either earlier
independent review).

Location: `jam_app/commit.rs:243`, `record_committed_action()` and its queue/Undo
callers. The existing checked next-sequence calculation ends in `expect`.
Normal restore and replay accept a nonzero unique `u32::MAX` sequence at the
actual restored Immediate/beat12/bar4/phrase1/sceneNone boundary. The next
ordinary toggle with available ActionId2 panics after adding a committed queue
entry absent from Session. A separate typed Source Monitor Undo panics after
restoring Blend→Source, marking its action Undone and consuming its snapshot,
without appending the structural Undo marker.

A bounded generated public-Core/App executable demonstrates both cases. The
coordinator read its complete source, independently rebuilt and reran it in a
fresh temporary owner: `/tmp/riotbox-1587-sequence-root-{build,run}.log`.
Source: `/tmp/riotbox-1587-sequence-diagnostic-wi5GUT/main.rs`, SHA-256
`838fc0ea164903df167ecfc54b5aa9b62598d92738e7ba56deb536142dfd9623`.
The diagnostic catches the panics to inspect state; exit0 means it demonstrated
the defect, not a correctness pass. Both disk files remain byte-identical; no
post-panic save or runtime/device operation was performed. Static comparison
confirms the same checked calculation/expect and admission on base main961dceff;
this is pre-existing and independent of ActionId exhaustion. No executed
old-main run is claimed.

Disposition: deferred to the separately scoped immediate follow-up
[RIOTBOX-1588](https://linear.app/riotbox/issue/RIOTBOX-1588), not waived or
included in1587's repair claim. Per-boundary whole-batch capacity and direct
Undo/export admission need their own typed Core/Session contract before
mutation, while preserving valid exhausted history and other boundaries.
This changes a distinct commit-capacity domain and stays out of the checked
ActionId PR. It does not invalidate the focused ID regressions or green CI,
which did not cover this previously untested sequence boundary.

## Bounds

No new ActionCommand, persistent App state, Session/replay schema, dependency,
DSP/renderer/sound policy, recording/observer/artifact version, numeric audio
gate or public test-only constructor. No source/capture/Holdout/commercial
audio access, source-directory discovery, old ignored-attempt reads, host
metadata/service/device operations, playback, DAW, Windows work or fresh
trial permission. This is not an original ALSA-cause, latency, endurance,
source-general, musical, hardness, human-listening or release pass.
RIOTBOX-1041/1036 remain open.

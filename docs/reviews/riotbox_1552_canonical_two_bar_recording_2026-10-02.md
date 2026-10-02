# RIOTBOX-1552 — canonical two-bar recording and DAW handoff

Date: 2026-10-02
Issue: RIOTBOX-1552, child of RIOTBOX-1036 / P016
Branch: `feature/riotbox-1552-canonical-two-bar-recording`
Base tree: PR #1654 head `f5ee01a7b862c87b3898c18b7ab07cefa13f9ce5`,
merged unchanged in `654791993a744c5c2b6e1948bdb71e0566ad48fb`
Decision: RBX-423

## Outcome and evidence boundary

Future default and explicit two-bar recordings use V4, with the canonical
runtime-f32 geometry already used for the bounded long-window V3. V4 recordings
can enter the separately versioned DAW V2 handoff without changing their audio
bytes. Eight/sixteen-bar recording stays V3. Historical V1/V2/V3 evidence remains
unchanged; usable V2 recordings retain the DAW V1 path. An old V2 receipt that
fails its frozen full-readiness check stays non-ready; no automatic migration
or reassessment is performed.

This maintenance slice removes the previously observed producer/consumer
timing disagreement from future two-bar recording. It does not change DSP,
transport, the callback tap, physical timing tolerances or allocation limits.
It is generated-PCM/metadata engineering evidence only: no source/capture/
Holdout/commercial audio, source-directory discovery, host playback, DAW import
or human review. Existing listening judgments do not qualify a fresh V4 take.
RIOTBOX-1036 remains open.

## Reproduced defect and version decision

At 48 kHz, 121.5/166.5 BPM serialize the actual runtime frame span as
42187/57812 nanobeats, while legacy V2 integer readiness expects 42188/57813.
Fractional f32 tempos also expose whole-frame disagreements: at 44.1 kHz,
80.6632 BPM gives canonical 262424 versus legacy 262425 frames, whereas
80.6838 gives canonical 262358 versus legacy 262357. The DAW consumer has its
own frame recomputation, so changing only one readiness comparison would not
repair the chain consistently.

RBX-423 chooses a recording V4 and DAW V2 successor instead of redefining V2
proofs or attaching an ambiguous reinterpretation flag to historical evidence.
The existing physical half-frame/start gates and bounded endpoint arithmetic
stay intact; the new recording version selects that arithmetic explicitly.

## Acceptance mapping

- Core owns the V4 and DAW V2 Action/receipt boundaries. V4 requires explicit
  `two_bars`; V3 still admits only `eight_bars`/`sixteen_bars`. Readiness selects
  legacy or canonical timing by version. Restore/replay reject missing or
  contradictory duration, Action/receipt identity and geometry without audio
  access; external recording/export replay still does not repeat file I/O.
- The existing producer plans V4 before queueing, requires the exact runtime
  BPM to roundtrip through micro-BPM, and binds the plan's version/duration to
  the pending Action before publication. It requires full V4 readiness before
  commit. No new production V2 entrypoint exists; the explicit test-only legacy
  queue retains the existing shared writer for generated historical controls.
- A small DAW version adapter maps V2 recording to DAW V1 and V4 recording to
  DAW V2. It is derived from Core identities, not another persisted truth.
  Selection pins the latest supported two-bar receipt and version at queue
  time. Invalid latest supported evidence blocks; V3 long windows are not a
  supported two-bar input and do not replace the selected receipt.
- Same-opened-byte hash/decode/proof validation, source version, tempo, format,
  metrics and stored lineage must agree. The shared archive preserves exact
  WAV bytes, four-member structure, eight-beat clip, XML/readback gates,
  no-clobber and Session-save rollback. Recorded micro-BPM XML formatting is
  unchanged. Host-import, audible-output and release blockers remain present.
- CLI success reports its receipt's committed Action version, not a global
  default or later Action. Failed attempts do not invent a receipt boundary.
  Existing observer/read-only reports and exhaustive UI labels distinguish
  legacy and successor identities; no new TUI control or command flag.

## Verification

- Core recording contract family: 31 passed; Core replay family: 122 passed;
  strict Core all-target Clippy passed.
- App recording family: 21 passed, including legacy V2 controls, V3 long
  windows, V4 rounding ties, save/restore, no-clobber, rollback, version/end
  tamper and unrepresentable-tempo rejection.
- App DAW family: 15 passed: nine explicit legacy V2-to-DAW-V1 cases and six
  canonical/mixed-version families. Includes exact WAV bytes, XML validation,
  metadata-only restore, 121.5/166.5 ties and 90.016 BPM's 255955 versus legacy
  255954 frame case, queued version pinning, latest-invalid refusal and rollback.
- Actual production transport-clock plus capture tap: five rate/tempo cases,
  each with 128, 1024 and varied callback partitions, produce exact frames and
  canonical serialized timing readiness with no faults. Generated PCM only.
- CLI family: 152 passed; historical/successor UI-label cases: two passed.
- Formatting and diff checks passed. The first full source-free `just ci`
  passed tests and smokes, then strict Clippy rejected repeated suffixes in a
  private test enum. Its variants now name the exact generated V2/V4/V3
  contracts; no lint waiver or behavior change. The final full rerun passed
  (`/tmp/riotbox-1552-ci-final.log`), including strict all-target/all-feature
  Clippy. Intermediate shared-tree compiles before
  peer test-module creation and initial fixture-shape failures are superseded
  by the focused green runs, not claimed as final verification.

## Branch review and closeout

Independent reviews cover Core/DAW against the specs (Spec/Evidence/API),
Core/producer/CLI against failure and restore paths (Adversarial/Performance),
and producer/actual-clock tests for version compatibility. Each implementation
author excludes their own changes from independent evidence. All three reviews
retain zero actionable P0–P3 findings. The producer reviewer independently
reran all 21 recording tests and the 15-case actual-clock test; the Spec/Evidence
reviewer reran six Core V4 cases and all 15 DAW cases. No introduced finding
was deferred or rejected. Coordinator compatibility/maintainer, product and
risk self-review likewise found no additional defect: source identities are
pinned, version tamper fails closed, legacy evidence stays immutable, and no
synthetic result is presented as human or host qualification.

Semantic ownership remains in the existing Core/recording/DAW modules; new
files own version adaptation or focused tests. No dependency, textual include,
second recorder/persistence/Session, or durable JamAppState field was added.
Existing large receipt/artifact modules were reviewed for semantic ownership;
the changes extend their owned contracts rather than mechanically splitting
them. No mandatory refactor was identified; the new DAW version adapter
centralizes cross-module mappings without adding persisted state.
The current-state architecture checkpoint was RIOTBOX-1551; this is the first
substantive successor, not another whole-repository audit.

Local/native CI, PR review/merge, main synchronization and branch/Linear
closeout remain required. Keep this issue in recent Done history after merge;
archive its final context before any later deletion. Any real-host V4 or DAW
playback evidence requires its own exact access and listening contract.

# RIOTBOX-1503 — Post-maintenance architecture checkpoint

Classification: maintenance/regression. Production baseline:
`db91297b4d1368215edd176aaebae2f2bd465921`; integrated main:
`5197e419` (only the RIOTBOX-1415 archive differs). The checkpoint follows five
substantive slices after RIOTBOX-1499: capture isolation (1500), commit ownership
(1414), typed source identity (1409), numeric contracts (1420), and runtime
telemetry (1415). It samples the current system, not just the latest diff.

This is a solo risk-directed review using separate technical lenses, **not**
independent reviewer approval or an exhaustive audit. Source-free public-API
probes were rebuilt and executed on 2026-09-30 after the old temporary files
became unavailable. No real source, active holdout, commercial reference, audio
device, DAW or human playback was accessed.

## Findings and priority

Two new P2 input/process robustness defects are demonstrated. The sampled
changes retain the existing Core/Session spine; neither finding establishes a
regression introduced by those five slices. No P0/P1 finding survives this
checkpoint. Fix original-source admission first (RIOTBOX-1505), then bounded
Sidecar writes (RIOTBOX-1504). Both are autonomous maintenance work rather than
audible product progress or a reason to claim P023 complete.

### F1 [P2] Sidecar request writes precede the operation timeout

Originating lens: **Performance and operations**.

Location: `crates/riotbox-sidecar/src/client.rs:310-314`, followed by
`read_response():317-334`; the public graph-building/analysis methods perform
the write before entering that response wait. The technology-stack spec
(`docs/specs/technology_stack_spec.md:94-98`) promises distinct bounded control
and analysis operations. Blocking `ChildStdin::write_all` and `flush` do not
consume or enforce either timeout.

Reproduction uses the public `StdioSidecarClient` API:

1. Spawn a fresh temporary Python peer and complete a normal protocol-0.1 ping.
2. The peer stops reading stdin, sleeps one second and exits deliberately.
3. Install 50 ms control and analysis budgets after the warm handshake.
4. Call `build_source_graph_stub` with a synthetic `SourceDescriptor` whose
   metadata `path` string contains 1 MiB of `x`, enough to fill the pipe.
5. Bound the whole diagnostic externally with `timeout --kill-after=1 5`.

Observed result:

```text
probe=sidecar budget_ms=50 elapsed_ms=1002 result=stdio transport failed: Broken pipe (os error 32)
```

The descriptor is arbitrary public-API metadata, **not** a valid real source
path or an observed user incident. No source is opened. The intended mock-peer
exit releases backpressure; without it the synchronous write has no client
deadline. The diagnostic proves the missing write bound, not a timing benchmark
or every possible transport failure.

Impact: an unresponsive analysis process can strand its caller before response
timeout handling runs. Repair under **RIOTBOX-1504**: a semantic transport owner
must enforce the operation deadline across write/flush and response wait,
invalidate an unusable peer, and clean up without leaving blocked workers or
reusing partial requests/late responses. Preserve separate budgets and existing
request-ID/protocol/provider validation. A thread moved behind an unbounded
join is not a repair. The regression needs warm-handshake pipe backpressure,
typed timeout, a generous outer safety bound and normal-path coverage.

The stdout reader also uses an unbounded channel and `read_line` buffer. This
was inspected, but no additional independent failure was exercised here; it
remains a resource-bound verification gap, not another demonstrated finding.

### F2 [P2] Original-source restore can wait indefinitely on a FIFO

Originating lens: **Adversarial Implementation Reviewer**.

Location: `crates/riotbox-app/src/jam_app/persistence.rs:298`, within
`load_source_audio_cache_for_graph`; the direct audio loader has the same
admission gap at `crates/riotbox-audio/src/source_audio/cache.rs:60-63`.
Both call `std::fs::read` without admitting the opened file type first.

Reproduction uses a fresh temporary FIFO named `synthetic.wav`, a new minimal
Session with an embedded hash-bound Source Graph and matching SourceRef
metadata, and the public `JamAppState::from_json_files` path. A producer thread
waits 750 ms, writes `not a WAV` and closes; an eight-second outer timeout bounds
the process. The restore returns only after that producer arrives:

```text
probe=fifo elapsed_ms=750 status=Unavailable { path: "<temporary>/synthetic.wav", reason: "invalid WAV source audio: header shorter than RIFF/WAVE" }
```

No invalid cache is admitted, and the existing same-buffer content-hash check
is not bypassed. The defect happens before decoding/hash rejection: without a
producer, opening the FIFO waits indefinitely instead of reaching the visible
unavailable state. This is local invalid-input robustness, not demonstrated
data corruption, a remote exploit or an audible-quality issue.

Repair under **RIOTBOX-1505**: use a shared original-source byte-read boundary,
open nonblocking on supported Unix targets, inspect the **opened descriptor**
for a regular file, and decode/hash the same returned buffer. Path metadata
followed by an ordinary reopen leaves a substitution window. Preserve valid
original-source symlinks to regular WAVs; do not accidentally impose capture's
stricter no-follow policy on original sources. Include bounded FIFO and
symlink-to-FIFO rejection, ordinary WAV/symlink success and hash-drift restore
tests. Capture/export admission rules remain independently owned.

## System model

| Owner | Entry/data flow and authority |
| --- | --- |
| Core | Source Graph, typed IDs, Session state, committed action/replay, capture and export identities; no App/Audio/Sidecar dependency |
| App | CLI orchestration and `JamAppState` facade; resolves Session-owned identities, hydrates control-side caches, commits external side effects |
| Audio | Core-dependent source codec and callback render state; prepared buffers and bounded capture handoff, no analysis/process/filesystem calls from callback |
| Sidecar | Core-dependent request/response transport to offline Python analysis; graph results are admitted through protocol/provider checks |

Analysis yields a Source Graph; committed actions update Session truth; App
materializes artifacts and prepares runtime state; Audio renders those prepared
inputs. Restore binds metadata to bytes before cache admission. F1 breaks the
bounded offline-process failure contract; F2 delays failure at the original
source input boundary. Neither requires a second persistence/replay model.

## Sampled boundaries and retained protections

- **Graph transaction/recovery:** `persistence/graph_transaction.rs` preserves
  the on-disk Session's exact generation when a corrupt alias is recovered and
  subsequently saved. Authority does not come from the edited Session or a
  directory scan. Non-JSON I/O errors still propagate. Existing tests include
  corrupt-alias recovery → edit → save → reload and symlink collisions. Core
  generation publication explicitly reports unsupported hardlinks; no new
  filesystem compatibility or crash-durability claim is made.
- **Capture ownership/identity:** opaque exclusively allocated files avoid
  session-local `cap-NN` collisions. Session locators are published after the
  complete write. Restore checks persisted identity from the decoded buffer,
  refuses changed/legacy-unverified artifacts and invalidates duplicate capture
  IDs rather than choosing one. Capture admission already rejects nonregular
  opened descriptors; F2 concerns the original-source path, not this reader.
- **Typed source relationships/scenes:** relationship endpoints resolve against
  the actual graph catalog; ambiguous node IDs are rejected. Explicit scene
  source bindings do not silently fall back to legacy inference. Supported
  decode profiles retain a narrow wire adapter; ambiguous historical custom
  names cannot acquire a standard profile by guesswork.
- **Commit/replay ownership:** Core owns action classification, timing and
  durable consequences; App dispatches materialized side effects. Existing
  differential/restore tests protect the moved boundary. This checkpoint
  samples those owners rather than repeating every 1414 permutation.
- **Runtime telemetry:** poison is observable in public health without
  fabricating a device error count. Running poisoned telemetry is faulted;
  stopped remains stopped. Timing-only reads use existing atomics, not the
  diagnostic mutex. Live-master recording requires running planned output and
  clean stream/scratch health, so the sampled consumer does not admit a
  telemetry-faulted runtime as healthy.
- **Live-master export:** the Session-owned receipt pins one exact WAV and
  proof; opened regular artifacts are hash-bound and semantically validated.
  Metadata-only restore avoids source/capture hydration and preserves graph
  references. This is contract inspection and synthetic test coverage, not
  actual DAW import or live-device qualification.
- **QA numeric ownership:** shared exact-mix predicates enforce finite values,
  producer format identity and the documented Fill conjunction. The shared
  professional-suite policy is distinct from runtime gain. Existing limiter
  provenance and reverse-count discrepancies remain **RIOTBOX-1501/1502**, not
  duplicated findings or authorization to tune source results.

## Verification and limits

The standalone probes use only temporary synthetic metadata/FIFO input and a
bounded mock process. Source: `/tmp/riotbox-1503-boundary-probes.Ag3Cqy/src/main.rs`;
build: offline Cargo against the public App/Core/Sidecar APIs. Logs:
`/tmp/riotbox-1503-sidecar-write-timeout.log` and
`/tmp/riotbox-1503-fifo-restore.log`. These paths are operational evidence,
not durable dependencies; the exact setup/results above and the follow-up
regression requirements preserve the finding if `/tmp` is cleared again.

`just ci` passed (`/tmp/riotbox-1503-ci.log`): App 767, Audio 273, Core 470 and
Sidecar 14 library tests passed, along with binary tests, Python/contract
fixtures, source-free synthetic audio/observer/manifests, tracked JSON,
formatting and strict Clippy. `git diff --check` passed. No runtime code changed.
No synthetic test grants a musical/human pass. Coverage excludes exhaustive
Python provider review, all decoder fuzz cases, filesystem/OS matrices,
power-loss durability, transport memory-stress testing, host audio and DAW
integration. Existing structural backlog (CLI textual includes/import ownership)
is not reprioritized above the demonstrated failure paths merely by file size.

## Branch review and disposition

The branch contains this review and the correction of RIOTBOX-1415's archive
deletion date: feature completion was 2026-09-22, but actual Linear deletion was
2026-09-30 after a transient HTTP 503 retry. No runtime code changes.

F1/F2 are **deferred to the explicit Todo follow-ups**, not declared fixed by
this audit. A sequential documentation self-review found no retained branch
finding: locations, timing evidence, priority, limits and ownership are
consistent with the inspected baseline. Separate processes/independent review
were not used, honoring the user's current constraint.

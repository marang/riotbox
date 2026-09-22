# RIOTBOX-1499 — Periodic architecture review

Classification: maintenance/regression. Inspected production baseline:
`c364ea46cf8cab04db46fbdfe083ac21972b7f8b`; integrated documentation baseline:
`cebd5b7459b7772ea5babd616b25aa626b74f24a` (subsequent changes are archive and
Sidecar test/fixture work). This is a risk-directed current-state review, not a
whole-repository line-by-line audit or a claim of independent reviewers.

## Result and immediate priority

One new **P1** correctness finding is confirmed with an executable public-API
probe: two Sessions in one directory can overwrite each other's capture WAV.
Tracked as **RIOTBOX-1500**, ahead of the existing structural maintenance work.
The recent capture identity check detects the replacement on restore, but
cannot recover the overwritten bytes. This is a pre-existing writer/locator
problem, not evidence that the identity check introduced it.

### F1: Session-local capture IDs become shared mutable filenames

Evidence at the inspected baseline:

- `jam_app/capture_helpers.rs:614` allocates `cap-NN` from the current Session's
  capture count; line 104 derives `captures/{capture_id}.wav` from that ID alone.
- `jam_app/capture_artifacts.rs:377` resolves that relative locator against the
  Session file's parent directory, not a Session-specific namespace.
- `jam_app/capture_identity.rs:81` publishes via `std::fs::write`, replacing any
  existing destination. Both source-window capture and bus-print persistence
  use this writer.

All three paths are under `crates/riotbox-app/src/`. A normal new Session starts
at `cap-01`; another Session in the same directory therefore targets the same
file. Distinct Session JSON filenames do not protect the audio. Bus prints can
contain audio that cannot be reconstructed from the original source.

The diagnostic used a temporary directory, two newly generated eight-second
48 kHz mono WAVs (constant levels 0.15 and -0.4), embedded valid Source Graphs,
and real content hashes. Each Session used the public `queue_capture_bar`,
`commit_ready_actions` and `save` paths. Session A initially restored with its
capture `Loaded`. After Session B captured and saved, the probe compared A's
original bytes and restored A again:

```text
same_path=true overwritten=true first_session_status=Some(Changed)
```

This is direct reproduction of data replacement, not a hypothetical malicious
file race. Temporary generated audio was removed by the probe's `TempDir`.
The local diagnostic source/log are `/tmp/riotbox-1499-probe.xX5WBd/src/main.rs`
and `/tmp/riotbox-1499-collision-probe.log`; they are not durable dependencies.
RIOTBOX-1500 must turn the sequence into an in-repository regression asserting
unchanged A bytes and successful restore of both Sessions, cover the shared
bus-print writer, and prevent clobbering on failure as well as success. Existing
legacy locators must remain readable. A hash rebaseline after replacement is
not a fix. No implementation change is included in this review branch.

## System model and ownership

| Owner | Boundary and durable responsibility |
| --- | --- |
| Core | Source Graph, Session, Action/queue/commit/replay, typed provenance and identities; no App/Audio dependency |
| App | CLI entry, `JamAppState` facade, control-side orchestration, filesystem/cache hydration and action side effects |
| Audio | Core-dependent render state, WAV codec, device callback and bounded publication handoff; no Sidecar/App calls |
| Sidecar | Core-dependent framed request/response client and analysis process; outside realtime |

The executable entry delegates to `cli::run`. Source analysis becomes a Graph;
committed actions change the Session; App materializes durable artifacts and
publishes control state; Audio renders it. Restore must bind metadata to the
same artifact bytes before activating audio. F1 breaks artifact ownership at
the filename/publication seam, even though the later identity check works.

## High-risk boundaries inspected

- **External Graph transaction/recovery:** App `persistence/graph_transaction.rs`
  and Core `persistence/graph_generation.rs` preserve immutable generations,
  validate exact hash-bound recovery, and publish the Session last. Recovery
  does not scan for an arbitrary newer generation. Hardlink support is an
  explicit storage condition; no power-loss/fsync or filesystem-matrix claim.
- **Original/capture identity:** original-source restore and
  `capture_identity.rs` decode and hash one byte buffer. Capture restore refuses
  changed or legacy-unverified content rather than silently trusting a path.
  Capture writer, hydration, bus-print inputs and commit consumers were traced
  far enough to reproduce F1. Migration attests current bytes, not historical
  originality; no contradictory claim was found in the sampled contract.
- **Restore history:** the RIOTBOX-1491 App history validator and Core replay
  indexing preserve durable Session truth and first-record/diagnostic behavior.
  Indexes are transient implementation details, not a second replay model.
- **W-30 publication:** RIOTBOX-1412 gives the shared state/cache a semantic
  module owner. The actual callback borrows a cached complete revision, while
  refreshing callback-owned timing separately. Stable revision checks and
  bounded retries retain the last complete payload on a racing writer.
- **Other realtime seams (sampled):** Source Monitor's ArcSwap snapshots retain
  callback-held old payloads on the control side; live-master capture writes
  into a preallocated bounded sample array. The sampled callback path does not
  call analysis, filesystem I/O or model services. This is not a full realtime
  certification of every callback contributor.
- **Sidecar (sampled):** response request IDs and operation-specific timeout
  policy remain distinct. RIOTBOX-1498 fixes fixture startup accounting only;
  production timeout behavior is unchanged. Reader resource limits and every
  Python provider were not exhaustively audited.

## Existing risks retained, not duplicated

| Existing ticket | Revalidated boundary | Disposition |
| --- | --- | --- |
| RIOTBOX-1409 | String relationship endpoints in `source_graph/model_and_helpers.rs`; string `SourceRef.decode_profile` versus Graph's typed profile | Existing typed-contract follow-up; no new reproduced failure |
| RIOTBOX-1414 | Core `live_performance_policy.rs:12` imports a view helper; App commit dispatch invokes multiple lane handlers | Existing ownership/dispatch follow-up; no speculative rewrite |
| RIOTBOX-1415 | `tr909_tail_telemetry.rs:292` uses a poison-sensitive mutex for stream errors | Existing robustness follow-up; distinguish error handling from normal render callback |
| RIOTBOX-1337 | `cli.rs` composes many lexical `include!` fragments | Existing semantic-module migration; file splitting alone is not completion |

These remain worthwhile, but none supersedes the reproduced ordinary-workflow
capture data replacement. New persistence work should deepen one shared
artifact-publication owner rather than add another app-local identity model.

## Coverage, verification and limits

Reviewed workspace/crate dependencies, entry point, the named production files,
their relevant tests and the Session/replay/audio/module contracts. Earlier
slice evidence remains in the RIOTBOX-1491, RIOTBOX-1412 and RIOTBOX-1498 review
artifacts; it is not presented as new hardware or listening evidence here.
The new public-API probe ran successfully and asserted the existing defect.

Not performed: whole-repository line coverage, DAW interoperability, real-device
xrun measurements, Windows/filesystem matrix, human playback, source catalog
qualification, Holdout access or commercial-reference access. No frozen
musical algorithm or threshold was changed. All new audio was synthetic and
temporary. TUI interaction behavior and the full export/provider surface were
outside this bounded audit.

Solo correctness, ownership, test-evidence and workflow lenses produced F1;
disposition: tracked in RIOTBOX-1500 for the next implementation slice. The
follow-up self-review of this documentation-only diff found no additional
issue. This report records risk, not a musical or release pass.

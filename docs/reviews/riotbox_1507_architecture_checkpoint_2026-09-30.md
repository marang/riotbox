# RIOTBOX-1507 — Post-hardening and semantic-module checkpoint

Date: 2026-09-30
Classification: maintenance/regression; solo risk-directed `review-codebase`.
Reviewed feature baseline: `847d700e98d96e3868dfa7cf40a0b9a2eeda1fee`.
Integrated production baseline: `e54b644e546bfcada10c9a5c6897fd4125c7bd20`
(RIOTBOX-1411 squash merge, PR #1555). Code, scripts and Cargo manifests/lock
are identical between these two commits (`git diff --exit-code` verified).

This checkpoint follows five substantive architecture/contract slices after
RIOTBOX-1503: original-source admission (1505), Sidecar deadlines (1504), shared
Hook/Chop evidence (1502), semantic CLI ownership (1337) and semantic UI/Jam
regression ownership (1411). The module migrations materially replace lexical
ownership, so they advance the cadence; fixture-only 1506 and archive branches
do not. This is current-state sampling, not another review of the last diff,
an exhaustive audit, or independent reviewer approval.

## Findings and priorities

No new evidence-backed P0–P3 defect survives this checkpoint. The sampled system
retains the Core/Session spine and realtime/control-side boundary. The two
demonstrated findings from RIOTBOX-1503 are fixed and pass fresh bounded
regressions; they are not merely deferred or reclassified.

No rewrite or new product-state owner is justified by this evidence. Continue
with an existing autonomous maintenance slice such as RIOTBOX-1340's explicit
Audio-runtime imports, preserving callback behavior and production scope. Do
not promote file length or uncapped inputs into urgent findings without an
applicable operational contract and evidence. P023 source qualification,
structured listening and live DAW/host acceptance remain separate gates; this
checkpoint does not satisfy them.

## System model and authority

| Owner | Entry, data flow and failure authority |
| --- | --- |
| Core | Source Graph, typed identities, Session, queue/commit classification and replay. Its production dependencies remain serde, serde_json and sha2, not App/Audio/Sidecar. |
| App | Public CLI launch and shell presentation over `JamAppState`; materializes side effects, restores Session-bound graph/source/capture identities and prepares runtime inputs. New private CLI/UI owners do not duplicate Session truth. |
| Audio | Core-dependent source decoder and prepared callback state. File admission/decoding happen offline; callback mixing consumes prepared snapshots/buffers and records bounded capture samples/atomic telemetry. |
| Sidecar | Control-side protocol-0.1 process exchange with distinct control/analysis budgets, one pipe-I/O deadline and explicit unusable-peer cleanup. No audio-callback dependency. |

Analysis produces Source Graph evidence; committed actions change Session truth;
App materializes artifacts and projects render state; Audio renders prepared
inputs. Restore verifies exact graph generations and encoded WAV identities
before cache admission. QA metadata can reject inconsistent evidence but cannot
authenticate provenance or provide human musical approval.

## Sampled boundaries and evidence

### Input/process hardening

- `source_audio/file_io.rs` opens once, admits regular metadata from the opened
  descriptor and uses Unix nonblocking open to reject FIFOs without a producer.
  Original-source symlinks to regular files remain valid. Both direct Audio
  loading and App original-source restore use this same boundary. App
  `persistence.rs::load_source_audio_cache_for_graph` decodes and hashes the same
  returned byte buffer and never installs a cache on an identity mismatch.
- Sidecar `transport.rs::exchange/read_line` uses one absolute deadline across
  writes/flush/read, scans only newly received frame bytes, retains coalesced
  trailing frames and supports final EOF frames without a newline. Completed
  frame capacity is released. `client.rs::exchange/verify_request_id` invalidates
  malformed/desynchronized peers; complete synchronized Sidecar errors retain
  usability. `spawn_python` inherits stderr rather than leaving an undrained
  private stderr pipe. Timeout cleanup closes parent pipes and reaps the direct
  child, without detached workers or a blocking worker join.
- Fresh prior-finding regressions: App source admission **3 passed**, direct
  Audio admission **4 passed**, Sidecar deadline/framing/lifecycle **9 passed**.
  Logs: `/tmp/riotbox-1507-source-admission.log`,
  `/tmp/riotbox-1507-audio-admission.log`,
  `/tmp/riotbox-1507-sidecar-deadlines.log`. These include externally bounded
  no-producer FIFO children and warm-peer pipe backpressure. The old delayed
  FIFO writer probe was not rerun after the admission fix.

### Persistence, recovery and capture identity

- `persistence/graph_transaction.rs` keeps Session publication as the commit
  point over immutable hash-bound graph generations. A corrupt alias is
  replaceable only after checking the on-disk Session's exact generation, not
  the edited Session or a directory scan. Missing fresh output aliases preserve
  embedded-to-external conversion/relocation. Non-JSON I/O failures remain
  errors; symlink collisions and overlapping destinations are rejected.
- Fresh graph-transaction family: **16 passed**, including corrupt alias →
  restore → edit → save → reload, wrong recovery authority, injected publication
  failures and unsupported publication. Log:
  `/tmp/riotbox-1507-graph-recovery.log`. Hardlink support remains an explicit
  storage requirement, not silently replaced by an incomplete-copy fallback.
- `capture_identity.rs` writes newly allocated opaque capture files and publishes
  locators after complete writes; an old locator contributes only its directory.
  Restore uses Core-persisted complete-encoded-WAV identity and the same decoded
  buffer, with visible legacy-unverified/changed/unavailable status. Duplicate
  capture IDs invalidate the ambiguous cache instead of selecting one.
- `capture_identity_migration.rs` adopts only explicitly selected captures,
  checks every selected input before saving, rejects existing identity mismatch
  and records adopted-current provenance without inventing historical identity.
  Preview does not save; accepting current bytes does not rewrite WAVs or graphs.
- Metadata-only export hydration bypasses graph/source/capture audio admission.
  Restore history validation uses transient indexes, not another persistence
  model; typed undo and replay authority remain in the existing action history.

### Runtime, ownership and QA contracts

- `runtime/shared_transport_tr909.rs::build_silent_output_stream` allocates the
  scratch buffer before callback installation. Overflow outputs silence and
  increments diagnostics instead of resizing in the callback. Prepared lane
  snapshots feed the actual mixer, source-monitor policy and unchanged master
  limiter before capture/output/telemetry. No sampled callback path opens files,
  invokes analysis or calls a model.
- Source-monitor publication serializes control-side writers and retains old
  snapshots off the callback. Live-master capture checks allocation size, uses
  preallocated atomic sample storage and reaps retired buffers on the control
  side. Telemetry timing reads avoid the diagnostic mutex; poisoned full health
  remains observably degraded. This is inspection plus existing synthetic
  regression evidence, not allocation tracing or real-device timing proof.
- CLI/UI facades preserve public entrypoints and consume existing App/Core
  state. Ordinary semantic children replace the former shared include namespace;
  explicit imports expose dependencies and test-only fixture contracts. The
  selected includes are gone; current inventory is **94 sites / 13 owners**.
  Large cohesive existing regression/recipe families remain visible review
  signals, not defects inferred from line count.
- `hook_chop_diagnostic_contract.py` owns the existing two-reverse evidence
  floor. Suite dense/matrix/tonal and child validators share it. Missing/invalid
  evidence remains null/failing rather than omitted from minima, while valid
  zero/one evidence is preserved as failure evidence. Fresh seven fixture tests
  pass: `/tmp/riotbox-1507-hook-evidence.log`. No renderer, musical threshold or
  frozen Stage-A contract was tuned from source results.

## Verification, limits and disposition

Full source-free CI for the unchanged reviewed runtime passed under RIOTBOX-1411:
`/tmp/riotbox-1411-ci.log` (App 770, Audio 279, Core 470, Sidecar 24 library tests,
subprocess/synthetic smokes, Python/contract fixtures, formatting, tracked JSON
and strict Clippy). Fresh targeted regressions above are additional checkpoint
evidence. Native Ubuntu Rust CI and Windows Sidecar checks for that exact feature
head passed in run `36775673935`; PR #1555 merged without outstanding review
comments. This branch changes only the review record; exact native PR-head CI
and normal closeout still gate merge.

No real source, active holdout, commercial reference, device, DAW or playback
was accessed. Coverage excludes exhaustive Python provider review, decoder
fuzzing, malicious-input memory stress, real-terminal/device interaction, OS/
filesystem matrices, power-loss durability and concurrent external writers.
Original/capture regular-file reads and Sidecar frames remain uncapped; pipe
deadlines do not preempt JSON CPU work, process spawn, kernel stalls or descendant
process lifetime. Those documented limits remain verification/contract gaps,
not newly demonstrated corruption or end-to-end timeout guarantees.

No new architectural decision or constraint arose, so this routine checkpoint
does not append a Decision Log entry. A sequential documentation branch review
and short self-review check evidence totals, scope/authority, prior-finding
disposition and completion obligations. No unresolved branch finding remains.
The substantive-slice cadence resets when this checkpoint is merged and closed.

# Current-state architecture checkpoint — RIOTBOX-1568

Date: 2026-10-09. Reviewed implementation: `ab6af50201ffbbd96d3b8e03f143984393c16a7f`
(clean, synchronized `main`, PR #1670). Risk-directed current-state review,
mapped across four Rust crates, Python Sidecar and silent-host operators; not
an exhaustive repository audit or a review of every DSP/UI path. This ticket
changes only this report. Implementation repairs require separate tickets.

## Retained finding

### P2 — Product-mix export can commit bytes different from its validated handoff

Independent origin: Core/App auditor, Adversarial Implementation Reviewer lens.
Coordinator verified the complete writer, reachable callers and owning contract.
Evidence is static code-path analysis, not an executed race reproduction.

`crates/riotbox-app/src/jam_app/product_export/product_mix_export_commit.rs:201–204`
parses the proof, `:209–216` independently hashes its artifact, and `:229`
independently hashes the proof. New-bundle publication at `:250–257` subsequently
copies both paths. `copy_file_new` at `:303–317` reopens each input without
validating the copied bytes. Success then creates the receipt from the earlier
contract/hash and commits the action at `:122–177`.

Concrete counterexample: artifact A passes its expected hash, then a handoff
producer or other process edits/replaces it before the copy. Copying B succeeds,
but the receipt still claims A's identity/readiness. Changing the proof between
parse, hash and copy can similarly publish unvalidated proof bytes. This path is
reachable through the musician-facing `E` trigger (`cli/event_loop.rs:484` →
`cli/controls.rs:305`), not just a dormant private helper.

Action Lexicon §6.8 (`docs/specs/action_lexicon_spec.md:416–418`) requires successful
exported bytes to remain hash-identical to the validated proof artifact. The
existing no-overwrite behavior does not bind the different input reads together.

Minimal repair: parse/hash the same proof bytes, copy into exclusively owned
staging while checking the bytes actually written, then admit publication and
commit only when their identities agree. Preserve complete identical-bundle
idempotency, incomplete/different-bundle rejection, and cleanup ownership; never
remove unrelated/pre-existing files. Add deterministic generated mutation-between-
gates tests, including proof mutation and partial failure. No real source or DAW
is needed. Tracked as [RIOTBOX-1569](https://linear.app/riotbox/issue/RIOTBOX-1569/bind-product-mix-export-publication-to-the-exact-validated-proof-and),
Todo, Medium. Duplicate searches found no matching open repair.

No other P0–P3 findings survived this bounded sample. This is not a claim that
unsampled paths are defect-free. The missing mutation regression remains a
verification obligation of 1569, not evidence supplied by passing ordinary tests.

## System model and sampled coverage

- Manifest dependencies remain acyclic: App → Audio/Sidecar/Core; Audio and
  Sidecar → Core. Core has no application-crate dependency.
- Core owns Source Graph, Session, actions, queue/commit records, capture/export
  identities and replay. Sampled replay planning, history validation and export
  contracts; no shadow persistence/action truth was found in these paths.
- App owns orchestration and control-side publication. Sampled source ingest and
  restore, exact-generation graph recovery, capture identity/hydration, recording
  readback/rollback, queued DAW receipt selection and product-mix export.
  Runtime transport/lane projections are prepared from the product spine.
- Audio owns prepared PCM and callback projections. Sampled prepared scratch,
  borrowed SPSC readers, monitor replacement/reclamation, capture admission and
  teardown. Callback overflow explicitly silences output. Finish/abort closes
  capture admission before detachment; runtime stop drops the stream before
  shared owners disappear. I/O, analysis and Session mutations remain outside
  the sampled realtime paths.
- Rust Sidecar owns request/version correlation, bounded frames, shared pipe
  deadlines and direct-child lifecycle. Fatal framing/desynchronization invalidates
  the peer; complete synchronized source errors preserve reuse.
- Python separates frozen limits, regular descriptor reads, pure WAV decoding
  and provider graph assembly. Hash, metadata and features use the same captured
  bytes. App compares its separate Rust read with the provider hash before
  enrichment/publication; restore hashes/decodes one admitted buffer. These are
  hash-bound reads, not an atomic filesystem snapshot.
- Silent-host V2 separates environment containment, process watchdog/group
  reaping, route/lifetime identity, Pulse module cleanup, deferred signals and
  transcript validation. The public CLI remains disabled before operations,
  including the old opt-in. Generated orchestration requires an explicit owner.
  V1 records/budgets remain unchanged; observed replacement rejects and uncertain
  cleanup retains containment. No prospective real-host attempt was performed.
- Command/spec/CI routing was inspected. Existing Windows transport CI remains
  separate; no new platform scope was added. CLI/UI ownership was mapped, not
  audited screen-by-screen.

## Evidence attribution

The independent Core/App auditor executed these generated-only Rust modules:

| Focus | Passed |
|---|---:|
| Product export actions | 23 |
| Queued DAW writer receipt admission | 4 |
| Live recording | 21 |

Logs: `/tmp/riotbox-1568-core-app-{product-tests,writer-admission,recording}.log`.

The independent Audio/Sidecar auditor executed:

| Focus | Passed |
|---|---:|
| Python Sidecar, ResourceWarnings fatal | 23 |
| Generated silent-host metadata/process/version/signal fixtures | 89 |
| Rust Sidecar client | 25 |
| Exact production-callback synthetic/heap checks | 5 |
| Live-master capture lifecycle/timing | 26 |
| Source-monitor replacement/reclamation | 6 |
| Rust regular-descriptor byte admission | 9 |
| Rust↔Python encoded/container/sample admission | 3 |
| Canonical capture-admission Loom models | 2 |
| Rust PCM resource admission | 4 |

All 192 checks passed. Logs:
`/tmp/riotbox-1568-{sidecar-python,silent-host,transport,callback,capture,source-monitor,rust-file-admission,sidecar-integration,admission-loom,decode-resource}-audit.log`.
The Loom checks model capture admission, not the full runtime/triple-buffer.
Neither auditor changed repository files or ran full CI.

The coordinator independently inspected manifests, prior checkpoints, relevant
contracts, graph recovery, source/capture identity and reachable export paths.
The first source-free local `just ci` reached `decision-search-fixtures` after
passing the preceding gates, then failed because the sandbox made
`/run/user/1000/just` read-only. This is an execution-context failure, not a green
full CI result. The same source-free command passed completely when repeated
with the needed local filesystem permissions, including final strict all-target/
all-feature Clippy; no host-audio or source access was added. Logs:
`/tmp/riotbox-1568-ci.log` and `/tmp/riotbox-1568-ci-host.log`. Completion of the
repeat is verified; native PR CI remains required before merge.
The auditors' focused test counts are separate from full CI and must not be
summed as unique suite coverage.

Separate report-only branch review applied Spec And Evidence Auditor and
Product Pragmatist lenses. It checked attribution/counts against the focused
logs, the retained static trace against the writer/contract, and the disabled
host/qualification boundaries. Zero report-change P0–P3 findings were retained;
the coordinator's short self-review also found none. No Rust implementation
change or additional test execution was part of that branch review.

## Cadence, limits and next step

Since the [1563 checkpoint](riotbox_1563_current_state_cadence_2026-10-03.md),
four substantive slices landed: 1564 (queued writer receipt), 1565 (probe startup),
1566 (bounded silent-host observation) and 1567 (versioned lifecycle contract).
This checkpoint is pulled forward from the fifth-branch cadence because the
recent export/host boundary changes merit review before another slice. Report-only
maintenance does not count as a new substantive product feature.

The earlier DAW receipt mismatch is repaired: `daw_session_export_commit.rs:56–81`
rejects stale/missing queued identity before any writer publication. Its four
generated regressions passed. The sampled graph recovery→save and same-byte
source/capture integrity owners remain intact. No broad architecture rewrite is
justified by this evidence.

Known exclusions remain exclusions: power-loss/concurrent Session-writer
guarantees, aggregate/RSS and writer allocations, Python object cost, scheduling
or kernel stalls, full Sidecar process-tree containment, atomic host compare-and-
unload, physical-device/endurance, actual DAW import and human taste qualification.
They are not newly proven defects without a violated supported contract.

Finish 1568's report review/CI/merge closeout, then begin 1569 through its own
Linear-first implementation slice. The verified export integrity defect is the
next autonomous priority. A future real-host observation still requires separate
prospective authorization; the V2 disable boundary is not lifted by this review.

No real source/Holdout/commercial/capture audio, source-directory discovery,
actual host-audio metadata/services/devices/DAW access, playback, V1 attempt material
or retry was used. Generated PCM and generated process/metadata fixtures are
synthetic evidence only. No new musical, source-general, hardness, host, DAW or
release qualification is granted.

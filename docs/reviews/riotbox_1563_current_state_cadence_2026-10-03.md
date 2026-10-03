# Current-state architecture checkpoint after RIOTBOX-1563

Date: 2026-10-03. Scope: risk-directed, mapped and sampled current state of all
four Rust crates and the Python Sidecar over `abdb8492` plus RIOTBOX-1563's
worktree. Main `e6b6f67a` contains that base after PR #1665 merged. This is not an
exhaustive repository audit and does not substitute for the separate branch
review in [the implementation report](riotbox_1563_python_sample_admission_2026-10-03.md).

## Finding

### P2 — Local writer commit can target a receipt different from its queued action

Review lens: Data and contracts / correctness and resilience. Independent
origin: Core/Audio/App current-state auditor. Coordinator verified the cited
queue, writer, commit and owning contract; evidence is static, not a newly
executed queue/edit/commit regression.

`crates/riotbox-app/src/jam_app/product_export/daw_session_export_queue.rs:82–100`
pins receipt A in `ActionParams::DawSessionExport`. The local writer commit at
`product_export/daw_session_export_commit.rs:56–74` ignores that queued identity:
`write_daw_session_writer_proof_skeleton` chooses the latest receipt, and commit
attaches proof/gate evidence to `latest_daw_session_receipt_index`.

If receipt B arrives between queue and commit, the committed action still names
A while generated files and evidence can describe B. The writer's package-match
check compares artifact paths/hashes, so a later receipt carrying those same
artifact references is not excluded by that check. This violates Action Lexicon
§ export.daw_session's matching-receipt queue/commit contract. Adjacent host-import
and audible-proof commits already resolve the queued identity and have tests
where a later receipt arrives.

Proportionate repair: validate the queued receipt before any filesystem writes;
explicitly target it throughout the writer or reject a stale selection without
publication. Add generated metadata-only success/stale/missing-identity tests
proving action, receipt and proof alignment. No actual DAW is needed. Tracked as
[RIOTBOX-1564](https://linear.app/riotbox/issue/RIOTBOX-1564/bind-local-daw-writer-commits-to-the-queued-receipt-before-publishing),
Todo, Medium. This pre-existing defect is separate from the 1563 source change.

No additional P0–P3 findings were retained in this bounded sample.

## System model and sampled coverage

- Dependency direction is acyclic in the four manifests: App → Audio/Sidecar/
  Core; Audio and Sidecar → Core; Core has no application-crate dependency.
- Core owns Source Graph, Session, action/commit records, capture/export identity
  and replay. Sampled queue semantics, replay validation and transient indexes;
  no second persistence/action truth was found in these paths.
- App owns orchestration/publication: protocol readiness → Python analysis →
  same-read Rust identity/decode → graph enrichment → graph/Session publication.
  Sampled exact-generation recovery, Session commit authority, source/capture
  integrity, recording readback/hash-owned rollback and DAW proof transactions.
- Audio owns prepared PCM and callback projections. Sampled control-side
  preparation, borrowed triple-buffer readers, capture admission closure, fixed
  scratch overflow handling and teardown. File I/O, provider analysis and Session
  mutation remain outside the sampled callback paths.
- Rust Sidecar owns correlated requests, bounded framing, shared pipe deadlines
  and peer invalidation. Fatal framing/desynchronization rejects the peer;
  complete source errors preserve reuse. Python's pure policy leaf, descriptor
  reader, pure WAV decoder and provider graph assembly have separate semantic
  ownership; Core remains the durable model owner.
- Inspected command/CI/spec routing. Local/Linux CI discover Python test modules;
  existing Windows transport coverage remains separate, with no platform scope
  expansion. CLI/UI interfaces were mapped through crate ownership, not audited
  screen-by-screen or exhaustively reviewed.

## Evidence attribution

The Core/Audio/App auditor inspected current code, contracts, manifests, prior
checkpoint and graph-interruption, capture-identity, callback-publication and DAW
queue tests. No tests or runtime experiments were executed by that auditor.

The independent ingest/Sidecar auditor executed 23 Python tests, five already
compiled transport regressions and four already compiled App identity tests.
No fresh Cargo build was run by that auditor. Inspected Rust implementations,
framing-limit tests, manifests, CI and persistence contracts. Python log:
`/tmp/riotbox-1563-cadence-python.log`. The new fifth App regression and full
freshly built Rust suites belong to coordinator branch evidence, not that older
compiled-test sample.

The coordinator inspected the manifest dependency graph, prior checkpoint,
current ingest/decoder/policy paths and the retained queue/commit/writer finding.
Focused fresh builds and full CI are recorded separately in the implementation
report. The current-state auditors did not implement the code under review.

## Cadence and next steps

Since RIOTBOX-1558, substantive slices 1559 (source identity), 1560 (response
framing), 1561 (encoded reads), 1562 (container admission) and 1563 (sample
admission) reach the required fifth-branch checkpoint.

The previous checkpoint's mixed-byte ingest finding is repaired by 1559; response
framing and early encoded/sample admission now also have explicit owners and
tests. These improvements do not erase the stated limits: aggregate/RSS and
writer memory, Python object cost, scheduling/process-tree guarantees, retained
publication PCM, real-device endurance and actual DAW/host qualification remain
unproven. Known exclusions are not relabeled as new defects without evidence of
a violated contract.

Finish 1563's separate review/CI/merge closeout, then address 1564 through its own
Linear-first slice. No broader architecture rewrite is justified by this sample.
No real audio assets were accessed, no source directories discovered, and no
device, DAW or human listening session was used. The checkpoint grants no new
musical, source-general, hardness or release qualification.

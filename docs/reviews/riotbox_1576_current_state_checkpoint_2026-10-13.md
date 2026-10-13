# Current-state architecture checkpoint — RIOTBOX-1576

Date: 2026-10-13. Frozen implementation:
`fe92445e87e0f5301877da658a5d44c41599d658` (merged PR #1678).
Coordinator and Audio/Sidecar auditor verified that its tree equals the
provisional reviewed head `9774908a26da3a54eecadac54d85c4103698b16b`.
This is risk-directed sampling across four Rust crates, Python Sidecar and
silent-host operators, not an exhaustive DSP, UI or repository audit.

## Findings and dispositions

No P0–P3 defect is retained in the sampled current implementation. Independent
Core/App and Audio/Sidecar/operator audits apply Adversarial, Spec/Evidence,
Rust/API and operability lenses; the coordinator checks contracts, traces,
dependencies, test attribution and workflow/access boundaries.

- The 1568 product-mix handoff TOCTOU is repaired: one captured proof buffer
  and a hash-checked private artifact descriptor supply final verified writes.
  Mutable handoff paths are not reopened for publication. Generated mutation,
  hash mismatch, exclusive-create collision and partial-copy controls pass.
- The queued DAW writer checks its queued receipt identity before publishing;
  generated missing/stale identity controls pass. Existing graph-generation
  recovery/save and same-byte source/capture identity owners remain intact.
- The 1575 recorder control owner now borrows the facade, shares real App/Core
  queue/publication/save decisions with generated observations, and reports
  observed stopped lifecycle. Its ordering, timeout and failure controls pass;
  this does not establish real callback or host health.

One contract question was investigated, not promoted to a defect: public
`queue_product_mix_export(Some(A))` followed by a commit to B does not compare
the optional queued destination with the writer argument. `None` is supported
as a deferred location; the current `E` ingress commits directly and auto-queues
its single launch destination. The owning Action Lexicon §6.8 does not pin an
override/matching rule for the separate two-call API. Before a future consumer
relies on that rule, specify whether Some is binding or advisory and test it.
This is not an executed race, a demonstrated current `E` defect, a pending-action
restore claim, or an urgent repair ticket. No speculative refactor is proposed.

## System model and sampled boundaries

- Manifest dependencies remain acyclic: App → Audio/Sidecar/Core; Audio and
  Sidecar → Core. Core owns Source Graph, Session, actions, queue/commit records,
  capture/export identities and replay. Sampled history validation, export
  receipt admission, recording duration contracts and committed replay.
- App orchestrates ingest/restore, timing-fed commits, recording and external
  publication. Existing writers own artifact verification, receipts, Session
  save and rollback. The private recorder port varies device observations,
  existing projection operations and time, not those policies. No new persistent
  App state, parallel action system or shadow replay truth was found.
- Audio prepares callback state off the realtime path. The sampled actual
  callback uses prepared scratch, borrowed readers and atomic telemetry; capture
  observes post-limiter samples. Admission closes before detachment/finalization,
  and retired ownership protects reclamation. Source-monitor replacement and
  regular-descriptor audio admission remain bounded at their documented seams.
- Rust Sidecar owns negotiation, bounded framing, deadlines and direct-peer
  lifecycle. Fatal framing/desynchronization invalidates reuse; synchronized
  analysis errors preserve it. Python hashes, decodes and analyzes one admitted
  buffer. App checks its separate Rust read against provider identity before
  enrichment; restore hashes and decodes its same admitted bytes.
- The silent-host operator owns containment, watchdog/group cleanup, route
  lifetimes and bounded transcripts. V4 adds a fixed-capacity child-only ALSA
  observer; Python validates measurements after verified group cleanup. The
  ordinary execution CLI remains disabled and consumed phases remain consumed.
  This QA observer is not production callback/Session truth or a C product fork.

## Executed evidence and attribution

Core/App auditor's generated selections pass:

| Focus | Passed |
| --- | ---: |
| Product export actions | 29 |
| Verified copy/cleanup | 2 |
| Queued DAW receipt admission | 4 |
| Graph transaction/recovery | 16 |
| Capture identity | 5 |
| Shared recorder control | 12 |
| Core replay | 123 |
| Core queue | 8 |
| Recording duration contracts | 29 |

Logs: `/tmp/riotbox-1576-core-app-{product,copy,writer,graph,capture,recorder,replay,queue,duration-corrected}.log`.
Replay also reports one ignored informational benchmark. An initial duration
selector ran zero tests and is excluded; the corrected selection passes 29.
These selections may overlap; they are not summed as unique coverage.

Audio/Sidecar/operator auditor's 153 generated selections pass:

| Focus | Passed |
| --- | ---: |
| Exact production callback/heap controls | 5 |
| Capture reader/lifecycle | 8 |
| Actual admission-source Loom models | 2 |
| Source-monitor replacement/reclamation | 6 |
| Regular-descriptor byte admission | 13 |
| Rust Sidecar client | 25 |
| Python byte/resource/container/provider controls | 23 |
| Rust↔Python admission integration | 3 |
| Silent observation driver | 12 |
| V3 framing/ownership and V4 diagnostic/operator/version controls | 56 |

Logs: `/tmp/riotbox-1576-aa-{callback,capture-reader,admission-loom,monitor-replacement,file-admission,sidecar,python-source,sidecar-byte-integration,sidecar-wave-integration,driver,operator}.log`.
Loom models the actual admission source, not the full runtime/triple-buffer.
The coordinator inspected bounded result lines; neither auditor ran full CI or
edited implementation. The preceding 1575 full source-free local/native CI is
separate evidence on the identical implementation, not this report's PR gate.
Independent report-only review applies Spec and Evidence Auditor plus Product
Pragmatist/Adversarial Evidence lenses; zero findings survive. The coordinator's
short self-review also retains none. Report-branch source-free `just ci` passes
(`/tmp/riotbox-1576-ci.log`), including strict all-target/all-feature Clippy.

## Cadence, limits and next step

Since the 1568 checkpoint, four substantive slices landed: 1569 (byte-bound
export), 1571 (multiline ownership/V3 contract), 1574 (bounded ALSA observer/V4
contract), and 1575 (private recorder-control ownership). This risk-directed
checkpoint is pulled forward from the fifth-branch cadence to check their
combined trust/control boundaries. Operational/report-only 1570/1572 and
generated attribution-only 1573 do not inflate the count; this report-only
checkpoint does not advance it either.

No broad redesign is justified by the sampled evidence. Atomic filesystem
snapshots, power-loss/concurrent-writer guarantees, aggregate/RSS/writer memory,
Sidecar process-tree/kernel stalls, atomic host compare-and-unload, unsampled
DSP/UI paths, physical-device/endurance, actual DAW import and human taste remain
exclusions, not newly established defects. The original 1572 ALSA trigger stays
unexplained and RIOTBOX-1041 remains open.

No real source/capture/Holdout/commercial audio, source-directory discovery,
ignored prior attempt, host-audio/service/device observation or launch, DAW,
human playback, broad QA, phase retry or Windows expansion was performed. No
new source-general, musical, hardness, host, endurance or release qualification
or future host authorization follows. Finish independent report review, local
source-free CI and native PR/merge/main/Linear closeout before completion; exact
gate results belong in Linear and the project journal.

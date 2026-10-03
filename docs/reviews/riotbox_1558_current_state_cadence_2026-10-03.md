# Current-state architecture checkpoint after RIOTBOX-1558

Date: 2026-10-03. Scope: risk-directed, mapped and sampled current state of
the four Rust crates and Python sidecar, over `170a88a6a24ce97046cbbdf130851e097422b3de`
plus RIOTBOX-1558's working tree. This is not an exhaustive repository audit or
a substitute for its separate branch review.

## Finding

### P2 — Source ingest can persist mixed-byte analysis evidence

Review lens: Adversarial implementation / provenance and trust boundaries.
Independent origin: current-state audit agent; coordinator verified the exact
code paths. Evidence is static, not an executed source-replacement reproduction.

`python/sidecar/json_stdio_sidecar.py:439–445` reads/hash-identifies the WAV,
then opens the path again for decoding. App
`crates/riotbox-app/src/jam_app/live_source_timing.rs:54–66` opens it again for
Rust timing analysis without matching that read against the graph content hash.
`crates/riotbox-app/src/jam_app/persistence.rs:193–200` publishes the graph and
Session before its subsequent restore integrity gate.

An atomic source replacement between completed reads can therefore bind the
first file's identity to another file's analysis/timing/hook evidence and persist
it. A currently mismatched source may be rejected during restore, but that is
after publication. If the original bytes return, the hash gate accepts them
without detecting previously mixed derived evidence. This is pre-existing,
not a regression introduced by RIOTBOX-1558 or proof that the restore identity
fix was wrong.

Proportionate repair: decode the same bytes Python hashed; require Rust's
bounded same-read hash/decode to agree before graph mutation/publication.
Generated substitution regressions must preserve the previous saved pair and
unchanged-source ingest. Tracked as
[RIOTBOX-1559](https://linear.app/riotbox/issue/RIOTBOX-1559/bind-python-and-rust-source-ingest-evidence-to-one-verified-byte),
`Todo`, Medium. It is not implemented as part of this approved three-task batch.

No P0/P1 or additional P2/P3 finding was retained in the bounded sample.

## System model and coverage

- Core owns Source Graph, Session, actions/commit records and replay truth.
  Sampled replay/cursor projections, generation JSON persistence and
  recording/DAW receipt validation; new replay indexes remain transient.
- App orchestrates source ingest, audio hydration, graph/session transactions,
  recording and artifact/DAW publication. Sampled exact-generation recovery,
  same-buffer restore/capture identity, no-clobber/hash-owned rollback and
  receipt/action alignment. It does not own a second persisted product model.
- Audio owns bounded Rust WAV admission/decoding, prepared PCM and callback
  projections. Sampled runtime adapter/stop lifecycle, source/capture publication,
  capture finalization/admission and test boundaries. The production data
  callback remains isolated from Core/Session mutation and file/sidecar work.
- Rust Sidecar and Python provide correlated protocol requests/provider analysis.
  Sampled framing/provider validation/deadlines and the multi-read identity gap.
- Inspected manifests, command catalog, existing Linux/Windows CI, owning specs
  and prior checkpoint/resource-boundary reports. No platform expansion.

The source-monitor implementer performed the broader sample, so their source
publication code/extraction is not independently qualified by this audit.
Separate Rust/concurrency and source/factory/spec reviewers cover that branch
boundary; the coordinator adjudicates all reports. No Cargo or runtime checks
were executed by the current-state auditor. Root branch tests/CI are recorded in
the separate RIOTBOX-1558 evidence report.

## Cadence and residual limits

The preceding checkpoint was RIOTBOX-1551. Substantive slices RIOTBOX-1552,
1555, 1556, 1557 and 1558 reach the fifth-branch cadence. Host-only evidence
1553 and mechanical summary correction 1554 do not advance that counter.

Known limits are not relabeled as findings or covered by this slice: Python
allocations before Rust admission, uncapped sidecar response size, aggregate
caches/process memory, writer/export memory, retained publication-slot PCM,
and DAW host qualification. No universal latency/deadline, source-general,
physical-device/endurance, musical/human or release qualification follows.

No real Development/Holdout/commercial/source/capture assets were opened,
read, hashed, rendered, classified or played; no source-directory discovery,
device/runtime/DAW launch or human playback occurred. The repair sequence is
finish RIOTBOX-1558's separate review/CI/closeout, then consider the bounded
RIOTBOX-1559 identity follow-up under a new Linear-first slice.

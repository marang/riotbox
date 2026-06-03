# Riotbox Session File Spec

Version: 0.1  
Status: Draft  
Audience: realtime, session, TUI, Ghost, QA

---

## 1. Purpose

This document defines the Riotbox session format.

It exists so that:

- a session can be saved and restored deterministically
- action history, source analysis, and live state remain coherent
- Ghost, TUI, and device lanes share the same persistence model
- future migrations happen intentionally instead of by accident

---

## 2. Core Rule

A Riotbox session is not just a preset dump.

It is the minimum complete state needed to restore:

- source references
- analysis references
- action history
- live configuration
- capture lineage needed for replay-safe continuation

If a state change matters to musical behavior after reload, it belongs in the session model or in a referenced artifact.

---

## 3. Session Goals

- deterministic restore
- explicit versioning
- replay-safe action history
- compact enough for routine save/load
- resilient to partial provider upgrades
- compatible with future migrations

---

## 4. Top-Level Shape

Canonical shape:

```text
SessionFile {
  session_version
  session_id
  created_at
  updated_at
  app_version
  source_refs
  source_graph_refs
  runtime_state
  action_log
  snapshots
  captures
  ghost_state
  notes
}
```

---

## 5. Session Identity

Required fields:

- `session_version`
- `session_id`
- `created_at`
- `updated_at`
- `app_version`

Rules:

- `session_version` governs migration
- `session_id` must be stable for the life of the session
- `app_version` records the writing version but does not replace explicit schema versioning

---

## 6. Source References

MVP note:

- Riotbox MVP currently supports exactly one active source per session
- `source_refs` remains plural in the schema so later migrations can grow without rewriting the top-level shape
- app/runtime code should reject sessions that silently contain multiple active sources during MVP

```text
SourceRef {
  source_id
  path_hint
  content_hash
  duration_sec
  decode_profile
}
```

Rules:

- session restore should prefer `content_hash` verification over path trust
- `path_hint` may help the UI, but path alone must not be the authority

---

## 7. Source Graph References

The session does not need to inline every raw analysis payload.

It must preserve enough to restore or validate the graph used by the session.

```text
SourceGraphRef {
  source_id
  graph_version
  graph_hash
  storage_mode
  embedded_graph
  external_path
  provenance
}
```

Allowed `storage_mode` values:

- `embedded`
- `external`

MVP preference:

- start with `embedded` unless graph size becomes a real problem

MVP note:

- Riotbox MVP currently supports exactly one active source graph reference per session
- that graph reference must match the single active `source_ref`
- plural shape is retained for forward compatibility, not to imply current multi-source support in the app/runtime

---

## 8. Runtime State

`runtime_state` is the live musical state that must exist after load even before replay begins.

```text
RuntimeState {
  transport
  source_monitor
  source_timing
  capture
  macro_state
  lane_state
  mixer_state
  scene_state
  lock_state
  pending_policy
  undo_state
}
```

### 8.1 Transport

- tempo basis
- playback position if persisted
- current scene

### 8.1.1 Source monitor

The source monitor mode is musical runtime state and must be restorable:

- `source`: decoded source only
- `blend`: decoded source plus Riotbox lanes
- `riotbox`: generated / performed Riotbox lanes only

If the monitor mode changes what the user hears after reload, it belongs in the
session and replay path, not in app-local UI state.

### 8.1.2 Source grid trust

User-confirmed timing trust must be represented explicitly:

- `source_timing.confirmed_grid.source_id`
- `source_timing.confirmed_grid.hypothesis_id`
- `source_timing.confirmed_grid.confirmed_by_action`
- `source_timing.confirmed_grid.confirmed_at`
- enough provenance to distinguish analyzed confidence from user acceptance
- undo / revert consequence

The session must preserve the original Source Graph evidence. Confirming a grid
does not silently rewrite the analysis payload.

Source-window consumers must read this session trust state through the shared
consumer readiness contract. A source graph that still requires manual
confirmation must not become a bar-accurate capture / W-30 reuse source merely
because it has a BPM estimate; the matching `confirmed_grid` state is what
turns that same analyzed timing into user-accepted source-window truth.

### 8.1.3 Capture intent

Capture length is musician-facing runtime state, not app-local UI state:

- `capture.length_intent`
- `capture.length_set_by_action`
- `capture.length_set_at`

Allowed length intents are `one_beat`, `one_bar`, `four_bars`, and `phrase`;
`four_bars` is the default. The commit path for source-window capture reads this
state when the capture action does not carry explicit bars, and replay must apply
`capture.set_length` before later capture actions that depend on it.

### 8.2 Macro state

Examples:

- source retain
- chaos
- 202 touch
- W-30 grit
- 909 slam

### 8.3 Lane state

Per lane, store only the state required to reproduce behavior:

- MC-202 role, phrase references, and committed phrase variant
- W-30 preview intent plus bank/pad focus and current capture reference
- TR-909 takeover, pattern, and reinforcement state

MC-202 role and phrase-variant fields use stable compatibility labels in
Session v1 persisted JSON. Current examples include `leader`, `follower`,
`answer`, `pressure`, `instigator`, and `mutated_drive`.

These labels are behavior-relevant persisted contract values, not ad-hoc
implementation strings. Session v1 intentionally keeps the compatibility-label
JSON shape for MC-202 lane state and MC-202 undo snapshots, while the in-memory
session model stores MC-202 role state as `Mc202RoleState`. Behavior consumers
must use typed core helpers such as `Mc202RoleState`, `Mc202PhraseIntentState`,
and `Mc202PhraseVariantState` before making queue, replay, render, observer, or
QA decisions.

Rules:

- old Session v1 JSON must continue to restore and replay
- unknown MC-202 role or phrase labels must reject or degrade explicitly at typed
  conversion boundaries
- raw label strings may be used for persisted JSON, display text, and artifact
  names, but not as unreviewed behavior branches
- no MC-202 Session v1 JSON shape migration is planned
- any future typed-field JSON migration must be part of a documented
  session-version migration with legacy fixture load, roundtrip, restore, replay,
  undo snapshot, TUI/observer label, and audio-output proof where applicable

The migration boundary is documented in
`docs/reviews/mc202_typed_contract_migration_plan_2026-05-10.md`.

### 8.4 Mixer state

- per-bus levels
- sends
- mute / solo if relevant
- profile flags that alter render behavior

### 8.5 Scene state

- active scene
- scene list or scene references
- restore pointers

### 8.6 Lock state

- locked objects
- lock owner or actor if relevant

### 8.7 Undo state

Undo state stores bounded restore snapshots for committed moves whose audible state must roll back.

Current MVP use:

- MC-202 commit-time lane snapshots keyed by action id
- previous role, phrase reference, phrase variant, and touch

Rules:

- snapshots must be explicit session state, not callback-local memory
- snapshots are only for undo restore, not a second arrangement or phrase system
- undo must refresh the typed render projection after applying a snapshot

### 8.8 Export receipts

Product export actions create explicit export receipts instead of hiding
file-writing truth in app-local state.

Minimum receipt fields:

- receipt id
- created by action id
- created at timestamp
- export scope, distinct from export role; current P016 receipts use
  `product_mix`
- pack id / render recipe identity; current P016 receipts use
  `feral-grid-demo`
- export role
- export boundary
- artifact path or artifact URI
- proof path
- optional manifest path
- export hash
- normalized manifest hash
- `artifact_set[]` entries with role, local path or URI, media type, sha256,
  optional normalized manifest hash, optional source graph ref, source capture
  refs, capture-lineage refs, optional fallback/source comparison evidence,
  optional audio metrics, and optional sample rate, channel count, and duration
  milliseconds when known
- readiness status
- unsupported-scope flags

Rules:

- export receipts describe completed side effects; they are not musical undo
  snapshots
- replay may validate receipt metadata or report missing artifacts, but must not
  blindly rewrite files without an explicit export request
- replay validation treats optional per-artifact lineage as backward-compatible
  when absent, but rejects present `source_graph_ref` or `timing_grid_ref`
  values whose source id, graph hash, or present hypothesis id is blank
- recovery / hydration preflight must validate local-path entries from
  `artifact_set[]` as well as the legacy product-mix artifact/proof fields;
  URI entries are identity-only until a fetch/cache contract exists
- future stem-package manifest and proof entries use the same recovery /
  hydration preflight rule: local `export_manifest` and proof JSON
  `artifact_set[]` entries must exist as files, while URI manifest/proof
  entries remain identity-only until a fetch/cache contract exists. This is
  preflight evidence only and does not make `export.stem_package` runnable.
- the first P016 receipt boundary is `full_grid_mix` from the deterministic
  Feral-grid generated-support export proof
- current `export.product_mix` receipts populate artifact-set entries for the
  full-grid WAV and the copied product-export proof JSON while preserving the
  legacy `artifact_path`, `proof_path`, `export_hash`, and
  `normalized_manifest_hash` fields for older readers
- current `export.product_mix` proof entries use role `product_export_proof`,
  media type `json`, local proof path, and SHA-256 of the copied proof file so
  the receipt pack can reason about all written files through `artifact_set[]`
- `ExportArtifactSetEntry::export_manifest` is the typed local JSON identity
  helper for export manifest files; `ExportArtifactSetEntry::stem_package_proof`
  is the typed local JSON identity helper for future stem-package proof files
  using the existing product-export proof role. These helpers only build
  receipt artifact identity; they do not write files or claim a runnable stem
  export.
- Receipt-side manifest/proof JSON artifact entries keep the written file
  SHA-256 values. The in-memory `StemPackageManifest` and `StemPackageProof`
  payloads deliberately copy only the JSON role, location, and media type for
  their embedded manifest/proof identities. They do not embed the manifest JSON
  file hash or proof JSON file hash, because those hashes are produced from the
  final written payload bytes.
- `riotbox-core::stem_package_manifest::StemPackageManifest` reserves the
  future stem-package manifest contract with schema id
  `riotbox.stem_package_manifest` and schema version `1`. It stores package id,
  `export_scope: stem_package`, receipt id, creating action id, claimed stem
  roles, one typed WAV artifact per claimed stem role, explicit manifest JSON
  identity, explicit proof JSON identity, and receipt QA gate summaries.
- The stem-package manifest constructor rejects blank package ids, empty or
  non-stem role claims, missing / duplicate / unclaimed stem artifacts, blank
  artifact locations or hashes, non-WAV stem artifacts, wrong manifest/proof
  roles, and non-JSON manifest/proof identities. This is a schema/identity
  contract for future package writers; it does not create files, mutate
  Session, or make `export.stem_package` runnable.
- `StemPackageManifest::from_receipt` builds that manifest value from
  `ExportReceiptState` only when `export_scope: stem_package`. It reads claimed
  stem roles from the `stem_package_artifact_set_evidence` receipt gate,
  requires one stem WAV artifact per claimed role, requires exactly one
  `export_manifest` JSON identity and one proof JSON identity in
  `artifact_set[]`, and preserves receipt QA gate summaries. The helper does
  not embed receipt-side JSON file hashes into the manifest payload, write a
  manifest file, rewrite receipt state, or infer package metadata from app-local
  state.
- `StemPackageManifest::normalized_json_bytes` returns deterministic in-memory
  pretty JSON bytes for future proof hashing. It prepares a stable proof input
  and must not be treated as a package writer, filesystem side effect, or
  completed stem export.
- `StemPackageManifest::normalized_json_sha256` computes the deterministic
  SHA-256 identity of those normalized bytes. It is a proof identity helper for
  future manifest/proof artifact wiring, not a second serializer and not a
  package writer. Because embedded manifest/proof JSON identities omit their own
  file hashes, this hash is non-circular and can be used by
  `StemPackageProof.manifest_sha256`.
- `riotbox-core::stem_package_proof::StemPackageProof` reserves the future
  stem-package proof JSON contract with schema id `riotbox.stem_package_proof`
  and schema version `1`. It stores package id, `export_scope: stem_package`,
  receipt id, creating action id, manifest SHA-256, claimed stem roles, and
  manifest/proof JSON identities. This is an in-memory proof payload contract
  only; it does not write proof files, mutate Session, or make
  `export.stem_package` runnable.
- `StemPackageProof::from_manifest` is the current CI-safe bridge from the
  typed manifest value to the typed proof value. It derives fields from
  `StemPackageManifest` and calls `normalized_json_sha256` for the manifest
  identity; it does not reserialize through a second proof path and does not
  write files. The proof payload also omits its own eventual proof-file SHA;
  that file hash belongs in the receipt `artifact_set[]` after writing.
- The current CI-safe stem-package manifest fixture builds an in-memory receipt
  with claimed drums and bass stems, manifest/proof JSON identities, and a
  deferred `stem_package_artifact_set_evidence` gate. It roundtrips the
  manifest JSON, derives and roundtrips the proof JSON payload, and asserts
  receipt readiness remains blocked, so it proves contract wiring without
  claiming full stem export readiness.
- Future stem-package writer planning contract:
  - first allowed writer boundary: `stem_package.local_ci_package_v1`. It is a
    future app-side side-effect boundary for an explicit Session export request,
    not a currently runnable `export.stem_package` command. The source of stems
    is a declared set of deterministic offline stem render providers rooted in
    Session/Core truth; the first implementation boundary should start with
    proven drums/bass roles and reject any claimed role without an implemented
    renderer, lineage source, metrics path, and fallback-comparison proof.
  - destination layout: write through a staging directory under the requested
    local export destination and promote only after validation. Final receipt
    identities use `stem_package/stems/<stem_role>.wav`,
    `stem_package/stem_package_manifest.json`, and
    `stem_package/stem_package_proof.json`; Session stores final local paths,
    media types, hashes, metrics, lineage, fallback evidence, and QA gates, not
    temp-file mechanics.
  - reusable product-export pieces: explicit side-effect action commit after
    success, local destination directory handling, artifact SHA-256 helpers,
    copied proof/artifact identity in `artifact_set[]`, source graph and
    timing-grid receipt attachment patterns, audio-metrics attachment after a
    local WAV can be decoded, recovery preflight for local artifact-set paths,
    and observer projection from Session receipts rather than app-local export
    state
  - new stem-package pieces: per-stem render recipe/boundary, one WAV artifact
    per claimed stem role, per-stem source/capture/capture-lineage evidence,
    per-stem source-vs-fallback comparison evidence, manifest JSON file
    emission, proof JSON file emission, package id / render profile identity
    beyond the current `feral-grid-demo` product proof, and receipt construction
    that can validate all stem-package QA gates before commit
  - writer gate order: validate params and claimed roles; render/write stems
    outside realtime audio; decode/measure written WAVs; hash each stem; attach
    lineage and fallback evidence from Session/Core truth; build the receipt
    draft, manifest payload, and proof payload; write and hash manifest/proof
    JSON; validate receipt QA gates; then commit the action, commit record, and
    receipt together
  - unsupported-scope rule: the draft may carry `unsupported_scopes:
    [stem_package]` while writer evidence is incomplete, but the committed
    ready receipt must remove that flag only after every required stem-package
    QA gate records `passed`. A failed writer must not leave behind a receipt
    that `validate_stem_package_receipt_readiness` can report as ready.
  - current skeleton boundary:
    `riotbox-core::stem_package_writer::plan_stem_package_local_ci_package`
    plans final local artifact identities for `stem_package.local_ci_package_v1`
    without constructing `ExportReceiptState`, writing files, hashing artifacts,
    or removing the stem-package unsupported-scope flag. It currently supports
    only the bounded drums/bass role set and rejects unsupported claims before
    side effects.
  - current CI writer proof:
    `riotbox-app::jam_app::stem_package_writer` exercises the first bounded
    file-emission path for `stem_package.local_ci_package_v1`. The proof writes
    deterministic drums/bass fixture WAVs in a staging directory, promotes only
    the validated package layout, measures and hashes final artifacts, writes
    manifest/proof JSON, and records a ready receipt only after the required
    stem-package QA gates pass. This receipt shape is proof evidence, not a
    general user-facing export command.
  - realtime boundary: the writer must not run blocking filesystem work,
    decoding, hashing, QA analysis, Ghost/model calls, or observer emission on
    the realtime audio callback
  - replay/restore boundary: replay and recovery may validate receipt metadata
    and local artifact availability, but must not regenerate stems, rewrite
    packages, or mutate missing files without a fresh explicit export action
  - manifest/proof identity rule: receipt `artifact_set[]` entries own the
    written manifest/proof JSON file hashes; manifest/proof payload identities
    own only JSON role, location, and media type. The writer must preserve that
    boundary so manifest/proof hashes are computed from final payload bytes
    without a self-hash cycle.
- current `export.product_mix` artifact-set entries also carry the same
  normalized manifest hash as per-artifact evidence, while older artifact-set
  entries default that field to absent
- current `export.product_mix` artifact-set entries attach the active
  `source_graph_ref` when the Session has one, preserving source id, graph
  version, and graph hash without duplicating embedded graph storage in the
  receipt
- current `export.product_mix` artifact-set entries attach the confirmed
  `timing_grid_ref` when `runtime_state.source_timing.confirmed_grid` exists,
  preserving source id, optional hypothesis id, confirming action, and
  confirmation timestamp without claiming arrangement placement ranges
- current `export.product_mix` artifact-set entries attach deterministic
  `audio_metrics` plus sample rate, channel count, and duration when the
  written full-grid local artifact can be decoded as PCM WAV outside realtime
  audio; older or unreadable receipts keep those fields absent rather than
  becoming incompatible
- current `export.product_mix` receipts explicitly store `export_scope:
  product_mix` so later stem package, live recording, and DAW session work does
  not infer export scope from the full-grid mix role
- `export_scope: stem_package` is a reserved typed receipt value for future
  stem-package receipts. Its presence alone does not remove
  `unsupported_scopes[]`, does not make `export.stem_package` runnable, and
  does not claim readiness without the required artifact-set and QA gates.
- current `export.product_mix` receipts explicitly store `pack_id:
  feral-grid-demo` from the product-export proof so replay and observer
  surfaces retain the deterministic recipe identity that produced the artifact
- current `export.product_mix` receipts populate `qa_gates[]` with
  `product_export_reproducibility_smoke: passed` for the `full_grid_mix`
  artifact role after the product-export proof and artifact hash gate accepts
  the export
- receipt QA gate status values are typed as `passed`, `failed`, or `deferred`;
  a `deferred` gate is inspectable evidence that still blocks readiness
  acceptance
- stem-package artifact-set QA reports may be recorded as
  `stem_package_artifact_set_evidence` receipt gates with the claimed stem
  roles and a concise summary; a structurally accepted skeleton with deferred
  audio or fallback proof records `deferred`, while structural evidence
  failures record `failed`
- per-stem hash-stability QA reports may be recorded as
  `stem_package_per_stem_hash_stability` receipt gates. This gate validates
  that every claimed stem has exactly one nonblank artifact SHA-256 identity,
  but records `deferred` while repeated package-writer/render hash comparison
  is still aspirational. Missing, duplicate, non-stem, or hashless claimed
  stems record `failed`.
- per-stem non-silence QA reports may be recorded as
  `stem_package_per_stem_non_silence` receipt gates. This gate records
  `passed` only when every claimed stem has metrics proving audible activity,
  `deferred` when metrics are missing, and `failed` when metrics prove silence,
  cannot prove activity, or claimed stem roles are missing/duplicated/non-stem.
- per-stem lineage QA reports may be recorded as
  `stem_package_per_stem_lineage` receipt gates. This gate records `passed`
  only when every claimed stem artifact carries Session/Core lineage evidence
  through source graph, source capture, or capture-lineage refs, and `failed`
  when claimed roles are missing, duplicated, non-stem, lineage-free, or carry
  blank lineage identities. It validates receipt evidence only and does not
  write package files.
- per-stem fallback-comparison QA reports may be recorded as
  `stem_package_per_stem_fallback_comparison` receipt gates. This gate records
  `passed` only when every claimed stem artifact carries typed source-vs-
  fallback comparison evidence with a nonblank reference identity and at least
  one metric field, and `failed` when claimed roles are missing, duplicated,
  non-stem, comparison-free, blank, or metricless. It is structural receipt
  evidence only; render comparison thresholds remain separate.
- `validate_stem_package_receipt_readiness` is the typed receipt-level guard
  for future stem-package readiness. It reports `blocked` while the receipt is
  not `export_scope: stem_package`, while `unsupported_scopes[]` still includes
  `stem_package`, or while any required stem-package QA gate is missing,
  `failed`, or `deferred`. The required gate ids are
  `stem_package_artifact_set_evidence`,
  `stem_package_per_stem_hash_stability`,
  `stem_package_per_stem_non_silence`, `stem_package_per_stem_lineage`, and
  `stem_package_per_stem_fallback_comparison`. A receipt can report `ready`
  only when all of those gates are present with `passed` status and the
  unsupported scope flag has been removed.
- The CI-safe ready stem-package receipt fixture is a contract fixture only. It
  uses explicit per-stem artifact evidence, metrics, lineage, fallback
  comparison, manifest/proof JSON identities, and `passed` receipt gates to
  exercise the positive readiness path without rendering, writing, or surfacing
  a runnable stem-package export.
- The fixture may mark artifact-set and hash-stability gates as passed by
  fixture writer proof while the real package writer and repeated-render proof
  are still absent. That keeps the readiness positive path testable without
  claiming file-writing, audio proof, or listening approval.
- stem package export, live recording export, DAW export, and host-audio soak
  require later receipt fields and QA gates before they are claimed

Additional receipt fields required before wider export scopes:

- future export scope variants beyond current `product_mix` and reserved
  `stem_package`, such as `live_recording` or `daw_session`
- wider artifact set roles, source/capture lineage links, and
  stem/DAW/live-recording media roles beyond the current full-grid WAV entry
- stem-package receipts must include one artifact-set entry per claimed stem
  role (`stem_drums`, `stem_bass`, `stem_music`, `stem_vocals`), plus manifest
  or proof entries needed to verify the package; each stem entry must carry
  role, local path or URI, media type, SHA-256, sample rate, channel count,
  duration, and audio metrics when the WAV can be decoded
- stem-package manifests must mirror that receipt evidence through the typed
  schema contract rather than inferring stem identity from filenames, folder
  names, or observer-only state
- bar/beat placement ranges and richer timing placement evidence beyond the
  current confirmed grid source/action reference
- source capture refs and capture-lineage refs for source-backed stems or
  resample-derived artifacts; these refs live on each `artifact_set[]` entry,
  default empty for older receipts, and must point back to Session/Core truth
  rather than filenames, app-local state, or observer-only metadata
- stem-package receipts must record which QA gates passed, failed, or remained
  deferred, including claimed-role structure, per-stem hash stability, per-stem
  non-silence, per-stem lineage evidence, and source-vs-fallback comparison when
  required; a receipt without those gates, or with deferred gates, must not
  claim `stem_package` readiness
- observer lifecycle records for stem-package exports must be derived from the
  action log, queue/history, pending queue, and Session export receipts. The
  observer may expose readiness and gate summaries from receipts, but it must
  not infer missing package state or become a second receipt truth.
- arrangement scene refs and bar/beat ranges for arrangement or DAW packages
- new render profile or recipe ids beyond current `feral-grid-demo` so replay
  can validate which deterministic path produced the artifacts
- stronger QA gate ids and results that prove future claimed artifact roles are
  not silent, fallback-collapsed, misplaced, or hash-unstable beyond the current
  product-export reproducibility smoke gate
- host-audio evidence refs for live recording only, including device/host,
  callback-gap summary, stream errors, and recording duration

These fields must remain in Session/Core models. They must not be hidden in
JamAppState or observer-only state.

---

## 9. Action Log

The session must persist the replay-relevant action history.

```text
ActionLog {
  actions[]
  commit_records[]
  replay_policy
}
```

Each action must follow the Action Lexicon contract.

Minimum stored fields:

- action ID
- actor
- command
- params
- target scope
- requested time
- quantization
- final status
- committed time if committed
- undo payload or undo reference

Commit records preserve replay-relevant musical boundary metadata separately
from human-readable result summaries:

- action ID
- boundary kind
- beat index
- bar index
- phrase index
- scene ID when known
- commit sequence within that boundary
- committed timestamp

Rules:

- uncommitted or transient UI noise does not belong in the durable log by default
- committed musical state changes do
- budget and replay logic must consume structured commit records, not parse
  result-summary strings

---

## 10. Snapshots

Snapshots provide coarse restore points in addition to action replay.

```text
Snapshot {
  snapshot_id
  created_at
  label
  action_cursor
  payload?
}

SnapshotPayload {
  payload_version
  snapshot_id
  action_cursor
  runtime_state
}
```

Rules:

- snapshots should not replace the action log
- snapshots accelerate restore and allow safer rollback
- snapshot payloads, when present, should be typed and versioned rather than
  external state dumps
- the first payload boundary should carry replay-relevant `RuntimeState` at the
  snapshot cursor and no separate action, lane, or arrangement model
- snapshot restore must validate payload version, `snapshot_id`, and
  `action_cursor` before using the payload as an anchor
- session load validation must reject present payloads whose `snapshot_id` or
  `action_cursor` does not match the owning snapshot; missing payloads remain
  valid until a snapshot-anchored restore path explicitly requires one
- snapshot payload hydration should happen before the existing replay suffix
  executor runs; the payload must not execute actions itself
- the current save path may fill missing payloads only for existing snapshots at
  the latest action cursor, because the top-level `runtime_state` represents only
  latest materialized state

---

## 11. Captures

Captured material must be representable without losing provenance.

```text
CaptureRef {
  capture_id
  type
  source_origin_refs
  source_window?
  lineage_capture_refs
  resample_generation_depth
  created_from_action
  storage_path
  assigned_target
  notes
}
```

Examples:

- W-30 pad capture
- loop promotion
- internal resample

Minimum provenance:

- source object references when available
- source audio window metadata when the capture maps directly back to the loaded source
- explicit capture-to-capture lineage when the material is internally reused
- resample generation depth for internally derived material
- generating action ID
- resulting assigned pad or bank if applicable

`source_window` is optional for backward compatibility and for captures that are derived from internal resampling rather than a direct source range. When present, it should preserve source id, start/end seconds, and start/end source frames so later raw playback can resolve audio without guessing from UI state.

For committed source-backed captures loaded from a session file, `storage_path` should be backed by a real PCM WAV artifact relative to the session file directory unless it is absolute. Artifact writing belongs to the non-realtime app commit path; the audio callback must never write capture files.

For internally printed W-30 resample captures, `storage_path` should point to the printed bus artifact rather than the source-window input artifact. Such captures should preserve input ownership through `lineage_capture_refs` and `resample_generation_depth`; `source_window` should be omitted unless the printed result is intentionally still a literal source-window copy. This keeps reload and later pad playback pointed at the exact printed audio instead of reconstructing it from source metadata.

Artifact-hydration identity boundary:

- `capture_id` is the stable session identity for the captured material
- `storage_path` is the durable audio artifact locator and must point at the exact WAV to hydrate
- `created_from_action` links the artifact to the committed action that created or printed it
- direct source-backed captures should keep `source_window` so the source range remains auditable
- internally derived W-30 captures should keep `lineage_capture_refs` and `resample_generation_depth` so ownership and derivation depth survive reload
- future replay or recovery hydration must validate this identity before using an artifact; it must not silently reconstruct a different artifact from UI state or source-window guesses

---

## 12. Ghost State

Ghost state should be persisted only to the extent that it affects deterministic continuation or user trust.

```text
GhostState {
  mode
  budget_state
  suggestion_history
  lock_awareness_state
}
```

MVP rule:

- persist accepted or active constraints
- Watch-mode suggestions are read-only proposal objects until a later Assist flow accepts them.
- Session `suggestion_history` should store compact user-facing records for continuity, not raw Ghost reasoning or queue/action objects.
- do not persist opaque internal deliberation blobs

---

## 13. Notes and User Metadata

Optional user-facing notes may include:

- session title
- tags
- short comments
- favorite captures

These are secondary and must never replace core replay state.

---

## 14. Versioning and Migration

Versioning is mandatory from the first saveable session.

Rules:

- every session file must carry an explicit schema version
- breaking changes require migration code or explicit incompatibility handling
- graph and action schema versions must be validated during load

If load detects mismatch, Riotbox should:

- explain the mismatch
- offer degraded load where safe
- refuse silent corruption

---

## 15. Save Rules

Save must be:

- atomic enough to avoid half-written sessions
- explicit about embedded vs external artifacts
- stable under repeated writes

MVP expectation:

- one session save path
- one clear artifact layout
- no hidden autosave complexity before the format is stable
- current JSON saves serialize first, write beside the target, then rename into place; this is the MVP crash-safety seam, not a full multi-file transaction

---

## 16. Load Rules

Load must validate:

- schema version
- source hash compatibility when possible
- source graph compatibility
- action log readability
- referenced capture existence where required

Load may degrade only when the resulting state remains honest and usable.

Current MVP crash-recovery boundary:

- truncated or partial session JSON must fail with an explicit parse error
- load must not silently repair or replace the requested file with guessed state
- adjacent valid session files can still be loaded manually, but automatic fallback selection is not part of MVP yet

### 16.1 MVP crash recovery policy

The MVP recovery model is explicit manual recovery, not hidden automatic repair.

Current save behavior:

- session JSON is serialized before touching the target path
- the serialized payload is written to a hidden sibling temp file named like `.<target-file-name>.tmp-<pid>-<nonce>`
- the temp file is renamed over the target only after the write succeeds
- if rename fails, the temp file is best-effort removed and the existing target remains the authority

Orphan temp-file policy:

- hidden sibling files matching `.<target-file-name>.tmp-*` are treated as interrupted writes for that target
- load must not choose an orphan temp file automatically
- UI or CLI diagnostics may list orphan temp files as recovery clues, but they must be clearly marked as untrusted candidates
- deleting orphan temp files is safe only after the user has confirmed the canonical target and any desired manual backup have been inspected

Autosave policy:

- autosave is not automatic in MVP
- when autosave lands, it should use an explicit sibling name such as `session.autosave.json` or timestamped `session.autosave.<ISO-UTC>.json`
- autosave files must use the same schema validation and replay/commit-record checks as normal session files
- autosave must not overwrite the canonical session without explicit user action

Manual fallback selection:

- if the requested session fails to parse or validate, Riotbox should fail the load with the concrete path and error
- a user may explicitly retry another candidate path, such as a manual backup or future autosave file
- the app must not silently fall back to an adjacent file because that would hide which replay truth was actually loaded

First implementation seam:

- add a non-mutating recovery-candidate scanner that reports canonical target status, orphan temp files, and explicit autosave candidates
- keep the scanner separate from `load_session_json` so normal load remains deterministic and side-effect free
- project scanner results into an explicit manual recovery surface before adding any automatic recovery action

Current implementation:

- `scan_session_recovery_candidates` reports the canonical target plus matching hidden temp and autosave siblings
- candidates are parse-checked as missing, parseable session JSON, invalid session JSON, or unreadable
- the scanner is read-only and does not load, replace, delete, or choose a recovery candidate
- `JamAppState::scan_session_recovery_surface` converts scanner output into TUI/CLI-facing labels, trust levels, details, and action hints
- the recovery surface keeps `selected_candidate` empty and states that Riotbox did not choose, load, replace, or delete any candidate
- load-mode TUI may attach a recovery surface when the requested canonical session loads successfully but adjacent manual candidates exist
- the TUI recovery prompt is guidance only: it must not select, load, replace, delete, or promote a candidate
- operator-facing snapshot-payload and unsupported-suffix label guidance lives in `docs/recovery_notes.md`

---

## 17. MVP Requirements

Session v1 must support:

- one source file
- one embedded Source Graph
- macro and lane state
- action log for committed actions
- basic snapshots
- capture references
- Ghost watch / assist state sufficient for continuity

It does not yet need:

- multiple simultaneous source files
- distributed asset storage
- collaborative session metadata
- performance-take libraries

---

## 18. Validation Requirements

Required validation:

- save / load smoke tests
- deterministic restore from same session file
- action-log replay consistency
- migration guard tests
- missing-asset error-path tests

---

## 19. Open Follow-Ups

This draft should be followed by:

1. exact on-disk layout
2. session migration policy
3. snapshot frequency policy
4. recovery prompt for failed canonical loads, likely as a CLI/manual report before the full TUI can start
5. autosave strategy after the format stabilizes

# Audio QA Manifests And Artifacts

Parent: [Audio QA Workflow Spec](../audio_qa_workflow_spec.md)

---

## Pattern origin and primitive-renderer boundaries
Pack-level `source_backed: true` states that source audio, availability, or
timing participates in the tested path; it does not imply that every musical
pattern in that pack was selected from source evidence. Pattern-level
provenance remains authoritative. In particular, a source-backed pack whose
fixed Fill is timed but not selected by source evidence must still label that
Fill `primitive_renderer` and carry the matching promotion boundary.

Every musical pattern used by a listening pack, benchmark, demo, or generated
artifact must carry an explicit origin. The allowed origin labels are:

- `source_derived`: derived from Source Graph, Source Timing, capture, source
  windows, anchors, transient evidence, or section evidence
- `user_confirmed`: explicitly accepted or performed by the musician
- `primitive_renderer`: transparent engine or preset vocabulary, useful as a
  renderer/control surface but not proof of source-aware musical intelligence
- `fixture`: deterministic QA material created to exercise one specific seam
- `fallback`: degraded, non-product, or explicitly silent placeholder chosen
  because better evidence is unavailable
- `compatibility_silent`: an output slot kept for manifest/schema continuity
  while the musical implementation is intentionally silent

Generated packs must not present `primitive_renderer`, `fixture`, `fallback`, or
`compatibility_silent` output as source-derived behavior. A source showcase can
include those lanes only when the manifest and README keep the origin visible and
when source-independent support is not loud enough to mask source-backed output.
If a fixed pattern is a renderer vocabulary or preset, label it as such; if a
pattern is claimed to react to a source, prove the source relation in the
manifest.

The `fallback` label must not authorize musical replacement output on Riotbox
product paths. If trusted source-backed material is unavailable, product output
must expose unavailable / degraded state or silence rather than playing
synthetic substitute music. Fallback audio may appear only as a clearly labeled
non-product diagnostic control or compatibility artifact, and it must carry
`quality_proof: false`.

Any listening manifest that contains `pattern_origin: "primitive_renderer"`
must also include `primitive_renderer_boundary`. Two roles are valid:

- `non_product_diagnostic_control` keeps `product_output_allowed: false`
- `product_primitive_vocabulary` permits `product_output_allowed: true` only
  for a typed, versioned instrument vocabulary reached by an explicit committed
  performer gesture. Every primitive record must also carry a versioned
  primitive schema registered for product output by a schema-specific shared
  validator, versioned recipe ID, non-empty typed selection inputs, a
  JSON-pointer activation reference resolving to the committed command, action
  ID, boundary, and candidate WAV, and affected artifacts declared by the
  manifest. The schema-specific validator rejects unknown version-looking
  schemas or recipes and enforces the registered input combinations, committed
  performer command, exact RuntimeMix paths, and candidate-artifact linkage.
  Source-modulated product primitives use
  `riotbox.primitive_renderer_boundary.v2`, which distinguishes fixed recipe
  provenance (`recipe_derivation_claimed: false` and
  `pattern_selection_claimed: false`) from truthful source-responsive output
  (`source_output_modulation_claimed: true`). Its registered modulation object
  names the source feature, derived policy values, resolved render inputs, and
  every affected pressure/focus path. The boundary also records
  `source_failure_fallback: false`, the exact activation references and
  promotion target, the affected RuntimeMix paths, and the same artifact set.

Both roles keep `demo_readiness: unverified`, `quality_proof: false`, and
`promotion_blocked: true`; for product primitive vocabulary the blocked target
is `source_derived_musical_intelligence`, not basic performer-triggered product
output. The boundary's `affected_paths` are exact JSON paths to every
`primitive_renderer` origin in the manifest, while `affected_runtime_paths` and
`affected_artifacts` identify the actual live seam and WAVs. Missing or stale
primitive-boundary metadata is a manifest validation failure, because otherwise
a fixed renderer can slip into musician-facing proof as if it were source-derived
output.

Product primitive vocabulary is not a fallback exception. It must never start
automatically because source evidence is absent, and it must not be described as
source-selected, source-composed, or proof of Riotbox musical intelligence. A
source-derived pressure or timbre modulation is allowed only when declared as
such; it does not turn a fixed step recipe into source-selected composition. A
fixed drum-machine Fill explicitly committed by the musician may be a valid
instrument gesture; the same Fill silently substituted for unavailable source
material is forbidden fallback output.

## Named validator ownership
Large professional-output JSON contracts belong in named repo-local validators,
not in oversized inline `jq` blocks inside `Justfile`. `just` recipes may keep
small smoke assertions and compact negative mutations, but cross-report musical
thresholds, evidence-boundary checks, artifact existence checks, and failure-code
names should live in validator scripts so future sound-quality gates stay
reviewable without weakening proof.
The sound-quality readiness smoke follows the same rule: the `Justfile` recipe
should generate the current artifacts and call
`scripts/validate_sound_quality_readiness_smoke.py`; Markdown worklist checks
and stale/missing-context mutation fixtures belong in that validator, not as a
large inline shell block.
Weak-output routing fixtures follow the same rule:
`just weak-output-fix-routing-fixtures` should generate the routing report and
call `scripts/validate_weak_output_fix_routing_smoke.py`; routed-case checks,
production-fix candidate checks, Markdown assertions, stale-count mutations,
unknown-case/manifest fixtures, and duplicate-category rejection belong in that
validator.
Rendered weak professional-output fixtures follow the same rule:
`just rendered-weak-professional-output-fixtures` should generate the negative
diagnostic report and call
`scripts/validate_rendered_weak_professional_outputs_smoke.py`; evidence-boundary
checks, required destructive failure codes, stale-count mutations,
quality-claim rejection, and rendered-artifact checks belong in that validator.
Destructive-variation professional smoke follows the same rule:
`just destructive-variation-professional-smoke` should generate the dense-break
performance report, generate the destructive-variation report, and call
`scripts/validate_destructive_variation_professional_smoke.py`; destructive
threshold comparisons, diagnostic-boundary checks, invalid flat-stutter
failure-code checks, stale metric mutations, quality-claim rejection, and
Markdown boundary checks belong in that validator.
Professional-output listening verdict import fixtures follow the same rule:
`just professional-output-listening-verdict-import-fixtures` should call
`scripts/validate_professional_output_listening_verdict_import_fixtures.py`;
keep-verdict import checks, human-label corpus field checks, unverified-review
rejection, and stale artifact-hash rejection belong in that validator.
General listening-review label import fixtures follow the same rule:
`just listening-review-label-import-fixtures` should call
`scripts/validate_listening_review_label_import_fixtures.py`; valid label import
checks, human-label corpus validation, and missing-metadata rejection belong in
that validator.
Human-listening label corpus fixtures follow the same rule:
`just human-listening-label-corpus-fixtures` should call
`scripts/validate_human_listening_label_corpus_fixtures.py`; valid corpus
summary checks, verdict-count checks, source-family checks, and invalid-corpus
rejection belong in that validator.

The P023 professional-source WAV pack, edge-source diagnostics, non-dense
professional proof pack, dense-break performance pack, and agent musical review
pack follow the same rule: their smoke recipes must call report validators with
mutation fixtures instead of duplicating source-family coverage, human-verdict,
diagnostic-only, artifact, silence, identity-collapse, source-derived policy,
source-character, visual-review, and weak-routing checks as shell `jq`.
Passing those validators proves the diagnostic boundary and contract shape only;
it does not promote scripted professional-source, edge, non-dense, dense-break,
or agent-review renders to product-quality proof.

## 6. Output Layout

Local audio QA output should use a stable structure.

Recommended shape:

```text
artifacts/
  audio_qa/
    2026-04-18/
      tr909-smoke/
        fills_phrase_drive/
          baseline.wav
          candidate.wav
          metrics.json
          notes.md
        takeover_controlled_phrase/
          baseline.wav
          candidate.wav
          metrics.json
          notes.md
```

Every rendered case should include:

- fixture or case ID
- seed
- action list or render-state source
- baseline reference if one exists
- metrics
- optional human review notes

---


## Export, receipt, observer, and proof boundaries

The following rules were formerly embedded inside the repository-status
inventory. They are normative here. In particular, missing, duplicate, stale,
silent, hashless, lineage-free, unsupported, deferred, or failed evidence
blocks the corresponding scope; observer and report surfaces remain projections
of Session/Core truth rather than second readiness engines.

- generated-pack manifest validation can require referenced artifact and metrics files to exist via `--require-existing-artifacts`
- `just offline-render-reproducibility-smoke` is a CI-safe bounded reproducibility check that renders the same deterministic source-backed W-30 output twice and compares WAV hashes; it is an offline render smoke, not the full export workflow
- `just p011-exit-evidence-manifest` validates the current machine-checkable P011 evidence index across replay, recovery, export reproducibility, and stage-style stability categories, including proof-file existence and repo-local `just` recipe references; it is an evidence index, not an execution gate by itself
- `just p011-exit-evidence-manifest-validator-fixtures` keeps the P011 evidence-index validator honest with the live manifest plus a negative fixture for a missing `just` recipe reference
- `just p011-exit-evidence-gate` is the aggregate bounded executable category gate for the P011 evidence index. It validates the manifest, selects every category, globally deduplicates repeated proof commands, and runs them without shell expansion. It is the CI entrypoint for category-level P011 exit evidence, not a host-audio soak, full arrangement export gate, or endurance gate.
- `just p011-replay-evidence-gate` is the first bounded executable category gate for the P011 evidence index. It validates the manifest, selects the `replay` category, deduplicates that category's proof commands, and runs them without shell expansion. It is CI-safe replay evidence, not a full all-category exit gate.
- `just p011-recovery-evidence-gate` is the second bounded executable category gate for the P011 evidence index. It validates and runs the `recovery` category's proof commands, covering generated recovery observer drills and the recovery-surface test family without claiming automatic startup recovery or real interrupted-host-session rehearsal.
- `just p011-export-evidence-gate` is the third bounded executable category gate for the P011 evidence index. It validates and runs the `export_reproducibility` category's proof commands, covering stable normalized manifest data and audio artifact hashes for the current deterministic Feral grid product-export seam without claiming full arrangement export readiness.
- `just p011-stage-style-evidence-gate` is the fourth bounded executable category gate for the P011 evidence index. It validates and runs the `stage_style_stability` category's proof commands, covering generated repeated-run restore-diversity observer/audio evidence and stable full-mix hashes without claiming host-audio soak or multi-hour endurance coverage.
- `just p011-exit-evidence-category-gate-fixtures` keeps the category runner honest with a replay dry-run and a negative missing-category fixture.
- `just full-grid-export-reproducibility-smoke` / `just product-export-reproducibility-smoke` is a CI-safe bounded export reproducibility check that renders the deterministic Feral grid/source-first plus generated-support pack twice from generated source material, validates both listening manifests, rejects collapsed full-mix output metrics, compares the exported generated-support WAV hashes, and validates a normalized product-export proof with temp paths removed; it is still not the full arrangement export workflow
- `riotbox-core::export_readiness` turns the current product-export
  reproducibility proof into a typed `ExportReadinessContract` for P016. The
  first contract scope is `product_mix`; the first boundary is the
  deterministic Feral-grid generated-support export with `full_grid_mix` as the
  current export role, `feral-grid-demo` as the current pack/recipe identity,
  `reproducible` status, and explicit unsupported-scope flags for stem package
  export, live recording export, DAW export, and host-audio soak.
- The current internal stem-package local CI proof uses a separate receipt and
  package identity: `export_scope: stem_package`,
  `pack_id: stem-package-local-ci`, `export_role: package_manifest`, and
  boundary `stem_package.local_ci_package_v1`. Its readiness proof is the
  stem-package artifact-set, hash-stability, non-silence, lineage, and
  fallback-comparison gate set; it must not reuse the product-mix
  `feral-grid-demo/full_grid_mix` identity.
- `just stem-package-local-ci-report-smoke` is the CI-safe operator-report
  smoke for that proof path. It creates a temp local CI package through the CLI,
  validates the ready read-only report, removes one stem file, and validates the
  blocked missing-file report. It proves report/readiness plumbing only, not
  final DAW export UX or listening approval.
- the first export action boundary is `export.product_mix`, which writes a
  `full_grid_mix` product artifact plus proof receipt only after the existing
  product-export reproducibility proof and artifact hash check succeed. It is not a
  stem package, live recording, DAW session, host-audio capture, automatic
  arranger export, or automatic Ghost export gate.
- the observer export surface derives `requested`, `started`, `completed`, and
  `failed` lifecycle records from the existing `export.product_mix`
  ActionCommand, queue/action history, and export receipts. Completed records
  include receipt id, export scope, pack id, role, artifact/proof paths,
  hashes, the full-grid WAV artifact-set entry, the product-export proof JSON
  artifact-set entry, per-artifact normalized manifest hash evidence,
  per-artifact source graph lineage when the Session has it, per-artifact
  confirmed timing-grid lineage when the Session has it, per-artifact audio
  metrics and WAV format evidence when the written local product artifact can
  be decoded safely, QA gate id/result evidence, readiness status, and
  unsupported scopes; failed records include the action id and failure reason.
  This is an observer projection, not a second export truth.
- wider P016 export scopes require stronger gates before they are claimed:
  stem packages require per-stem non-silence, role labeling, hash stability, and
  source/capture lineage checks against the per-artifact evidence fields; live
  recordings require real-session `live_recording_host_audio_refs[]` evidence
  with host/device, callback-gap, stream-error, and duration summaries plus a
  Core/Session readiness report that blocks missing evidence, blank host/device
  identity, zero duration, callback-gap threshold breaches, stream errors, and
  still-unsupported live-recording scope flags; DAW session export requires
  tempo-map and arrangement placement validation against the Source
  Graph/Session timing truth. None of those are covered by the current
  product-export reproducibility smoke.
- live-recording observer receipt snapshots project
  `live_recording_host_audio_readiness` from the Core/Session receipt report
  alongside `live_recording_host_audio_refs[]`. This projection explains
  blocked/ready evidence states for real committed receipt lifecycles only; it
  does not make `export.live_recording` runnable or synthesize lifecycle from
  observer-only state.
- `riotbox-app --live-recording-readiness-report --session <session.json>` is a
  read-only operator report over Session receipts. It writes no files, emits no
  observer events, launches no host, captures no audio, and does not make
  `export.live_recording` runnable; it only reports whether the latest
  `export_scope: live_recording` receipt has sufficient host-audio evidence for
  the current Core/Session readiness contract.
- `just live-recording-readiness-report-smoke` exercises that report through
  the built `riotbox-app` binary for ready and blocked receipt evidence.
- `export.stem_package` remains reserved until an implementation can provide a
  package receipt whose `artifact_set[]` contains every claimed stem role, the
  package manifest/proof entries, per-stem hashes, per-stem WAV format/audio
  metrics, and the policy-required source/capture lineage and fallback
  comparison evidence. A UI, Ghost, or CLI path must not show it as ready while
  those gates are absent.
- `ExportScope::StemPackage` is no longer only a future receipt label: the
  current app has an internal `stem_package.local_ci_package_v1` commit proof for
  deterministic drums/bass CI stems. That proof may remove the stem-package
  unsupported-scope flag only when the written artifact set, per-stem hashes,
  non-silence metrics, lineage, and fallback-comparison gates pass. It still
  does not make a full musician-facing DAW/stem export workflow ready.
- `riotbox-core::session::validate_stem_package_receipt_readiness` is the
  current receipt-level guard: missing, failed, or deferred
  `stem_package_artifact_set_evidence` gates keep readiness blocked, and even a
  passed gate stays blocked while the receipt still carries the
  `stem_package` unsupported-scope flag.
- Stem-package QA gates must fail when a claimed role is missing, duplicated,
  mislabeled, hashless, locationless, silent by metrics, or missing required
  lineage/fallback evidence. Hash stability and non-silence must be checked per
  stem, not only on a package-level manifest.
- `riotbox-core::export_qa::validate_stem_package_artifact_set_evidence`
  is the current CI-safe stem-package gate skeleton. It validates only
  structure: claimed roles must be stem roles, each claimed role must have
  exactly one artifact-set entry, and each entry must carry a location plus
  sha256. When a claimed stem artifact includes audio metrics, the gate fails
  metrics that prove silence or do not contain enough activity evidence. Missing
  metrics keep per-stem non-silence deferred/aspirational. Callers may enable
  the structural lineage policy to require each claimed stem artifact to carry
  source graph, source capture, or capture-lineage evidence before a wider stem
  scope is accepted. The default gate remains compatible with current
  product-mix callers. Callers may also enable the structural fallback policy
  to require typed source-vs-fallback comparison evidence before accepting a
  claimed stem artifact. Enabled structural policies reject blank lineage
  identities, blank fallback reference identities, and fallback comparison
  payloads with no metric fields, but threshold interpretation and real render
  comparison remain deferred. Passing this skeleton does not claim a full stem
  package export.
- `riotbox-core::session::ExportReceiptQaGateResult::stem_package_artifact_set_evidence`
  records that skeleton as `stem_package_artifact_set_evidence` in receipt
  `qa_gates[]`. Structural failures become `failed`; structural acceptance with
  deferred audio/fallback proof becomes `deferred`, not `passed`, so receipts
  can explain why a stem package is blocked without claiming readiness.
- `riotbox-core::export_qa::validate_stem_package_hash_stability_evidence`
  is the current CI-safe per-stem hash identity gate. It requires one nonblank
  SHA-256 identity for each claimed stem role, fails missing / duplicate /
  hashless / non-stem claims, and records
  `stem_package_per_stem_hash_stability` in receipt `qa_gates[]`. Successful
  identity evidence remains `deferred`, not `passed`, until a package writer or
  repeated render proof can compare stable hashes across actual outputs.
- The current local CI package writer records the hash-stability gate as
  `passed` only for its deterministic repeated fixture proof boundary. Wider
  stem renderers must supply their own repeated-output hash evidence before they
  can claim the same gate.
- `riotbox-core::export_qa::validate_stem_package_non_silence_evidence`
  is the current CI-safe per-stem non-silence receipt gate. It records
  `stem_package_per_stem_non_silence` as `passed` only when every claimed stem
  has audio metrics that prove activity, `deferred` when metrics are absent,
  and `failed` when metrics prove silence, cannot prove activity, or claimed
  roles are missing, duplicated, or non-stem. It is metrics evidence, not a
  package writer or listening-pack approval.
- `riotbox-core::export_qa::validate_stem_package_lineage_evidence`
  is the current CI-safe per-stem lineage receipt gate. It records
  `stem_package_per_stem_lineage` as `passed` only when every claimed stem
  artifact carries source graph, source capture, or capture-lineage evidence
  from Session/Core receipt fields. Missing, duplicate, non-stem, lineage-free,
  or blank lineage identities record `failed`. It validates traceability
  evidence only and does not claim package writing or listening approval.
- `riotbox-core::export_qa::validate_stem_package_fallback_comparison_evidence`
  is the current CI-safe per-stem source-vs-fallback comparison receipt gate.
  It records `stem_package_per_stem_fallback_comparison` as `passed` only when
  every claimed stem artifact carries a nonblank fallback reference identity
  and at least one comparison metric field. Missing, duplicate, non-stem,
  comparison-free, blank, or metricless evidence records `failed`. It is
  structural comparison evidence only; real render thresholds remain separate.
- `riotbox-core::session::validate_stem_package_receipt_readiness` requires the
  receipt to be `export_scope: stem_package`, to have no remaining
  `stem_package` unsupported-scope flag, and to carry `passed` status for all
  required stem-package QA gates: artifact-set evidence, per-stem hash
  stability, per-stem non-silence, per-stem lineage, and per-stem fallback
  comparison. Missing, `deferred`, or `failed` gates keep readiness blocked with
  typed blockers.
- The CI-safe ready stem-package receipt fixture exercises that positive
  readiness path with explicit stem artifact identities, active audio metrics,
  lineage, fallback comparison, manifest/proof identities, and all required
  gates passed. It is a contract fixture only: no files are written, no package
  writer runs, no audio proof or listening verdict is produced, and the
  artifact-set/hash-stability gates are fixture writer-proof placeholders until
  real writer and repeated-render evidence exist.
- `riotbox-core::stem_package_manifest::StemPackageManifest` is the current
  CI-safe schema contract for future stem-package manifests. It serializes
  stable schema id/version, `export_scope: stem_package`, receipt/action
  references, claimed stem roles, typed per-stem WAV artifact identities,
  manifest JSON identity, proof JSON identity, and QA gate summaries. Its
  constructor enforces identity consistency, but it is not a writer, not a
  render path, and not evidence that the actual stem audio is non-silent or
  fallback-safe.
- `StemPackageManifest::from_receipt` is the current CI-safe bridge from
  Session receipt truth to that manifest value. It accepts only
  `export_scope: stem_package`, uses the
  `stem_package_artifact_set_evidence` gate for claimed roles, requires the
  corresponding stem WAV artifact identities plus manifest/proof JSON
  identities in `artifact_set[]`, and carries the receipt QA gate summaries
  forward. Receipt-side manifest/proof JSON file hashes stay in
  `artifact_set[]`; the manifest payload carries only JSON role, location, and
  media type for those identities. This still proves identity wiring only;
  audio non-silence, fallback-safety, and future file writing remain separate
  gates.
- `StemPackageManifest::normalized_json_bytes` is the current CI-safe proof
  input helper. It returns deterministic in-memory pretty JSON bytes and proves
  that stable manifest values serialize identically while artifact identity
  changes alter the proof input. It does not write files or claim package
  export readiness.
- `StemPackageManifest::normalized_json_sha256` is the current CI-safe proof
  identity helper. It hashes `normalized_json_bytes` directly so future
  manifest/proof artifacts can use the same deterministic identity without a
  parallel serializer. Embedded manifest/proof JSON identities omit their own
  eventual file hashes, so this manifest hash is non-circular. It does not write
  package files or claim stem export readiness.
- `riotbox-core::stem_package_proof::StemPackageProof` is the current CI-safe
  stem-package proof JSON schema contract. It records package, receipt/action,
  manifest SHA-256, claimed roles, and manifest/proof JSON identities, but it
  remains an in-memory proof payload only. It does not write files, render
  stems, or make `export.stem_package` ready for musicians.
- `StemPackageProof::from_manifest` builds that proof payload from
  `StemPackageManifest` and its `normalized_json_sha256` helper. It is proof
  identity wiring only; it does not embed the eventual proof-file SHA, write a
  proof JSON file, or claim package export readiness.
- The current stem-package manifest fixture is in-memory and CI-safe: it uses
  claimed drums and bass stems, manifest/proof identities, and deferred QA gate
  evidence, then roundtrips the manifest JSON, derives and roundtrips the proof
  JSON payload, and checks receipt readiness stays blocked. It is a contract
  fixture, not a listening pack, package writer, or proof that
  `export.stem_package` is ready for musicians.
- Future stem-package writer QA contract:
  - first allowed writer boundary: `stem_package.local_ci_package_v1`, a future
    local app-side package writer for an explicit Session export request. Its
    stem source is deterministic offline stem render providers declared by
    role; the first implementation boundary should start with roles that
    already have receipt/fixture proof, and must reject unsupported role claims
    instead of producing placeholder or fallback-only stems.
  - current code-level skeleton only plans that boundary. It validates local
    destination and bounded drums/bass claims, then returns final artifact
    identities without writing WAV/JSON files, hashing outputs, running audio
    metrics, or producing listening-review evidence.
  - current CLI dry-run exposes that plan through
    `riotbox-app --stem-package-local-ci-dry-run`. It requires an explicit
    local destination and claimed roles, reports planned artifact paths,
    supported/unsupported role claims, and readiness blockers, and must keep
    `writes_files: false`. This proves only the control-plane plan and negative
    blocking surface; it cannot satisfy stem-package audio QA gates or claim
    musician-ready export output.
  - current CLI execute exposes the bounded writer proof through
    `riotbox-app --stem-package-local-ci-execute`. It requires an explicit
    Session, local destination, and supported claimed roles, writes the
    deterministic local CI package through the committed writer/action path,
    saves the Session receipt, and can emit observer evidence. This satisfies
    the internal local CI writer proof for drums/bass only; it is still not a
    structured listening-review pack, DAW export workflow, or general
    musician-facing `export.stem_package` control.
  - current CLI operator report exposes the written proof summary through
    `riotbox-app --stem-package-local-ci-report --session <session.json>`. It is
    read-only: it reports the latest stem-package Session receipt, stem roles,
    manifest/proof identities, QA gate status, Core readiness blockers, local
    artifact availability, and missing package files without writing observer
    events, regenerating artifacts, or treating product-mix receipts as stem
    packages. This helps operators inspect the internal proof boundary; it is
    still not a listening review, DAW export workflow, or musician-facing export
    readiness claim.
  - current CI writer proof emits files for that boundary inside
    `riotbox-app::jam_app::stem_package_writer`: deterministic drums/bass WAVs
    are written through a staging directory and promoted into the final package
    layout, decoded for format and non-silence metrics, hashed from the final
    paths, represented in `artifact_set[]`, and paired with manifest/proof JSON
    hashes. A repeated fixture run proves stable per-stem hashes for identical
    inputs. No structured listening-review pack exists for this proof; it is
    CI-safe written-artifact evidence with `human_verdict: unverified`.
  - current musician-facing surface gate is separate from receipt readiness:
    a ready local CI stem-package receipt can satisfy the internal writer proof,
    per-stem QA gates, observer projection, operator report, and package
    identity, but the TUI/Ghost/user export surface remains disabled while the
    package is developer-proof-only, DAW placement is missing, or structured
    listening review is not verified. Reserved UI/Ghost-style attempts must
    reject with those blockers and must not write files or commit a receipt.
  - current arrangement / DAW placement contract skeleton reserves
    `daw_session` receipt identity and `arrangement_placement_refs[]` so future
    DAW export QA can prove scene/bar/beat placement separately from local file
    availability. It is a Session receipt contract and observer/recovery report
    surface only; it writes no DAW files and does not approve an audible DAW
    handoff.
  - current DAW tempo-map contract skeleton adds `daw_tempo_map_ref` to
    `daw_session` receipts so future DAW export QA can prove tempo-map evidence
    separately from arrangement placement and local file availability. It stores
    receipt evidence for the existing confirmed timing boundary; it writes no
    DAW tempo map and does not approve an audible DAW handoff.
  - current DAW operator report exposes the combined receipt/preflight status
    through `riotbox-app --daw-export-readiness-report --session
    <session.json>`. It is read-only: it reports the latest DAW-session receipt,
    placement readiness, tempo-map readiness, unsupported-command blockers,
    local artifact preflight, missing/unreadable files, proof-gate status for
    JSON package integrity, writer proof, host-import proof, and audible-output
    proof, and release blockers without writing DAW files, observer events, or
    musician-facing export state. `developer_proof_only` remains fixed; writer,
    host-import, and audible-output blockers clear only when their own proof
    gates pass. The report's `proof_stack` summary makes stacks explicit when
    they are complete but still developer-proof-only. A `ready_for_writer`
    report means the receipt is ready for the next DAW-writer implementation
    gate only.
  - `just daw-export-readiness-report-smoke` is the CI-safe operator-report
    proof for that path. It runs the real binary against a temporary Session,
    validates the ready-for-writer report, validates a complete
    developer-proof-only proof stack that still leaves musician-facing DAW
    export disabled, removes the manifest file, and validates the missing-file
    blocker.
  - current DAW writer plan skeleton exposes deterministic planned identities
    through `riotbox-app --daw-session-writer-plan --session <session.json>
    --daw-session-destination <dir>`. It is a dry-run only: it reuses the
    operator readiness report, reports planned arrangement manifest, tempo-map,
    and DAW-session proof JSON paths under `daw_session/`, carries placement
    refs and tempo-map refs forward, and keeps `daw_writer_missing` explicit.
    It writes no files, creates no destination directory, emits no observer
    events, and is not a musician-facing DAW export action.
  - `just daw-session-writer-plan-smoke` is the CI-safe proof for that planning
    path. It runs the real binary, validates the ready-for-writer dry-run plan,
    removes the arrangement manifest source file, validates the missing-file
    blocker, and proves the destination directory was not created.
  - current DAW-session manifest/tempo-map/proof payload contracts live in
    `riotbox-core::daw_session_manifest`,
    `riotbox-core::daw_session_tempo_map`, and
    `riotbox-core::daw_session_proof`. They provide deterministic normalized
    JSON/hash contracts for the planned DAW manifest, tempo map, and proof from
    receipt evidence, placement refs, tempo-map refs, source artifact
    identities, and planned DAW JSON identities where applicable. They are
    schema and proof-input contracts only: no DAW files are written, no Session
    is mutated, no audio output is produced, and no musician-facing DAW export
    is enabled.
  - current DAW writer plan payload preview exposes those contracts through the
    read-only CLI report. A ready preview reports planned manifest/tempo/proof
    paths, normalized manifest and tempo-map hashes, and proof/manifest hash
    linkage; a blocked preview carries typed upstream blockers and emits no
    hashes. This proves payload shape only, not DAW file export completion, DAW
    placement correctness in a host, or audible output.
  - current DAW JSON writer proof is internal and CI-safe only. It writes the
    manifest, tempo-map, and proof JSON payloads through staging, promotes the
    package, hashes final JSON files, and verifies proof/manifest linkage. This
    proves local JSON file emission only, not DAW audio output, host import
    correctness, observer lifecycle, Session mutation, or a musician-facing
    `export.daw_session` workflow.
  - current DAW JSON package execute CLI exposes that proof explicitly through
    `riotbox-app --daw-session-json-package-execute --session <session.json>
    --daw-session-destination <dir>`. It writes only the local JSON package,
    reports hashes plus the package report, and keeps Session mutation,
    observer lifecycle, host import correctness, audible output, and
    musician-facing DAW export out of scope. `just
    daw-session-json-package-execute-smoke` is the bounded proof for that
    real-binary path.
  - current DAW JSON package report reads a local `daw_session/` package
    without writing, validates expected schema ids, verifies proof/manifest hash
    linkage, reports final JSON SHA-256 values, and surfaces typed blockers for
    missing files, invalid JSON, schema mismatch, or hash mismatch. This proves
    local JSON package integrity only, not DAW host import correctness or
    audible output.
  - current DAW JSON package receipt evidence records that local package result
    in Session/Core receipt truth via `artifact_set[]` entries for
    `export_manifest`, `daw_session_tempo_map`, and `product_export_proof`, plus
    the `daw_session_json_package_integrity` QA gate. This proves receipt
    handoff of JSON package evidence only; it still does not prove DAW host
    import correctness, observer lifecycle completion, or audible output.
    `riotbox-app --daw-session-json-package-evidence-apply --session
    <session.json> --daw-session-destination <dir>` is the current real-binary
    path for that handoff, and `just
    daw-session-json-package-evidence-apply-smoke` proves the saved Session
    receipt evidence after the package execute path.
  - current DAW session surface gate keeps musician-facing DAW export disabled
    even when receipt and JSON package evidence are ready. The remaining
    blockers explicitly name developer-only status, missing DAW writer, missing
    DAW host-import proof, and missing audible-output proof so JSON package
    success is not mistaken for a playable DAW export.
  - current DAW host-import proof evidence is a reserved receipt QA gate:
    `daw_session_host_import_proof`. Missing or failed proof keeps
    `daw_host_import_proof_missing` visible; passed proof removes only that
    blocker and still does not prove audible output or make `export.daw_session`
    runnable. The gate may pass only after a passed `daw_session_writer_proof`
    exists on the same receipt; out-of-order host-import evidence records
    `daw_writer_proof_missing` and remains failed.
  - current DAW host-import proof apply path is explicit and evidence-only:
    `riotbox-app --daw-session-host-import-proof-apply --session
    <session.json> --daw-session-host-import-proof <proof.json>` reads a local
    `riotbox.daw_session_host_import_proof` JSON report and mutates only the
    latest DAW-session receipt's host-import QA gate. It is not a DAW host
    runner, not a DAW writer, not an observer lifecycle event, and not audible
    output proof.
  - current `export.daw_session` host-import-proof action boundary is
    `host_import_proof_v1`. It commits an existing local
    `riotbox.daw_session_host_import_proof` JSON report through queue/history,
    Session action log, commit record, and matching receipt evidence only after
    the same receipt has passed writer proof. It attaches only
    `daw_session_host_import_proof`, emits observer lifecycle evidence, writes
    no files, launches no host, captures no audio, and leaves
    `developer_proof_only` plus `audible_output_proof_missing` visible.
  - current DAW audible-output proof evidence is a reserved receipt QA gate:
    `daw_session_audible_output_proof`. Missing or failed proof keeps
    `audible_output_proof_missing` visible; passed proof removes only that
    blocker and still does not launch a host, capture audio, write DAW files, or
    make `export.daw_session` runnable. The gate may pass only after passed
    `daw_session_writer_proof` and `daw_session_host_import_proof` gates exist
    on the same receipt; out-of-order audible-output evidence records missing
    prerequisite blockers and remains failed.
  - current DAW audible-output proof apply path is explicit and evidence-only:
    `riotbox-app --daw-session-audible-output-proof-apply --session
    <session.json> --daw-session-audible-output-proof <proof.json>` reads a
    local `riotbox.daw_session_audible_output_proof` JSON report and mutates
    only the latest DAW-session receipt's audible-output QA gate. It is not a
    DAW writer, not a host runner, not live audio capture, and not an observer
    lifecycle event. `just daw-session-audible-output-proof-apply-smoke` proves
    the real-binary Session mutation while `export.daw_session` stays disabled.
  - current `export.daw_session` audible-output-proof action boundary is
    `audible_output_proof_v1`. It commits an existing local
    `riotbox.daw_session_audible_output_proof` JSON report through
    queue/history, Session action log, commit record, and matching receipt
    evidence only after the same receipt has passed writer proof and
    host-import proof. It attaches only `daw_session_audible_output_proof`,
    emits observer lifecycle evidence, writes no files, launches no host,
    captures no live audio, and leaves `developer_proof_only` visible.
  - first DAW session writer/action boundary is reserved as
    `daw_session.local_project_writer_v1` for the first bounded
    `export.daw_session` local-writer commit path. It sits after the CI-safe
    `daw_session.json_package_writer_v1` JSON package proof and before
    host-import or audible-output proof. Current code also has a typed reserved
    `export.daw_session` queue-history guard that rejects attempts with the DAW
    surface-gate reason and records destination/receipt intent without writing
    files, receipts, observer lifecycle records, host checks, or proof
    artifacts.
  - current local-writer `export.daw_session` commit path uses the same
    `daw_session.local_project_writer_v1` boundary only after the DAW writer
    plan is ready and `daw_session_json_package_integrity` has passed. It runs
    the existing staged local writer proof, records a committed action and
    commit record, attaches `daw_session_writer_proof` to the matching receipt,
    and still emits no host-import proof, audible-output proof, live capture, or
    final musician-facing export enablement.
    `just daw-session-writer-export-execute-smoke` runs the real binary through
    this queue/commit path, saves the Session mutation, and verifies the
    optional observer lifecycle while still writing only local writer proof
    JSON files.
  - current observer export snapshots include `export.daw_session` lifecycle
    records only when a real queued DAW-session action exists. Rejected reserved
    attempts produce requested / started / failed records without a receipt;
    committed local-writer proof actions produce requested / started /
    completed records with the matching DAW-session receipt and proof-gate
    summary; committed host-import-proof actions do the same for
    `host_import_proof_v1`; committed audible-output-proof actions do the same
    for `audible_output_proof_v1`. This is observer evidence, not live capture,
    host launch proof, or musician-facing DAW export readiness.
    `just daw-session-host-import-proof-export-execute-smoke` proves the
    host-import action path through the real binary without launching a host or
    writing DAW files.
  - current DAW-session writer proof skeleton writes only bounded local proof
    artifacts through `riotbox-app --daw-session-writer-proof-execute
    --session <session.json> --daw-session-destination <dir>`. The proof
    requires a passed JSON package gate, uses staging, emits
    `daw_session_writer/local_project_skeleton.json` and
    `daw_session_writer/writer_proof.json`, mutates no Session, emits no
    observer events, launches no host, and captures no audio. `just
    daw-session-writer-proof-smoke` proves that real-binary path.
  - current DAW-session writer proof apply path mutates only receipt evidence:
    `riotbox-app --daw-session-writer-proof-apply --session <session.json>
    --daw-session-destination <dir>` attaches `daw_session_writer_proof` and a
    writer-proof artifact entry to the latest DAW-session receipt. That writer
    proof is surfaced by the DAW operator report under
    `proof_gates.writer_proof`, including gate status and matching artifact
    availability. It is still not host-import proof and not audible-output
    proof.
  - current observer export snapshot also projects the latest DAW-session
    receipt's `proof_gates` and `proof_stack` so observer consumers see the same
    JSON package, writer-proof, host-import, and audible-output proof state as
    the operator report. This read-only receipt summary is not an
    `export.daw_session` lifecycle claim; lifecycle records are emitted only
    when a real queued action exists.
  - DAW-session release blockers are cleared only by their own proof layer:
    JSON package evidence clears only JSON package blockers, writer proof
    clears only `daw_writer_missing`, host-import proof clears only
    `daw_host_import_proof_missing`, audible-output proof clears only
    `audible_output_proof_missing`, and `developer_proof_only` stays visible
    until a later musician-facing release policy removes it. Any PR that
    implements the writer must state whether structured listening review exists
    or whether the handoff remains `human_verdict: unverified`.
  - reusable product-export evidence: local artifact hashing, local proof file
    hashing, receipt-side `artifact_set[]` projection, source graph and
    timing-grid receipt evidence, safe post-write WAV metric extraction,
    recovery preflight for local artifact paths, and observer export lifecycle
    projection from queue/history plus Session receipts
  - current live-recording export is a reserved contract only:
    `export.live_recording`, `export_scope: live_recording`,
    `live_recording.receipt_contract_v1`, `live_recording_capture`, and
    `live-recording-receipt-contract` are stable typed identities, but no live
    capture, WAV writer, observer completion, Session receipt mutation, or
    musician-facing runnable command exists yet. The app queue guard rejects
    attempts with a typed `export.live_recording` action and failed observer
    lifecycle so operators can see the boundary; because it writes no audio,
    current QA is control-path and side-effect proof only.
    `just live-recording-reserved-action-lifecycle-smoke` proves that rejected
    lifecycle path stays distinct from read-only live-recording receipt
    projection: it creates no receipt, writes no destination, and reports the
    explicit future-capture-writer reason.
  - new evidence required before readiness: one written WAV per claimed stem
    role, per-stem format metrics, per-stem non-silence, per-stem hash
    stability across repeated writer/render output, per-stem source/capture or
    capture-lineage evidence when policy requires it, per-stem source-vs-
    fallback comparison evidence when policy requires it, manifest JSON file,
    proof JSON file, and a package/render profile identity distinct from the
    current product-mix Feral-grid proof
  - gate order: render/write stems outside realtime audio; decode/measure
    written WAVs; hash stems; attach lineage and fallback evidence from
    Session/Core; build manifest and proof payloads from receipt evidence; write
    and hash manifest/proof JSON; run stem-package artifact-set, hash-stability,
    non-silence, lineage, and fallback-comparison gates; then commit the receipt
    only if the scope is no longer unsupported and all required gates pass
  - minimal output proof before any UI, Ghost, or CLI path may surface
    `export.stem_package` as runnable: a CI-safe writer proof that writes the
    final package layout, records format and audio metrics for every claimed
    stem, proves per-stem non-silence, proves repeated writer/render hash
    stability for identical inputs, proves per-stem source/capture lineage, and
    proves source-vs-fallback comparison evidence for every claimed stem. If the
    writer changes audible behavior beyond exporting already-proven buffers, it
    also needs the current structured listening-review status or an explicit
    `human_verdict: unverified` note.
  - realtime boundary: filesystem writes, hashing, decoding, metric extraction,
    QA comparison, Ghost/model calls, and observer emission must remain outside
    the realtime audio callback
  - replay/restore boundary: replay may validate package metadata and artifact
    availability, but must not regenerate stems or rewrite package files without
    a fresh explicit export request
  - manifest/proof identity rule: receipt `artifact_set[]` entries own written
    manifest/proof JSON file hashes; manifest/proof payload identities own only
    JSON role, location, and media type. The writer must keep this boundary so
    proof `manifest_sha256` and the eventual proof-file SHA are computed from
    final payload bytes without self-hash cycles.
- Observer export snapshots project those receipt `qa_gates[]` values as-is,
  including non-product stem-package evidence. For `export_scope: stem_package`
  receipts, observer snapshots also project `stem_package_readiness` with the
  Core-derived readiness status, typed blockers, and blocker labels from
  `validate_stem_package_receipt_readiness`. The observer surface is evidence
  projection from Session/Core receipt truth, not a second readiness engine and
  not permission to surface `export.stem_package` as runnable.
  For `export_scope: daw_session` receipts, observer snapshots also project the
  latest DAW-session receipt as top-level `daw_session_receipt` evidence so
  package/placement/tempo-map proof plus proof-stack state is visible before
  `export.daw_session` exists. This must remain read-only evidence projection
  and must not invent DAW export lifecycle records.
  Current app observer lifecycle projection includes `export.stem_package` and
  `export.daw_session` actions from action log, queue history, and pending queue
  state. Failed reserved attempts have typed failure reasons and no receipt;
  completed receipts expose readiness only from the Session receipt.
  Observer snapshots also include the shared stem-package musician surface gate
  with `status`, `runnable`, typed blockers, and musician labels. That gate
  explains product surfacing and is not permission to infer or mutate package
  artifacts.

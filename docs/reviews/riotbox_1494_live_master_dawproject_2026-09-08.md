# RIOTBOX-1494 — Committed live master to DAWproject

Status: merged; PR/remote-CI success, independent review and corrected
handoff-02 verified. Handoff-01 remains failed external XML evidence.
Contracts: RBX-372 and required serialization correction RBX-373.

## Intended musician path

Export a previously committed bar-aligned live recording with
`just live-master-dawproject <session> <destination.dawproject> [observer]`.
The DAW clip contains the exact complete master, at the recorded tempo for
eight beats; it is not a semantic W-30 hook or editable stems. The command
does not start audio, rerender material, or claim a successful DAW import.

## Ownership and proof

Use the existing Action/Session/export receipt spine and shared archive writer.
The required metadata-only hydration mode belongs to the existing restore/save
pipeline and preserves graph refs without external graph/source/capture reads.
Proof must cover exact WAV/proof identity, recorded timing, lineage, typed XML,
queue pinning, no-clobber publication and Session-save rollback. Ordinary
runtime loading and W-30 export remain regression gates.

## Exact local evidence boundary

Only these pre-existing non-synthetic inputs are authorized, under
`artifacts/development/riotbox-1492/take-02` in the main workspace:

- `session.json` (read as metadata; work on a fresh copy).
- `runtime-master.wav`, SHA-256
  `8d598b852a2811c7614de8755d7a9da870339b75582a922510e6c82b1d77f4c0`.
- `runtime-master.wav.riotbox.json`, SHA-256
  `2ae77d457959cc31093c604ddeddc6a2b23665d2e0d300a2de67bd01f995a622`.

Use a new bounded access log and hash-check before the handoff. No source graph
file, capture WAV, source directory, Holdout or commercial reference may be
opened. Do not change the original Session, take, or hash-bound human review.

RIOTBOX-1492's keep applies only to the exact hook-plus-beat composite. The
handoff must prove byte identity, not create a new verdict or replay unchanged
audio. DAW import and host playback remain unverified.

The verified result is document/archive readiness only. It does not assert
DAW-host import, playback, release, isolated-source or hardness qualification.

## Implementation review notes

- Adversarial implementation review identified incomplete proof/window
  cross-checking in the initial input validator: matching duration/BPM alone
  did not bind every shared original timing-window field or reject every
  contradictory capture fault/clip counter. The correction must check all
  shared timing and health fields, scene/action ownership, and lineage, with
  negative tests that also update the proof-file hash so they exercise semantic
  checks rather than merely hash drift. Disposition: corrected; focused
  regression verification passed.
- The new export policy initially combined orchestration, input admission and
  document construction in one large module. Split those semantic owners into
  real modules; do not copy the shared archive mechanism or add textual shards.
  Disposition: corrected within this slice. The root owns queue, publication,
  commit and rollback; `input` owns complete admission; `document` owns typed
  proof, XML model and receipt construction. No textual production shards.
- Independent review found missing arrangement placement and tempo-map refs
  in the initial live-master receipt, which would leave the shared operator
  report blocked after a successful archive. The receipt now carries beat-zero
  two-bar placement and the recorded tempo/grid identity. Regression tests must
  retain host-import, audible-output and release blockers while recognizing
  the completed archive gate.
- Exact artifact reads now open with `O_NOFOLLOW | O_NONBLOCK`, verify regular
  file type on that same handle, then hash/decode/embed its read bytes. This
  closes the separate path-check/open race and avoids waiting on a FIFO before
  file-type rejection. `libc` was already transitively present; the app now
  declares its direct Unix flags dependency.
- Independent review found ambiguous duplicate source receipt IDs: selecting
  the latest receipt for queueing and the first matching ID at commit could
  substitute a different receipt. Both phases now require exactly one matching
  stored ID; regression cases cover duplicates before and after queueing.
- Independent review found that generic archive readiness could accept an
  incomplete live-master artifact set. A dedicated Core metadata check now
  requires the four unique roles, their media and member locations, hash
  identities and one complete passed archive gate. Operator, proof, release and
  surface summaries use it for this boundary; historical W-30 behavior is
  unchanged. Ten tamper cases exercise missing/duplicate/mismatched evidence.
- Initial full CI passed tests and synthetic QA but failed strict Clippy on
  an unnecessary explicit lifetime in input admission. The lifetime was elided;
  no behavior or QA threshold changed. Subsequent full CI passed. The later
  XML schema test helper also needed to reap its child on a stdin-write error;
  that cleanup was fixed, independently reviewed, and the complete final CI
  passed (`/tmp/riotbox-1494-ci-release.log`).

## Failed exact handoff and independent schema correction

The first complete corrected-code CI passed (`/tmp/riotbox-1494-ci-final.log`)
and the final focused suite passed 12 tests, including save/reload/commit/replay
plan identity without archive rewriting. The exact CLI handoff then wrote
`artifacts/development/riotbox-1494/handoff-01` with archive SHA-256
`ad2fb7012ac3f826e6ae2643b6f96f20ce2f1bd995d5edc3a797ba42f32a07cb`.
It is retained as **failed/unqualified**, not a completed musician handoff.

The independent audit confirmed the pinned embedded WAV SHA, one exporter open
each of the exact master WAV and proof, and zero source/graph/lineage-capture
reads, directory enumeration or network connections. The original Session SHA
at the copy boundary was
`dcd6b6bcbefadc57df9491dc444ceddbfe41511b7e018462904ef76738f63091`;
the original was never passed as a write target. No runtime or playback ran.

External XML inspection then rejected `<ProjectType>`; metadata similarly had
`<MetaDataType>`. These are generated Rust type names, not the document elements
declared by the [official Project schema](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/Project.xsd)
and [MetaData schema](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/MetaData.xsd).
The dependency's own writer/reader accepted the same erroneous representation.
This is a pre-existing shared serialization defect also preserved by RIOTBOX-1493,
not a change to the approved recording.

RBX-373 versions the correction and requires canonical XML-document evidence
for new W-30 and live-master readiness. Independent mandatory synthetic XSD
tests use pinned upstream schemas with their MIT license and `xmllint --nonet`.
The failed handoff is not patched or reclassified; a corrected writer must pass
synthetic review/CI before a fresh bounded exact handoff is attempted.

## Corrected handoff-02 and final evidence

`artifacts/development/riotbox-1494/handoff-02/riotbox-live-master.dawproject`
passed independent ZIP/member, byte-identity, official XSD, clip/timing,
Session/action/receipt and observer verification. Its access log and
`handoff-verification.json` are local ignored evidence, not redistributed audio.

- Archive SHA-256:
  `fb08afbb76118180c1bf3420f5116d1695f68b24faf71aa500877ef4fb2c3458`.
- Project XML SHA-256:
  `c489e58e55700a719acc8615f34ff9922e45f00b639e922ab5c659b211ab96a8`.
- Embedded DAW proof SHA-256:
  `2884a4b263d710b5e8e9eac32adabdd101bc560e0f4d031b66a7e5c85199cafa`.
- Embedded WAV retains the pinned RIOTBOX-1492 SHA above: 192,000 frames,
  stereo float32, 48 kHz, four seconds, eight beats at recorded 120 BPM.
- Source receipt `export-receipt-a-0008` becomes DAW receipt
  `export-receipt-a-0009`; copied saved Session SHA-256:
  `97dd65f0727e7ab128fd6b827237049cbb3daee6cf4fe83f7af6b503de40be45`.
- Exactly one exporter open each of the exact WAV and proof; no graph,
  original-source or lineage-capture reads, directory enumeration, sound-device
  opens or network connections. No audio runtime or playback.
- Original Session metadata was read for the copy and reread for the final
  metadata comparison; its original hash is unchanged. A helper initially
  expected lowercase `committed`; the frozen Rust enum serializes `Committed`.
  Only that verifier expectation was corrected. Final verification reused the
  existing output, with no re-export or additional original WAV/proof read.
- One successful CLI observer event binds the same saved receipt. Both unique
  archive and versioned XML-document gates are passed. No host gate is granted.

Final local validation:

- `just ci`: pass, `/tmp/riotbox-1494-ci-release.log`.
- App library: 738 passed, `/tmp/riotbox-1494-xml-app.log`.
- Final focused DAW tests: 24 passed,
  `/tmp/riotbox-1494-xml-schema-final.log`.
- Core: 449 tests passed in full CI. Strict all-target/all-feature Clippy,
  formatting, synthetic audio/manifest, access and contract checks passed.
- Independent Core/metadata and adversarial app/Rust reviews: zero outstanding
  P0–P3 findings after corrections; the reviewers did not access real audio.
- Short final self-review confirms the existing Action/commit/Session/replay
  spine, necessary semantic module ownership, metadata-only hydration and
  preserved no-clobber/rollback behavior. No new ActionCommand or DSP.

Listening state remains the existing RIOTBOX-1492 composite keep. No repeated
listening or new musical verdict was requested or inferred. Actual DAW import
and playback remain the next host-dependent evidence boundary.

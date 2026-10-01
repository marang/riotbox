# RIOTBOX-1539 QA integrity and product spine architecture checkpoint

Date: 2026-10-01. Classification: maintenance/regression.
Current integrated code: `e7f874371661b58f0bcc9620c093db089520fdc9`.
Integrated archive-only follow-up: `59edfc3c1c6e1f7f9c2ad991ef1fc0f61963cc97`.
Previous checkpoint: `782cacfe7c9b0f45f0f541b92abf9a8927d218a4`.
Solo risk-directed current-state review under review-codebase, not a full
line-by-line audit or independent panel.

This checkpoint reviews the recent QA ownership and input-preservation work
against the wider product spine. It finds one additional, reproducible P2
output-integrity defect, recorded for the immediate next bounded slice. The
new source guards work within their stated input-preservation contract; this
finding is a separate gap between two output files, not a source-loss regression.

## Findings and priorities

### P2 W-30 metrics can overwrite a physically aliased output WAV

Location: `w30_preview_render/main.rs`, between its source-only preflight and
success return; `artifact_io::write_pcm16_wav` followed by
`artifact_io::write_metrics_markdown`.
Reviewer lens: Adversarial Implementation Reviewer, applied solo.

Two actual CLI runs on current integrated code use preexisting hardlinked
preview.wav and preview.metrics.md. Both return zero, but the second write
replaces the WAV with Markdown: prefix `52494646` becomes `2320572d`.
A separate-output counterfactual returns zero and retains a valid RIFF WAV.
The refined feedback loop also verifies the exact synthetic input hash stays
unchanged. The shared guard currently compares each output only against the
source; it never establishes mutual output distinctness. Thus success and
metrics do not prove the named WAV contains the reported rendering.

Tight unattended loop:
`bash /tmp/riotbox-qa-output-alias-probe.6WrEPt/repro.sh alias`.
Each invocation allocates its own isolated temporary output directory and uses
only the exact generated synthetic-control.wav from the W-30 parity proof.
The loop expects fail-closed exit 1 and unchanged prior output; it currently
goes red. `separate` is the positive counterfactual.

Proportionate repair: preflight both actual W-30 outputs against each other's
physical identity before rendering/publication, in explicit-source and explicit
no-source diagnostic mode, reusing the existing binary-only filesystem owner.
Preserve normal bytes, source/capacity/error order and equal-content distinct-file
support; prove hardlink/symlink, late/error and no-source behavior at the actual
CLI seam. RIOTBOX-1540 is Todo and the immediate next implementation slice.
No fix is silently mixed into this audit. Other renderers' mutual-output behavior
is not inferred from this one confirmed probe.

No other demonstrated P0–P3 finding survives the sampled checkpoint. Existing
RIOTBOX-1509 remains Todo awaiting causal Windows startup evidence. Green
transport runs do not close that intermittent issue. Remaining textual includes
are explicit legacy ownership, not a new urgent defect based on size alone.

## System model and boundaries

| Owner | Inspected responsibility and direction |
| --- | --- |
| App facade | Committed actions and Core/Session truth feed runtime projection, persistence and artifact hydration; no QA-local state becomes replay truth. |
| Core | Source/Graph identities, capture provenance, Session, action/commit records and replay planning/execution own product history. Replay indexes actions and validates unique commit identities. |
| Audio library | Admitted source bytes feed PCM cache and prepared lane snapshots; callback consumes snapshots and preallocated scratch. No QA filesystem preflight enters the callback. |
| Offline source guards | One binary-only physical-file identity implementation; Feral nineteen-file and Before/After fourteen-file plans own their write paths. W-30 uses its two existing path variables. |
| Comparison | Both parsed metrics and convention-derived WAVs precede report/manifest publication checks; writer naming helpers remain authoritative and output self-aliases reject. |
| Capture storage | New exclusive files are published into Session only after complete write; same-buffer hash/decode hydration, explicit legacy adoption and runtime-only observed status. |
| Graph persistence | Exact hash-bound generation and persisted Session path authorize recovery/save; Session publication remains commit point, never latest-file scanning. |
| Sidecar | Protocol/request identity and one absolute pipe-exchange deadline; invalid peers cannot be silently reused or become trusted source qualification. |

Cargo direction remains App -> Audio/Core/Sidecar and Audio/Sidecar -> Core.
The new same-file dependency is Audio-local and already lockfile-present; no
new version, executable or public library interface. Ordinary binary helpers
hide filesystem mechanics while keeping renderer layout and domain policy local.

## Contracts and sampled evidence

Read current QA router/artifact contract, Session capture/persistence/load rules,
replay authority, original-source admission and module/include policy, plus the
recent decision records and prior checkpoint. Inspected actual renderer plans,
entrypoints, preflight/publication ordering and naming/write consumers, compared
retained shared helper/safety tests, and traced App graph transaction/recovery,
original-source same-buffer restore, capture creation/verification, Core
capture/replay ownership, actual callback and Sidecar exchange/client contracts.

Exact Git comparison against the previous checkpoint is empty for Core, App,
Sidecar and Audio library/runtime/source-cache paths. Audio changes are scoped
to offline binaries, their regressions and the explicit crate-local dependency.
Cargo.lock adds only the Audio dependency edge to the already-pinned same-file
package; no package version changes. Frozen benchmark contracts are unchanged.
RIOTBOX-1538 was initially pending, then passed exact-head Rust/Windows-sidecar
CI and merged. Candidate code/contracts equal integrated main; four disjoint
predecessor archive/report files explain the whole-tree difference.

Source admission hashes the same bytes decoded; capture hydration similarly
verifies one read, with no untrusted source-window substitute. Legacy adoption
records current bytes, not historical authenticity. Graph recovery save uses
the on-disk Session's exact generation/path, never edited state or a directory
scan. Unsupported external-generation hardlinks fail explicitly before mutable
publication; this is a declared compatibility limit, not a copy fallback.

The callback retains prepared snapshots, scratch-overflow silence/telemetry,
source-monitor policy, limiter and capture/output order. No blocking filesystem
I/O, hashing, analysis or model calls are added. Sidecar deadlines cover pipe
write/flush/read, not arbitrary JSON CPU, spawn/kernel delay or descendants.
Regular-file reads and Sidecar frame sizes remain uncapped; no new resource
budget or malicious-peer resilience claim is invented.
The include guard validates five retained sites in two known legacy owners.

## Fresh integrated verification

- Five QA binaries and eight CLI targets: 127 executed cases,
  `/tmp/riotbox-1539-qa.log`.
- Core library: 470, `/tmp/riotbox-1539-core.log`.
- Graph transaction/recovery: 16, `/tmp/riotbox-1539-persistence.log`.
- Capture identity/publication: eight, `/tmp/riotbox-1539-capture-all.log`.
- Actual source-file admission: three, `/tmp/riotbox-1539-source-admission.log`.
- Callback scratch sizing/telemetry: two, `/tmp/riotbox-1539-callback.log`.
- Sidecar: 24 library cases plus one working-directory-independent process
  integration, `/tmp/riotbox-1539-sidecar.log`.
- Fresh integrated standalone binaries: `/tmp/riotbox-1539-build.log`.

Sequential, fully completed normal CLI reruns match all 141 records:
Feral 31, W-30 32, Before/After 31 and comparison 47. All 85 exact hashes match
prior baselines respectively 39, nine, 29 and eight. Only already-generated
synthetic controls are opened; no source directory discovery. Comparison
pass/drift manifests validate existing artifacts; current renderer tests retain
literal destination catalog and normal artifact coverage.
Early lookup/capture diagnostics are not accepted proof. Test counts reflect
executed cases, not zero-test filter success; final parity capture completes
before hash comparison. The first narrower capture run is superseded by the
complete eight-case identity/publication run.

Full source-free `just ci` completed with actual exit zero:
`/tmp/riotbox-1539-ci-final.log`. Final accepted logs are scanned for warnings
and errors; fmt/diff/include gates pass. Green finite fixtures do not contradict
the separately reproduced output-alias defect absent from those old cases.
Native report PR review/merge and archive/cleanup remain separate gates.

## Coverage limits and disposition

No exhaustive repo audit, decoder fuzzing, huge allocation stress, hostile
namespace races, power-loss durability, OS/filesystem/device matrix, allocation
tracing, live TUI/device, DAW or playback. No real Development/Holdout/commercial
audio or human verdict. Windows-sidecar CI is not Windows Audio/identity proof.
No musical/source/diversity/hardness/release pass or P023 completion.

No new durable architecture/algorithm/threshold decision arises from the audit
itself, so no routine Decision Log entry is added. Immediate next priority is
RIOTBOX-1540's confirmed output-integrity regression; optional further ownership
cleanup stays behind it. This documentation branch is ready as an audit with
that explicit open P2, not as a claim the product is release-ready. Solo
code-review of scope/counts/contracts/source boundaries and short self-review
find no additional actionable report defect; no independent panel or subagents.

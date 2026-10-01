# RIOTBOX-1516 — Post-ownership architecture checkpoint

Date: 2026-10-01. Integrated production baseline:
`348a13e88db9dd81e9d6e0af2be9e65463ec77ae` (RIOTBOX-1514, PR #1572).
Reviewed feature/CI head: `0ac68ba9cba9ff06b00c652139fdf218fa8b4bd4`.
Code, Cargo manifests/lock and scripts are identical between these heads
(`git diff --exit-code` verified). Classification: maintenance/regression;
solo risk-directed `review-codebase`, not an exhaustive audit or independent panel.

This follows five substantive ownership slices since RIOTBOX-1507: Core Jam
projections (1508), timing candidates (1510), App lane projections (1512),
Core TR-909 policy (1513) and Observer/audio diagnostics (1514). Test-only 1511,
import-only maintenance, diagnosis-only 1509 and archive branches do not count.
Preparation began while #1572 CI ran; completion uses its merged integrated head.

## Findings and priorities

**P2 — Existing Observer/audio anchor-count overflow remains open, RIOTBOX-1515.**
`observer_audio_correlate/source_timing_anchor_evidence.rs:35–39` sums three
untrusted JSON `u64` counts with unchecked addition. Declared total/kick count
`u64::MAX`, backbeat count one and transient count zero cause Debug CLI status
101, empty stdout and `attempt to add with overflow`. The caller is intended
to reject inconsistent anchor metadata, not panic or accept a wrapped total.
The exact retained pre-migration impl establishes this is not introduced by
RIOTBOX-1514. Overflow-disabled wrapped acceptance is code-derived, not yet a
release-run result. Fresh integrated-head reproduction reads only named synthetic
JSON and committed observer NDJSON; no referenced source/artifact audio is opened.

Use checked sum validation and reject malformed evidence through the existing
strict/local error semantics. Preserve valid bytes, schemas and thresholds.
Test both overflow positions and valid boundary totals under Debug/Release;
do not saturate away contradictions or infer a musical failure from this parser
defect. RIOTBOX-1515 is ready Medium/P2 Todo for the next autonomous bug slice.

No additional demonstrated P0–P3 finding survives this sampled checkpoint.
Core/Session authority, recovery identities, control-side preparation and the
actual mixer boundary remain intact. No rewrite or additional product-state
owner is justified. Fix the demonstrated parser error before optional further
QA-bin structural migration; do not elevate file length into a correctness bug.
RIOTBOX-1509 remains open: current green native Sidecar checks do not establish
a causal fix for previously observed intermittent Windows startup failures.

## System model and ownership

| Owner | Existing authority and downstream boundary |
| --- | --- |
| Core | Source Graph, Session/action/replay truth; typed timing/TR-909 policy and Jam views. Depends on serde/JSON/SHA, not App/Audio/Sidecar. |
| App | Facade over Core truth; verified source/capture admission, recovery and prepared lane/sample projections. No new persisted state in projection children. |
| Audio | Prepared snapshots and preallocated callback mix/capture buffers; Core dependency, no App/Sidecar/model dependency. |
| Sidecar | Control-side versioned request/response exchange with absolute pipe-I/O deadline and invalid-peer cleanup. Never a callback dependency. |
| QA binary | Named observer/manifest metadata -> derived report -> evidence/presentation. Can reject inconsistency, not authenticate source provenance or grant human approval. |

## Sampled cross-slice boundaries

- **Jam and timing:** explicit public facades retain existing paths. Jam assembly
  consumes Session, graph and pending/committed actions; presentation does not
  become replay truth. Timing model assembly consumes scoring/hypothesis/grid
  owners; strict readiness and short-loop/manual-confirm classification remain
  in Core. Reviewed candidate-model/grid-use bodies and sampled Jam assembly/
  source-timing summaries; thresholds/algorithm behavior were not tuned.
- **Lane/policy mapping:** App TR-909 maps existing Core policy into Audio enums;
  MC-202 keeps source-plan/section checks and silent unavailable state. Scene
  context consumes persisted projection movement and never promotes a restore
  event into the active sound profile. W-30 cached material prepares samples on
  the control side; its transform data has no dependency back to preview policy.
  Core TR-909 retains public typed vocabulary and source/scene/transport inputs.
- **QA depth:** typed report data/scalar readers are leaves; metadata collectors
  feed composition/alignment, evidence consumes the report and presentation
  consumes report/evidence/labels. No evidence -> rendering or model -> builder
  cycle is introduced. Existing lane-recipe/observer-envelope children remain.
  The same single Cargo binary target and all 21 CLI byte/status cases hold on
  the integrated head. Numeric parser trust remains the explicit finding above.
- **Persistence:** Session publication remains the commit point over hash-bound
  immutable graph generations. Recovery save verifies the on-disk Session's
  exact generation, not the edited graph or a scanned newest file. JSON/invalid
  UTF-8 alias damage may recover; unrelated I/O errors stay errors. Generation/
  alias/Session collisions and symlink redirects fail closed. Hardlink storage
  compatibility remains explicit; no incomplete-copy fallback is inferred.
- **Source/capture identity:** original-source descriptor admission is shared,
  with one buffer used for decode and graph/Session hash comparisons. Capture
  identity stays in Core; fresh opaque artifacts are published only after writes
  finish. Legacy capture adoption requires explicit selected IDs and current
  byte acceptance, not invented historical identity. Duplicate/changed/unverified
  captures cannot populate trusted playback caches. Persistence, identity,
  original-source and Sidecar implementations are unchanged since RIOTBOX-1507;
  targeted commit comparison and current boundary reads verify that statement.
- **Realtime/process failure:** sampled the actual stream callback's scratch
  allocation before callback installation, oversized-buffer silence/telemetry,
  prepared lane mixing, source-monitor policy, limiter and capture/output order.
  No file/hash/analysis/model work is added to that callback. Sidecar exchange
  retains one write/flush/read deadline, incremental frame scanning and trailing
  frames; malformed/desynchronized responses invalidate/reap the direct peer.
  Kernel/process/JSON CPU bounds and allocation tracing are not proven here.

## Fresh regression and integrated evidence

Fresh bounded tests pass, with all seven validation/build logs explicitly
scanned warning/error-free:

- Core library: **470** — `/tmp/riotbox-1516-core.log`.
- Graph transaction/recovery: **16** — `/tmp/riotbox-1516-graph-recovery.log`.
- Capture identity/adoption/publication: **8** —
  `/tmp/riotbox-1516-capture-identity.log`.
- Original-source file admission: **4** — `/tmp/riotbox-1516-source-admission.log`.
- Callback scratch capacity/overflow telemetry: **2** —
  `/tmp/riotbox-1516-callback-scratch.log`.
- Sidecar library/process fixtures: **24** — `/tmp/riotbox-1516-sidecar.log`.
- Integrated Observer binary rebuild: `/tmp/riotbox-1516-observer-build.log`;
  **21** metadata-only CLI stdout/stderr/exit cases equal the captured baseline.

Source/capture/process tests use fresh synthetic temporary files/peers, not
Development or Holdout audio. Full source-free CI for the identical reviewed
code passes in `/tmp/riotbox-1514-ci.log`: App 770 plus one unchanged ignore,
Audio 279 plus one unchanged ignore, Core 470, Sidecar 24, Observer 61, synthetic
smokes, Python/contracts, formatting, tracked JSON and strict Clippy. Native
exact-head run **36795220369** passed Ubuntu Rust and Windows Sidecar jobs;
PR #1572 merged with zero reviews/comments. Windows transport evidence is not
Windows audio, filesystem-matrix or device proof. This docs-only checkpoint's
native exact-head CI, merge and closeout remain separate completion gates.

## Limits and disposition

No real source, active holdout, commercial reference, device, DAW or playback
was accessed. This is current-state sampling, not whole-repo line-by-line
coverage. Excludes exhaustive Python-provider review, numeric/decoder fuzzing,
malicious-input memory stress, allocation tracing, real-terminal/device behavior,
OS/filesystem matrices, power-loss durability and concurrent external writers.
Regular-file byte reads and Sidecar frames remain uncapped; pipe deadlines do
not bound JSON CPU work, process spawn, kernel stalls or descendant lifetime.
Those known limits are not newly demonstrated corruption/timeout guarantees.

No new architectural decision/constraint arose, so this routine checkpoint
adds no Decision Log entry. RIOTBOX-1515 owns the demonstrated validation fix;
RIOTBOX-1509 retains its unresolved causal diagnosis. Sequential documentation
review/self-review checks evidence scope, finding disposition and closeout.
Reset the substantive architecture cadence only after this checkpoint is merged
and closed; it grants no musical/source/human qualification.

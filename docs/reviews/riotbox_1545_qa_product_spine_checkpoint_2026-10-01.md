# RIOTBOX-1545 QA artifact integrity and product-spine checkpoint

Date: 2026-10-01. Classification: risk-directed current-state maintenance review.
Integrated code: `fdd5651c0b069d6e2835bfea572a88504149628d`.
Previous checkpoint code: `e7f874371661b58f0bcc9620c093db089520fdc9`.
Previous checkpoint archive baseline: `59edfc3c1c6e1f7f9c2ad991ef1fc0f61963cc97`.
Solo review-codebase; no independent panel or exhaustive line-by-line audit.

## Findings and disposition

No new demonstrated P0–P3 finding survives this sampled checkpoint. The
previously reproduced output-role coupling is addressed at each actual public
CLI seam: W-30 in RIOTBOX-1540, Feral in RIOTBOX-1541 and Before/After in
RIOTBOX-1542. Source preservation is distinct from mutual-output integrity;
these checks prove neither WAV content identity nor musical qualification.

The shared pair owner and subsequent renderer/report/Markdown owners retain
their stated boundaries. RIOTBOX-1509 remains Todo: native Windows Sidecar
success is finite evidence, not causal reproduction or repair of its sporadic
startup failure. No timeout tuning or green-retry closeout is justified.

Remaining three textual includes are explicit legacy ownership, not an urgent
defect inferred from line count. The next safe optional maintenance is the
separately bounded remaining stem/render/validation/publication owner, followed
by its retained test/helper ownership. Preserve complete algorithms, all tests
and actual CLI/artifact bytes; do not mix policy or DSP changes into migration.
Human limiter calibration and real-host DAW proof retain their existing gates.

## System model and trust boundaries

| Owner | Current responsibility and dependency direction |
| --- | --- |
| App facade | Core/Session actions and history feed restore, artifact hydration and prepared runtime projection. Observed load status is runtime-local; expected identity is not. |
| Core | Session, capture provenance, Source/Graph references and typed action/commit/replay records own persisted product truth. Action-ID index and uniqueness sets protect replay planning. |
| Audio | One admitted original-source buffer feeds hashing/PCM decode. Prepared snapshots and preallocated scratch feed the actual callback; QA file guards never enter realtime. |
| QA layouts | Feral 19, Before/After 14, W-30 2 and comparator 2 output roles retain actual writer-derived plans/naming. One binary-private owner admits regular entries and checks physical pairs. |
| Feral orchestration | Root main delegates through explicit renderer dependencies. The 31-field report is shared ephemeral evidence, not a second replay or musical-state model. |
| Markdown publication | Two direct sibling-imported publishers own file presentation; nine formatters stay private. Presentation consumes existing timing/measurement values, never computes new product decisions. |
| Capture persistence | Fresh exclusive allocation precedes Session reference/identity installation. One read buffer is hash-checked and decoded; unverified/changed content cannot hydrate a trusted cache. |
| Graph persistence | Exact persisted Session hash/path authorizes recovery→save. Immutable generation first, mutable alias next, Session rename last; no newest-file scan or copy fallback. |
| Sidecar | Protocol and request identity plus one absolute pipe write/flush/read deadline. Invalid peers are invalidated; finite fixtures are not source qualification. |

Cargo remains App → Audio/Core/Sidecar and Audio/Sidecar → Core. Exact Git
comparison since the checkpoint is empty for Core, App, Sidecar, Audio library/
runtime/source admission and Cargo manifests/lockfile. Changes are offline QA
owners, their public regression tests and owning contracts/review/archive docs.
Frozen Stage-A protocol, matrix, registry and pre-admission review are unchanged.

## Current-state review and counterchecks

Read current artifact/source/mutual-output and Session save/load/capture rules,
module/include policy, previous checkpoint and the owning recent decisions.
Trace actual renderer entrypoints, nineteen/fourteen destination iterators,
writer/manifest naming and source/format/window/preflight/publication ordering.
All outputs are admitted before pair comparison; bounded existing pairs use
the same read-only identity owner. Absent/singleton sets do not open pair
handles, preserving explicit no-source diagnostic behavior. Dangling/nonregular/
unknown destinations fail closed; independent equal-content files are valid.

Trace graph load/recovery authority, generation publication and the persisted
Session path; unrelated alias I/O failures remain errors. Unsupported hardlinks
produce the declared compatibility error before mutable publication. Capture
creation never replaces a prior locator; complete bytes are written before
identity/cache installation. Hydration and explicit selected legacy adoption
use one read buffer. Adoption establishes current bytes, not past authenticity.
Missing/changed/unverified captures have no implicit source-window substitute.

Inspect Core replay index/uniqueness validation, actual callback scratch-
overflow silence/telemetry, snapshot/control-side retention, source-monitor,
limiter and capture/output ordering. No new I/O, analysis, hashing, QA policy
or model calls enter that callback. Inspect original-source regular descriptor
admission and PCM decode/writer checked alignment arithmetic. Original sources
permit regular symlinks; capture artifacts retain their separate no-follow rule.

Inspect Sidecar exchange/invalidation and transport write/flush/read loops,
deadline checks and completed-frame allocation release. Deadlines do not promise
bounded JSON CPU, spawn/kernel delay or child descendants. Source/capture reads
and received frames have no new universal byte cap; huge-allocation and hostile-
peer resilience are verification limits, not newly demonstrated regressions.
An initially suspected unknown-Session-version migration gap is rejected:
SessionVersion is a closed typed V1 enum, so deserialization already rejects
unknown variants. No speculative finding or fix ticket is created for it.

## Accepted verification

Fresh source-free product suites, completed against the exact predecessor
candidate whose whole tree equals normally merged integrated main:

- Core: 470, `/tmp/riotbox-1545-core.log`.
- Graph recovery/transaction: 16, `/tmp/riotbox-1545-graph.log`.
- Capture identity/publication: eight, `/tmp/riotbox-1545-capture.log`.
- App original-source admission: three, `/tmp/riotbox-1545-source-app.log`.
- Audio regular/directory/symlink/FIFO admission: four,
  `/tmp/riotbox-1545-source-audio.log`.
- Actual callback scratch size/telemetry: two, `/tmp/riotbox-1545-callback.log`.
- Sidecar: 24 library and one cwd-independent process integration,
  `/tmp/riotbox-1545-sidecar.log`; empty binary/doc targets do not count.

Accepted RIOTBOX-1544 proof for the identical integrated tree: same 143 focused
Debug/Release names/statuses, all eleven presentation bodies, fresh standalone
binaries and completed sequential 141 CLI records / 85 baseline-identical
hashes. The original Feral collision loop preserves input/prior outputs; both
comparison pass/drift manifests validate with existing artifacts required.
All final accepted logs are warning/error-free. Native exact-head run
36850899763 passes Rust and Windows Sidecar before feature #1632 normally merges;
whole-tree equality is independently checked. No unmerged feature stack.

Full source-free audit-branch just ci completes with actual exit zero:
`/tmp/riotbox-1545-ci-final.log`. Final accepted logs are warning/error-free;
fmt, diff and the three-site include guard pass. Native report/merge/archive/
cleanup remain separate obligations, not inferred from predecessor proof. No new
durable architecture/algorithm/threshold decision follows from this audit,
so no routine Decision Log entry is added.

## Coverage limits

This is a risk-directed current-state review, not a complete repo audit,
decoder fuzzing, allocation tracing, huge-file stress, hostile namespace race,
multiwriter/power-loss durability or OS/filesystem/device matrix. QA guards
assume a stable namespace and readable/identifiable outputs; directory creation
can precede rejection. They are not atomic whole-pack publication. Windows
Sidecar CI is not Windows Audio or physical-file-identity runtime proof.

Only exact generated synthetic controls and retained fixtures; no source-
directory discovery, real Development/Holdout/commercial audio, DAW/device/
playback, subagents or human verdict. No source-general/diversity/musical/
hardness/release pass or P023 completion. Solo code-review of report scope,
counts, contracts and evidence plus short self-review finds no open report
defect; source/human/host gates are not waived by this maintenance checkpoint.

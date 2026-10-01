# RIOTBOX-1532 — Architecture checkpoint after Feral policy ownership

Date: 2026-10-01. Integrated baseline:
`3704304664ecef657141d6f2473dae4fdb5dccea` (RIOTBOX-1531, PR #1606).
Classification: maintenance/regression. Solo risk-directed `review-codebase`
checkpoint, not an exhaustive audit or independent reviewer panel.
The subsequent archive-only merge `6f37e4837c64b0198d51d3f9e131f6befda8057c`
adds no code changes and is synchronized into this report branch.

RBX-402 requires this checkpoint after five structural slices since RIOTBOX-1524:
W-30 QA owners (1527), TR-909 QA owners (1528), decoded-cache window selection
(1529), MC-202 QA owners (1530) and mix computation/policy/evidence (1531).
Capacity fixes 1525/1526 and archive branches do not count. This is a bounded
series obligation, not an invented global numeric cadence.

## Findings and priorities

No additional demonstrated P0–P3 correctness or architecture finding survives
the sampled checkpoint. The actual-owner directions remain coherent; no
rewrite, second product-state owner or new source/quality authority is justified.
Prior P2 Feral capacity rejection is present, and W-30 duration/vector/PCM16 RIFF
preflight is retained before optional source hydration/render/output I/O.
Those guards are representability checks, not arbitrary memory budgets.

Remaining root orchestration, rendered-mix TR-909 pressure and regression/assertion
families are explicit legacy opportunities, not proven urgent bugs. A next
structural slice must remain bounded, preserve behavior and improve actual
responsibility/dependency ownership; low line count alone is not a finding.
RIOTBOX-1509 stays Todo awaiting causal Windows startup evidence. Green native
Windows transport runs do not prove that intermittent startup failure fixed.

## System model and dependency direction

| Owner | Existing responsibility and inspected dependency direction |
| --- | --- |
| W-30 QA | Shared measurements/events feed window/chop, slice choice, trigger, accent and playback policy; JSON presentation consumes typed evidence. Root retains hydration/render/publication. |
| TR-909 QA | Source profile consumes Grid/config/filter/spectral/runtime metrics. Pressure consumes profile/Grid and the existing library renderer; presentation consumes proofs. Fractional frame conversion belongs beside Grid. |
| Window selection | Search preparation/ranking consume decoded cache, Args/Grid and scalar measurements. Root still owns file admission/publication; selector/serializer-only fields stay private. |
| MC-202 QA | Contour/profile feeds phrase/state and low-body/pressure policy. Pressure consumes actual render measurements/library renderer; presentation consumes typed proofs. No reverse root/mix/stem dependency. |
| Mix QA | Components own scalar MixPolicy/render/ratio computation. Balance policy consumes components and MC-202 contour; movement evidence consumes both plus RMS measurements. Product-stem production imports actual component/filter owners. |
| Core and App | Source Graph, Session/action/replay/timing and capture identity remain product truth. App admission checks content identity; recovery/save uses the persisted Session's exact generation. |
| Actual Audio runtime | Prepared snapshots and preallocated scratch feed lane mixing, Source Monitor, limiter, capture and output. QA ownership does not enter the callback. |
| Sidecar | One absolute deadline covers synchronous pipe write/flush/read; protocol/request identity and transport invalidation remain explicit. Stub transport cannot become source qualification. |

Shared helper/evidence visibility is binary-bound. Single-owner ranking,
classification, DSP and serialization internals stay private. Explicit root
compatibility aliases serve untouched consumers, including retained test paths;
they are not concealed production-owner back-dependencies or a claim that all
regression families migrated. The manual include guard validates nine remaining
sites in two owners, eight root plus one shared legacy test helper.

## Product and failure boundaries

An exact Git comparison against the merged RIOTBOX-1524 checkpoint
`670261d2a3d6113f3646cdb59806c409f8014da4` is empty for Core, App and actual
Sidecar paths. Every changed Audio source path is under `src/bin/`; library
runtime/source admission/DSP and frozen Stage-A implementation are unchanged.
W-30's shared PCM16 format constants are local to its standalone QA helper,
not a library-WAV rewrite. Only QA ownership and the two scoped capacity fixes
change in the sampled execution series.

Session publication remains the commit point over hash-bound graph generations.
Recovery save verifies the on-disk Session's exact graph/path, not the edited
in-memory state; unrelated alias I/O errors remain errors. Original WAV decode
and hash use the same bytes. Capture verification also binds decode/hash to one
read; explicit legacy adoption records current bytes as AdoptedLegacyV1, never
inventing historical identity or silently substituting source-window audio.

The actual callback in shared_transport_tr909.rs:404–499 retains preallocated
scratch, oversized-buffer silence/telemetry, prepared lane snapshots, source
monitor policy, limiter, capture and output order. No analysis, hashing, blocking
file I/O or model work was introduced. Reads and scratch tests do not prove
real-device behavior. Sidecar retains bounded pipe waiting/invalid-peer cleanup;
the deadline does not promise bounded JSON CPU, process spawn, kernel stalls or
descendants. Regular-file payload reads and Sidecar frame size remain uncapped.

Private QA mix callers prepare Grid-sized stems and retain final shape checks.
The generic slice helpers are not newly promoted to public validated-input
interfaces. Existing directly serialized metrics and fixed phrase/profile/mix
vocabulary remain diagnostic controls, not product intelligence or a human pass.

## Fresh integrated verification

All targeted tests below passed on the reviewed predecessor candidate
`6b908e51cf06f01388589792538f7a26997ffd82`. Its entire tree was verified equal
to merged main before synchronization; the post-merge standalone build and
CLI/hash rerun then used the integrated baseline above.

- Five QA binaries: 95 unit tests plus six actual CLI integration tests,
  101 total, `/tmp/riotbox-1532-qa-tests-reviewed.log`
- Core library: 470, `/tmp/riotbox-1532-core.log`
- Graph transaction/recovery: 16, `/tmp/riotbox-1532-persistence.log`
- Capture identity: eight, `/tmp/riotbox-1532-capture.log`
- Actual original-source admission: three,
  `/tmp/riotbox-1532-source-admission.log`
- Callback scratch sizing: two, `/tmp/riotbox-1532-callback.log`
- Actual `riotbox-sidecar`: 24 library tests and one working-directory-
  independent process integration, `/tmp/riotbox-1532-sidecar-reviewed.log`
- Verification command contract: two Python tests, including local Just argv
  probes, `/tmp/riotbox-1532-verification-argv.log`
- Post-merge standalone build: `/tmp/riotbox-1532-build.log`

Early incorrect QA-target and Sidecar-package lookups aborted before those tests;
their logs are diagnostic, excluded from green proof. Final target/package names
come from actual file/Cargo metadata, and every count above reflects executed
cases, not zero-test success. All final logs above were explicitly scanned
warning/error-free. No removed/ignored tests or source/device smoke substitution.

Fresh execution of all 31 actual CLI status/stdout/stderr cases is identical to
RIOTBOX-1531. All 39 hashes remain identical at the exact already-generated
paths under `/tmp/riotbox-1531-byteproof.MRxbva`, including 16 output WAVs,
complete reports/README/manifests and the four-second synthetic input. Both
manifests validate existing paths. No source directory is searched or real audio
opened; these are technical reruns, not newly qualified listening artifacts.

Full source-free `just ci` passed with actual exit zero and a warning/error-free
log, `/tmp/riotbox-1532-ci-final.log`. Exact-head native Rust/Windows Sidecar
review/merge remain separate gates, followed by archive and exact cleanup.
Windows transport CI does not prove Windows audio/filesystem/device behavior.

## Coverage limits and disposition

No real Development source, Holdout, commercial reference, device, DAW or
playback. This audit excludes whole-repo line-by-line review, decoder/numeric
fuzzing, huge-memory allocation, OS/filesystem matrices, power-loss/concurrent-
writer durability, allocation tracing and live terminal/device use. Finite but
impractically large renders remain a separate resource-policy question.

No new architecture/algorithm/threshold decision arose, so routine checkpoint
evidence adds no Decision Log entry. No new proven bug requires a follow-up;
RIOTBOX-1509 remains open. Reset this series count only after merge/closeout.
This maintenance grants no P023 musical, hardness, source or release claim.

Sequential docs branch/code-review checks scope, actual counts, source boundaries,
dependency/privacy claims, prior guard preservation and disposition. Incorrect
initial command diagnostics are excluded, and callback sizing is not mislabeled
as a device run. Follow-up self-review: zero remaining findings, no independent
panel. Remaining native/merge/archive/cleanup gates are not waived.

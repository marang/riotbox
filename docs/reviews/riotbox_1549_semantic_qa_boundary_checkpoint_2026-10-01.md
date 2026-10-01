# RIOTBOX-1549 semantic QA boundary checkpoint

Date: 2026-10-01. Classification: risk-directed current-state maintenance review.
Integrated code: `4c5212690136ae01adae799c409d22cbef4de4f7`.
Accepted candidate: `2e1446b094814784fa86ecf5973d1b8421dc4d2a`.
Previous checkpoint: [RIOTBOX-1545](riotbox_1545_qa_product_spine_checkpoint_2026-10-01.md),
code `fdd5651c0b069d6e2835bfea572a88504149628d`, report merge
`8629de2c8889f689e6fdc61cc6dc2fb3e6f7763b`.
Solo review-codebase; no independent panel or exhaustive line-by-line audit.

## Findings and disposition

No new demonstrated P0–P3 finding survives this sampled checkpoint. The final
semantic migrations preserve actual dependency ownership, complete definitions,
test identities and CLI/artifact behavior. Zero textual includes is a useful
guardrail outcome, not evidence of whole-repository architecture or audio quality.

The Feral root now owns composition/main, not hidden sibling compatibility
aliases. The remaining stem include becomes cohesive trigger rendering,
validation, artifact I/O and reproduction-command owners. Regression and
synthetic-fixture ownership retains stable test identities and actual imports.
There is no size-driven demand for further splitting or a replacement framework.

An initially suspected absent-output collision is rejected after tracing the
actual CLI contracts: W-30 has only one output argument and derives a distinct
metrics filename; Feral/Before/After have fixed distinct filenames; the
comparator explicitly rejects equal report/manifest paths. The shared physical
pair guard is not mistaken for that writer-level naming contract. No speculative
bug ticket or production change follows from this countercheck.

RIOTBOX-1509 remains Todo: successful Windows Sidecar runs are finite evidence,
not minimized reproduction or repair of its sporadic fixture startup failure.
Human limiter calibration and real-host DAW proof remain gated. A separate
existing historical-review documentation task, RIOTBOX-1416, is a possible next
source-free slice after validating its original evidence; TUI work is not
automatically promoted over the user's late-priority preference.

## System model and dependency boundaries

| Owner | Actual responsibility and dependency direction |
| --- | --- |
| Feral entrypoint | CLI parse/help and render_pack delegation; ordinary private modules, no root evidence/schema/test alias namespace. |
| Pack builder | Source/format/window/capacity admission and existing analysis/render/validate/publication order; composes owners, not a second product engine. |
| Trigger renderer | Existing W-30 chop/trigger preparation and rendering; policy/data types come from actual siblings and Audio owners. |
| Pack validation | Existing report assertions and complete validation thresholds stay together. No new product qualification or numerical policy. |
| Artifact I/O | WAV/metric writing and private formatting; reproduction-command owner produces quoted command text, never executes it. |
| Evidence/publication | Ephemeral 31-field report, unchanged manifest, Markdown/README presentation; direct owner imports, not Session/replay truth. |
| Regression/fixtures | Ordinary test-only owners, retained names/assertions and synthetic controls; no production fallback or new test facade. |
| QA path safety | Four CLI-derived plans admit destinations and then compare existing physical identities. No file guard enters realtime. |
| Core/Session/App | Core owns actions, commits/replay and persisted Source/Graph/capture identity. App projects/hydrates these contracts; runtime status is not persisted truth. |
| Audio callback | Prepared snapshots and preallocated scratch, lane/monitor/fill/limiter/capture/output ordering; no offline QA I/O, analysis or hashing. |
| Sidecar | Protocol/request identity and absolute exchange deadline; invalid transport is discarded, fixtures do not qualify source intelligence. |

Cargo remains App → Audio/Core/Sidecar and Audio/Sidecar → Core. Exact Git
comparison since the previous report merge is empty for Core, App, Sidecar,
Audio library/runtime/source admission, Cargo manifests/lockfile and owning
specs/benchmarks. Production changes are confined to the Feral offline binary
family. Frozen Stage-A protocol/matrix/registry/pre-admission evidence is unchanged.

## Current-state tracing and counterchecks

Read canonical module/include, artifact/source/mutual-output and Session
identity/save/load contracts, previous checkpoint and recent owning decisions.
Trace current root, pack builder, trigger/validation/I/O/reproduction owners,
manifest/presentation and actual regression imports rather than relying on the
include inventory. Unchanged analysis/error/publication order is preserved.
Directory creation can precede rejection; WAV/metrics can precede final report
validation. These guards do not promise atomic whole-pack publication.

Trace the four actual entrypoints and writer-derived plans: Feral nineteen,
Before/After fourteen, W-30 two, comparator two. Source protection and mutual
output protection remain separate. Only NotFound means an absent destination;
dangling/nonregular/unknown entries fail closed. Existing regular paths use the
same read-only physical identity owner; independent equal-content files remain
valid. Absent/singleton sets require no pair handles. Stable namespace remains
an explicit assumption, not an adversarial race guarantee.

Recheck graph recovery→save against the on-disk Session's exact path/hash,
immutable-generation publication and alias/Session commit order. Corrupt/missing
aliases can recover only through the authoritative generation; unrelated I/O
errors remain errors. Hardlink support has its declared typed failure, no unsafe
copy fallback or newest-generation scan. Capture creation uses fresh exclusive
allocation; hydration hashes and decodes the same admitted buffer. Legacy
adoption establishes current bytes, not retrospective authenticity. Original
source identity and capture no-follow admission remain distinct contracts.

Recheck replay's Action-ID index and uniqueness sets, actual callback scratch
overflow silence/telemetry, prepared snapshots and output ordering. Recheck
Sidecar write/flush/read deadline and invalidation. No second action/replay/
persistence model, app-local expected identity, source fallback, library/DSP
change or realtime dependency on offline QA is introduced by these migrations.

## Accepted verification

RIOTBOX-1548 exact accepted candidate equals its normally merged whole tree.
Fresh completed proof: 49 identical resolved manifest bindings; fourteen whole
manifest definitions; unchanged verification/BPM/W-30 definitions (3/10/10);
eleven product-stem definitions with only the explicitly accounted same-function
call qualification; all 58 root definitions match by stable module identity.
See [the migration report](riotbox_1548_feral_root_direct_owners_2026-10-01.md).

Fresh Debug/Release retain the same 143 executed identities/statuses. Completed
standalone build precedes all 141 parsed CLI records and 85 baseline-identical
artifact hashes. Parsed JSON normalizes CLI JSON whitespace only; artifact
hashes are byte-exact. The original Feral alias rejection preserves generated
input/prior outputs; both comparison manifests require existing artifacts.
Full source-free candidate just ci exits zero, warning/error-free:
`/tmp/riotbox-1548-ci-final.log`. Native run 36858818314 passes both Rust and
Windows Sidecar before feature #1640 merges; fresh comments/reviews are empty.

The unchanged product-spine suites retain the previous checkpoint's executed
528-test evidence (Core 470, graph 16, capture eight, App source three, Audio
source four, callback two, Sidecar 24+1). These targeted suites are not claimed
as separately rerun for this report. Fresh full source-free report-branch
just ci completes with actual exit zero, warning/error-free:
`/tmp/riotbox-1549-ci-final.log`. Final fmt/diff and zero-include guard pass.
Native report/archive CI, fresh reviews, merge/main and exact cleanup remain
separate obligations. No routine Decision Log entry is appropriate for this
checkpoint; it accepts no new architecture/algorithm/threshold contract.

## Coverage limits

Risk-directed current-state review, not exhaustive repo audit, decoder fuzzing,
allocation tracing, huge-source/frame stress, hostile peers/namespaces, atomic
pack publication, multiwriter/power-loss durability or OS/filesystem/device
matrix. Source/capture/frame reads have no newly established universal byte cap;
Sidecar deadlines do not bound JSON CPU, spawn/kernel delay or descendants.
Windows Sidecar CI is not Windows Audio or physical file-identity proof.

Only generated controls/retained fixtures. No real Development/Holdout/commercial
audio, source-directory discovery, DAW/device/playback, subagents or human verdict.
No source-general/diversity/music/hardness/release pass or P023 completion. Solo
branch review checks report accuracy, contract/evidence separation and
remaining workflow obligations; short self-review finds no open report defect.
Source/human/host gates are not waived.

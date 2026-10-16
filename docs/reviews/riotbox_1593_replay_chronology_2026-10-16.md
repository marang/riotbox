# RIOTBOX-1593 — durable replay chronology across navigation

Date: 2026-10-16. Maintenance protecting P023 recall, performer state and
recording/export lineage under RIOTBOX-1041. Base: merged RIOTBOX-1592,
`6f76df8eecb60cd048107bf3c57a67465574edee`. Contract: RBX-454,
[Replay §7.2](../specs/replay_model_spec.md#72-action-ordering) and Session §9.

## Reproduced defects

The reference-free public App sequence commits Blend at beat12, Seek0 at beat12,
then Source at beat0. Ordinary save/load preserves its history, but replay sorted
durable action order 1,2,3 into 3,1,2 and restored Blend instead of Source.
The correct-seam standalone regression failed with exit101, not a fixture setup
error: `/tmp/riotbox-1593-restore-order-red.log`. Source witness
`/tmp/riotbox-1593-restore-order-red.rs`, SHA-256
`9a3fc21384871fe81512a01ef41674cee7ac236bf5467456d620b47ef6dfb0ea`.
Earlier stale-library link failures were not counted as regression evidence.

Three additional Core RED tests demonstrate wrong full order, snapshot/target
suffix order and revisited-boundary order, with IDs and wall clocks deliberately
unrelated to chronology. Two Undo RED tests demonstrate timestamp rollback
refusing a valid marker and an MC-202 barrier wrongly blocking an independent
monitor target. Logs: `/tmp/riotbox-1593-core-chronology-red.log` and
`/tmp/riotbox-1593-undo-chronology-red.log`. Their retained tests now pass.

## Ownership and compatibility

Fresh ActionLog/Session defaults declare required
`ReplayPolicy::DeterministicCommittedOrderV2`. The existing Session V1 envelope
and ActionLog fields stay unchanged; old enum readers reject this new value.
An optional ignored field could not safely protect against old-reader downgrade.
Append chronology is validated against the complete committed/undone action
subsequence before executable filtering. Each full boundary retains its own
increasing, noncontiguous sequence; neither musical position, ActionId nor wall
clock supplies global chronology. Structural Undo uses existing typed domains
and transient per-domain stacks, not a new history or persisted cursor.

Legacy policies retain declared sorting and ordinary roundtrip semantics.
Known explicit backward Seek/Stop histories fail with upgrade-required evidence;
ordinary legacy source-map backward gestures refuse before enqueue/ID allocation.
The conservative explicit Core upgrade changes only the policy after old
canonical, stored-record and action orders agree and all contracts validate.
Incomplete or ambiguous histories refuse without repair or partial mutation.
It is never invoked by load/save/recording. This is not a general adoption tool
for arbitrary historical restore-and-edit histories or unknown provenance.
The frozen generated loaded-recorder V1 producer explicitly pins legacy policy;
the historical partial-history test factory does likewise rather than weakening
fresh V2 validation or inventing its missing record.

## Verification

- Public App regressions exercise real queue/commit, save/load and suffix
  restore, including metadata-only SourceMap PreviousBar. New controls exercise
  real Undo with clock rollback, post-Undo payload anchors, an MC-202 timing
  barrier with independent monitor Undo, and unchanged legacy file bytes.
- A generated in-memory PCM control compares actual Source Monitor output
  before and after backward-history restore, and distinguishes the old incorrect
  Blend result. It is offline output-path evidence, not callback/host proof.
- V2 all-prefix scan-oracle comparisons cover every target/snapshot cursor,
  repeated/backward/forward boundary coordinates, typed Undo stacks, timestamp
  rollback, rejected actions/results, unsafe anchors and out-of-range cursors.
  Existing explicitly legacy permutation/oracle tests remain intact.
- Core capture reconstruction and export receipt planning controls retain
  action-bound identities across backward coordinates and JSON roundtrip,
  without opening referenced artifact paths or republishing side effects.
- Compatibility controls cover old enum rejection, required policy, legacy
  roundtrip, safe/idempotent explicit upgrade, refused ambiguous/incomplete
  upgrade nonmutation, missing records, orphan Undone targets, local sequence
  order and structural/failed-result records. No production validation is relaxed.
- Full Core library: 542 passed, one existing informational benchmark ignored
  (`/tmp/riotbox-1593-core-suite-final.log`). The generated preparation suite
  passes five tests, including all six 44.1/48kHz windows and identity/mutation
  controls (`/tmp/riotbox-1593-generated-producer.log`). Full local/native gates
  and final delta review remain mandatory before merge; no pending result is
  inferred from these focused checks.
- Initial independent read-only Rust/adversarial and spec/evidence reviews retain
  no production findings. Their remaining coverage questions prompted the new
  cursor, post-Undo, generated-output and capture/export controls above.

## Limits

No actual host/device/service observation or runtime, real source/capture,
Holdout/commercial/ignored-attempt access, DAW or human playback. All audio
evidence uses newly generated fixtures or in-memory PCM. No new command, App
state, DSP/recipe/threshold, recording/observer schema, dependency, Windows work
or source/musical/human/hardness/latency/endurance/release verdict. Consumed
RIOTBOX-1589 stays consumed; RIOTBOX-1041/1036 and the original ALSA cause remain
open. Temporary diagnostic logs are local operational evidence; durable tests
and this contract travel with the PR.

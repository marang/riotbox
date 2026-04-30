# Riotbox Recovery Notes

Status: current MVP manual-recovery guidance

This note explains the current recovery-facing meaning of replay and snapshot
payload diagnostics. It does not define automatic recovery behavior.

## Current Rule

Riotbox treats recovery as an explicit operator action for now.

- normal session load is deterministic and side-effect free
- adjacent temp or autosave-like files may be shown as clues, not chosen automatically
- snapshot payloads accelerate restore only when an explicit restore helper uses them
- the action log and structured commit records remain the replay truth

If Riotbox says a session is parseable or a payload is ready, that is evidence
for a possible recovery path. It is not permission to silently replace the
requested session file or skip replay validation.

## Artifact Availability Labels

Manual recovery candidates may also show W-30/capture artifact availability:

- `artifacts n/a | no captures`
  The parseable session does not reference persisted capture artifacts.
- `artifacts ready: N capture(s)`
  Every referenced capture `storage_path` resolves to a file next to the
  candidate session or through an absolute path.
- `artifacts blocked: X of N | ...`
  One or more referenced captures have missing identity, missing files, or
  unreadable/non-file paths. Recovery must report this as a blocker before any
  future artifact hydration path can use those captures.
- `artifacts unchecked`
  The candidate is not parseable session JSON, so artifact availability was not
  inspected.

These labels are diagnostics only. They do not hydrate, regenerate, repair, or
choose artifacts.

## Artifact Hydration Preflight

Before a future W-30 artifact hydrator can use a `CaptureRef`, the app-level
preflight must reject missing storage identity, unavailable session-relative
paths, missing files, unreadable paths, and paths that are not files.

The current preflight only proves the boundary and keeps cache refresh aligned
with it. It does not decode the artifact into replay state or synthesize a
replacement.

## Snapshot Payload Labels

The app runtime view may show these labels:

- `payload none | full replay`
  No usable snapshot anchor exists for the current target. Recovery would need
  replay from the origin action log.
- `payload missing | snapshot restore blocked`
  A snapshot anchor exists, but it has no payload. The session may still load
  normally, but snapshot-payload restore cannot use that anchor.
- `payload ready | snapshot restore ok`
  The selected snapshot anchor has an identity-matched payload. An explicit
  snapshot-payload restore helper can hydrate from that payload, then run the
  target suffix.
- `payload invalid | snapshot restore blocked`
  The selected anchor's payload identity does not match the owning snapshot.
  Persisted sessions with this mismatch should already be rejected during load.

## Unsupported Suffix Labels

Payload readiness is not the whole recovery decision. A ready payload can still
be blocked if the action suffix after the snapshot contains commands the replay
executor cannot safely apply yet.

Current unsupported examples include artifact-producing W-30 actions such as
`w30.loop_freeze`. These must reject without partially mutating the app session.
That is intentional until capture/resample artifact hydration has a durable
replay boundary.

## Current Verification Seams

Use these probes when changing recovery behavior:

- `cargo test -p riotbox-app app_snapshot_payload_restore_rejects -- --nocapture`
- `cargo test -p riotbox-app runtime_view_surfaces_snapshot_payload_readiness -- --nocapture`
- `cargo test -p riotbox-app recovery_surface_reports_capture_artifact_availability_for_parseable_candidates -- --nocapture`
- `cargo test -p riotbox-app capture_artifact_hydration_preflight -- --nocapture`
- `cargo test -p riotbox-app save_materializes_payload_for_latest_explicit_snapshot_and_restore_uses_it -- --nocapture`
- `cargo test -p riotbox-app w30_snapshot_payload_restore_runner_matches_committed_app_preview_output -- --nocapture`
- `cargo test -p riotbox-core snapshot_payload_hydration -- --nocapture`
- `just ci`

The first probe verifies app-level failure paths for missing payloads and
unsupported suffixes. The second verifies the visible diagnostic labels. The
third verifies the current save-time producer boundary for latest explicit
snapshots. The W-30 probe verifies payload-backed restore can hydrate a
source-backed W-30 anchor, apply a safe suffix, and match committed preview
output. The core probe verifies the lower-level hydration contract.

## Out Of Scope Today

Current Riotbox recovery does not yet provide:

- automatic startup recovery selection
- automatic repair of invalid payloads
- automatic snapshot creation or snapshot-frequency policy
- capture/resample artifact recreation during replay
- a full replay-from-origin runner for every action family

Those behaviors should be added only through bounded slices that preserve the
single replay truth: frozen source references, frozen Source Graph references,
structured committed action history, optional snapshots, and explicit artifacts.

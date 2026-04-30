# Riotbox Recovery Notes

Status: current MVP manual-recovery guidance

This note explains the current recovery-facing meaning of replay and snapshot
payload diagnostics. It does not define automatic recovery behavior.

## Current Rule

Riotbox treats recovery as an explicit operator action for now.

- normal session load is deterministic and side-effect free
- adjacent temp or autosave-like files may be shown as clues, not chosen automatically
- interrupted-save shapes with a broken requested session plus adjacent
  parseable temp/autosave files must remain manual-review surfaces only
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

## Candidate Decision Labels

Manual recovery candidates may show a compact decision diagnostic:

- `decision: normal load path`
  The candidate is the requested session path and can be loaded normally.
- `decision: reviewable | explicit user choice required`
  The candidate is parseable and not blocked by replay or artifact checks, but
  still must not be selected automatically.
- `decision: reviewable | full replay required`
  The candidate is parseable, but snapshot-payload restore is unavailable and a
  future recovery action would need full replay.
- `decision: blocked | artifacts unavailable`
  The candidate references capture artifacts that are missing, unreadable, or
  missing storage identity.
- `decision: blocked | replay unsupported`
  The candidate contains an unsupported replay origin or suffix command.
- `decision: blocked | replay and artifacts`
  The candidate has both an unsupported replay command and unavailable capture
  artifacts.
- `decision: broken candidate`
  The candidate is unreadable or invalid session JSON.
- `decision: normal target missing`
  The requested normal session path does not exist.

These decision labels are deliberately read-only. They summarize why a candidate
needs manual review and must not choose, repair, or replace a session file.

When artifacts are ready but replay is blocked by an unsupported
artifact-producing command such as `capture.now` or `capture.loop`, the manual
recovery UI may say that audio artifacts are present while replay remains
blocked until that command has an explicit artifact hydrator. That hint is
explanatory only; it must not select a candidate or silently repair the session.

## Artifact Hydration Preflight

Before a W-30 artifact hydrator can use a `CaptureRef`, the app-level preflight
must reject missing storage identity, unavailable session-relative paths,
missing files, unreadable paths, and paths that are not files.

The current app preflight only proves the file boundary and keeps cache refresh
aligned with it. The core replay contract seam is
`plan_w30_artifact_replay_hydration`; it validates explicit session identity for
artifact-producing W-30 actions before any file is loaded. `w30.loop_freeze` and
`promote.resample` can now use that identity through the replay executor to
point W-30 state at an already persisted artifact; they still do not recreate
capture audio or synthesize a replacement.

Recovery artifact-availability labels should use this same preflight classifier
so UI diagnostics and future hydration gates cannot drift.

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

Current unsupported examples include broader artifact-producing capture actions
such as `capture.now` and `capture.loop`. These must reject without partially
mutating the app session. `capture.bar_group`, `w30.capture_to_pad`,
`w30.loop_freeze`, `promote.resample`, and metadata-only capture promotion
suffixes have bounded replay support when explicit persisted identity is
present.

## Current Verification Seams

Use these probes when changing recovery behavior:

- `cargo test -p riotbox-app app_snapshot_payload_restore_rejects -- --nocapture`
- `cargo test -p riotbox-app runtime_view_surfaces_snapshot_payload_readiness -- --nocapture`
- `cargo test -p riotbox-app recovery_surface_reports_capture_artifact_availability_for_parseable_candidates -- --nocapture`
- `cargo test -p riotbox-app capture_artifact_hydration_preflight -- --nocapture`
- `cargo test -p riotbox-app reloaded_session_uses_capture_artifact_cache_without_source_audio -- --nocapture`
- `cargo test -p riotbox-app committed_w30_internal_resample_prints_reusable_bus_artifact -- --nocapture`
- `cargo test -p riotbox-app save_materializes_payload_for_latest_explicit_snapshot_and_restore_uses_it -- --nocapture`
- `cargo test -p riotbox-app w30_snapshot_payload_restore_runner_matches_committed_app_preview_output -- --nocapture`
- `cargo test -p riotbox-app w30_snapshot_payload_restore_hydrates_loop_freeze_artifact_preview_output -- --nocapture`
- `cargo test -p riotbox-app w30_snapshot_payload_restore_hydrates_promote_resample_artifact_preview_output -- --nocapture`
- `cargo test -p riotbox-app w30_snapshot_payload_restore_replays_promote_capture_to_pad_for_resample_artifact -- --nocapture`
- `cargo test -p riotbox-app snapshot_payload_restore_hydrates_capture_bar_group_artifact_preview_output -- --nocapture`
- `cargo test -p riotbox-core snapshot_payload_hydration -- --nocapture`
- `just ci`

The first probe verifies app-level failure paths for missing payloads and
unsupported suffixes. The second verifies the visible diagnostic labels. The
third verifies the current save-time producer boundary for latest explicit
snapshots. The W-30 probe verifies payload-backed restore can hydrate a
source-backed W-30 anchor, apply a safe suffix, and match committed preview
output. The loop-freeze and promote-resample artifact probes verify replay can
reload a session from JSON, hydrate persisted W-30 WAV artifacts, and avoid
fallback oscillator collapse. The promote-capture-to-pad probe verifies the
explicit `[p]` gesture can be replayed for a printed resample artifact and still
drive artifact-backed W-30 playback. The capture-bar-group probe verifies a
persisted source-window capture can be replayed into W-30 `last_capture` /
live-recall state without recreating audio. The internal-resample probe verifies
that same gesture can assign a printed resample artifact to a W-30 pad before
source-less reload. The core probe verifies the lower-level hydration contract.

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

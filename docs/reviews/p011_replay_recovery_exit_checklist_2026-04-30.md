# P011 Replay / Recovery Exit Checklist

Date: 2026-04-30
Scope: P011 Pro Hardening checkpoint after snapshot-payload restore readiness,
manual recovery notes, and stage-style payload-readiness smoke coverage.

## Verdict

P011 is still active and not exit-ready.

The current repo has useful replay and recovery evidence, but that evidence is
bounded. It proves important seams, not complete product-grade recovery.

## Green Gates To Keep Running

- `just ci`
  Runs formatter, full Rust tests, audio QA smoke gates, observer/audio
  correlation probes, stage-style probe, offline render reproducibility smoke,
  and clippy.
- `just stage-style-jam-probe`
  Exercises a longer generated W-30 source-diff output, observer/audio
  correlation, and the current latest-explicit-snapshot payload readiness check.
- `just offline-render-reproducibility-smoke`
  Proves one deterministic source-backed W-30 render emits byte-stable WAV
  output for the same generated source.
- `cargo test -p riotbox-app app_snapshot_payload_restore_rejects -- --nocapture`
  Proves app-level snapshot-payload restore failure paths are explicit and
  non-mutating.
- `cargo test -p riotbox-app runtime_view_surfaces_snapshot_payload_readiness -- --nocapture`
  Proves the app runtime view surfaces selected-anchor payload readiness labels.
- `cargo test -p riotbox-app save_materializes_payload_for_latest_explicit_snapshot_and_restore_uses_it -- --nocapture`
  Proves the current save path materializes payloads for latest explicit
  snapshots and reload reports payload readiness as ready.
- `cargo test -p riotbox-app w30_snapshot_payload_restore_runner_matches_committed_app_preview_output -- --nocapture`
  Proves a source-backed W-30 snapshot payload can hydrate an anchor, apply a
  safe trigger suffix, and match committed preview output.
- `cargo test -p riotbox-core snapshot_payload_hydration -- --nocapture`
  Proves the core snapshot-payload hydration boundary and rejection paths.

## Proven Today

- Session load validates structured commit-record integrity instead of trusting
  malformed replay metadata.
- Restore diagnostics can explain anchor, suffix, unsupported commands, and
  selected-anchor payload readiness without mutating runtime state.
- Snapshot-payload hydration exists as an explicit helper: payload first, then
  the existing target-suffix executor.
- Source-backed W-30 snapshot-payload restore has a bounded preview parity
  probe for the browse-anchor plus trigger-suffix path.
- W-30 artifact hydration has an explicit identity boundary in the session and
  replay specs before any future artifact-producing replay support is added.
- Manual recovery candidates can report capture artifact availability as
  read-only diagnostics before any future hydration path uses those artifacts.
- The app has a W-30 capture artifact hydration preflight that rejects missing
  storage identity or missing artifact files before decoding or cache use.
- Recovery artifact-availability diagnostics use the same preflight classifier
  as cache/hydration readiness, reducing drift between UI and restore gates.
- Reloaded sessions can repopulate W-30 capture audio cache from an existing
  artifact and keep artifact-backed pad playback audible without source audio.
- Internally printed W-30 resample artifacts can survive save/reload with
  lineage/depth intact, pass preflight, and drive artifact-backed pad playback.
- The existing explicit `[p]` / `promote.capture_to_pad` gesture can assign a
  printed resample artifact to the focused W-30 pad before that source-less
  reload path.
- Missing payloads, mismatched payload identity, and unsupported suffixes reject
  instead of silently falling back.
- Save may materialize payloads only for existing explicit snapshots at the
  latest action cursor.
- Manual recovery scanning is read-only and does not choose, load, replace,
  delete, or repair candidates.
- Stage-style QA now includes bounded output-path evidence plus a payload
  readiness guard.
- Offline render reproducibility has one CI-safe smoke gate.

## Still Not Proven

- Full replay-from-origin for every replay-relevant action family.
- Automatic startup recovery selection.
- Interactive guided recovery that can safely choose a candidate with user
  confirmation.
- Capture/resample artifact hydration for artifact-producing W-30 actions.
- Full arrangement export, stems, recording, or manifest-normalized export
  reproducibility.
- Long-run soak behavior over extended live sessions.
- Real user-session recovery drills after an interrupted multi-file save.

## Exit-Ready Requirements

P011 should not be called exit-ready until these are true:

- A full replay or staged replay path can reconstruct a useful session without
  relying only on persisted latest runtime state.
- Snapshot-payload restore and replay-from-origin converge for the important
  command families that are replay-safe today.
- Artifact-producing W-30 / capture actions either hydrate from durable
  artifacts or reject with clear user-facing recovery instructions.
- Manual recovery can guide a user through an explicit candidate choice without
  hiding replay truth.
- Long-run and stage-style probes include enough action diversity to catch
  drift beyond the current smoke-level W-30 source-diff run.
- Export reproducibility exists for the real export surface, not only a helper
  render.

## Next Slice Guidance

Prefer small P011 slices that strengthen one seam at a time:

- enrich read-only manual recovery candidate diagnostics
- add one command-family replay convergence proof at a time
- expand stage-style probes only when they catch a named risk
- add artifact hydration only when the artifact identity and provenance are
  explicit in session state

Do not add automatic fallback selection or silent repair before the manual
recovery path is explicit and testable.

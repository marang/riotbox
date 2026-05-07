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
  Exercises a longer generated W-30 source-diff output, generated app-level
  multi-boundary observer evidence, summary-level commit boundary assertions,
  observer/audio correlation, and the current latest-explicit-snapshot payload
  readiness check.
- `just stage-style-snapshot-convergence-smoke`
  Proves a supported Scene / MC-202 / TR-909 stage-style sequence can restore
  from a mid-run snapshot payload and converge to the same final mixed buffer
  as the originally committed path.
- `just offline-render-reproducibility-smoke`
  Proves one deterministic source-backed W-30 render emits byte-stable WAV
  output for the same generated source.
- `just product-export-reproducibility-smoke`
  Proves the current Feral grid generated-support full-mix export seam is
  reproducible after normalizing temp/source paths away from the listening
  manifest and comparing stable audio artifact hashes.
- `cargo test -p riotbox-app app_snapshot_payload_restore_rejects -- --nocapture`
  Proves app-level snapshot-payload restore failure paths are explicit and
  non-mutating.
- `cargo test -p riotbox-app snapshot_payload_restore -- --nocapture`
  Proves the current app-level snapshot-payload restore parity suite across
  W-30, TR-909, MC-202, Scene Brain, and failure-path seams.
- `cargo test -p riotbox-app runtime_view_surfaces_snapshot_payload_readiness -- --nocapture`
  Proves the app runtime view surfaces selected-anchor payload readiness labels.
- `cargo test -p riotbox-app save_materializes_payload_for_latest_explicit_snapshot_and_restore_uses_it -- --nocapture`
  Proves the current save path materializes payloads for latest explicit
  snapshots and reload reports payload readiness as ready.
- `cargo test -p riotbox-app mc202_snapshot_payload_restore_hydrates_answer_projection -- --nocapture`
  Proves an MC-202 snapshot payload can hydrate a follower phrase anchor, apply
  an answer suffix, and match committed bass render output.
- `cargo test -p riotbox-app mc202_snapshot_payload_restore_hydrates_phrase_mutation_projection -- --nocapture`
  Proves the current MC-202 snapshot-payload restore chain reaches the
  phrase-mutation suffix with committed bass render parity.
- `cargo test -p riotbox-app scene_restore_snapshot_payload_restore_matches_committed_movement_projection -- --nocapture`
  Proves graph-aware Scene restore snapshot-payload hydration converges across
  Scene state, Jam projection, TR-909 / MC-202 render state, and mixed output.
- `cargo test -p riotbox-app tr909_snapshot_payload_restore_hydrates_takeover_release_projection -- --nocapture`
  Proves a TR-909 snapshot payload can hydrate a takeover anchor, apply a
  release suffix, and match committed drum render output.
- `cargo test -p riotbox-app w30_snapshot_payload_restore_runner_matches_committed_app_preview_output -- --nocapture`
  Proves a source-backed W-30 snapshot payload can hydrate an anchor, apply a
  safe trigger suffix, and match committed preview output.
- `cargo test -p riotbox-app w30_snapshot_payload_restore_hydrates_damage_profile_preview_output -- --nocapture`
  Proves a source-backed W-30 snapshot payload can hydrate a browse anchor,
  apply a damage-profile suffix, and match committed grit/preview output.
- `cargo test -p riotbox-app snapshot_payload_restore_hydrates_capture_now_artifact_preview_output -- --nocapture`
  Proves an immediate source-window capture can hydrate from a persisted
  artifact and match committed W-30 preview output without falling back.
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
  replay specs, and `w30.loop_freeze` plus `promote.resample` are now the first
  narrow artifact-backed replay suffixes that can hydrate from persisted capture
  identity.
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
  reload path, and replay can now restore that pad assignment when it appears as
  a snapshot suffix.
- `promote.capture_to_scene` can replay metadata-only assignment for an existing
  capture and explicit scene target without creating or regenerating audio.
- `w30.capture_to_pad` can replay a persisted source-window-backed pad capture
  artifact and drive artifact-backed W-30 playback without source audio.
- `capture.now` and `capture.loop` can replay persisted source-window-backed
  loop capture artifacts into W-30 `last_capture` / live-recall state and drive
  artifact-backed W-30 playback without source audio.
- App-level snapshot-payload restore now proves the immediate `capture.now`
  variant against committed W-30 preview output and fallback-collapse controls.
- Recovery scanning now reports broken immediate `capture.now` artifact suffixes
  as read-only, non-selected candidates with explicit missing artifact identity
  guidance instead of silently falling back.
- `capture.bar_group` can replay a persisted source-window-backed pad capture
  artifact into W-30 `last_capture` / live-recall state and drive
  artifact-backed W-30 playback without source audio.
- W-30 damage-profile suffix replay can converge through snapshot-payload
  restore on the same committed grit state and preview output as the normal app
  commit path.
- TR-909 takeover/release suffix replay can converge through snapshot-payload
  restore on the same committed support/takeover state and drum render output
  as the normal app commit path.
- MC-202 answer suffix replay can converge through snapshot-payload restore on
  the same committed phrase lane state and bass render output as the normal app
  commit path.
- MC-202 pressure, instigator, and phrase-mutation suffixes can now also
  converge through snapshot-payload restore on committed lane state and bass
  render output, giving the current MC-202 recipe chain app-level restore
  parity coverage.
- Scene restore can converge through graph-aware snapshot-payload restore on
  committed active/restore scene state, Jam scene projection, TR-909 / MC-202
  render state, and mixed output.
- A supported stage-style Scene / MC-202 / TR-909 sequence can now converge
  from a mid-run snapshot payload through the latest-snapshot summary path with
  no unsupported suffix commands and final mixed-output parity.
- App test helpers now include a graph-aware snapshot restore anchor path so
  Scene-family parity proofs do not have to hand-roll anchor runtime state.
- Missing payloads, mismatched payload identity, and unsupported suffixes reject
  instead of silently falling back.
- Save may materialize payloads only for existing explicit snapshots at the
  latest action cursor.
- Manual recovery scanning is read-only and does not choose, load, replace,
  delete, or repair candidates.
- A bounded interrupted-save drill now covers a broken requested session path
  with parseable adjacent temp/autosave candidates and proves the recovery
  surface still selects nothing and mutates no candidate file.
- Recovery observer evidence now has CI-safe file-backed startup drills for
  interrupted temp/autosave files and missing canonical session paths, both
  preserving the read-only manual recovery boundary.
- Recovery candidate diagnostics now expose a compact replay-family label, so
  read-only startup and help surfaces can distinguish no-replay, Scene,
  MC-202, TR-909, W-30, and mixed suffix candidates without executing restore.
- Stage-style QA now includes bounded output-path evidence plus a payload
  readiness guard.
- Offline render reproducibility has one CI-safe smoke gate.
- The current Feral grid generated-support full-mix seam has a normalized
  product-export reproducibility proof. This is still bounded and does not
  cover full arrangement export, stem packages, or live recording export.

## Snapshot Restore Parity Coverage Matrix

This matrix is the current P011 snapshot-payload restore coverage index. It is
not a full P011 exit declaration.

The machine-checkable companion index lives at
`docs/benchmarks/p011_replay_family_manifest.json` and is validated by:

```bash
just p011-replay-family-manifest
```

That validator checks that each claimed proof points at existing repo-local test
files and test functions. It does not replace the tests themselves and must not
be read as a full replay-from-origin claim.

| Area | Covered suffix / family | Proof file | Output proof |
| --- | --- | --- | --- |
| Core hydration boundary | payload clone, cursor/identity rejection, target suffix execution | `crates/riotbox-core/src/replay/target_execution.rs` | state convergence and explicit rejection paths |
| App failure boundary | missing payload, unsupported suffix, invalid payload identity | `crates/riotbox-app/src/jam_app/tests/snapshot_payload_restore_failures.rs` | non-mutating rejection proof |
| W-30 cue path | browse anchor -> trigger suffix, browse anchor -> promoted audition suffix | `crates/riotbox-app/src/jam_app/tests/w30_replay.rs` | committed preview buffer parity and browse-vs-trigger / browse-vs-audition deltas |
| W-30 damage path | browse anchor -> damage-profile suffix | `crates/riotbox-app/src/jam_app/tests/w30_replay.rs` | committed grit/preview parity and browse-vs-damage delta |
| W-30 artifact path | loop-freeze, promote.resample, promote.capture_to_pad for resample artifacts | `crates/riotbox-app/src/jam_app/tests/w30_loop_freeze_artifact_replay.rs`, `w30_resample_artifact_replay.rs`, `w30_resample_promotion_replay.rs` | artifact-backed preview/pad buffer parity and fallback-collapse controls |
| Capture artifact path | capture.now, capture.loop, capture.bar_group, w30.capture_to_pad | `crates/riotbox-app/src/jam_app/tests/w30_capture_now_replay.rs`, `w30_capture_loop_replay.rs`, `w30_capture_bar_group_replay.rs`, `w30_capture_to_pad_replay.rs` | artifact-backed W-30 preview/live-recall/pad output parity |
| TR-909 support path | fill -> slam, takeover -> release, reinforce -> scene lock | `crates/riotbox-app/src/jam_app/tests/tr909_replay.rs` | committed drum render parity and pre/post movement delta |
| MC-202 phrase path | follower -> answer -> pressure -> instigator -> phrase mutation | `crates/riotbox-app/src/jam_app/tests/mc202_restore_replay.rs` | committed bass render parity and adjacent phrase-shape deltas |
| Scene Brain movement path | scene.launch anchor -> scene.restore suffix | `crates/riotbox-app/src/jam_app/tests/scene_replay.rs` | Scene/Jam/runtime convergence plus mixed-output parity and launch/restore deltas |
| Stage-style supported seam | scene.launch + MC-202 answer anchor -> TR-909 fill/slam + scene.restore suffix | `crates/riotbox-app/src/jam_app/tests/stage_style_snapshot_convergence.rs` | latest-snapshot readiness, unsupported-count zero, final mixed-output parity, and initial/anchor deltas |
| Save / reload payload readiness | latest explicit snapshot payload materialization | `crates/riotbox-app/src/jam_app/tests/persistence_runtime_view.rs` | reload readiness and app restore usability |
| Recovery diagnostics | missing artifact identity, app-invalid payload candidates, replay-family labels, read-only candidate status | `crates/riotbox-app/src/jam_app/tests/recovery_*` and `artifact_hydration_preflight.rs` | user-facing guidance and non-mutating scans |

Coverage remains intentionally command-family based. The matrix should grow one
bounded replay-safe family at a time rather than becoming a speculative feature
list.

## Recovery Observer Drill Coverage Matrix

This matrix is the current P011 recovery-observer coverage index. It documents
startup evidence shapes only; it is not a promise that Riotbox can automatically
choose or restore any candidate.

| Shape | Proof | Evidence | Explicit boundary |
| --- | --- | --- | --- |
| Normal recovery scan with adjacent clues | `cargo test -p riotbox-app recovery_surface -- --nocapture` | read-only candidate list, no selected candidate, no file mutation | surface diagnostics only, no observer stream required |
| Manual choice dry-run | `cargo test -p riotbox-app recovery_surface_dry_runs_manual_choice_without_selecting_or_mutating_files -- --nocapture` | candidate labels mirrored into a dry-run result with `selected_for_restore == false` | preflight only, no restore execution |
| App-invalid autosave candidate | `cargo test -p riotbox-app --bin riotbox-app recovery_observer -- --nocapture` | observer snapshot reports `app-invalid session`, `BrokenClue`, and no dry-run candidate | broken clue only, no repair or payload rewrite |
| Recoverable autosave observer evidence | `cargo test -p riotbox-app --bin riotbox-app recovery_observer -- --nocapture` | observer snapshot includes `manual_choice_dry_run` for a recoverable autosave clue | dry-run inspection only, no automatic selection |
| Interrupted temp/autosave file set | `just interrupted-session-recovery-probe` | generated canonical session, invalid temp file, parseable autosave, validated observer stream | file-backed drill only, not a real host crash/kill soak |
| Missing normal session target | `just missing-target-recovery-probe` | generated missing canonical path plus parseable autosave clue, validated observer stream | autosave remains a manual clue, not a fallback load target |

These drills are intentionally narrow. They make startup recovery evidence
auditable for CI and future TUI work, but automatic startup recovery,
user-confirmed restore execution, and real interrupted-host-session rehearsal
remain open P011 work.

## Still Not Proven

- Full replay-from-origin for every replay-relevant action family.
- Automatic startup recovery selection.
- Interactive guided recovery that can safely choose a candidate with user
  confirmation.
- Remaining artifact hydration for broader artifact-producing W-30 /
  capture / promote actions beyond `w30.loop_freeze`, `promote.resample`,
  bounded `w30.capture_to_pad`, bounded `capture.bar_group`, and bounded
  `capture.now` / `capture.loop`.
- Full arrangement export, stem package export, live recording export, or
  product export commands beyond the current Feral grid generated-support
  full-mix boundary.
- Long-run soak behavior over extended live sessions.
- Real user-session recovery drills against an actual interrupted host run.

## Exit-Ready Requirements

P011 should not be called exit-ready until these are true:

- A full replay or staged replay path can reconstruct a useful session without
  relying only on persisted latest runtime state.
- Snapshot-payload restore and replay-from-origin converge for the important
  command families that are replay-safe today.
- Artifact-producing W-30 / capture actions either hydrate from durable
  artifacts through explicit per-command hydrators or reject with clear
  user-facing recovery instructions.
- Manual recovery can guide a user through an explicit candidate choice without
  hiding replay truth.
- Long-run and stage-style probes include enough action diversity to catch
  drift beyond the current smoke-level W-30 source-diff run. The current
  stage-style probe asserts `Phrase`, `Bar`, and `Beat` boundary
  coverage through observer/audio summary JSON, but is still not a soak test.
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

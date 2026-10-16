# RIOTBOX-1597 — legacy live-history admission

Date: 2026-10-16. Base: `c8f71c3c3929a5b5e81f17a5ea9daaa41da394f1`
(RIOTBOX-1600's separately reviewed manifest-ownership correction). P017
maintenance protecting P023 recall and saved-history integrity under
RIOTBOX-1041. RBX-456 and the owning Session/replay/module contracts were frozen
before production implementation.

## Failure and repair

The RIOTBOX-1596 checkpoint demonstrated that supported legacy files could be
edited into histories their declared Core policy refused: a Source Monitor Undo
after wall-clock rollback, or generic Stop/backward Seek committed through the
public queue. Saving those accepted operations succeeded; subsequent restore
failed. That was not filesystem-byte corruption or an actual user-data incident.
The existing Source Map gesture guard did not cover these other public paths.

Non-append policies now stage exact prospective committed metadata on existing
Queue/Session clones and invoke the unchanged Core planner before original
mutation. A failed batch preserves its entry queue, Session, runtime, projections,
snapshots, allocator and saved bytes. An already successful independent enqueue
remains legitimate pending intent. Immediate Play/Pause includes its new draft
and the whole ready drain before enqueue. Both transport timing entrances admit
the prospective boundary/timestamp before changing the clock or acknowledgment
flags. Earlier external audio delivery is not transactionally rolled back.

Typed Undo has one private semantic owner under the existing commit facade. Its
same Session operation normalizes missing snapshots, restores typed state,
changes the target and appends the exact allocated marker/record. Non-append
policy installs the candidate only after Core admission; V2 retains direct
mutation. One canonical record append serves actual and staged histories. The
existing Queue remains the only ID and commit allocator; capacity-before-ID
precedence and no-target normalization are preserved. V2 and empty ready drains
do not clone history for legacy admission.

`JamAppError::ReplayAdmission` carries the declared policy and Core error.
Commit/timing APIs return it honestly; ordinary interactive key refusals are
visible/nonfatal. Failed operations publish no successful observer commit.
Timing failure still stops the runtime, and the recorder still stops/aborts an
acquired capture without publishing a take. Error text offers conservative
explicit sibling copying only conditionally for unambiguous legacy history;
SnapshotOnly gets no upgrade promise. No automatic migration, timestamp repair,
record reordering, relaxed validator or general save-validation framework.

## Evidence

- Genuine public Undo RED: one compiled test failed because rollback returned
  `Ok(Some(marker))`; `/tmp/riotbox-1597-undo-red.log`, SHA-256
  `46b510e97eb31b22cfe0a1f8b38238c2529f4c09cde81c677da3c3fe1f01f221`.
- Genuine direct-commit RED: one compiled test failed because Stop at beat12
  returned a committed reference; `/tmp/riotbox-1597-commit-red-actual.log`,
  SHA-256 `608fc99d533a5df7a34eb7914a5cf4833de6d8760a6b7676a1b613c95fc68433`.
  The earlier short-name `--exact` command selected zero tests and is not RED
  evidence. Both actual regression tests subsequently passed.
- Seven Undo controls passed in `/tmp/riotbox-1597-undo-controls.log`: valid
  legacy/SnapshotOnly forward and V2 rollback, exact marker/save/reload, both
  non-append rollback refusals, independent domain barriers, capacity/ID
  precedence, staged missing-snapshot normalization and no-target behavior.
- Public commit controls cover Stop/Seek, mixed ready/nonready batches,
  preexisting unsafe toggle work, capacity precedence, all-policy valid
  navigation, V2 Stop→Monitor with clock rollback, explicit-copy retry and a
  historical Stop-only copy control. Five timing tests cover both entrances,
  prospective beat/bar boundaries, full refusal nonmutation, valid progress,
  stale/unavailable/stopped observations and pending-position delivery.
- The existing neutral direct CI stem writer commits only its own receipt,
  leaves unsafe pending transport intent untouched and produces reloadable
  declared history. Only newly generated CI PCM/files are used.
- Shell-control/observer coverage proves visible refusal and only an intent
  event, not a success commit; no real terminal/event loop is launched. The
  existing event-loop error match was separately inspected for nonfatal Undo
  admission. Two generated recorder-controller tests cover pre-capture refusal
  and acquired-capture Stop-before-Abort with no Finish/publication.
- Complete App library: **932 passed, one existing ignored benchmark**, no
  failures, `/tmp/riotbox-1597-app-full.log`. This run includes the later-added
  historical-copy and neutral-writer controls, unlike the earlier focused logs.

## Review and limits

Independent Rust/Adversarial/Maintainer and Spec/Evidence reviews inspected the
complete production change and final control-test delta; neither retained a
P0–P3 finding. Coordinator self-review covered mutation boundaries, error
propagation, semantic Undo/admission ownership, compatibility and drift. Full
source-free local/native CI remain mandatory gates; final exact-head results
belong to PR/Linear closeout and are not inferred from focused tests.

No new ActionCommand, Session/replay/recording/observer schema, persistent App
state, second history/validator/allocator/clock, dependency or source/DSP
algorithm/threshold/recipe. No actual user Session migration, source/capture/
Holdout/commercial/ignored-attempt access or discovery, host/device/service
observation/runtime, consumed-phase repeat, playback, DAW or Windows/TUI
expansion. Generated maintenance evidence is not musical, human, source-general,
hardness, latency, endurance or release qualification. RIOTBOX-1599 and1598
remain separate checkpoint follow-ups;1041/1036 and original ALSA cause remain
open. RIOTBOX-1589's passed, consumed host observation is unchanged.

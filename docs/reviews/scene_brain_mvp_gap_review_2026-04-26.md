# Scene Brain MVP Gap Review 2026-04-26

## Scope

This review re-audits `P008 | Scene Brain` after `P007 | W-30 MVP` closed.

Reviewed against `docs/phase_definition_of_done.md` Phase 6:

- a track yields multiple usable scenes
- scene changes sound musical
- restore logic works
- default arrange no longer feels like a static 8-bar loop
- scene actions remain replay-safe

Reviewed implementation and evidence:

- `crates/riotbox-app/src/jam_app/scene_ops.rs`
- `crates/riotbox-app/src/jam_app/side_effects.rs`
- `crates/riotbox-app/src/jam_app/projection.rs`
- `crates/riotbox-core/src/view/jam.rs`
- `crates/riotbox-core/src/tr909_policy.rs`
- `crates/riotbox-audio/src/runtime.rs`
- `docs/jam_recipes.md`
- `docs/benchmarks/lane_recipe_listening_pack_2026-04-26.md`
- `docs/reviews/scene_launch_audio_coupling_2026-04-25.md`

## Current State

Scene Brain is no longer only a display concept.

Implemented and covered today:

- deterministic scene candidates are projected from ordered Source Graph sections into session scene state
- `scene.launch` queues on `NextBar`, commits through the existing action queue, and updates session active scene plus transport current scene
- launch target selection can prefer the first known energy-contrast scene over adjacent same-energy movement
- `scene.restore` queues on the same `NextBar` seam and uses the explicit session restore pointer
- restore swaps active/current scene and restore target without introducing a hidden scene stack
- Jam and Log show active scene, next target, restore target, energy direction, pending scene actions, and landed scene trail
- TR-909 source-support can derive its support profile from the projected scene target and falls back to transport-bar context when mapping is unknown
- MC-202 render projection already uses projected scene/source section context for contour hints and hook-response restraint
- scene state and scene regression fixtures are covered across core view, app state, shell rendering, and persistence-oriented regression paths

## Verification Run

Commands run during this review:

```bash
cargo test -p riotbox-core scene -- --nocapture
cargo test -p riotbox-app scene -- --nocapture
cargo test -p riotbox-audio scene -- --nocapture
```

Results:

- `riotbox-core`: 5 scene-filtered tests passed
- `riotbox-app`: 27 scene-filtered tests passed across app library and shell binary tests
- `riotbox-audio`: 2 scene-filtered runtime tests passed

Important passing coverage includes:

- `prefers_contrast_next_scene_when_energy_data_is_available`
- `queue_scene_select_prefers_energy_contrast_candidate`
- `committed_scene_select_projects_target_scene_into_tr909_source_support`
- `committed_scene_restore_projects_target_scene_into_tr909_source_support`
- `scene_fixture_backed_committed_state_regressions_hold`
- `scene_fixture_backed_shell_regressions_hold`
- `scene_target_context_adds_bounded_support_accent`

## Phase 6 Assessment

### Multiple usable scenes

Status: partial.

Riotbox can derive multiple deterministic scene ids from source sections and keep them in session state. That satisfies the first structural requirement, but it is still a projected section list, not a richer Scene Graph with transition roles, scene budgets, or arrangement memory.

MVP implication:

- enough exists for first Scene Brain launch/restore flow
- not enough to claim a finished Scene Graph

### Scene changes sound musical

Status: partial and still the main blocker.

The current audio consequence is real but narrow:

- TR-909 can switch `SourceSupport` profile from a projected scene target and add a bounded scene-target support accent
- MC-202 can derive contour and hook-response from projected scene/source section context

The missing proof is sequence-level and musician-facing:

- no focused regression currently replays the documented `jump -> restore` flow and compares before/jump/restore mixed render output as one Scene Brain behavior
- no explicit Scene transition policy yet decides how strongly each lane should change on launch versus restore
- the listening-pack document still describes Scene Brain only through the TR-909 support-accent seam, which is conservative but now incomplete because MC-202 scene/source contour exists too

MVP implication:

- current scene changes can affect sound
- P008 should not close until there is a stronger combined output-path proof that a launched/restored scene creates a meaningful mixed-lane contrast

### Restore logic works

Status: mostly satisfied for MVP.

`scene.restore` uses the explicit restore pointer, queues on `NextBar`, blocks overlapping scene transitions, updates active/current scene, and refreshes the restore pointer. Jam and Log distinguish `restore waits` from `restore ready`.

MVP implication:

- restore semantics are real enough for the first Scene Brain loop
- later work can refine transition feel without reopening the restore pointer model

### Default arrange no longer feels static

Status: satisfied for bounded MVP.

Scene launch/restore now records a replay-safe `last_movement` in session scene state. The movement names launch/restore kind, source scene, target scene, `rise/drop/hold` direction, bounded intensity, and TR-909 / MC-202 lane intent. The app render projection consumes that movement without creating a full arranger: TR-909 phrase variation and bounded slam floor follow the movement, while MC-202 contour/touch follows the same landed movement.

MVP implication:

- this closes the P008 default-arrange blocker for the current bounded MVP
- full source playback repositioning, strip/build/slam scheduling, automatic scene chains, and a richer Scene Graph remain follow-up work

### Scene actions remain replay-safe

Status: mostly satisfied.

Scene actions use typed `ActionCommand`s, queue/commit boundaries, session state, transport state, and fixture-backed regressions. No second scene action path was found.

MVP implication:

- replay-safety architecture is on course
- follow-up audio work should stay on the same action and render-state seams

## Findings

### 1. Scene Brain needs sequence-level output proof

Severity: major for P008 closeout.

Isolated state, shell, and lane tests are good, but the product claim is a musician-facing `jump -> restore` flow. The next slice should replay that flow through the current app state and nearest audio render seams, then compare mixed output before launch, after launch, and after restore.

Acceptance shape:

- queue and commit `scene.launch`
- queue and commit `scene.restore`
- keep TR-909 source support and MC-202 follower active so both current scene-coupled lanes can respond
- render before / launched / restored buffers
- prove the launched buffer is non-silent and meaningfully different from the before buffer
- prove the restored buffer moves back toward the restore target instead of staying identical to the launched state
- assert scene state, restore pointer, Jam view, and render diagnostics alongside the audio metrics

Status:

- closed by `RIOTBOX-325`
- evidence: `scene_jump_restore_replay_proves_state_and_mixed_audio_path`
- proof shape: renders before / launched / restored mixed TR-909 + MC-202 buffers, proves launch and restore signal-delta thresholds, and proves restore returns to the baseline render

### 2. Scene transition policy is still implicit

Severity: major for P008 closeout.

Scene changes currently affect lanes through each lane's local source/scene context rules. That is safer than a shadow arranger, but it is not yet an explicit Scene Brain transition policy. There is no single bounded policy that says launch should be a rise/drop/hold move with controlled TR-909, MC-202, and later W-30 implications.

Acceptance shape:

- start with a small typed policy object or projection, not a full Scene Graph
- derive it from current scene energy, target scene energy, and action kind
- keep the policy read-only and deterministic
- feed existing lane render seams instead of creating a new audio path

Status:

- closed for projection by `RIOTBOX-326`
- evidence: `SceneTransitionPolicyView` derives launch/restore `rise`, `drop`, or `hold` from scene energy and names bounded `909` plus `202` lane intent
- output proof remains covered by `RIOTBOX-325`; `RIOTBOX-326` keeps the policy on the existing Jam view and UI surface instead of adding a shadow audio path

### 3. Listening-pack wording is conservative but stale

Severity: minor.

`docs/benchmarks/lane_recipe_listening_pack_2026-04-26.md` says Scene Brain is represented only through TR-909 `scene_target` support accent. That was true for the older coupling review, but the current projection also lets MC-202 contour and hook-response follow projected scene/source sections.

Acceptance shape:

- update the listening-pack note when sequence-level Scene Brain audio proof lands
- do not overstate this before the mixed replay proof exists

Status:

- updated by `RIOTBOX-325`
- the listening-pack note now reflects the mixed TR-909 + MC-202 Scene replay proof while keeping full transition-engine claims out of scope

## Recommended Next Slice

Add a bounded Scene Brain recipe replay output regression.

The slice should not build a full arranger. It should prove the existing `jump -> restore` flow as a musician would use it:

1. load a fixture Source Graph with at least intro/drop/break energy contrast
2. set TR-909 source support and MC-202 follower active
3. render a baseline mixed-lane buffer
4. queue and commit `scene.launch`
5. render the launched mixed-lane buffer and assert scene-target diagnostics
6. queue and commit `scene.restore`
7. render the restored mixed-lane buffer and assert restore diagnostics
8. compare signal-delta metrics across the three buffers

If that proof is too weak with the current lane rules, the same slice should introduce the smallest explicit scene-transition policy needed to make launch/restore audibly meaningful.

## Conclusion

`P008 | Scene Brain` is exit-clean for the current bounded MVP. Launch, restore, energy cues, replay-safe state, mixed-lane `jump -> restore` output proof, typed transition-policy projection, and replay-safe Scene movement state now all exist on the main Source Graph / Session / ActionQueue / Jam view / render-state spine.

This does not mean Riotbox has a finished arranger. It means the MVP phase can close honestly because a Scene move now produces a persisted, inspected, and audibly verified movement instead of only changing labels or local diagnostics.

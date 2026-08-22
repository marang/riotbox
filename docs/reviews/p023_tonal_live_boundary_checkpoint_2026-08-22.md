# P023 Tonal Live Boundary Checkpoint Review

Date: 2026-08-22

Scope:

- current-state architecture and design review of the P023 live-performance
  character policy, TR-909 projection, tonal performer journey, exact RuntimeMix
  proof, and the adjacent restart/recall seam
- reviewed
  `crates/riotbox-core/src/live_performance_policy.rs`,
  `crates/riotbox-app/src/jam_app/lifecycle.rs`,
  `crates/riotbox-app/src/jam_app/projection/tr909_projection.rs`, and
  `crates/riotbox-app/src/bin/dense_break_live_path_render/`
- review type: focused current-state `review-codebase` checkpoint, complemented
  by the RIOTBOX-1454 branch review; it is not a diff-only review

## Verdict

No blocking architecture, correctness, security, or realtime-performance
finding remains in the reviewed seam.

The tonal path keeps one product spine: source evidence produces a typed Core
policy; `JamAppState::refresh_view` projects that policy into the existing audio
render state; committed W-30 actions own Pitch Dive, ordinary re-entry, and
restart recall; and the offline tool exercises the same queue/commit and exact
RuntimeMix path. The review did not find a second action, persistence, replay,
arrangement, or audio-rendering authority.

## Boundary Review

- Architecture and boundaries: `LivePerformanceTr909Intent::StayOut` is a typed
  Core decision at
  `crates/riotbox-core/src/live_performance_policy.rs:87`; the tonal assignment
  is derived with the other source-character policy at
  `crates/riotbox-core/src/live_performance_policy.rs:270`. The app projection
  consumes that decision at
  `crates/riotbox-app/src/jam_app/projection/tr909_projection.rs:143` and returns
  the existing idle render state at
  `crates/riotbox-app/src/jam_app/projection/tr909_projection.rs:150`. Policy
  ownership therefore remains in Core while audio-type conversion remains in
  the app layer.
- Design pattern consistency: explicit performer Fill, Takeover, Slam, and Scene
  movement override the held `StayOut` default at
  `crates/riotbox-app/src/jam_app/projection/tr909_projection.rs:144`. This
  preserves the established rule that a source-character default may restrain
  accompaniment but may not suppress a committed performer action.
- Technical debt and maintainability: the exact tonal manifest is a cohesive
  538-line proof module, and the older shared live-flow module remains a larger
  review-cost hotspot. RIOTBOX-1454 moved tonal journey construction and
  validation into semantic modules at
  `crates/riotbox-app/src/bin/dense_break_live_path_render/tonal_journey.rs:17`
  and
  `crates/riotbox-app/src/bin/dense_break_live_path_render/tonal_live_manifest.rs:33`.
  No mechanical split is justified at this checkpoint; future unrelated proof
  families should receive their own module instead of growing either module.
- Cross-module dependencies: the app lifecycle continues to rebuild render
  projections from Session and Source Graph state at
  `crates/riotbox-app/src/jam_app/lifecycle.rs:64`. The diagnostic renderer
  drives public queue/commit behavior and reloads the persisted session through
  the shared restart helper at
  `crates/riotbox-app/src/bin/dense_break_live_path_render/tonal_journey.rs:32`
  and
  `crates/riotbox-app/src/bin/dense_break_live_path_render/alpha_arc.rs:20`;
  it does not retain a parallel product state.
- Security and performance: no credential, network, Holdout, or commercial
  reference path enters this seam. Policy projection performs bounded in-memory
  work; file access, source decoding, long renders, hashing, and manifest output
  remain in the offline diagnostic binary. No blocking I/O was added to an
  audio callback.
- Proof quality: the exact tonal validator compares 128- and 257-frame callback
  partitions, proves W-30-only contributor ownership across held, Pitch Dive,
  and re-entry stages, and checks W-30-only restart recall at
  `crates/riotbox-app/src/bin/dense_break_live_path_render/tonal_live_manifest.rs:50`.
  The controlled Development matrix separately protects dense, tonal, and
  sparse source-character behavior. Human acceptance remains artifact-bound in
  `docs/reviews/riotbox_1454_tonal_live_journey_v2_acceptance_2026-08-22.md`.

## Follow-Up Boundary

No new Linear issue is warranted by this review. The only maintainability note
is preventative: keep future proof families in semantic modules and revisit a
split only when responsibilities actually diverge. This checkpoint does not
reopen Pitch Dive sound design, repeat its five-source qualification, or grant
Holdout or release claims.

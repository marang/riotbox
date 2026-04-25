# Scene Launch Audio Coupling Review 2026-04-25

## Scope

This review checks what currently changes when a Scene Brain launch lands, after the contrast-target work in `RIOTBOX-226` and the explanation work in `RIOTBOX-228`.

Reviewed paths:

- `crates/riotbox-app/src/jam_app/scene_ops.rs`
- `crates/riotbox-app/src/jam_app/side_effects.rs`
- `crates/riotbox-app/src/jam_app/projection.rs`
- `crates/riotbox-core/src/tr909_policy.rs`
- existing Scene Brain and TR-909 source-support tests

## Current Behavior

When `scene.launch` commits:

- session active scene is updated
- transport current scene is updated
- restore scene is set to the previous scene when different
- runtime transport current scene is refreshed after the side effect
- committed Log result names the landed scene
- after `RIOTBOX-228`, contrast-selected launches can be named as contrast launches

The TR-909 render state also receives `current_scene_id` from transport projection.

## Audio Coupling Finding

The audible TR-909 `SourceSupport` profile is still derived from the current transport bar's Source Graph section:

- `derive_tr909_source_support_profile()` finds the section whose `bar_start..bar_end` contains `transport.bar_index`
- it does not currently use the committed Scene target's projected section index
- therefore a Scene launch can update Scene state and Log truth while the support profile still follows the transport's bar position

This is deterministic and replay-safe, but it means Scene Brain target selection is not yet fully coupled to audible section-profile changes.

## Product Impact

This explains why Scene Brain can now feel more intentional in the TUI without necessarily producing a strong audible contrast every time.

The current product truth is:

- Scene target selection is real
- Scene state and restore pointer are real
- Log confirmation is real
- TR-909 source-support audio is real
- target-Scene audio coupling is still incomplete

## Recommended Next Slice

Add a bounded target-scene audio-coupling slice:

- when a Scene launch or restore lands, project the target Scene's Source Graph section into the runtime render context
- keep transport-bar fallback when Scene-to-section mapping is missing or unknown
- add a regression proving a launched high/medium/low target changes the relevant support profile even if transport bar position has not crossed into that section yet

This should stay inside the existing Source Graph, Session, transport, and TR-909 policy seams. It should not introduce a second arranger model.

## Follow-up Status

`RIOTBOX-230` closes the bounded gap identified here for projected Scene ids:

- `scene-NN-label` targets now project into the matching sorted Source Graph section for TR-909 `SourceSupport`
- unmapped, legacy, or out-of-range Scene ids still fall back to the transport-bar section
- this keeps the new behavior inside the existing Source Graph, Session, transport, and TR-909 policy seams

## Verification Used

- code inspection of the current Scene launch side-effect path
- code inspection of TR-909 source-support projection
- existing test coverage around Scene launch state updates
- existing test coverage around source-support profile tracking by transport section

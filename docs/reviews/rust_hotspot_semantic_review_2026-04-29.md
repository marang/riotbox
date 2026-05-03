# Rust Hotspot Semantic Review 2026-04-29

## Scope

This review checks the largest Rust files after the repo adopted the "roughly
500 lines" soft review/context budget.

The goal is not to force every file below a hard limit. The goal is to identify
only semantic split candidates that would reduce future review and agent-context
cost without creating numbered shards or artificial modules.

Command used:

```bash
find crates -name '*.rs' -type f -print0 | xargs -0 wc -l | sort -nr | sed -n '1,40p'
```

## Largest Files

| LOC | File | Current recommendation |
| ---: | --- | --- |
| 500 | `crates/riotbox-core/src/view/jam/tests/build_view_model_tests.rs` | Refactor candidate |
| 499 | `crates/riotbox-audio/src/bin/lane_recipe_pack/tr909_support_case.rs` | Leave for now |
| 494 | `crates/riotbox-audio/src/bin/feral_grid_pack/render_stems.rs` | Leave for now |
| 488 | `crates/riotbox-app/src/jam_app/runtime_view.rs` | Watch / later semantic split |
| 487 | `crates/riotbox-app/src/jam_app/w30_queue.rs` | Leave for now |
| 481 | `crates/riotbox-core/src/view/jam/build_view_model.rs` | Refactor candidate |
| 474 | `crates/riotbox-audio/src/bin/feral_grid_pack/pack_builder.rs` | Leave for now |
| 468 | `crates/riotbox-app/src/ui/first_run_capture.rs` | Leave for now |
| 460 | `crates/riotbox-app/src/bin/observer_audio_correlate.rs` | Leave for now |
| 450 | `crates/riotbox-core/src/tr909_policy/render_policy.rs` | Leave for now |
| 450 | `crates/riotbox-app/src/ui/screen_bodies_footer_start.rs` | Leave for now |
| 450 | `crates/riotbox-app/src/ui/scene_timing_rail.rs` | Leave for now |

## Findings

### 1. Jam view-model test is one mega-fixture

- **Location**: `crates/riotbox-core/src/view/jam/tests/build_view_model_tests.rs:2`
- **Category**: scope
- **Severity**: major
- **Title**: One test constructs a full SourceGraph, Session, Queue, and asserts many unrelated view surfaces
- **Description**: `builds_minimal_jam_view_model` spans almost the whole file and covers source summary, scenes, macros, W-30, TR-909, capture, pending actions, and Ghost projection in one fixture. This keeps behavior covered, but future small Jam changes will repeatedly load a 500-line test context and risk unrelated assertion churn.
- **Suggestion**: Extract semantic fixture builders such as `sample_graph_with_feral_material`, `session_with_committed_lane_state`, and `queue_with_lane_pending_actions`, then split assertions into behavior-focused tests by surface: source/scene, W-30 pending, TR-909 pending, capture, Ghost.

### 2. Jam view-model builder is the next projection hotspot

- **Location**: `crates/riotbox-core/src/view/jam/build_view_model.rs:3`
- **Category**: scope
- **Severity**: major
- **Title**: One builder still owns lane pending extraction, source projection, scene policy, capture projection, and Ghost projection
- **Description**: `JamViewModel::build` remains one central projection seam. That is architecturally legitimate because Jam has one typed view model, but the function now carries multiple independent projection responsibilities. The W-30 pending extraction block alone starts at `crates/riotbox-core/src/view/jam/build_view_model.rs:52` and runs through several lane-specific pending summaries before the final view assembly.
- **Suggestion**: Do not split mechanically. When the next Jam projection change lands, extract by semantic responsibility into helper functions or child modules, for example `pending_lane_projection`, `source_scene_projection`, and `capture_projection`. Keep `JamViewModel::build` as the orchestration point.

### 3. Runtime view mixes runtime health and lane warning policies

- **Location**: `crates/riotbox-app/src/jam_app/runtime_view.rs:57`
- **Category**: scope
- **Severity**: minor
- **Title**: Runtime projection and lane warning derivation are coupled in one file
- **Description**: `JamRuntimeView::build` assembles audio/sidecar health and all lane render summaries, while later helpers derive TR-909, MC-202, W-30 preview, and W-30 resample warnings. This is still coherent today because it all belongs to runtime projection, but future render-warning growth will make the file a recurring hotspot.
- **Suggestion**: Leave it intact now. If the next runtime-warning slice grows it, split warning derivation into a semantic `runtime_warnings` module or helpers, not numbered shards.

### 4. W-30 queue surface is large but semantically coherent

- **Location**: `crates/riotbox-app/src/jam_app/w30_queue.rs:3`
- **Category**: scope
- **Severity**: suggestion
- **Title**: Many W-30 queue methods share one responsibility
- **Description**: The file is near the soft budget, but its responsibility is clear: user-facing W-30 queue commands. Splitting now would mostly move methods around without reducing conceptual cost.
- **Suggestion**: Do not refactor yet. Revisit only if repeated ActionDraft boilerplate becomes error-prone enough to justify a `w30_action_draft` helper.

### 5. Listening-pack bin tools are large but acceptable as bounded tool modules

- **Location**: `crates/riotbox-audio/src/bin/lane_recipe_pack/tr909_support_case.rs:77`
- **Category**: scope
- **Severity**: suggestion
- **Title**: Render, metrics, markdown, and manifest writing live together in the bin helper
- **Description**: Several near-500 files are CLI/listening-pack helpers. They combine render setup, offline metrics, markdown output, and manifest generation. That is not ideal long-term, but these are bounded QA tools rather than product runtime seams, and splitting them now would mostly produce overhead.
- **Suggestion**: Leave them intact unless the next audio QA slice edits the same file heavily. If they grow, split by real artifact responsibility such as `render_case`, `metrics_report`, and `manifest`, with semantic names.

## Follow-Up

Create one bounded follow-up for the Jam view-model projection/test hotspot. It
should not change behavior. It should extract semantic builders/projection
helpers only where they make future Jam feature slices smaller and easier to
review.

Do not create follow-ups for the bin tools or W-30 queue file yet.

## Decision

The repo does not need a broad cleanup pass right now. The next code-size
refactor should target the Jam view-model test/projection seam because it is a
real product-path hotspot and already receives frequent feature slices.

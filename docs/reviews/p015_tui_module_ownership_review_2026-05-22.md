# P015 TUI Module Ownership Review - 2026-05-22

Scope: `crates/riotbox-app/src/ui`

Context:

- Review cadence follow-up after the P015 TUI module split batch.
- Current `main` includes RIOTBOX-920 through RIOTBOX-922.
- RIOTBOX-923 is in flight and extracts W-30 preview labels from `diagnostics_mc202_w30_logs.rs`.

## Summary

The recent split batch materially improved review shape: formerly mixed UI shards now have semantic child modules for footer renderer behavior, footer lane lines, W-30 cue labels, scene timing labels, and W-30 slice-pool labels. The remaining risk is not behavior drift; it is the still-large UI root and several mixed test/diagnostic files that keep broad private scope and context cost high.

No immediate product-spine or audio/replay blocker was found in this review.

## Findings

### 1. UI Root Still Owns Too Much Module Wiring

- Location: `crates/riotbox-app/src/ui.rs:1`
- Category: scope
- Severity: minor
- Title: `ui.rs` remains a mixed module registry and include root
- Description: `ui.rs` has started moving semantic helpers into explicit modules, but it still combines root type inclusion, child-module registration, cross-shard imports, and the remaining `include!` shards. The include list at `crates/riotbox-app/src/ui.rs:63` still makes the root namespace the shared private API for several unrelated surfaces.
- Suggestion: Continue converting one include shard at a time into real modules only when the boundary is semantic. Avoid a mechanical include purge. Good next candidates are diagnostics and first-run/capture routing helpers because they already have coherent sub-responsibilities.

### 2. Footer/Gesture Tests Are Still A Mixed Fixture Hotspot

- Location: `crates/riotbox-app/src/ui/tests/footer_gesture_fixture_types.rs:34`
- Category: scope
- Severity: minor
- Title: Footer style assertions and suggested-gesture behavior share one large test shard
- Description: This file mixes low-level footer style/token assertions with musician-facing suggested gesture behavior and fixture-heavy state setup. The file is under the soft 500-line limit, but it remains one of the largest UI test files and has more than one review responsibility.
- Suggestion: Split into `footer_style_tokens.rs` for footer renderer and key-token styling, and `suggested_gesture_cues.rs` for feral/source-aware gesture cue behavior.

### 3. Capture/W-30 Tests Mix Preview Labels, Pending Capture, And Lane Cues

- Location: `crates/riotbox-app/src/ui/tests/capture_w30_cues.rs:1`
- Category: scope
- Severity: minor
- Title: Capture/W-30 tests still combine several independent surfaces
- Description: The file begins with source-window and pending-capture assertions, then covers W-30 audition/source-readiness helper behavior around `crates/riotbox-app/src/ui/tests/capture_w30_cues.rs:108`, and later covers broader Capture shell cue rendering. This keeps targeted W-30 preview-label changes coupled to Capture surface snapshots.
- Suggestion: After RIOTBOX-923 lands, split W-30 preview/source-readiness assertions into a dedicated `w30_preview_cues.rs` test shard. Keep Capture shell flow assertions in `capture_w30_cues.rs`.

### 4. First-Run Capture Routing Is A Good Next Production Split

- Location: `crates/riotbox-app/src/ui/first_run_capture.rs:263`
- Category: scope
- Severity: minor
- Title: Capture routing lines combine first-run surface and W-30 routing detail
- Description: `first_run_capture.rs` owns first-run onramp staging, capture summaries, pending capture lines, and W-30 routing/resample detail. The routing block starting around `capture_routing_lines` is cohesive enough to become a child module without changing behavior.
- Suggestion: Extract capture routing lines and the capture-heard-path helper into `ui/first_run_capture/routing.rs`. Keep `FirstRunOnrampStage` and first-run stage selection in the parent module.

### 5. Diagnostics Still Has A W-30 Resample Helper Cluster

- Location: `crates/riotbox-app/src/ui/diagnostics_mc202_w30_logs.rs:332`
- Category: scope
- Severity: minor
- Title: W-30 resample labels remain grouped with diagnostics line assembly
- Description: RIOTBOX-923 removes the preview-label cluster, but the resample tap/source/lineage/mix helpers still sit in the diagnostics shard. They are cohesive and used by both Log diagnostics and Capture routing.
- Suggestion: Follow RIOTBOX-923 with a narrow W-30 resample label extraction if the diagnostics shard remains above the desired review budget after that PR merges.

## Recommended Next Slices

1. Split `ui/tests/footer_gesture_fixture_types.rs` into footer style-token tests and suggested-gesture cue tests.
2. After RIOTBOX-923 merges, split W-30 preview/source-readiness tests out of `ui/tests/capture_w30_cues.rs`.
3. Extract `ui/first_run_capture/routing.rs` for Capture routing and heard-path lines.
4. Extract W-30 resample label helpers from `diagnostics_mc202_w30_logs.rs` if still needed after RIOTBOX-923.

## Drift Check

- New `ActionCommand`: no
- Queue path changed: no
- Commit or side-effect path changed: no
- Session/replay consequence: none
- User-visible or observer surface changed: no
- Audio-producing behavior changed: no
- `JamAppState` state added: no
- Shadow-system risk: none; review only

# W-30 MVP Exit Review 2026-04-26

Context:

- ticket: `RIOTBOX-321`
- phase: `P007 | W-30 MVP`
- review type: current-state W-30 MVP exit review via `review-codebase`
- source criteria:
  - `docs/phase_definition_of_done.md`
  - `docs/specs/audio_qa_workflow_spec.md`
  - `docs/reviews/w30_mvp_gap_review_2026-04-26.md`

## Summary

W-30 is no longer metadata-only. Since the previous gap review, the main storage and resampling blockers have been closed:

- committed source-backed captures now write real `captures/cap-*.wav` artifacts
- focused W-30 hit playback can consume the committed capture artifact even when the original source file/cache is unavailable
- committed `promote.resample` now prints a bounded W-30 bus artifact, reloads it into the capture cache, and keeps lineage/generation metadata explicit
- source-vs-fallback and raw-vs-printed output checks exist around the new audible seams

However, `P007 | W-30 MVP` is not exit-clean yet. The remaining blocker is pad playability: the current W-30 audio path still renders one focused preview state backed by a fixed `2048`-sample preview window, not a duration-aware loop/pad playback seam.

## Phase 5 Criteria

| Criterion | Status | Evidence | Remaining Gap |
| --- | --- | --- | --- |
| useful loops can be captured | Satisfied for MVP | committed captures write `captures/cap-01.wav`, reload through `SourceAudioCache`, preserve duration/sample-rate/channel metadata, and survive session reload | capture is source-window-backed, not live bus recording; acceptable for first MVP |
| pads are playable | Still blocked for MVP exit | W-30 hit/audition/live-recall actions are real and the focused hit path renders artifact-backed non-silent audio that differs from fallback | playback is still one focused `W30PreviewRenderState` with a fixed `W30_PREVIEW_SAMPLE_WINDOW_LEN` preview, not duration-aware loop/pad playback |
| internal bus resampling works | Satisfied for MVP | committed `promote.resample` creates a `CaptureType::Resample`, writes `captures/cap-02.wav`, reloads it, and proves output differs from raw capture and synthetic resample-tap control | render policy is bounded MVP, not final multitrack/export resampling |
| captured material can be reused without leaving flow | Mostly satisfied | capture -> promote -> audition/hit -> resample can be driven through existing queue/commit actions without leaving the Jam model | reuse is constrained by the fixed preview-window playback seam |
| provenance for captured material is not lost | Satisfied | `source_origin_refs`, `source_window`, `lineage_capture_refs`, generation depth, target metadata, storage path, and notes remain explicit; resample captures no longer pretend to be literal source windows | none for MVP |

## Findings

### 1. Focused W-30 pad playback is audible, but still preview-window based

Severity: blocker for Phase 5 exit

The current render projection prefers committed capture artifacts, but it reduces the artifact into `W30PreviewSampleWindow`, whose buffer is fixed at `W30_PREVIEW_SAMPLE_WINDOW_LEN`. `source_preview_from_interleaved` samples from the full artifact into that fixed window, and the runtime renders the focused preview state.

Evidence:

- `build_w30_capture_artifact_preview` prefers `capture_audio_cache` before source-window projection
- `source_preview_from_interleaved` caps playback material to `W30_PREVIEW_SAMPLE_WINDOW_LEN`
- `focused_w30_pad_trigger_uses_capture_artifact_preview_when_source_cache_unavailable` proves the seam is non-silent and artifact-backed, but also asserts the fixed preview sample count

Impact:

- the user can trigger a focused W-30 pad and hear source-derived material
- the sound is not yet a real loop-length pad playback result
- multiple pad targets are represented as session state, but the audio path still owns one focused preview voice

Next implementation should make the focused W-30 pad seam duration-aware before closing P007. Keep it bounded: one focused pad voice is enough, but it should render from the capture artifact over its own duration/loop policy instead of a fixed diagnostic preview window.

### 2. Capture artifacts are now real enough for MVP

Severity: resolved blocker

Committed capture side effects now write session-relative WAV artifacts from source windows on the app/control path, then reload them into `capture_audio_cache`.

Evidence:

- `persist_capture_audio_artifact` writes and caches committed capture artifacts
- `committed_source_backed_capture_writes_wav_artifact` verifies `captures/cap-01.wav` exists, reloads it, checks sample rate/channel count/duration/non-silence, and saves/reloads the session

Impact:

- `CaptureRef.storage_path` is no longer decorative for normal source-backed captures
- captured loops are concrete files beside the session
- later sampler, export, and QA paths can trust the artifact seam

### 3. Internal resampling is now a reusable artifact seam

Severity: resolved blocker

Committed `promote.resample` now writes a bounded offline bus print to the new resample capture path and caches it for later W-30 use.

Evidence:

- `persist_w30_bus_print_artifact` writes/caches the resample artifact on commit
- `capture_ref_from_action` keeps resample captures lineage-safe and omits `source_window` for printed resamples
- `committed_w30_internal_resample_prints_reusable_bus_artifact` verifies `captures/cap-02.wav`, reloads it, checks non-silent metrics, and compares it against raw capture and synthetic resample-tap controls

Impact:

- resampling now produces material, not only lineage
- printed resamples can be reused through the same capture-cache seam
- full multitrack recording remains rightly out of scope

## Required Follow-Up Before P007 Close

Create and complete one bounded implementation ticket:

`Make focused W-30 pad playback duration-aware from capture artifacts`

Acceptance shape:

- use the committed capture artifact as the playback source for the focused W-30 pad
- preserve the existing queue/commit action path for `w30.trigger_pad`, audition, and recall
- render more than the fixed preview sample window when the artifact is longer
- define a first loop/retrigger policy for the focused pad voice
- prove control path plus output path with source-vs-fallback and short-vs-duration-aware buffer comparisons
- keep realtime callback input bounded and preloaded; do not perform file I/O in the callback

## Non-Blocking Follow-Ups

These should not block the first W-30 MVP exit once focused duration-aware pad playback lands:

- full multi-pad polyphony
- richer pad-bank UI and cross-pad browsing
- final W-30 sound-design pass
- live bus recording from the audio callback
- export/multitrack recording
- automatic perceptual audio comparison

## Conclusion

W-30 has crossed the important line from "logs and lineage" into reusable audio artifacts. The remaining MVP blocker is narrower and concrete: the focused pad path must stop being only a fixed-size preview window and become a duration-aware artifact playback seam. After that, P007 can likely be closed as a first honest W-30 MVP, with full sampler expansion moved to later phases.

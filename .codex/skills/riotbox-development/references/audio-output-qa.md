# Audio Output QA

Use this reference when a Riotbox change can affect sound.

## Required Proof Shape

Every audio-producing slice needs two independent proofs:

- Control proof: queue/action/log/session/render-state/provenance confirms the intended path landed.
- Output proof: rendered samples, WAV artifact, or metrics confirm the audible output is correct enough for the current stage.

If the feature prepares state but does not render directly, verify the nearest downstream render seam that consumes that state.

## Common Failure Modes

- The log says `src`, but the renderer still sounds fallback-like.
- Transport says started, but timing does not advance.
- A preview is technically non-silent but too short, too quiet, or too repetitive to be useful.
- A deterministic source-backed path always produces the same weak tick because the source window is too small or read too slowly.
- A UI recipe queues actions faster than commit boundaries can land.

Turn repeated failures into fixture cases, thresholds, or listening-pack controls.

## Current Useful Checks

Check the repo `Justfile` before using these commands. If a target is missing, use the direct cargo helper in the repo docs or add the missing wrapper as part of the slice.

General audio/runtime checks:

```bash
cargo test -p riotbox-audio
cargo test -p riotbox-app audio_timing_snapshot
```

Use source-backed determinism when the same source should reproduce exactly:

```bash
just w30-smoke-source-qa "data/test_audio/examples/Beat03_130BPM(Full).wav" local
```

Use source-vs-fallback diff when the source-backed path must prove it did not collapse to synthetic fallback:

```bash
just w30-smoke-source-diff "data/test_audio/examples/Beat03_130BPM(Full).wav" local-source-diff
```

Inspect generated artifacts under:

```text
artifacts/audio_qa/<date>/w30-preview-smoke/raw_capture_source_window_preview/
```

Expected files:

- `baseline.wav`
- `candidate.wav`
- `baseline.metrics.md`
- `candidate.metrics.md`
- `comparison.md`

For current W-30 source-diff checks, pay attention to:

- `rms` delta
- `sum` delta
- `peak_abs`
- active sample count

These metrics are not musical quality by themselves; they are a guard against false positives.

## Generalizing Beyond W-30

Apply the same control-vs-output pattern to every lane:

- TR-909: prove fill/reinforce/takeover state landed and that event density, peak/RMS, or rendered buffer behavior changed as intended.
- MC-202: prove phrase/follower/answer state landed and that the generated lane has audible pitch/rhythm variation when the audio seam exists.
- W-30: prove capture/promote/recall provenance landed and that source-backed output differs from fallback/control when it claims to.
- Scene Brain: prove scene choice/restore state landed and that any claimed audio consequence is visible in the downstream lane output.
- Future source playback/mixer: prove transport/source state landed and that the actual source/mix bus emits expected audio.

## Musician-Usability Check

For every audible feature, also answer:

- Can a musician hear what changed without reading logs?
- Does the gesture feel playable at the current latency and quantization point?
- Is the result musically useful, or only technically non-silent?
- Does the UI make the next meaningful action clear?
- Is there a short recipe that produces a satisfying result from a real source file?

If the answer is no, treat the feature as technically partial even when tests pass.

## User-Reported Audio Mismatch

When the user reports that output sounds wrong:

1. Reproduce with the same command and source.
2. State the intended audible behavior.
3. Verify transport/timing first.
4. Verify action/log path.
5. Render a deterministic offline artifact for the suspected seam.
6. Compare output against fallback/control/baseline.
7. If metrics pass but it still sounds bad, record the weakness and create a tighter fixture, threshold, UX cue, audio policy change, or implementation fix.

Do not answer "works internally" as the final conclusion for an audible feature.

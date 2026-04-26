# W-30 Preview Smoke Listening Pack

Date: `2026-04-26`

Status: initial local-only convention

Related:

- `docs/specs/audio_qa_workflow_spec.md`
- `docs/benchmarks/audio_qa_artifact_convention_2026-04-26.md`
- `cargo run -p riotbox-audio --bin w30_preview_render`
- `cargo run -p riotbox-audio --bin w30_preview_compare`

## Purpose

This file defines the first W-30 local listening-pack convention without claiming a full listening-pack harness.

It gives agents and humans one stable case ID, output path, command, and note shape for the current deterministic W-30 preview smoke render.

## Pack Manifest

- Pack ID: `w30-preview-smoke`
- Case ID: `raw_capture_source_window_preview`
- Render helper: `cargo run -p riotbox-audio --bin w30_preview_render`
- Metrics comparison helper: `cargo run -p riotbox-audio --bin w30_preview_compare`
- Render type: deterministic local candidate WAV
- Review type: manual listening plus sibling metrics
- CI status: local-only, not a CI gate

## Output Convention

Use this directory shape for generated local artifacts:

```text
artifacts/audio_qa/YYYY-MM-DD/w30-preview-smoke/raw_capture_source_window_preview/
  baseline.wav
  baseline.metrics.md
  candidate.wav
  candidate.metrics.md
  comparison.md
  notes.md
```

Generated audio QA artifacts are intentionally ignored by Git. Preserve durable conclusions in `docs/benchmarks/`, not by committing generated WAV files. See `audio_qa_artifact_convention_2026-04-26.md` for baseline-vs-candidate naming.

## Render Command

Use the short Justfile wrappers for the normal local path:

```bash
just w30-smoke-candidate 2026-04-26
just w30-smoke-baseline 2026-04-26
```

Render a candidate into the convention path:

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --date 2026-04-26 \
  --role candidate
```

The helper also writes:

```text
artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/candidate.metrics.md
```

Render a baseline from a known-good commit with the same date and `--role baseline`:

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --date 2026-04-26 \
  --role baseline
```

Use `--out PATH` only when you need to override the convention path for an ad hoc render.

To render the same smoke case from an existing PCM16 or PCM24 WAV source window instead of the deterministic synthetic window, use the source-backed wrapper:

```bash
just w30-smoke-source-qa "data/test_audio/examples/Beat03_130BPM(Full).wav" 2026-04-26
```

That command renders both roles from the same source-backed path, so it is useful as a determinism check.

When the question is whether the current source-backed W-30 path is audibly different from the safe synthetic fallback, use the source-diff wrapper instead:

```bash
just w30-smoke-source-diff "data/test_audio/examples/Beat03_130BPM(Full).wav" 2026-04-26-source-diff
```

That command renders:

```text
baseline.wav  = synthetic fallback preview
candidate.wav = source-backed preview from the WAV window
```

It then requires minimum RMS and sum deltas so the run fails if the source-backed candidate collapses back into the same fallback-like preview.

Equivalent direct command:

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --date 2026-04-26 \
  --role candidate \
  --source "data/test_audio/examples/Beat03_130BPM(Full).wav" \
  --source-start-seconds 0.0 \
  --source-duration-seconds 0.25
```

The current source-backed helper uses the existing non-realtime WAV loader. Float, compressed, and non-PCM WAV support remain out of scope for this smoke path.

Current W-30 preview renders use a bounded `2048`-sample mono source-window preview. This is intentionally larger than the first tiny callback-safe sketch, but it is still a short preview excerpt rather than full pad-bank sample streaming.

## Compare Command

After rendering both baseline and candidate metrics for the same date, compare their metric deltas:

```bash
just w30-smoke-compare 2026-04-26
```

Equivalent direct command:

```bash
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --date 2026-04-26
```

The comparison helper reads `baseline.metrics.md` and `candidate.metrics.md` from the convention path, prints active-sample / peak / RMS / sum deltas, writes the same report to `comparison.md`, and exits non-zero when the default drift limits are exceeded. This is a narrow local metrics helper, not a waveform diff or listening-pack CI gate.

Use `--max-active-samples-delta`, `--max-peak-delta`, `--max-rms-delta`, or `--max-sum-delta` when a local branch intentionally changes the smoke render and you want to inspect bounded drift instead of requiring an exact match.

Use `--min-active-samples-delta`, `--min-peak-delta`, `--min-rms-delta`, or `--min-sum-delta` when the branch must prove that a source-backed render differs from a fallback or control render.

Use `--report PATH` only when you need to write the comparison report outside the convention path.

For a quick local smoke pack using the current commit for both roles:

```bash
just w30-smoke-qa 2026-04-26
```

## Notes Template

Create `notes.md` beside the rendered WAV when a human listens:

```markdown
# W-30 Preview Smoke Notes

- Date:
- Commit:
- Case: `raw_capture_source_window_preview`
- Listener:
- Playback path:
- Result: `pass` / `concern` / `fail`

## Metrics Snapshot

- Active samples:
- Peak abs:
- RMS:
- Sum:

## Listening Notes

- Rhythmic clarity:
- Source-window audibility:
- Harshness / clipping:
- Usefulness as a W-30 preview:

## Follow-Up

- None / ticket:
```

## Current Limits

- This is not a generalized fixture-pack runner.
- There is no baseline-vs-candidate waveform or perceptual comparison engine yet.
- There is no committed WAV baseline in the repo.
- There is no CI gate for generated audio artifacts.
- The useful durable artifact today is the convention plus any follow-up ticket created from listening.

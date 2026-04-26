# W-30 Preview Smoke Listening Pack

Date: `2026-04-26`

Status: initial local-only convention

Related:

- `docs/specs/audio_qa_workflow_spec.md`
- `docs/benchmarks/audio_qa_artifact_convention_2026-04-26.md`
- `cargo run -p riotbox-audio --bin w30_preview_render`

## Purpose

This file defines the first W-30 local listening-pack convention without claiming a full listening-pack harness.

It gives agents and humans one stable case ID, output path, command, and note shape for the current deterministic W-30 preview smoke render.

## Pack Manifest

- Pack ID: `w30-preview-smoke`
- Case ID: `raw_capture_source_window_preview`
- Render helper: `cargo run -p riotbox-audio --bin w30_preview_render`
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
  notes.md
```

Generated audio QA artifacts are intentionally ignored by Git. Preserve durable conclusions in `docs/benchmarks/`, not by committing generated WAV files. See `audio_qa_artifact_convention_2026-04-26.md` for baseline-vs-candidate naming.

## Render Command

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --out artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/candidate.wav
```

The helper also writes:

```text
artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/candidate.metrics.md
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
- There is no baseline-vs-candidate comparison engine yet.
- There is no committed WAV baseline in the repo.
- There is no CI gate for generated audio artifacts.
- The useful durable artifact today is the convention plus any follow-up ticket created from listening.

# Audio QA Artifact Convention

Date: `2026-04-26`

Status: local-only baseline / candidate convention

Related:

- `docs/specs/audio_qa_workflow_spec.md`
- `docs/benchmarks/w30_preview_smoke_listening_pack_2026-04-26.md`

## Purpose

This file defines the first baseline-vs-candidate artifact shape for local audio QA.

It is intentionally a convention, not an automated comparison engine. The goal is to make generated WAVs, metrics, and listening notes predictable before adding broader pack runners or CI gates.

## Directory Shape

Use this layout for local generated artifacts:

```text
artifacts/audio_qa/YYYY-MM-DD/<pack-id>/<case-id>/
  baseline.wav
  baseline.metrics.md
  candidate.wav
  candidate.metrics.md
  comparison.md
  notes.md
```

For the current W-30 smoke case:

```text
artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/
  baseline.wav
  baseline.metrics.md
  candidate.wav
  candidate.metrics.md
  comparison.md
  notes.md
```

## File Meanings

- `baseline.wav`: the previously accepted local reference render for this case.
- `baseline.metrics.md`: metrics generated for `baseline.wav`.
- `candidate.wav`: the newly rendered output from the current branch or commit.
- `candidate.metrics.md`: metrics generated for `candidate.wav`.
- `comparison.md`: local baseline-vs-candidate metrics comparison report.
- `notes.md`: human listening notes and pass / concern / fail result.

## Git Rule

Generated audio QA artifacts under `artifacts/audio_qa/` are ignored by Git.

Commit durable conclusions instead:

- benchmark convention docs in `docs/benchmarks/`
- workflow/spec updates in `docs/specs/`
- follow-up tickets for audible concerns
- small text summaries when a listening result matters later

Do not commit generated WAVs by default.

## Current W-30 Command

The current helper writes a candidate render:

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --date 2026-04-26 \
  --role candidate
```

To create a local baseline manually, run the same helper against `baseline.wav` from a known-good commit:

```bash
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --date 2026-04-26 \
  --role baseline
```

`--date` and `--role` derive the convention path. `--out PATH` remains available for ad hoc renders that should not use this directory shape.

Compare the resulting sibling metrics files:

```bash
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --date 2026-04-26
```

The comparison helper is a local metrics drift check only. It does not compare waveforms, promote baselines, or create a CI gate.

By default it writes the report to `comparison.md` beside the two metrics files. Use `--report PATH` for ad hoc output paths.

## Current Limits

- There is no automated baseline lookup.
- There is no baseline-vs-candidate waveform or perceptual diff engine.
- There is no CI gate for generated audio artifacts.
- Baseline promotion is still a human workflow decision.

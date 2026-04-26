# Lane Recipe Listening Pack 2026-04-26

Status: initial local QA harness  
Ticket: `RIOTBOX-299`

## Purpose

This pack is the first local audio-output harness for documented Jam routines outside the W-30 source-preview path.

It exists because the routine audit showed that W-30 now has source-vs-fallback proof, while TR-909, Scene Brain, and MC-202 still needed clearer audio-output accounting.

## Command

```bash
just lane-recipe-pack 2026-04-26
```

Equivalent direct command:

```bash
cargo run -p riotbox-audio --bin lane_recipe_pack -- --date 2026-04-26
```

Generated files live under:

```text
artifacts/audio_qa/<date>/lane-recipe-listening-pack/
```

Each case writes:

- `baseline.wav`
- `candidate.wav`
- `baseline.metrics.md`
- `candidate.metrics.md`
- `comparison.md`

The pack root also writes:

- `pack-summary.md`

For local human listening notes, create `notes.md` beside either the pack root or a specific case:

```bash
just audio-qa-notes artifacts/audio_qa/<date>/lane-recipe-listening-pack/notes.md
just audio-qa-notes artifacts/audio_qa/<date>/lane-recipe-listening-pack/tr909-support-to-fill/notes.md
```

Template source: `docs/benchmarks/audio_qa_listening_review_template_2026-04-26.md`.

## Covered Cases

Current cases:

- `tr909-support-to-fill`
  - baseline: steady source support
  - candidate: fill with mainline drive
  - covers Recipe 2 and Recipe 7
- `tr909-support-to-takeover`
  - baseline: source support
  - candidate: controlled phrase takeover
  - covers Recipe 2
- `scene-transport-to-target-support`
  - baseline: transport-bar support
  - candidate: Scene-target support accent
  - covers Recipe 10's current TR-909 support-accent seam

## Current Limits

This is not yet a full performance-recorder or TUI replay harness.

MC-202 is intentionally documented as non-audible in this pack because the current implementation proves follower/answer state and phrase generation, not a dedicated MC-202 audio lane.

Scene Brain is represented only through the existing TR-909 `scene_target` support-accent seam. This proves the bounded accent, not a finished Scene transition engine.

Generated WAVs remain local and untracked under `artifacts/audio_qa/`.

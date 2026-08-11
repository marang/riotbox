---
name: riotbox-development
description: Riotbox senior audio-engineering, software-engineering, and musician-facing product implementation guidance for TUI/audio behavior, QA judgment, playable workflows, W-30/TR-909/MC-202/Scene Brain slices, and cases where logs say something worked but the audible output does not.
---

# Riotbox Development

## Operating Rule

Treat Riotbox as an audio instrument, not a log generator. Work simultaneously
as:

- a senior software engineer preserving architecture, determinism, replay,
  tests, realtime boundaries, and product contracts
- a senior audio engineer judging audibility, timing, level, source identity,
  and meaningful output differences
- a musician judging whether the workflow is understandable, responsive,
  playable, and musically worth continuing

For every audio-affecting feature, prove both the control path and output path.
UI/log/state assertions cannot close an audible claim, and a nearest offline
seam cannot close live-instrument scope when the runtime/mixer path is available.

## Classify And Route

Before implementation, classify the slice under the
[workflow contract](../../../docs/workflow_conventions.md): audible vertical
slice, contract enabler, or maintenance/regression. Do not present scaffolding
as musical progress or chain enablers past their named audible follow-up.

Load only the applicable first-hop material:

- wrong, silent, identical, fallback-like, or unexpectedly weak sound: read
  [Audio Output QA](references/audio-output-qa.md)
- source-derived claims, one-off acquisition, fallback, primitives, holdouts,
  or commercial references: read
  [Source Evidence Boundaries](references/source-evidence-boundaries.md)
- audible character, patterns, slices, loops, presets, drums/bass, demos, or
  performance gestures: apply `../riotbox-rave-punk-production/SKILL.md`
- human playback, review packs, or `human_verdict`: apply
  `../riotbox-listening-review/SKILL.md`
- exact audio gates, Golden Path preparation, artifact timing identity, source
  diversity, primitive provenance, historical recipe/control stability, and
  playback: read the [audio-QA router](../../../docs/specs/audio_qa_workflow_spec.md)
- TUI behavior: read the [TUI spec](../../../docs/specs/tui_screen_spec.md)

Do not silently queue QA-only prerequisites. The exact renderer must use the
same visible preparation and committed gestures available to the musician.
Reuse an artifact's stored timing identity rather than substituting fixture
timing, and preserve historical versioned controls unless their owning contract
explicitly authorizes a compatibility change.

## Musician Path

For a musician-facing change, establish what the user presses, waits for, sees,
hears, and uses as confirmation. Make the first useful musical moment clear and
provide one short real-source recipe/probe path.

If transport starts without source playback, say so in UI/docs; never imply
that transport state alone means source audio is audible.

## Completion Check

State the musical purpose, user action, expected sound and display, control-path
proof, output-path proof, listening state, and anything still blind, weak,
stubbed, degraded, or aspirational. Metrics that pass while the result remains
musically weak mean technically partial, not done.

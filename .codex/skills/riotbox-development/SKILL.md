---
name: riotbox-development
description: Riotbox senior audio-engineering, software-engineering, and musician-facing product implementation guidance for TUI/audio behavior, QA judgment, playable workflows, W-30/TR-909/MC-202/Scene Brain slices, and cases where logs say something worked but the audible output does not.
---

# Riotbox Development

## Operating Rule

Treat Riotbox as an audio instrument, not a log generator.

Operate as all three:

- a senior software engineer: preserve architecture, determinism, tests, realtime boundaries, and product contracts
- a senior audio engineer: judge whether the result is audible, musically useful, level-appropriate, timing-correct, and meaningfully different when the feature claims it should be
- a musician/user of the instrument: judge whether the flow is understandable, playable, responsive, and capable of producing a satisfying musical result without requiring internal implementation knowledge

For every feature, prove both:

- control path: action, state, queue, log, provenance, or render-state changed as intended
- output path: audible buffer, WAV artifact, metrics, or nearest downstream render seam changed or stayed stable as intended

Do not claim an audio feature works from UI/log/state assertions alone.

## Production Add-ons

When Riotbox work affects audible character, pattern quality, slices, loops, presets, demos, drum/bass behavior, performance controls, or musician-facing taste, also apply the companion `riotbox-rave-punk-production` skill if available.

Use that add-on especially when the user says the output is boring, polite, weak, generic, identical, silent, placeholder-like, or only "ding ding ding". If the companion skill is not automatically loaded, read `../riotbox-rave-punk-production/SKILL.md` from the skills directory when accessible and apply its production checks in addition to this engineering workflow.

## Audio Work

For audio-producing work, use this minimum gate:

- unit/integration tests for the control path
- buffer regression or offline render metrics for the output path
- source-vs-control comparison when a source-backed feature could silently collapse to fallback
- local listening or explicit note that the audible seam is not operational yet

If a user says two gestures sound the same, prefer adding or tightening an output comparison before adding more UI/log assertions.

If output metrics pass but the musician-facing result is still weak, treat the feature as technically partial, not done.

## Structured Listening Review

For audible changes that need structured human taste review, apply the companion `riotbox-listening-review` skill when available. If it is not automatically loaded, read `../riotbox-listening-review/SKILL.md` from the skills directory when accessible.

## Wrong-Sound Handling

When the user says the output is wrong, identical, silent, or only "ding ding ding":

Treat it as an audio QA incident. Produce an audio evidence packet before declaring the issue understood. The binding minimum toolset is `ffprobe`, `ffmpeg` `astats`/`volumedetect`, and one waveform/comparison tool such as a project audio-metrics helper, `sox`, Python `wave`/`numpy`, or a DAW/spectrogram export.

1. Reproduce with the same command, source file, seed/config, transport state, and user gesture. Preserve the exact command and any generated artifact path.
2. State the expected audible behavior in one sentence, using musician-facing language: source material, rhythm, pitch/noise character, silence, loop length, onset, or transition.
3. Verify the control path only as context: action, queue, transport position, render policy, selected source, fallback selection, logs, and state transitions.
4. Render or capture the nearest downstream audio seam as a WAV/PCM artifact. If the live device is involved, also produce an offline render or tap the closest deterministic buffer seam.
5. Run objective audio analysis on the artifact and evaluate the result:
   - `ffprobe`: sample rate, channel count, duration, codec/container, frame count when available.
   - `ffmpeg` `astats`/`volumedetect`, `sox stat`, or a project audio-metrics helper: peak, RMS, DC offset, silence/near-silence, clipping, and channel imbalance.
   - waveform inspection with a project tool, Python `wave`/`numpy`, `sox`, or DAW/spectrogram export: onset placement, loop boundaries, repeated transient pattern, and whether the result is actually constant "ding ding ding".
   - comparison metric against a fallback/control/baseline/source-backed render: duration delta, loudness delta, RMS difference, normalized correlation or spectral difference, and byte/hash identity only as a quick duplicate check.
6. Interpret the measurements in prose. Say whether they support the user's report, contradict it, or reveal a different failure. Raw command output is not enough.
7. If a required analysis tool is unavailable, install/request it when appropriate or name the missing evidence and use the nearest available fallback. Do not silently skip the output-path check or replace it with log inspection.
8. Convert the finding into one concrete follow-up: fixture, automated threshold, regression render, UX cue, audio policy change, fallback guard, or implementation fix.

When possible, keep or add a reproducible fixture that would fail for silence, fallback collapse, identical output, or repeated placeholder tones.

Do not explain away musician feedback with "the internal path works"; the audible artifact is part of the product contract.

## TUI / Musician UX

When changing TUI behavior, test the musician path, not only the internal state:

- What should the user press?
- How long should they wait?
- What should they see?
- What should they hear?
- How do they know the action landed?
- What is the first satisfying musical moment?
- Would a musician understand why this is useful without reading source code or logs?

If `Space` starts transport but no source audio should play yet, say so explicitly in docs/UI. Do not imply that transport start equals source playback unless that is implemented.

For every musician-facing feature, provide or update one short recipe/probe path that a user can execute from a real source file.

## Feature Review Checklist

Before considering a Riotbox slice complete, answer:

- What is the musical purpose?
- What does the user press?
- What should the user hear?
- What should the user see?
- What proves the control path?
- What proves the output path?
- What remains blind, weak, stubbed, or aspirational?

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

A nearest downstream render seam is partial evidence. When a slice claims
musician-facing instrument progress and the live path is in scope, require the
exact live runtime / mixer path before calling it complete.

## Work Classes

Classify each slice before implementation:

- `audible_vertical_slice`: changes what the musician can hear or play; prove
  the product path, musician action, audible consequence, and listening state
- `contract_enabler`: changes a prerequisite contract; name exactly one
  directly enabled audible follow-up issue and outcome
- `maintenance/regression`: preserves behavior; do not present it as musical or
  instrument progress

Do not chain contract enablers without landing the named audible follow-up.
Offline binaries, scripted packs, fixtures, reports, and validators stay
diagnostic until their behavior is promoted into product ownership.

## Product Intelligence Rule

Riotbox must not confuse scaffolding with intelligence.

Hardcoded phrases, fixed templates, scripted demo paths, fingerprint-only
variation, and source-aware mutations are controls, regression scaffolds, or
architecture proofs; they are not product-quality source-derived behavior until
source evidence changes the musical decision and the audible output.

A narrow product-instrument exception exists for fixed, typed, versioned
vocabulary explicitly committed by the performer. Such vocabulary must be
identified by a stable schema and recipe ID, carry its typed selection inputs,
and link to the actual committed Action/Boundary and affected RuntimeMix
artifacts. It remains `primitive_renderer`, must never activate as a
missing-source fallback, and cannot claim source-derived recipe selection,
composition, or quality proof. If source evidence audibly modulates fixed
vocabulary without selecting its recipe, record that separately under a
versioned product-primitive boundary: exact source feature, derived policy
values, resolved render inputs, all affected RuntimeMix paths, and
`source_evidence_selects_pattern: false`. Never hide source-responsive pressure
or timbre behind an `availability_and_timing_only` claim.
The primitive schema must be explicitly registered for product output in the
shared listening-manifest validator. A version-looking schema or recipe string
without schema-specific recipe/input, source-modulation, activation, RuntimeMix, and candidate-WAV
checks remains diagnostic-only and cannot authorize product sound.

Do not implement hardcoded musical/audio fallback output as a Riotbox product
path. If source-backed generation cannot produce trusted material, surface
unavailable / degraded state to the musician instead of playing synthetic
replacement music. Diagnostic comparisons may use silence or explicitly labeled
non-product controls, but fallback sound must not exist on playable Riotbox
output paths.

For any feature that claims source-derived or intelligent behavior, prove all
five surfaces:

- source evidence: the decision consumed real source features such as timing,
  transients, low-band pressure, density, section role, hook/restraint context,
  slice identity, or captured audio material
- musical decision: the system chose a role, placement, contour, density,
  silence/stay-out decision, destructive gesture, or arrangement move from that
  evidence
- product spine: the decision is represented in Source Graph, Session, Action
  Lexicon, queue / commit, replay, or another documented contract as appropriate
- audible consequence: rendered audio changes in a way a musician can hear
- quality proof: same-source reproducibility and cross-source diversity are
  tested, and scripted/hardcoded artifacts stay labeled `quality_proof: false`
  until structured listening review accepts them

For shared audible DSP, mix, pattern, or performance-policy tuning, do not wait
for broad source-family expansion to catch overfitting. Keep one Golden Path as
the human taste target, but run at least three contrasting real sources through
the nearest exact product-path matrix before the next listening request. This
matrix is a safety/diversity gate, not a substitute for the Golden Path human
verdict.

If a slice only adds the spine or a deterministic scaffold, say that plainly in
the PR and docs. Do not describe it as complete musical intelligence.

## Production Add-ons

When Riotbox work affects audible character, pattern quality, slices, loops, presets, demos, drum/bass behavior, performance controls, or musician-facing taste, also apply the companion `riotbox-rave-punk-production` skill if available.

Use that add-on especially when the user says the output is boring, polite, weak, generic, identical, silent, placeholder-like, or only "ding ding ding". If the companion skill is not automatically loaded, read `../riotbox-rave-punk-production/SKILL.md` from the skills directory when accessible and apply its production checks in addition to this engineering workflow.

## Audio Work

For audio-producing work, use this minimum gate:

- unit/integration tests for the control path
- buffer regression or offline render metrics for the output path
- source-vs-control comparison when a source-backed feature could silently collapse to fallback
- local listening or explicit note that the audible seam is not operational yet

Default to this order: implement the audible product behavior, render or capture
the narrow seam, listen, then freeze the smallest regression that catches the
observed failure. Add new QA infrastructure first only when the current behavior
cannot otherwise be evaluated honestly.

If a user says two gestures sound the same, prefer adding or tightening an output comparison before adding more UI/log assertions.

An exact Golden Path renderer must execute only the documented musician preparation and gestures. Do not silently queue a lane mode, phrase decision, pattern, monitor route, or other prerequisite that the recipe/UI does not expose. If a prerequisite is required, make it a visible product step and prove its committed action.

When a QA gate reuses an existing source-backed render or manifest, validate it
against the timing identity stored in that artifact. Do not substitute a
generated-fixture BPM, sample rate, anchor, or source constant; that turns valid
cross-source evidence into a false failure and hides whether the gate is truly
source-general.

When tuning typed versioned audio vocabulary, keep historical review/control versions sample-stable unless an explicit compatibility decision says otherwise. Split recipe-local focus, gain, or articulation data by version instead of reusing one mutable constant across the historical and current IDs.

If output metrics pass but the musician-facing result is still weak, treat the feature as technically partial, not done.

After at most two consecutive generations of a review-ready candidate that
still has `human_verdict: unverified`, stop generation for that candidate and
perform or explicitly hand off structured human listening. Do not continue with
another report, threshold, fixture, queue, or validator unless the current
failure is genuinely unobservable.

## Structured Listening Review

For audible changes that need structured human taste review, apply the companion `riotbox-listening-review` skill when available. If it is not automatically loaded, read `../riotbox-listening-review/SKILL.md` from the skills directory when accessible.

Keep human QA playback bounded. Default to at most 10 seconds, and use only
2-5 seconds for an isolated capture or stem when that is enough to judge the
requested property. Exceed 10 seconds only when a named multi-bar development
claim genuinely requires it; state the exact duration and purpose before
requesting readiness. A live instrument loop is not a bounded audition: issue
and verify an explicit audible stop at the announced endpoint, and terminate
the runtime immediately if transport stop does not silence every active lane.

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
8. Convert the finding into one concrete follow-up. Prefer the audio policy or
   implementation fix when the failure is already observable; use a fixture,
   threshold, regression render, or UX cue when it closes a real evidence or
   musician-understanding blind spot.

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

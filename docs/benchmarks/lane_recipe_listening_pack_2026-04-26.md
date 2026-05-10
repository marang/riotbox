# Lane Recipe Listening Pack 2026-04-26

Status: initial local QA harness  
Ticket: `RIOTBOX-299`

## Purpose

This pack is the first local audio-output harness for documented Jam routines outside the W-30 source-preview path.

It exists because the routine audit showed that W-30 now has source-vs-fallback proof, while TR-909, Scene Brain, and MC-202 still needed clearer audio-output accounting. The current pack now includes TR-909, Scene-coupled TR-909, and explicit offline MC-202 render cases for follower-vs-answer, touch energy, pressure, instigator, phrase mutation, note budget, source-section contour hints, and hook-response restraint.

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

Each comparison checks both:

- normal RMS difference between the two rendered files
- sample-by-sample signal delta RMS, so a phrase change with similar loudness cannot silently collapse into the same fallback output

The pack root also writes:

- `pack-summary.md`

For local human listening notes, create `notes.md` beside either the pack root or a specific case:

```bash
just audio-qa-notes artifacts/audio_qa/<date>/lane-recipe-listening-pack/notes.md
just audio-qa-notes artifacts/audio_qa/<date>/lane-recipe-listening-pack/tr909-support-to-fill/notes.md
```

Template source: `docs/benchmarks/audio_qa_listening_review_template_2026-04-26.md`.

## Observer / Audio Correlation Gate

The current CI-safe Recipe 2 correlation gate is:

```bash
just recipe2-observer-audio-gate
```

It generates and validates a headless app-level Recipe 2 observer stream,
generates a fresh lane recipe listening pack, validates the listening manifest,
and then runs `observer_audio_correlate --require-evidence` against both
evidence streams.

The gate currently requires the MC-202 Recipe 2 cases to be present, passing,
non-collapsed, above their signal-delta thresholds, aligned to their internal
phrase grid, and attached to a selected Source Graph phrase slot.
It also requires the generated observer stream to include the live-observer
snapshot envelope on launch, runtime, key-outcome, and commit events.

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
- `mc202-follower-to-answer`
  - baseline: follower drive
  - candidate: answer hook
  - covers the first explicit MC-202 offline audio contrast for Recipe 2 and Recipe 5
- `mc202-touch-low-to-high`
  - baseline: follower drive with low touch
  - candidate: the same follower phrase with high touch
  - covers the `<` / `>` touch-energy gesture from Recipe 2
  - after note-budget policy, minimum RMS delta is `0.005500`; signal delta remains the stronger fallback-collapse guard at `0.006000`
- `mc202-follower-to-pressure`
  - baseline: follower drive
  - candidate: pressure cell
  - covers the `P` pressure gesture from Recipe 2
- `mc202-follower-to-instigator`
  - baseline: follower drive
  - candidate: instigator spike
  - covers the `I` instigate gesture from Recipe 2
  - local `riotbox-309-local` metrics after note-budget policy: signal delta RMS `0.009384`, RMS delta `0.002314`, result `pass`
- `mc202-follower-to-mutated-drive`
  - baseline: follower drive
  - candidate: mutated drive
  - covers the `G` phrase-mutation gesture from Recipe 2
- `mc202-neutral-to-lift-contour`
  - baseline: follower drive with neutral contour
  - candidate: the same follower role with lift contour
  - covers the first source-section contour hint on the MC-202 render seam
  - local `riotbox-310-local` metrics: signal delta RMS `0.007847`, RMS delta `0.000006`, result `pass`
- `mc202-direct-to-hook-response`
  - baseline: follower drive in direct mode
  - candidate: the same follower phrase with `answer_space` hook response
  - covers the first hook-response guardrail against doubling hook downbeats
  - local `riotbox-311-local` metrics: signal delta RMS `0.008681`, RMS delta `0.004777`, result `pass`

## Current Limits

This is not yet a full performance-recorder or TUI replay harness.

MC-202 now has bounded offline render cases in this pack. They prove that follower/answer, touch, pressure, instigator, mutated phrase shapes, note budget, source-section contour hints, and hook-response restraint can be rendered and compared as audio artifacts. They do not mean Riotbox has a finished source-aware bassline engine.

Scene Brain now has two bounded audio proof layers:

- the listening pack still represents Scene Brain through the TR-909 `scene_target` support-accent case
- `scene_jump_restore_replay_proves_state_and_mixed_audio_path` additionally proves the current app-level `jump -> restore` flow across mixed TR-909 + MC-202 output, including non-silence, signal-delta on launch/restore, landed Scene movement state, and bounded movement shaping instead of fallback collapse

These prove the current Scene-coupled lane seams and the first persisted movement seam, not a finished Scene transition engine or full arranger.

Generated WAVs remain local and untracked under `artifacts/audio_qa/`.

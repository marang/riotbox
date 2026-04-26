# Scene Post-Landed Energy Cue Baseline

- Timestamp: `2026-04-26`
- Commit SHA: `bcbde72`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `0cdc17f`

## Scope

This is the bounded readability baseline for the post-landed Scene Brain cue after it learned to use compact `scene/energy` labels and the compact TR-909 support hint.

It measures whether the current shipped shell keeps the immediate after-landing cue aligned with the rest of the restore-ready, live/restore, and Scene-driven TR-909 support language.

The baseline stays intentionally manual and repo-local. It records one current `Jam` cue, not a new benchmark harness.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- use the current Scene Brain path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - let the jump land
  - stay on `Jam`
  - read the post-landed Scene cue
  - press `[Y]` to queue restore
  - let restore land
  - read the post-landed Scene cue again

Current interaction seam assumptions:

- a landed scene jump establishes a restore target
- the post-landed cue appears on `Jam` after `scene jump` or `scene restore` lands
- the cue should speak the same compact `scene/energy` language as the ready and contrast cues
- when TR-909 support accent is driven by the landed Scene target, the cue may append `909 lift`

## Measured Values

### 1. Time to understand the current scene after a landed jump

Definition for this baseline:

- from the moment `scene jump` lands
- to the moment the player can answer which scene/energy state is now live

Current measured value:

- zero screen switches
- one direct shell read in `Jam`:
  - `changed: scene <scene>/<energy> | next [Y] restore <scene>/<energy> [c] capture | 909 lift`

Judgment:

- `Pass`

Why this is acceptable now:

- the cue no longer drops from `scene/energy` vocabulary back to scene-only labels after the action lands
- the player can read both the live result and the next restore option from one line
- `909 lift` gives one compact hint that TR-909 support is following the Scene target without forcing a switch to `Log`

### 2. Time to understand the current scene after a landed restore

Definition for this baseline:

- from the moment `scene restore` lands
- to the moment the player can answer which scene/energy state is live again

Current measured value:

- zero screen switches
- one direct shell read in `Jam`:
  - `changed: scene <scene>/<energy> | next [y] jump [c] capture | 909 lift`

Judgment:

- `Pass`

Why this is acceptable now:

- the restored result is named with the same compact energy label as the pre-restore target
- the next-step cue stays short enough to remain a post-landed hint rather than a second help overlay
- the same `909 lift` suffix applies after restore when the restored Scene target drives TR-909 support

## Qualitative Friction Notes

- the cue is still text-first and can wrap on narrower terminal sizes
- the line is useful as a confirmation cue, but it does not yet replace the need for a stronger visual timing instrument
- `909 lift` is intentionally a compact performance hint; `Log` and Inspect remain the diagnostic surfaces for `scene_target` versus fallback context
- the current language is now internally consistent across queued, ready, contrast, post-landed, and TR-909 support Scene Brain states

## Follow-Up

- compare future post-landed wording changes against this baseline before shortening the cue again
- use this alongside the restore-ready and restore-state contrast baselines so Scene Brain keeps one coherent `scene/energy` vocabulary
- use the TR-909 support context baselines when changing the deeper diagnostic labels behind the compact `909 lift` hint

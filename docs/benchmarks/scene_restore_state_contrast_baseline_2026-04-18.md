# Scene Restore State Contrast Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `ad233da`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the contrast between the two current restore
states on `Jam`:

- `restore waits for one landed jump`
- `restore <scene>/<energy> ready`

The goal is to keep those states from collapsing back into one ambiguous `Y` affordance as Scene
Brain wording evolves.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- observe the initial `Y` affordance before any landed jump
- press `[Space]`
- press `[y]` and let one scene jump land
- return to `Jam` if needed
- observe the new restore-ready cue on `Jam`
- open help with `[?]`
- read the restore block

Current interaction seam assumptions:

- before a landed jump, restore is not actionable
- after a landed jump, restore becomes actionable on the next bar
- the shipped shell currently distinguishes those states in both the footer and help overlay

## Measured Values

### 1. Time to distinguish wake-up from ready on the default shell

Definition for this baseline:

- from first seeing `Jam`
- to the moment the player can answer whether `Y` is still waiting on the first landed jump or can already queue restore

Current measured value:

- zero screen switches
- one direct shell read before the first landed jump:
  - `[Y] restore waits for one landed jump`
- one direct shell read after the first landed jump:
  - `[Y] restore <scene> now`
  - `Scene cue: restore <scene>/<energy> ready | Y brings back <scene>/<energy>`

Judgment:

- `Pass`

Why this is acceptable now:

- the shell exposes two different restore states on the default Jam surface
- the ready state carries the concrete target instead of only saying that restore is generally back

### 2. Time to confirm the same state contrast in Help

Definition for this baseline:

- from the moment the player opens `Help`
- to the moment the player can explain the current restore state in words

Current measured value:

- one explicit gesture: `[?]`
- before first landed jump:
  - `Scene restore`
  - `Y waits for one landed jump`
  - `land one jump, then Y can restore the last scene`
- after first landed jump:
  - `Scene restore`
  - `Y is live now for <scene>/<energy>`
  - `press Y to bring <scene>/<energy> back on the next bar`

Judgment:

- `Pass`

Why this is acceptable now:

- the help overlay mirrors the same contrast already visible on `Jam`
- the two restore states are explained without inventing a separate diagnostic model

## Qualitative Friction Notes

- the contrast is readable now, but still text-heavy compared to a future more graphical timing instrument
- the wake-up wording is intentionally more blocked and provisional than the ready wording, which is correct today but still easy to over-compress later
- this seam is now much clearer as a state transition than as a musical promise

## Follow-Up

- compare future wake-up or ready wording changes against this baseline before simplifying the restore affordance again
- use this alongside the restore-ready readability baseline so the shell keeps both sides of the restore state machine visible

# Scene Guidance Stack Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `9b97c12`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded baseline for the current queued-scene guidance stack across `Jam`, `Help`, and `Log`.

It measures whether the current shipped shell gives one compact guidance path for a queued `scene jump` or `scene restore`:

- a tiny always-visible cue in the `Footer`
- a compact explanation in the `Help` overlay
- a bounded confirmation path in `Log`

The baseline stays intentionally manual and repo-local. It measures the visible learning seam, not a new analytics layer.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- use the current Scene Brain seam:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - stay on `Jam` and read the footer cue
  - open help with `[?]` and read the scene timing block
  - close help
  - let the jump land
  - `[2]` switch to `Log` and confirm the landed trail
  - `[1]` go back to `Jam`
  - `[Y]` queue one restore and repeat the same path

Current interaction seam assumptions:

- the current `Jam` footer replaces the normal `Advanced:` row with a compact scene cue while `jump` or `restore` is pending
- the current help overlay adds a bounded `Scene timing` block only while a scene action is pending
- the current `Log` counts panel exposes the recent `trail ...` scene result cue

## Measured Values

### 1. Time to notice there is a scene-specific reminder without opening Help

Definition for this baseline:

- from the moment a queued scene action is pending on `Jam`
- to the moment the player can answer:
  - there is a queued-scene-specific reminder on the default shell
  - it names the current target and boundary

Current measured value:

- zero screen switches
- one direct shell read in `Footer`:
  - `Scene cue: launch <scene> @ next bar | <rise|drop|hold> + pulse, 2 trail`
  - or `Scene cue: restore <scene> @ next bar | <rise|drop|hold> + pulse, 2 trail`

Judgment:

- `Pass`

Why this is acceptable now:

- the player no longer needs to open help just to discover that a scene-specific seam exists
- the cue stays on a line that is already part of the default shell rhythm

### 2. Time to get the compact explanation of what to read on Jam

Definition for this baseline:

- from the moment a queued scene action is pending
- to the moment the player can answer:
  - which current `Jam` cues matter before the action lands
  - which next check matters after it lands

Current measured value:

- one explicit gesture: `[?]`
- zero screen switches
- one direct help read:
  - `Scene timing`
  - `<launch|restore> <scene>: lands at next bar`
  - `Jam: read launch/restore, pulse, live/restore energy`
  - `2: confirm the landed trail on Log`

Judgment:

- `Pass`

Why this is acceptable now:

- the help overlay points directly at the already-shipped scene seam, including the new compact energy contrast, instead of inventing a second diagnostic explanation
- the block is bounded to one small scene-specific chunk

### 3. Time to confirm the recent scene result after using the guidance stack

Definition for this baseline:

- from the moment a guided queued scene action lands
- to the moment the player can answer whether the last move was a jump or a restore

Current measured value:

- one screen switch: `[2]`
- one direct shell read in `Log`:
  - `trail j <scene>`
  - or `trail r <scene>`

Judgment:

- `Pass`

Why this is acceptable now:

- the guidance path stays compact and stable:
  - footer for reminder
  - help for explanation
  - log for confirmation

## Qualitative Friction Notes

- the footer cue is intentionally terse, so it works better as a reminder than as a full explanation
- the help cue still requires an explicit overlay open, which is acceptable now but not ideal forever
- the combined path is readable, but still more operator-ish than instrument-native

## Follow-Up

- compare any future scene-specific footer or help redesign against this bounded stack before adding heavier onboarding chrome
- use this baseline when deciding whether more scene guidance belongs in `Jam`, `Help`, or a future first-use surface

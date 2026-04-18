# Scene Restore-Ready Readability Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `612d417`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the `restore ready` seam on `Jam`.

It measures whether the current shipped shell makes the restore target legible at the moment `Y`
changes from a wake-up affordance into an actual ready action.

The baseline stays intentionally manual and repo-local. It measures one current shell seam, not a
new analytics layer.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- use the current Scene Brain recovery path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - let the jump land
  - stay on `Jam`
  - read the restore-ready cue
  - open help with `[?]`
  - read the restore-ready block
  - close help
  - press `[Y]` to queue restore

Current interaction seam assumptions:

- `Y` is not restore-ready before a landed jump establishes a restore target
- after one landed jump, `Jam` shows:
  - a compact restore-ready footer cue
  - a matching restore-ready help block
- `scene restore` still commits on `NextBar`

## Measured Values

### 1. Time to understand which scene `Y` will bring back

Definition for this baseline:

- from the moment the first scene jump lands
- to the moment the player can answer which scene `Y` will restore

Current measured value:

- zero screen switches
- one direct shell read in `Jam` footer:
  - `Scene cue: restore <scene>/<energy> ready | Y brings back <scene>/<energy>`
- optional one help overlay open:
  - `Scene restore`
  - `Y is live now for <scene>/<energy>`
  - `press Y to bring <scene>/<energy> back on the next bar`

Judgment:

- `Pass`

Why this is acceptable now:

- the restore target is named directly in both the compact footer and the matching help block
- the shell no longer relies on the player inferring the target from a generic `it` pronoun

### 2. Time to distinguish `restore not ready` from `restore ready`

Definition for this baseline:

- from the initial `Jam` shell before any landed jump
- to the moment the player can tell that restore has become actionable

Current measured value:

- before first landed jump:
  - `[Y] restore waits for one landed jump`
- after first landed jump:
  - `[Y] restore <scene> now`
  - `Scene cue: restore <scene>/<energy> ready | Y brings back <scene>/<energy>`

Judgment:

- `Pass`

Why this is acceptable now:

- the shell exposes two distinct states instead of one overloaded restore control
- the ready state now points at the concrete `scene/energy` target that became valid

## Qualitative Friction Notes

- the restore target is now easier to read, but the seam still relies on text rather than a stronger visual timing instrument
- the help overlay remains useful as confirmation, even though the footer now carries more of the first explanation burden
- the wording is more explicit, but the overall Scene Brain surface is still denser than a polished musician-facing instrument

## Follow-Up

- compare any future restore-ready wording changes against this baseline before broadening the help copy again
- use this alongside the queued-scene guidance-stack baseline so `pending` and `ready` scene cues stay coherent as one learning path

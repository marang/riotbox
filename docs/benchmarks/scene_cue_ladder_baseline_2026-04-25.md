# Scene Cue Ladder Baseline 2026-04-25

- Timestamp: `2026-04-25`
- Commit SHA: `5cc75dd`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the complete Scene Brain cue ladder on the current shipped shell.

It ties together the smaller baselines for queued guidance, post-landed confirmation, restore readiness, restore-state contrast, and Log trail confirmation.

The baseline stays docs-only and manual. It captures one coherent reading path through existing `Jam`, `Help`, and `Log` surfaces.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- run the current Scene Brain path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - read the queued footer cue
  - open `[?]` and read the scene timing block
  - close help
  - let the jump land
  - read the post-landed cue
  - read the restore-ready cue
  - `[Y]` queue restore
  - read the queued restore cue
  - let restore land
  - read the post-landed restore cue
  - `[2]` open `Log` and confirm the trail

Current interaction assumptions:

- scene jump and scene restore land on their current quantized boundary
- queued scene actions use the compact footer cue
- post-landed scene cues use compact `scene/energy` labels
- restore has a distinct wait-state before the first landed jump and a ready-state after it
- `Log` remains the confirmation surface for the recent trail

## Measured Values

### 1. Cue ladder length for a first jump

Definition for this baseline:

- from `[y]` queue to a readable landed jump result

Current measured path:

- `Jam` footer:
  - `Scene cue: launch <scene> @ next bar | <rise|drop|hold> + [===>], 2 trail`
- `Help` overlay:
  - `Scene timing`
  - `<launch|restore> <scene>: lands at next bar`
  - `Jam: read launch/restore, pulse, live/restore energy`
- post-landed `Jam` cue:
  - `changed: scene <scene>/<energy> | next [Y] restore <scene>/<energy> [c] capture`

Judgment:

- `Pass`

Why this is acceptable now:

- the player can read pending intent, landing timing, live result, and next restore affordance without leaving `Jam`
- `Help` is optional confirmation, not the only way to discover the queued action

### 2. Cue ladder length for restore readiness

Definition for this baseline:

- from initial launch to knowing whether `Y` is actionable

Current measured path:

- before a landed jump:
  - `[Y] restore waits for one landed jump`
- after a landed jump:
  - `Scene cue: restore <scene>/<energy> ready | Y brings back <scene>/<energy>`

Judgment:

- `Pass`

Why this is acceptable now:

- the shell exposes the negative and positive restore states as distinct readable states
- the ready state names the same `scene/energy` target used by the post-landed cue

### 3. Cue ladder length for restore landing and confirmation

Definition for this baseline:

- from `[Y]` queue to a readable restored result and trail confirmation

Current measured path:

- queued restore footer:
  - `Scene cue: restore <scene> @ next bar | <rise|drop|hold> + [===>], 2 trail`
- post-landed `Jam` cue:
  - `changed: scene <scene>/<energy> | next [y] jump [c] capture`
- `Log` trail:
  - `trail r <scene>`

Judgment:

- `Pass`

Why this is acceptable now:

- restore has the same basic reading ladder as jump:
  - queued intent
  - landed result
  - trail confirmation
- `Log` remains technical enough to confirm the action without forcing those details into `Jam`

## Qualitative Friction Notes

- the cue ladder is now internally coherent, but still text-heavy
- the current `Jam` shell can wrap the post-landed cue on narrower terminals
- the path teaches Scene Brain better than before, but a future visual timing indicator would still reduce reading load

## Follow-Up

- use this ladder baseline before changing Scene Brain cue wording across multiple surfaces
- compare future first-run UX work against this full path, not only against isolated footer or help lines

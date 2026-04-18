# Scene Timing Readability Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `78e2602`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first explicit readability baseline for the current Scene Brain timing cues on the perform-first Jam shell.

It measures whether the shipped Jam surface now answers three small timing questions without forcing the player into `Log`:

- when a queued `scene jump` will land
- where the player currently is in beat / bar / phrase terms while that jump is pending
- what the next sensible move is after a jump or restore lands

The baseline stays intentionally manual and repo-local. It measures the visible shell cues, not a new analytics subsystem.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
  ```

- wait until the Jam shell is visible
- use the current Scene Brain timing path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - stay on `Jam`
  - read the `Next` panel until the queued boundary and pulse are both visible
  - let the jump land
  - stay on `Jam` and read the post-commit `changed / next / then` guidance
  - `[Y]` queue one restore
  - again stay on `Jam` and read the same post-commit guidance after restore lands

Current interaction seam assumptions:

- `scene jump` commits on `NextBar`
- `scene restore` commits on `NextBar`
- the current Jam `Next` panel exposes:
  - pending scene boundary
  - compact beat / bar / phrase pulse
  - scene-specific post-commit guidance

## Measured Values

### 1. Time to understand queued jump timing on Jam

Definition for this baseline:

- from the moment a `scene jump` is queued on `Jam`
- to the moment the player can answer:
  - it lands on `next bar`
  - current timing context is visible as beat / bar / phrase

Current measured value:

- one explicit gesture: `[y]`
- zero screen switches
- one direct shell read:
  - `launch -> ... @ next bar`
  - `pulse b.. | bar .. | ph .. -> next bar`

Judgment:

- `Pass`

Why this is acceptable now:

- the queue boundary is no longer Jam-hidden
- the timing cue is present on the same surface as the pending action

### 2. Time to understand what changed after jump

Definition for this baseline:

- from the moment the jump lands
- to the moment the player can answer:
  - which scene is now live
  - which restore target is available
  - what the next recommended move is

Current measured value:

- zero screen switches
- one direct shell read:
  - `changed: <scene> | restore <scene>`
  - `next: [Y] restore  [c] capture`

Judgment:

- `Pass`

Why this is acceptable now:

- the post-commit guidance is now scene-specific instead of generic action text
- the recovery move is visible on the same Jam surface as the result

### 3. Time to understand what changed after restore

Definition for this baseline:

- from the moment the restore lands
- to the moment the player can answer:
  - which scene is now live
  - which jump target is available next
  - what one sensible next move is

Current measured value:

- zero screen switches
- one direct shell read:
  - `changed: <scene> | restore <scene>`
  - `next: [y] jump  [c] capture`

Judgment:

- `Pass`

Why this is acceptable now:

- restore is now legible as a live scene-state result, not just a technical command
- the shell points the player back into the scene loop without requiring Log-first interpretation

## Qualitative Friction Notes

- the timing cue is still textual, not graphical
- beat / bar / phrase are visible, but the shell still does not visualize motion or countdown toward the next bar
- post-commit guidance now fits the perform shell, but deeper onboarding still belongs to future first-use UX work

## Follow-Up

- compare future pulse-visualization work against this baseline before adding heavier timing graphics
- use this baseline when evaluating whether Scene Brain timing cues can stay perform-first without forcing frequent `Log` detours

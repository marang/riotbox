# Scene Timing Readability Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `1285add`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first explicit readability baseline for the current Scene Brain timing cues on the perform-first Jam shell.

It measures whether the shipped Jam surface now answers three small timing questions without forcing the player into `Log`:

- when a queued `scene jump` will land
- where the player currently is in beat / bar / phrase terms while that jump is pending
- what the next sensible move is after a jump or restore lands

It also checks whether the current shipped Scene Brain stack now keeps one compact answer for:

- which scene is live versus which scene is the restore target
- what recent `jump` / `restore` history looks like on `Log`

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
  - tiny ASCII countdown cue inside that pulse
  - scene-specific post-commit guidance
- the current Jam `Now` panel exposes:
  - compact `live <> restore` scene contrast
- the current `Log` counts panel exposes:
  - compact recent scene trail for the last committed `jump` / `restore` moves

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
  - `pulse [..>.] b.. | b.. | p..`

Judgment:

- `Pass`

Why this is acceptable now:

- the queue boundary is no longer Jam-hidden
- the timing cue is present on the same surface as the pending action
- the small countdown glyph gives one quick visual anchor without opening a larger timing widget

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
  - `live <scene> <> restore <scene>`
  - `changed: <scene> | restore <scene>`
  - `next: [Y] restore  [c] capture`

Judgment:

- `Pass`

Why this is acceptable now:

- the live scene and restore target are now contrasted in the `Now` panel before the player even reads the post-commit guidance
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
  - `live <scene> <> restore <scene>`
  - `changed: <scene> | restore <scene>`
  - `next: [y] jump  [c] capture`

Judgment:

- `Pass`

Why this is acceptable now:

- restore is now legible as a live scene-state result, not just a technical command
- the shell points the player back into the scene loop without requiring Log-first interpretation

### 4. Time to reconstruct recent scene moves on Log

Definition for this baseline:

- from the moment at least one `scene jump` or `scene restore` has landed
- to the moment the player can answer:
  - what the most recent scene move was
  - whether the last move was a `jump` or a `restore`

Current measured value:

- one screen switch: `[2]`
- one direct shell read in `Counts`:
  - `trail j <scene>`
  - or `trail r <scene>`

Judgment:

- `Pass`

Why this is acceptable now:

- the `Log` screen now preserves one bounded scene-result memory without inventing a second scene-history subsystem
- the trail stays compact enough to share space with the existing action-log truth surface

## Qualitative Friction Notes

- the countdown cue is still tiny and ASCII-first; it is helpful, but not yet a full motion or bar-proximity widget
- the `Log` trail is intentionally compressed (`j` / `r`) to fit the existing layout, so it optimizes for compactness over explanation
- post-commit guidance now fits the perform shell, but deeper onboarding still belongs to future first-use UX work

## Follow-Up

- compare any future larger timing widget against this compact countdown baseline before adding heavier graphics
- use this baseline when evaluating whether Scene Brain can stay perform-first without forcing frequent `Log` detours
- use the current `live <> restore` and `trail ...` cues as the small-layout baseline before introducing denser scene history or inspect-only scene detail

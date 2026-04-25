# Scene Footer Tick Readability Baseline 2026-04-25

- Timestamp: `2026-04-25`
- Commit SHA: `2bfd636`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the compact ASCII timing tick in the queued Scene Brain footer cue.

It measures whether the tick helps the player notice that a queued scene action is approaching a musical landing point, without requiring a larger timing widget.

The baseline stays docs-only and manual. It records one current `Jam` footer cue, not a new benchmark harness.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/DH_RushArp_120_A.wav"
  ```

- wait until the Jam shell is visible
- use the current queued Scene Brain path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - stay on `Jam`
  - read the footer cue
  - wait a beat and read the footer cue again

Current interaction assumptions:

- the footer cue appears only while a `scene jump` or `scene restore` is pending
- the compact tick is derived from the same beat position as the existing Jam pulse line
- the tick is a cue, not a precise transport ruler

## Measured Values

### 1. Time to notice the pending action has a timing shape

Definition for this baseline:

- from the moment a scene action is pending
- to the moment the player can answer whether the footer shows a changing timing marker

Current measured value:

- zero screen switches
- one direct footer read:
  - `Scene: launch <scene> @ next bar | <rise|drop|hold> [===>] | 2 trail`

Judgment:

- `Pass`

Why this is acceptable now:

- the footer now carries a visual marker instead of only the word `pulse`
- the marker is compact enough to fit inside the existing Scene cue line

### 2. Time to connect the tick with the larger Jam pulse line

Definition for this baseline:

- from the footer read
- to the moment the player can connect the small tick to the larger Jam pulse cue

Current measured value:

- zero screen switches
- two direct `Jam` reads:
  - footer: `... [===>] | 2 trail`
  - pulse line: `pulse [===>] b32 | b8 | p1`

Judgment:

- `Pass`

Why this is acceptable now:

- both cues use the same ASCII marker shape
- the footer remains the reminder while the main Jam pulse line remains the richer timing read

## Qualitative Friction Notes

- the tick is intentionally coarse and should not be read as sample-accurate timing
- the marker is more scannable than the word `pulse`, but the shell is still text-heavy
- this does not solve the future need for a stronger visual timing instrument

## Follow-Up

- compare future visual timing work against this baseline before adding a larger bar or graph
- keep the footer tick aligned with the main Jam pulse marker if the marker shape changes

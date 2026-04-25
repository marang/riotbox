# Jam Footer Color Hierarchy Baseline 2026-04-25

- Timestamp: `2026-04-25`
- Commit SHA: `54147ce`
- Fixture ID: `data/test_audio/examples/DH_RushArp_120_A.wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the Jam footer color and emphasis hierarchy.

It measures whether the first semantic palette makes the footer easier to scan without changing the text contract, layout, keymap, or monochrome snapshot readability.

The baseline stays docs-only and manual. It records the expected read order of the current footer, not a new screenshot or color-diff harness.

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
  - read the footer from top to bottom
- repeat once with a warning visible, if a source or runtime warning is already present

Current interaction assumptions:

- color is semantic reinforcement only
- the footer text must still be readable in monochrome terminals and test snapshots
- the first palette is intentionally small:
  - cyan + bold for primary perform controls
  - yellow + bold for active Scene timing or restore affordances
  - red + bold for warning labels, with yellow warning detail
  - green for clear/healthy confirmation
  - dark gray for lower-priority status diagnostics

## Measured Values

### 1. Time to find the primary live controls

Definition for this baseline:

- from first seeing the footer
- to the moment the player can identify the main play gestures without reading status diagnostics first

Current measured value:

- zero screen switches
- one footer read:
  - `Primary: [y] scene jump [g] follow [f] fill [c] capture [w] hit [u] undo`

Judgment:

- `Pass`

Why this is acceptable now:

- the `Primary:` label is cyan + bold
- the control text stays unchanged, so existing learning docs and monochrome snapshots remain valid

### 2. Time to find a pending Scene timing cue

Definition for this baseline:

- from the moment a Scene action is pending
- to the moment the player can separate the active Scene cue from lane ops and status text

Current measured value:

- zero screen switches
- one direct footer read:
  - `Scene: launch <scene> @ next bar | <rise|drop|hold> [===>] | 2 trail`

Judgment:

- `Pass`

Why this is acceptable now:

- the full Scene cue line is yellow + bold
- the wording still carries the action, target, boundary, timing tick, and trail count without relying on color alone

### 3. Time to separate warnings from status diagnostics

Definition for this baseline:

- from seeing the lower footer block
- to the moment the player can identify whether there is a warning or only status noise

Current measured value:

- zero screen switches
- one footer read:
  - clear case: `Warnings clear | source trust stable enough for shell work`
  - warning case: `Warning: <warning text>`

Judgment:

- `Pass`

Why this is acceptable now:

- clear-state text is green
- warning labels are red + bold, while warning detail is yellow
- status diagnostics are dark gray and therefore visually secondary

## Qualitative Friction Notes

- terminal color themes vary, so the palette must remain conservative
- the footer is still text-heavy; this baseline does not replace future layout simplification
- the current test snapshot path checks text, while the new unit test checks the style tokens directly

## Follow-Up

- compare future Jam footer styling changes against this baseline before adding more colors
- keep color labels semantic rather than decorative
- do not move warning or Scene meaning into color-only cues

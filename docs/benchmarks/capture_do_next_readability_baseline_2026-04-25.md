# Capture Do Next Readability Baseline 2026-04-25

- Timestamp: `2026-04-25`
- Commit SHA: `303a677`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the Capture screen after the `Do Next` hierarchy landed.

It measures whether a new user can find the current capture workflow without reading provenance, pinned-state, or routing diagnostics first.

The baseline stays docs-only and manual. It records the expected scan order of the current Capture surface, not a screenshot diff or audio QA harness.

## Procedure

Starting point:

- launch Riotbox from the current example-source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
  ```

- start transport with `[Space]`
- queue one first gesture such as `[f]` fill or `[g]` follow
- after the first result lands, press `[c]` to capture
- press `[4]` to open `Capture`
- read the top row from left to right

Current interaction assumptions:

- stored capture is not automatically the same thing as an audible W-30 hit
- promotion is still required before the normal `[w]` hit path is useful
- `Log` remains the truth surface for verifying queued and landed actions
- `Advanced Routing` is diagnostic support, not the first-use path

## Measured Values

### 1. Time to find the next Capture action

Definition for this baseline:

- from first opening `Capture`
- to identifying the next useful capture step without scanning the routing diagnostics

Current measured value:

- zero screen switches
- one top-row read:
  - `Do Next`
  - `1 [o] audition raw <capture>`
  - `2 [p] promote <capture>`
  - `3 [w] hit after promote`

Judgment:

- `Pass`

Why this is acceptable now:

- the first-action path is in the top row
- the wording is sequential and key-driven
- routing internals are no longer presented as the first action

### 2. Time to understand whether the latest capture is audible

Definition for this baseline:

- from opening `Capture`
- to separating stored material from material that can already be hit or auditioned

Current measured value:

- zero screen switches
- one `Latest Capture` read:
  - stored case: `hear <capture> stored [o] raw or [p]->[w]`
  - promoted-pad case: `hear <capture>->pad <bank>/<pad> [w]/[o]`

Judgment:

- `Pass`

Why this is acceptable now:

- the `hear ...` label no longer implies that stored material is already audible
- the promoted case points directly at the current hit and audition keys

### 3. Time to find diagnostics without mistaking them for the primary path

Definition for this baseline:

- from the moment the user needs provenance or routing details
- to finding the relevant diagnostics after reading the first action

Current measured value:

- zero screen switches
- lower-row read:
  - `Provenance`
  - `Advanced Routing`

Judgment:

- `Pass`

Why this is acceptable now:

- provenance remains visible on the Capture screen
- routing is explicitly named advanced, so it reads as support detail instead of the next thing to do

## Qualitative Friction Notes

- `Do Next` is still text-only; a later Capture help cue should teach it without requiring the user to infer the scan order.
- The screen is still dense, but the first action is no longer competing with pinned-state and routing internals in the top row.
- This baseline does not claim that unpromoted captures are directly audible; that still needs future audio/sampler work.

## Follow-Up

- compare future Capture screen changes against this baseline before adding more panels
- keep `Do Next` focused on one capture/promote/hit path
- keep provenance and routing available, but secondary to the first audible action

# Capture Pending Do Next Readability Baseline 2026-04-25

- Timestamp: `2026-04-25`
- Commit SHA: `8cc544f`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `readability`
- Previous baseline: `capture_do_next_readability_baseline_2026-04-25.md`

## Scope

This baseline records the Capture screen after `Do Next` became aware of pending capture-path actions.

It measures whether the screen describes the action the performer just queued instead of falling back to the latest committed capture state.

The baseline is docs-only and manual. It does not claim an automated screenshot diff, audio QA harness, or new capture behavior.

## Procedure

Starting point:

- launch Riotbox from the current example-source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
  ```

- start transport with `[Space]`
- press `[c]` to queue capture
- press `[4]` to open `Capture`
- read `Do Next` before provenance, routing, or recent captures

Promotion-state check:

- after a capture exists, press `[p]` to queue promotion
- open or stay on `Capture`
- read `Do Next`

W-30 reshape-state check:

- after a W-30-focused capture exists, queue a freeze or resample action
- open or stay on `Capture`
- read `Do Next`

Current interaction assumptions:

- pending state is intent, not committed state
- the boundary still decides when the queued action lands
- `Log` remains the truth surface for confirming the final committed result
- `Do Next` should show only the next useful handoff, not every diagnostic detail

## Measured Values

### 1. Queued capture is visible before stored-capture guidance

Definition for this baseline:

- from opening `Capture` with a pending capture action
- to identifying that the system is waiting for a capture commit

Current expected cue:

- `queued [c] capture @ next_phrase`
- `wait for commit`
- `then [p] promote keeper`
- `[2] confirm capture`

Judgment:

- `Pass`

Why this is acceptable now:

- the cue names the queued key and quantization boundary
- the next step is promotion, not blind routing inspection
- confirmation is explicitly delegated to `Log`

### 2. Queued promotion is visible before last-capture fallback

Definition for this baseline:

- from opening `Capture` with a pending promote-to-pad action
- to identifying the target and next audible handoff

Current expected cue:

- `queued [p] promote @ next_bar`
- `wait, then [w] hit`
- `target <lane-or-pad-target>`
- `[2] confirm promotion`

Judgment:

- `Pass`

Why this is acceptable now:

- the cue no longer asks the user to promote something that is already queued
- the next action is an audible W-30 hit after commit
- the target remains visible without making routing the primary path

### 3. Queued reshape stays secondary but legible

Definition for this baseline:

- from opening `Capture` with a pending W-30 loop-freeze or resample action
- to understanding that the next boundary is a W-30 reshape commit

Current expected cue:

- `queued W-30 reshape @ next_phrase`
- `wait for phrase seam`
- `target <capture-or-pad-target>`
- `[2] confirm result`

Judgment:

- `Pass`

Why this is acceptable now:

- freeze and resample share the same user-facing pending shape
- the cue is action-oriented instead of exposing lineage internals first
- confirmation still happens through the existing Log screen

## Qualitative Friction Notes

- `Do Next` is still plain text; later TUI work should make pending-vs-committed state visually distinct.
- The Capture screen still exposes several lower-priority panels; future reductions should preserve this pending-first scan order.
- This baseline does not solve blind listening. It only makes the current queue state harder to miss.

## Follow-Up

- compare future Capture redesigns against these pending-state cues
- keep pending intent ahead of committed fallback copy
- avoid adding more first-row diagnostics unless they directly answer what the performer should do next

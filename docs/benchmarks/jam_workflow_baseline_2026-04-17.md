# Jam Workflow Baseline 2026-04-17

- Timestamp: `2026-04-17`
- Commit SHA: `3d2e4aa`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `workflow`
- Previous baseline: `None`

## Scope

This is the first explicit workflow-benchmark baseline for the current Jam shell.

It records two roadmap/spec metrics in a form that is honest for the current prototype:

- time to first playable Jam state
- time to first successful capture

This baseline is intentionally derived from the shipped interaction seam and quantization model, not from a new stopwatch or analytics subsystem.

## Procedure

Starting point:

- launch Riotbox from the current shipped README path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
  ```

- wait until the Jam shell is visible
- use the current first-run path:
  - `[Space]` start transport
  - `[f]` queue one first fill
  - `[c]` queue one capture after the first landed result

Current interaction seam assumptions:

- `TR-909 fill` commits on `NextBar`
- `capture` commits on `NextPhrase`
- the local reference source is `128 BPM`
- with `4/4`, one bar is `1.875s`
- the current phrase model is treated as `4 bars`, so one phrase is `7.5s`

## Measured Values

### 1. Time to first playable Jam state

Definition for this baseline:

- from visible Jam shell and active transport
- to the first landed musical change from the default first-run path

Current measured value:

- one explicit gesture: `[f]`
- one quantization wait: `<= 1 next bar`
- budget at `128 BPM`: `<= 1.875s`

Judgment:

- `Pass`

Why this is acceptable now:

- the shell already exposes a musically useful first landed result on a single bounded queue/commit path
- the user does not need to discover multiple gestures before the first meaningful state change

### 2. Time to first successful capture

Definition for this baseline:

- from visible Jam shell and active transport
- through the first-result path
- to the first committed capture

Current measured value:

- gesture path: `[f]` then `[c]`
- quantization waits:
  - `fill` on `<= 1 next bar`
  - `capture` on `<= 1 next phrase`
- budget at `128 BPM`:
  - `<= 1.875s` to first landed fill
  - `<= 7.5s` from queued capture to committed capture
  - `<= 9.375s` total from transport start to first successful capture on the default path

Judgment:

- `Pass`

Why this is acceptable now:

- the first capture path is real, visible, deterministic, and recoverable through the current shell
- the result still depends on phrase timing, but the budget is now explicit instead of implicit

## Qualitative Friction Notes

- the minimal first-run path is now usable, but it is still intentionally narrow
- repeated use of the same source and the same first gesture can feel similar, which is why the recipe guide now exists
- `Log` remains the fastest truth surface for confirming that the benchmark path really landed
- the next benchmark-quality improvement should be a semi-automated stopwatch path for the same two workflows, not a new onboarding or analytics architecture

## Follow-Up

- use this baseline as the first reference point for later workflow-benchmark regressions
- compare future first-run/onramp or inspect-mode UX changes against the same fixture and path before expanding the benchmark family

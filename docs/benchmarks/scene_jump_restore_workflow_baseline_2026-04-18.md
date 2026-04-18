# Scene Jump / Restore Workflow Baseline 2026-04-18

- Timestamp: `2026-04-18`
- Commit SHA: `5a40ff0`
- Fixture ID: `data/test_audio/examples/Beat08_128BPM(Full).wav`
- Benchmark family: `workflow`
- Previous baseline: `None`

## Scope

This is the first explicit workflow baseline for the current Scene Brain recovery loop.

It records two current-shell measurements:

- time to first successful scene jump
- time to first successful scene restore after a landed jump

The baseline stays intentionally manual and repo-local. It measures the shipped interaction seam instead of introducing a new stopwatch or analytics subsystem.

## Procedure

Starting point:

- launch Riotbox from the current shipped source path:

  ```bash
  cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
  ```

- wait until the Jam shell is visible
- use the current Scene Brain learning path:
  - `[Space]` start transport
  - `[y]` queue one scene jump
  - `[2]` confirm the jump landed in `Log`
  - `[1]` return to `Jam`
  - `[Y]` queue one restore
  - `[2]` confirm the restore landed in `Log`

Current interaction seam assumptions:

- `scene jump` commits on `NextBar`
- `scene restore` commits on `NextBar`
- the local reference source is `128 BPM`
- with `4/4`, one bar is `1.875s`

## Measured Values

### 1. Time to first successful scene jump

Definition for this baseline:

- from visible Jam shell and active transport
- to the first committed `scene jump`

Current measured value:

- one explicit gesture: `[y]`
- one quantization wait: `<= 1 next bar`
- budget at `128 BPM`: `<= 1.875s`

Judgment:

- `Pass`

Why this is acceptable now:

- the first Scene Brain move is reachable from the default Jam shell without opening a secondary mode
- `Log` gives one clear truth surface for the landed result

### 2. Time to first successful scene restore

Definition for this baseline:

- from visible Jam shell and active transport
- through one landed scene jump
- to the first committed `scene restore`

Current measured value:

- gesture path: `[y]`, then `[Y]`
- quantization waits:
  - `scene jump` on `<= 1 next bar`
  - `scene restore` on `<= 1 next bar` after the restore target becomes meaningful
- budget at `128 BPM`:
  - `<= 1.875s` to first landed jump
  - `<= 1.875s` from queued restore to committed restore
  - `<= 3.75s` total from first jump request to first successful restore on the default path

Judgment:

- `Pass`

Why this is acceptable now:

- the recovery move is real, visible, and bounded by the same queue/commit seam as the initial jump
- the shell now exposes enough restore context that the loop can be measured honestly without hidden state

## Qualitative Friction Notes

- the restore path is still easier to understand on `Log` than on `Jam`
- the current shell does not yet visualize next-bar timing directly, so the user still learns the seam more by landed evidence than by an explicit timing display
- the jump and restore loop is usable now, but it still reads more as a structured recovery path than as a musically rich scene-performance surface

## Follow-Up

- use this baseline as the first reference point for later Scene Brain workflow regressions
- compare future timing-visibility, scene-energy, or source-comparison improvements against the same fixture and recipe path before broadening the benchmark family

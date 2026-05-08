# Source Timing Generated Probe Matrix

Date: 2026-05-08
Project: `P012 | Source Timing Intelligence`
Status: generated CLI-path QA matrix for source timing readiness

## Purpose

This benchmark records the generated `source_timing_probe --json` matrix used by
`just audio-qa-ci`.

The matrix protects three different timing-readiness boundaries:

- a strong, accented generated break can become `grid locked`
- silence / no evidence must stay degraded and manual-confirmed
- a flat metronomic pulse can expose stable beat evidence without becoming a
  trusted downbeat / phrase authority

This distinction matters because Riotbox must not let TR-909, MC-202, W-30,
bass, Scene Brain, or destructive gestures treat any plausible BPM as a
musically trustworthy grid.

## Commands

```bash
just generated-source-timing-probe-json-smoke
just generated-degraded-source-timing-probe-json-smoke
just generated-ambiguous-source-timing-probe-json-smoke
just audio-qa-ci
```

Each command writes deterministic generated audio into a temporary directory,
runs the real `riotbox-audio` `source_timing_probe --json` CLI, validates the
emitted `riotbox.source_timing_probe_cli.v1` JSON contract, and asserts the
expected timing-readiness boundary.

## Matrix

| Case | Source shape | Expected cue | Readiness | Manual confirm | Beat | Downbeat | Phrase | Key evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Strong generated break | 16s accented 128 BPM synthetic break | `grid locked` | `ready` | no | `stable` | `stable` | `stable` | BPM about `128.397`, beat score about `0.979`, downbeat score about `0.565` |
| Silence | 4s generated silence | `needs confirm` | `unavailable` | yes | `unavailable` | `unavailable` | `unavailable` | no BPM, no beat/downbeat scores, warnings include `low_timing_confidence` and `weak_kick_anchor` |
| Flat pulse | 16s evenly accented 128 BPM pulse | `needs confirm` | `weak` | yes | `stable` | `weak` | `ambiguous_downbeat` | BPM about `128.397`, beat score about `0.977`, downbeat score about `0.268`, alternate downbeat phases `3` |

## Interpretation

The flat-pulse case is the important middle case. It proves that stable beat
interval evidence is not enough to claim a musically locked grid. Downbeat and
phrase evidence must also be strong enough, or the UI / QA path must keep the
result manual-confirmed.

This is one of the safeguards against "ding ding ding" regressions: a generated
lane may be mathematically on a beat interval and still be musically wrong if the
downbeat or phrase authority is ambiguous.

## Boundary

This matrix is deterministic generated-audio QA. It is not a claim that arbitrary
user audio is solved.

Future detector work should expand this matrix with real-source and synthetic
fixture families, but it should not loosen the current safety rule merely to make
more examples report `grid locked`.

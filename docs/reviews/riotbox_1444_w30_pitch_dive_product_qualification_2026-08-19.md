# RIOTBOX-1444 W-30 Pitch Dive Product Qualification

Date: 2026-08-19  
Partition: Development only  
Frozen mechanism: `w30_pitch_dive_v1`  
Decision: RBX-303  
Result: qualified bounded product keep

## Scope

RIOTBOX-1444 rebuilt the unchanged RBX-299 Pitch Dive through the established
product spine as `w30.pitch_dive`. The performer action targets the current
focused promoted W-30 capture, queues on the next bar, persists typed
articulation/capture/start-beat state through Session and replay, remains
visible through observer/UI surfaces, and reaches the exact RuntimeMix W-30
path. Missing or mismatched source-backed material is refused or silent; no
musical fallback exists.

The frozen product contract is
`docs/benchmarks/w30_pitch_dive_product_qualification_v1.json`. Source results
were not allowed to change the playback curve, timeline, fade, source set,
thresholds, or stopping rule.

## Development Access Boundary

A fresh bounded access log existed before the first qualification-source open.
The runner opened only the four exact registered paths and verified their
hashes before and after each case. It did not discover source directories or
open Holdout or commercial-reference audio. The completed local access log is
`artifacts/development/riotbox-1444/access-log-v1.json` with SHA-256
`41875f212ffc99d772584f3f1a39d6c987c95a7963f0ac7020acfa5171261446`.

The representative review used a separate one-source access log at
`artifacts/development/riotbox-1444/review-access-log-v1.json` with SHA-256
`6989f5eb1f08fb7c258cda4428fe6377cbde96810dae3357c20a632daf07dbac`.

## Exact Product Matrix

All cases passed before human playback.

| Case | Product BPM | Final-four-beat delta RMS | Candidate peak | Clips / limited | Boundaries / callback |
| --- | ---: | ---: | ---: | ---: | --- |
| `dense_beat03_130` | `130.284943` | `0.088090` | `0.399897` | `0 / 0` | pass |
| `freesound_alastair_pursloe_183441` | `135.110321` | `0.161271` | `0.396724` | `0 / 0` | pass |
| `freesound_dr_skitz_353853` | `119.680878` | `0.111207` | `0.392098` | `0 / 0` | pass |
| `tonal_rusharp_120` | `120.000000` | `0.141839` | `0.268649` | `0 / 0` | pass |

For every case, the first eight relative beats were sample-exact to control,
the frozen final four beats were materially distinct, relative beat twelve and
later were silent, 128- and 257-frame callback partitions matched exactly, and
pre-limiter clips, limiter interventions, and post-limiter clips were zero.
Capture lineage, W-30 grit, and Source Monitor state remained unchanged; the
missing-source control produced zero active samples. Source-blind automated
tests cover Session JSON round-trip and replay equivalence.

## Formal Product Review

Exactly one representative composite was prepared after the matrix passed:
bounded source context repeated four times, one second of silence, A exact
product control repeated twice, one second of silence, and B exact product
Pitch Dive repeated twice. The stereo 48 kHz PCM16 artifact is `33.216`
seconds, measures `-22.0 LUFS` with `-4.8 dBFS` true peak, has no PCM clips,
and ends with exact silence. Its SHA-256 is
`4fe23c5fbe9ec00cc0b4ae382a2e2be9c2a4d23f8f7876c170e77cbd3c79432e`.

After exact-artifact preflight and fresh readiness, playback reached the
announced endpoint and stopped silently. The musician's verdict matched the
four prior transfer observations: the ordinary W-30 control remained useful,
the Pitch Dive supplied the stronger musical payoff, the hook stayed clear,
and the transformed source remained recognizable. No groove, clarity, timing,
or source-identity objection was reported. The structured local review records
`human_verdict: keep`, `hook_after_two_bars: clear`, and
`source_recognition: source_transformed_but_present`.

## Claim Boundary

This qualifies one performer-owned destructive W-30 transition and preserves
the exact frozen v1 behavior. It does not claim percussive hardness, universal
source quality, the complete all-lane P023 Golden Path, Holdout evidence, demo
readiness, or release readiness. Any mechanism change requires a new version
and Decision-Log entry rather than post-qualification retuning.

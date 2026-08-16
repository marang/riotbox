# RIOTBOX-1440 W-30 Hook-Turnaround Development Exploration

Date: 2026-08-16  
Partition: Development only  
Result: `provisional_keep`  
Holdout access: none  
Commercial-reference access: none

## Narrow Question

Can one W-30-owned sampler articulation remain recognizably source-backed while
changing the hook's rhythm and return clearly enough to be worth triggering
live?

The exploration used the existing source-backed W-30 pad renderer. It did not
add a product action, source analyzer, fallback voice, validation framework, or
new PCM transform. Roland's sampler manuals establish forward, reverse, gate,
and trigger playback as direct sampler articulations. Controlled groove studies
also support a bounded momentary rhythmic contradiction over continuous maximum
complexity. This informed one small hypothesis: preserve the metric anchor,
turn the source around briefly, choke the final beat, and return to the ordinary
hook.

## Bounded Source Access

Only the exact registered Development source `dense_beat03_130` was opened:

- path: `data/test_audio/examples/Beat03_130BPM(Full).wav`
- SHA-256: `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- format: stereo PCM24 at 44.1 kHz
- duration: `3.692313` seconds

The hash matched the existing registered contract. No source directory was
searched. No Holdout or commercial-reference audio was opened.

## Exact Artifact Evidence

The isolated exact RuntimeMix contributor inventory contained only the W-30
preview. TR-909, MC-202, the internal resample tap, and Source Monitor were
silent.

| Artifact | SHA-256 | Peak | RMS | Clipping |
| --- | --- | ---: | ---: | ---: |
| control | `7f79cad897c64d38aa91f5ac79b0cef3b858eef45e6c70a5e06d0d13483b2b8c` | `0.399897` | `0.075777` | `0` |
| candidate v1 | `9d1744875e9178b9fa28c738a826c703ddd8779c8578e1c7e846ab460cd07a75` | `0.399897` | `0.069294` | `0` |

Both artifacts are `5.538458` seconds, stereo PCM16 at 48 kHz. Full-artifact
delta RMS is `0.038530`. The opening four beats and final return are
sample-identical between control and candidate; only the middle four-beat
articulation differs. The candidate anchors its first relative beat, applies a
two-beat reverse pickup with `0.68` step gate, applies a one-beat forward choke
with `0.34` step gate, then returns to the ordinary hook.

The first diagnostic attempt was correctly rejected as digitally silent because
its isolated monitor route selected Source without a source contributor. The
fresh render used the established `Riotbox` control-only monitor route and
produced the evidence above. The rejected bytes were neither played nor used to
choose constants.

## Human Result

After separate bounded playback of the exact control and candidate, the
project musician found the turnaround clearly audible, musically successful,
and worth triggering live. The recognizable hook remained intact. This is a
provisional Development keep, not a product-quality, source-general, demo,
release, or hardness verdict.

## Promotion Boundary

`w30_hook_turnaround_v1` is frozen in
`docs/benchmarks/w30_hook_turnaround_development_v1.json` under RBX-296. The
exploration cannot be promoted directly. Product work must rebuild the behavior
source-blind from the frozen contract, preserve the existing Damage action,
integrate the new performer action through queue/commit, Session/replay,
observer/UI, and exact RuntimeMix, then qualify all three registered
Development sources before a formal structured human review. Qualification may
reject but may not tune v1.


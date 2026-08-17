# RIOTBOX-1440 W-30 Hook-Turnaround Development Qualification

Date: 2026-08-16  
Partition: Development only  
Result: `pass`
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

## Product Qualification

The source-blind rebuild adds the explicit `w30.hook_turnaround` performer
action on `H` through the established queue/commit, Session/replay,
observer/UI, source-backed W-30 projection, and exact RuntimeMix path. The
action is refused before queueing unless transport is running, trusted tempo is
positive and finite, the focused promoted capture matches the rendered pad,
and source-backed pad audio is available. Missing material remains silent with
no fallback. The existing Damage action and pre-gesture grit remain unchanged.

After the source-blind Core, Audio, and App suites passed, one fresh bounded
Development-only `StageAQualificationSession` opened exactly the three frozen
paths. All expected source hashes matched before decode or render. No source
directory, Holdout audio, or commercial reference was opened. The completed
access log is local at
`artifacts/audio_qa/riotbox-1440/w30-hook-turnaround-v1/access-log.json`
(SHA-256
`9ce23f1dda1433bac78f17786f3196c35a7331c12ecea301ea100c3cb632a4c6`).

| Case | Reverse delta RMS | Choke delta RMS | Peak | Clips / limited | Boundaries |
| --- | ---: | ---: | ---: | ---: | --- |
| `dense_beat03_130` | `0.067343` | `0.048255` | `0.399897` | `0 / 0` | sample-exact |
| `tonal_rusharp_120` | `0.137809` | `0.078858` | `0.268649` | `0 / 0` | sample-exact |
| `sparse_kicksnr_120` | `0.096941` | `0.028423` | `0.369202` | `0 / 0` | sample-exact |

Every case preserved the first relative beat and ordinary return from relative
beat four sample-exactly, differed from control in both frozen articulation
windows, produced identical output with 128- and 257-frame callback
partitions, preserved capture lineage/grit/Source Monitor, and produced zero
active samples in the missing-source control. Session JSON round-trip and
replay equivalence passed in source-blind automated tests.

Exactly one formal review-ready generation was produced from the dense product
path. Both artifacts are stereo PCM16 at 48 kHz, `5.538458` seconds, and contain
four control pre-roll beats, the four-beat gesture span, and four ordinary
return beats. Pre-roll and return are sample-identical; only the intended
middle differs.

| Review role | Artifact | SHA-256 | Peak | RMS |
| --- | --- | --- | ---: | ---: |
| A: product control | `07_review_A_control.wav` | `cf5c892b71cc9f32e29f572aa79d187506be30f8ed94791b627ffbdbcd6101bd` | `0.399897` | `0.068574` |
| B: `w30_hook_turnaround_v1` | `08_review_B_candidate_v1.wav` | `da0cbb1cc1cde2c14cd298e272d94881ec6a0073dcec070fa263a9e9d8ebd4b1` | `0.399897` | `0.064287` |

## Formal Human Result

After fresh readiness, A and B were played separately once and each stopped at
its declared `5.538458`-second endpoint with silence verified. The project
musician affirmed that the turnaround is a good, musically acceptable effect;
the combined structured response also affirmed clear usefulness, retained hook
identity, clean return, and willingness to trigger it live. No click, timing
damage, source loss, pasted-on quality, or other objection was reported. This
is a qualified positive product pass rather than a claim that the gesture alone
makes a beat harder, completes the Golden Path, or establishes demo/release
readiness.

# RIOTBOX-1473 W-30 Silence Cut Product Qualification

Date: 2026-08-26
Partition: Development only
Frozen mechanism: `w30_choke_silence_cut_v1`
Decisions: RBX-342, RBX-343, RBX-344
Result: `technically_ok_but_musically_weak`; product v1 removed

## Scope

RIOTBOX-1473 rebuilt the unchanged RIOTBOX-1472 mechanism source-blind through
the existing product spine as typed `w30.silence_cut`: performer queue and
next-bar commit, persisted Session/replay articulation, observer/UI surface,
and the isolated W-30 RuntimeMix path. Implementation commit `e547bd79` binds
that complete pre-access state.

The frozen product qualification contract is
`docs/benchmarks/w30_silence_cut_product_qualification_v1.json`, SHA-256
`925b483f91bdf87dcf4e04ec35540a5860c515877dba661ee4546bf87ca0df6c`.
It fixed the source set, 5 ms tapers, one-beat silence, thresholds, review
budget, and stopping rule before qualification-source access. Source and
listening results were not allowed to tune v1.

## Development Access Boundary

One fresh bounded access log existed before the first qualification-source
open. The runner opened only the four exact registered Development paths and
verified their hashes before and after each case. It did not discover source
directories or open Holdout or commercial-reference audio. The completed log
is `artifacts/development/riotbox-1473/access-log-2026-08-26-a.json`, SHA-256
`eb1e08f4cea485b1de0abe85bca514763ce750a8a52e4eaa3b637033957b2274`.

The representative source-first presentation used a separate bounded review
log at
`artifacts/development/riotbox-1473/review-access-log-2026-08-26-a.json`,
SHA-256
`70f5044c890cd6e2dae51fc690f97560d5f0c03f902268259fdcf83f929e2955`.
It records one exact representative source, no directory discovery, no
Holdout, and no commercial reference.

## Exact Product Matrix

All four cases passed before human playback.

| Case | Product BPM | Effect-window delta RMS | Candidate peak | Clips / limited | Zero / return / callback |
| --- | ---: | ---: | ---: | ---: | --- |
| `dense_beat03_130` | `130.0` | `0.075168` | `0.399897` | `0 / 0` | pass |
| `freesound_alastair_pursloe_183441` | `135.0` | `0.114483` | `0.396724` | `0 / 0` | pass |
| `freesound_dr_skitz_353853` | `120.0` | `0.083675` | `0.392098` | `0 / 0` | pass |
| `tonal_rusharp_120` | `120.0` | `0.102395` | `0.268649` | `0 / 0` | pass |

Every case preserved ordinary playback sample-exactly before the 5 ms
fade-out, emitted exact PCM zero for one beat, and returned to the continuously
advancing control sample-exactly after the 5 ms fade-in. The 128- and 257-frame
callback outputs matched exactly. Pre-limiter clips, limiter interventions,
and post-limiter clips were zero. Capture lineage, W-30 grit, music-bus level,
Source Monitor, MC-202, and TR-909 state remained unchanged. Missing-source
controls emitted zero active samples.

## Formal Product Review

After the complete matrix passed, one representative artifact presented
bounded source context twice, one second of silence, A exact product control
twice, one second of silence, and B exact product Silence Cut twice. Its
SHA-256 is
`504a0514bcab8fd3feb42a851db35045b38ced70e563960ae003853c24ff3538`.
It is stereo 48 kHz PCM16, `16.709208` seconds, `-22.0 LUFS`, and
`-4.8 dBTP`, with no full-scale PCM samples and a 50 ms terminal fade.

The exact A and B components are each `2.769229` seconds. Their SHA-256 values
are `678bca448d80a5edd5e402322dae3e0fc807d96241042f799ae0ed8e8c87002c`
and `f2cc97169c8ac200e7644a48aeea5f331d45201d7294d4a881f547e442ce6977`.
Both peak at `0.399872` (`-8.0 dBTP`). Their waveform correlation is
`0.896972`; they differ only from `1.841229` through `2.312646` seconds.
B contains exact silence from `1.846167` through `2.307729` seconds and adds
no spectral content: the delta is the broadband control material removed by
the fixed cut.

After exact-artifact preflight and fresh readiness, playback reached the
announced endpoint and stopped silently. The musician judged control A good
but candidate B weaker because the fixed silence makes the phrase feel uneven.
This is normalized as `technically_ok_but_musically_weak`: the failure is
musical continuity, not audibility, clipping, source loss, callback drift, or
an accidental return defect. Structured review SHA-256
`9d285e9830c2a245ada12dac5541b1e59e8b4616e89aad804f460a821fe360cb`
binds the artifact and verdict.

## Outcome And Claim Boundary

RBX-344 removes the unqualified Silence Cut v1 action, Session/replay profile,
observer/UI surface, RuntimeMix seam, product tests, qualification renderer,
and executable runner from the final tree. The ordinary W-30 control A and all
previously qualified W-30 gestures remain unchanged. The frozen contracts,
matrix identities, and negative review remain as evidence; v1 is not replayed
or retuned.

A future successor requires a separate Linear-first version and a materially
different phrase-aware causal contract frozen before fresh source access. This
result grants no product keep, source-general, Holdout, percussive-hardness,
automatic-arrangement, demo, release, universal-quality, or P023-completion
claim.

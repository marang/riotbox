# RIOTBOX-1428 H31 Stage A Rejected Experiment

Date: 2026-08-02
Work class: `audible_vertical_slice` experiment
Product status: rejected; no runtime integration

## Intended Claim

H31 Stage A attempted to reconstruct source impact events and establish a
pitch-stable, source-recognizable `percussive_hard` candidate before any live
W-30 takeover work. The candidate had to preserve event timing and become
unambiguously more forceful, not merely different.

## Stage A v1 Human Rejection

The exact Beat03 review artifact was a `4.926848 s` loudness-matched A/B with
eight sample-exact source events, a `300 ms` gap, and eight prepared events.
It measured `-16.2 LUFS` integrated with `-1.0 dBFS` true peak. Timing and the
three-source development checks passed mechanically.

Human verdict: `reject`. The source half sounded higher and the prepared half
lower; timing stayed aligned, but nothing sounded more forceful or harder. The
recipe's approximately `0.78x` body playback rate therefore changed pitch
instead of proving percussive force. The recipe was frozen immediately.

Review evidence remains local at:

`artifacts/audio_qa/local/listening-reviews/RIOTBOX-1428-H31-STAGE-A/review.json`

The reviewed local artifacts are identity-bound as follows:

- source train SHA-256:
  `ee0d6f02bfec431ca532c4d0611787bb0398ddc2d4eaae96aa5152ba4f85e801`
- prepared train SHA-256:
  `af70724d5074969dceebf44b6be2aaf949e39d19e7372cea9f22b34e17677ace`
- source-then-prepared review WAV SHA-256:
  `788f801b47f14d8b2d600c8d594c991738e09da01656f04f420ebca5ba36b7b3`
- structured review SHA-256:
  `1ec58f7b8388bdb8d67f8e727c4b53db6a9a607facfd58fce7fa0d0258adf68b`
- metrics SHA-256:
  `542bb14b93580dcec90d1ab3d90ae5e6af2b5d1304c79eb9997ee4a88e33bfaf`

These ignored local files are not committed or artifact-complete release
evidence. The hashes bind the historical observation but do not make it a
guaranteed executable backtest.

## Stage A v2 Mechanical Rejection

The second experiment removed resampling, kept the source cursor at `1.0x`,
and targeted a larger attack while retaining the body. It was rejected before
human playback because the three-source gate showed near-identity and/or body
loss rather than a credible new hardness mechanism:

| Source | Correlation | RMS delta | Attack ratio | Body ratio | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| Beat03 | 0.9745 | 0.226 | 1.144 | 0.810 | insufficient attack; body loss |
| Fat groove | 0.9839 | 0.180 | 1.243 | 0.880 | too similar |
| Prehistoric drums | 0.9712 | 0.240 | 1.232 | 0.872 | body loss |

No repeated listening was requested because the sound recipe had not crossed
its pre-registered technical gate.

## Closeout Decision

- Remove the diagnostic Rust renderer and compiler from the branch.
- Do not merge or promote either recipe into a product output path.
- Preserve the semantic hardness contract and the negative evidence.
- Move RIOTBOX-1428 out of active implementation until a dedicated research
  prerequisite defines a defensible perceptual model, experiment design,
  falsifiable algorithm hypotheses, and validation protocol for hard versus
  soft beats.
- Resume RIOTBOX-1428 Stage A to implement and falsify those hypotheses; only
  its implementation may produce a mechanism materially different from
  playback-rate, loudness, darkness, dirt, or fixed-float tuning.

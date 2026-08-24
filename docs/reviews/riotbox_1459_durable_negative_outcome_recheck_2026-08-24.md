# RIOTBOX-1459 Durable Negative-Outcome Recheck

- Date: 2026-08-24
- Source families: `weak_source`, `bad_timing`
- Exact product outcome: `degraded / needs_user_confirmation`
- Human product-handling verdict: `needs_fix`
- Demo readiness: `not_demo_ready`
- Promotion result: not promoted

## Scope And Access Boundary

This recheck used one exact registered Development source for both formal
families: `data/test_audio/examples/Beat20_128BPM(Full).wav`, SHA-256
`d3d86134e99dfb5889c9efe683ccd427cdf73e499ebfbc69dbd1f3a145bdf1e1`.
The corpus entries are `weak_source_beat20_128` and
`bad_timing_beat20_128`. Before opening the source, the ticket compared case
ID, exact path, and known source hash with the active Holdout metadata in the
frozen v2 and v3 rotation registries. No collision was found. No Holdout audio,
commercial reference, or source directory was opened or discovered.

The exact access log is
`artifacts/development/riotbox-1459/access-log-20260824T131216Z.json`,
SHA-256
`33935f5985d9cabf3633f28eda3428e3f84e6e97542d60bd5141d8e2850f3949`.

## Exact Product Proof

The live assignment supplied neither manual BPM nor manual downbeat. It
reported `degraded / needs_user_confirmation`, displayed
`ambiguous_downbeat`, and offered `listen first` before grid confirmation. The
transport remained stopped across 402 callbacks. TR-909, MC-202, and W-30
remained idle, no generated output or fallback music was configured, and the
queue and Session action log remained empty.

The bound artifacts are:

- Source Graph: SHA-256
  `56a6fa6f6d54d775a0b4afef95d0e8024b528993571f99a301c3467dc193d50f`
- Session: SHA-256
  `bd36b67c0e40eea3cb6c7f15be09c502db1c62623a92dd49086ef2407f5bb518`
- Observer: SHA-256
  `41c835168e4566561dfb1e5e21f120b30289f846914c59e1121e23e208432450`

A fresh process loaded a byte-identical Session with the explicit external
Source Graph. It preserved the same degraded state and reason across 594
callbacks, again with stopped transport, source-only routing, idle generated
lanes, no fallback, and no actions. The restart observer has SHA-256
`b68249c2e4a68894135a7c48049ddad2258218f9afa13029a886f9b3d2396572`.

An initial session-only restart without the required external graph argument
failed before observer creation or audio-runtime startup. The corrected run
supplied the exact graph already referenced by the Session; no product state or
audio evidence was produced by the failed attempt.

## Human Review And Independent Analysis

The exact source is 3.75 seconds of stereo 44.1 kHz 24-bit PCM. It was played
source-only in two bounded two-pass presentations, for 15 seconds total. No
generated Riotbox output was played, and endpoint silence was verified after
both presentations.

The reviewer perceived the file boundary as the musical downbeat and asked why
the system could not determine that itself. Offline inspection supports the
concern without granting a new timing claim:

- 3.75 seconds at the registered 128 BPM is exactly eight beats, matching a
  two-bar 4/4 container, although duration alone cannot prove phase;
- the probe estimated 128.39674 BPM but selected phase 3, placing its first bar
  at approximately 0.935 seconds;
- phase 1 at 0.0 seconds remained only an alternate hypothesis;
- all four downbeat phases remained present, and the selected phase score was
  too weak to earn a locked grid under the existing contract;
- a bounded energy/accent inspection found the strongest repeating accents on
  phase 4 and therefore did not independently justify the detector-selected
  phase 3. This simple inspection is diagnostic, not a replacement detector.

The existing RBX decision for Beat20 deliberately keeps it manual-confirm-only
until stronger musical evidence exists. The cautious block therefore remains
safe, but the human review did not accept `ambiguous_downbeat` as a sufficiently
accurate explanation of this source. Both structured records are
`needs_fix`, not reconstructed passes:

- `weak_source`: SHA-256
  `222886269f46f79b764ca5ceb363540aaa22df12bb1b8b24ce50b6c3b13a1793`
- `bad_timing`: SHA-256
  `ff1145ecb71ce01f679e833c788e6ca3572bb15c8dbc1344a7efb7a8c4e208ad`

The single human review was not duplicated; the two records bind the same
reviewed product state to distinct formal family contracts.

## Outcome

Neither record is eligible for negative-family promotion because the product
handling did not earn a human pass. `weak_source` and `bad_timing` therefore
remain unresolved in live aggregate readiness. No demo-ready, source-general,
quality, release, or Holdout claim is made.

The live source-family coverage recheck has SHA-256
`6fe9e9b1524bbe686c7ab0701117b1841b2937b727c5fcfe52d32a9d6c3ba45a`.
Both families remain `missing_candidate`. The aggregate readiness report has
SHA-256
`2053c4248ee2f41b36b1b437abef17d7b97d83b760458298d5102b8349885667`,
keeps `release_readiness: blocked`, and keeps `quality_claim_allowed: false`.
The hashes match the pre-review live state because the rejected records were
correctly not added to the live bank.

The smallest next product step is the existing RIOTBOX-1033 detector-quality
issue: test stronger musical downbeat evidence on Beat20 and varied registered
Development sources while preserving manual-confirm fallback. Any adopted
loop-boundary or phase-selection rule requires a versioned algorithm/contract
and a new Decision Log entry; this review does not change either.

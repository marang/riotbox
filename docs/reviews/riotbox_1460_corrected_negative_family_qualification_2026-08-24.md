# RIOTBOX-1460 Corrected Negative-Family Qualification

- Date: 2026-08-24
- Source families: `weak_source`, `bad_timing`
- Exact product outcome: `degraded / needs_user_confirmation`
- Human product-handling verdict: `pass`
- Demo readiness: `not_demo_ready`
- Holdout access: none
- Audio playback in this ticket: none

## Scope And Access Boundary

This qualification rebuilt the two negative-family records from the corrected
current product state; it did not modify or promote the rejected RIOTBOX-1459
records. Both families use the exact registered Development source
`data/test_audio/examples/Beat20_128BPM(Full).wav`, SHA-256
`d3d86134e99dfb5889c9efe683ccd427cdf73e499ebfbc69dbd1f3a145bdf1e1`.

Before opening that exact path, the ticket compared case ID, source path, and
expected SHA-256 against the active v2 and v3 Holdout metadata. No collision
was found. No Holdout audio, commercial reference, source-directory discovery,
candidate render, or fallback source was accessed. The bounded local access log
is
`artifacts/development/riotbox-1460/access-log-20260824T152224Z.json`, SHA-256
`7bed73e36432e3c4258fc9e3b2af6aa35a60ed896fd2485ab5b8119586b38e9f`.

## Exact Product Proof

The live Jam ingest supplied neither manual BPM nor manual downbeat. The frozen
`source-timing-probe.repeated-loop-boundary-prior.v1` cue suggests phase zero,
placing bar one at the file boundary, while retaining all three alternative
phases. The product remains `degraded / needs_user_confirmation`, exposes
`ambiguous_downbeat` plus `listen first`, and requires explicit grid
confirmation before confident bar-locked output.

Across every assigned observer snapshot:

- transport stayed stopped and the real audio callback ran;
- the source monitor remained `source_preview_only` / source-only;
- TR-909, MC-202, and W-30 generated lanes remained unconfigured;
- no generated output or fallback music was present;
- queue history and Session action history remained empty;
- automatic grid confirmation and confident bar-locked output remained false.

The bound live artifacts are:

- Source Graph SHA-256:
  `81050d0a99a811bdee9b67e8efab2706d0369fbedd3667bd0c632221f87bdac0`
- Session SHA-256:
  `c43f95ea8d068c808fede4eb9c10b5a0b292165677ba4f15e7464517a88c1970`
- Observer SHA-256:
  `de58b8934a15c5316fe4048bc4f329de35d2227c294bf644085a1a4146862e13`

A fresh process loaded the exact Session and external Source Graph. It
preserved phase zero, three alternatives, manual confirmation, stopped
transport, empty action history, source-only monitoring, idle generated lanes,
and absent fallback across 1,166 callbacks. The restart observer SHA-256 is
`d4a644122e79f81fdc439e52303de1305359bc7209ae0e73e4d7b8a133c78067`;
the Session remained byte-identical.

## Human Product Review

No audio was replayed. The source bytes and audible contributors are unchanged
from the exact source-only RIOTBOX-1459 review, and RIOTBOX-1033 already bound
the normalized timing observation that the file boundary is probably the
musical downbeat. Repeating the source would not test the current question,
which is whether the corrected product handling is understandable and safe.

After the exact current state was stated neutrally, the reviewer accepted the
phase-zero suggestion, retained ambiguity, visible degraded state, and explicit
confirmation action as understandable and appropriate. Distinct hash-bound
records preserve the two formal family contracts:

- `weak_source` review SHA-256:
  `9ee185df4a0eefb21eda5621471f3c17170266de329d10c6fd4711fcdea86daf`
- `bad_timing` review SHA-256:
  `c757ddb766e58576622841d87b5d6c27b6d398f4af5131ff7d3c819d7fee4c6a`

This is a pass for honest product handling, not for generated music or source
quality.

## Promotion And Remaining Boundary

The validated live bank at
`artifacts/audio_qa/local/live-review/RIOTBOX-1460/demo-bank.json`, SHA-256
`b4462f5de4be8a45fa5c7712afd627e4fe51cdb0ba149b4e7b510212957de3a8`,
contains the existing reviewed `pad_noise` unavailable outcome plus the fresh
`weak_source` and `bad_timing` degraded outcomes.

The live source-family report, SHA-256
`27c4d5852920c8765f6d103e9e4211ed58bcef36ef5f3ec3d91c39c3db8da875`,
counts all three as `reviewed_degraded_or_reject`. Only the positive musical
families `dense_break`, `sparse_drums`, and `tonal_riff` remain without their
required family success. The aggregate readiness report, SHA-256
`ddee298328951a25a51b344f9e32678405b1d34e1c589c9ab56c468e79647c55`,
removes the resolved negative edge blockers but keeps
`release_readiness: blocked` and `quality_claim_allowed: false`.

No Decision Log entry is required. This ticket applies the frozen RIOTBOX-1033
timing contract and the existing negative-family review contract without
changing an algorithm, threshold, access boundary, schema, or product policy.

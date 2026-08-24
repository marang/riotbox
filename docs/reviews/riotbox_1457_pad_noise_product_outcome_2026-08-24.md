# RIOTBOX-1457 Pad/Noise Product Outcome

- Date: 2026-08-24
- Source family: `pad_noise`
- Exact product outcome: `unavailable`
- Human product-handling verdict: `pass`
- Demo readiness: `not_demo_ready`
- Overall release readiness: `blocked`

## Scope And Access Boundary

This review covers one exact live `riotbox-app` assignment for the registered
Development case `pad_noise_fadapad_120`. The source was opened only through
its exact corpus path,
`data/test_audio/examples/DH_Fadapad_120_A.wav`, with SHA-256
`22e825c8bf59cfd71a02ce229d222d1d35f1bae6f7d1bafab6edeb4ff4829d8c`.
No source directory discovery, Holdout-audio access, or commercial-reference
access occurred. Holdout registry metadata was checked before opening the
source and showed no case-id, path, or hash collision.

The run supplied neither manual BPM nor manual downbeat. This prevents the
review from manufacturing trusted timing for a soft-attack pad merely because
its filename carries a tempo hint.

## Exact Product Proof

The source graph, Session, and observer artifacts are bound by these hashes:

- Source Graph:
  `1d46342b6498f52cf85fcea5c6534fe035e20d7aa37c295875b347ef93db7237`
- Session:
  `a669f70c0f687d60f9729002b32906d53172d3842e1ba3ceca56027a4299716f`
- Observer:
  `cb90919c5e451ea0da8137c696ad667a05d763a1218c1a16de839111163ddc76`

The live product displayed `unavailable | bar/live?`, explained the state as
`sparse onsets`, and offered source preview while stating that timing was
unavailable. Across 3,102 audio callbacks, transport stayed stopped, the queue
and Session action log stayed empty, no grid was confirmed, and TR-909,
MC-202, and W-30 remained unconfigured and silent. No generated output, live
source policy, or fallback music was present.

A fresh process then loaded a byte-identical copy of the Session without a
`--source`, manual BPM, or manual downbeat argument. The restored Session kept
SHA-256
`a669f70c0f687d60f9729002b32906d53172d3842e1ba3ceca56027a4299716f`.
Its observer, SHA-256
`4d9deec505322c9637c3abdb32d033044e50dbdbe4e00747c418adbe24a19d0b`,
again reported `unavailable / sparse_onsets`, stopped transport, zero queued or
committed actions, no confirmed grid, and silent idle generated lanes across
325 callbacks. The unavailable decision therefore survives process restart
without inventing replay state or fallback output.

Because the existing timing contract selected an unavailable outcome, no
candidate render was created. No audio was played to the reviewer. The review
therefore assesses the visible risk state, reason, and next safe action rather
than musical sound quality.

## Human Product Verdict

The exact unavailable state, sparse-onset reason, and source-preview path were
accepted as clear and correct product behavior. The structured review is
`artifacts/development/riotbox-1457/review-20260824T103610Z/review.json`,
SHA-256
`e4d96803ec79148dfde326088b2ee0f4fb8ee35b2aa180e6d871376b72fb20bc`.
Its prompt SHA-256 is
`d270432ca431c6301d96f449ca4548ed421bf6fb747b4eb94e3b019305d8ba4a`.

This is a human pass for honest product handling. The promoted bank entry uses
`human_verdict: fail` only in the demo/music sense: there is no demo-ready
musical candidate. It does not contradict the successful product-handling
review.

## Coverage And Contract Alignment

The demo-bank contract already allowed `pad_noise` to succeed by either a
demo-ready human pass or a reviewed degraded, unavailable, or reject outcome.
The coverage and aggregate-readiness implementations previously modeled it as
positive-demo-only. RIOTBOX-1457 aligns those consumers with the existing
contract and recognizes canonical `pad_noise` entries while retaining
`tonal_pad` as a compatibility alias.

The validated live bank at
`artifacts/audio_qa/local/live-review/RIOTBOX-1457/demo-bank.json` has SHA-256
`f058b229ce2d501d3c8aa11b0672b0229e6a454a7395de7f171d4652be1fd331`.
Its only entry is the hash-bound Fadapad unavailable review. The resulting
source-family coverage report has SHA-256
`6fe9e9b1524bbe686c7ab0701117b1841b2937b727c5fcfe52d32a9d6c3ba45a`
and reports `pad_noise: reviewed_degraded_or_reject`.

The aggregate sound-quality readiness report has SHA-256
`c4b5c7358bc5b4f43f150acdb2977596bb040c36cf1054723dfcdfb3a523f116`.
It remains blocked, permits no quality claim, and leaves dense-break,
sparse-drums, tonal-riff, weak-source, and bad-timing formal evidence open.
RIOTBOX-1457 closes only the pad/noise product-outcome gap.

An older RIOTBOX-1405 local bank still references expired temporary review
paths and therefore cannot pass the current full bank validator. It was not
silently repaired or promoted. The degraded-review promotion helper now
validates the complete candidate bank before writing, so invalid legacy entries
fail closed without overwriting the input bank.

## Validation

- Exact observer validation: pass.
- Session-only process restart: byte-identical Session and matching unavailable
  no-fallback state.
- Structured degraded-product review with required human pass: pass.
- Release-grade live demo-bank validation: pass.
- Live source-family coverage validation: pass; `pad_noise` satisfied through
  reviewed unavailable handling.
- Aggregate live sound-quality readiness validation: pass; release readiness
  remains blocked and `quality_claim_allowed` remains false.
- Canonical `pad_noise` and legacy `tonal_pad` fixture paths: pass for both
  demo-ready and reviewed-unavailable outcomes.
- Invalid existing-bank promotion: fails before write.

No Decision Log entry is required because this change implements the existing
dual-path pad/noise contract rather than changing an algorithm, threshold,
access boundary, or product decision.

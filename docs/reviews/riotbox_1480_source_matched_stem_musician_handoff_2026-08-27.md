# RIOTBOX-1480 Source-Matched Stem Musician Handoff Review — 2026-08-27

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1480
- Frozen qualification contract:
  `riotbox.source_matched_stem_musician_handoff_qualification.v1`, raw SHA-256
  `f158692da62e434e882fd802dab2bb750f9a410c75810f25062163ca6e0add42`
- Exact registered Development case: `dense_beat03_130`, source SHA-256
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Claim boundary: one exact V2 producer handoff, committed source-matched
  Action/Session package, and bounded human assessment of the exact source,
  full mix, drums, music, and bass artifacts
- Explicit non-goals: renderer or threshold tuning, source-general quality,
  transfer-source access without an initial keep, DAW placement, TUI/Ghost
  musician controls, live recording, Holdout evidence, commercial-reference
  evidence, release readiness, or hardness proof

## Access And Fail-Closed Recovery

The fresh access session opened the registered Development source exactly once
through the no-follow bounded accessor. It performed no directory discovery and
opened no Holdout or commercial-reference audio. The verified bytes reached the
in-process owner and produced a valid V2 bundle, but the owner transaction then
aborted when the Rust Session ingress rejected the Python proof's serialized
reconstruction-tolerance decimals. The access log remains honestly `aborted`;
it was not rewritten into a successful state. Its raw SHA-256 is
`6ef55766d9281409ad98d225037fc60a749b44ed703d5817ba946836f4898515`.

The failure exposed a real cross-language contract defect. Python published the
unchanged exact limits `3 / 32768` and `1.5 / 32768` as `0.000091552734` and
`0.000045776367`; Rust required binary equality within `f64::EPSILON`, although
the existing Python V2 validator already used `1e-12` for contract-field
comparison. RBX-353 aligns only that declared-field comparison with `1e-12`.
The exact limits, PCM measurement, reconstruction rule, renderer, and all
musical thresholds remain unchanged. Focused Rust tests accept the actual
published precision and reject material tolerance drift.

Recovery reused only the already-created private owner copy after revalidating
its exact source SHA-256 and PCM24 contract. It did not reopen the original
source or rerun the producer. The recovery record raw SHA-256 is
`807fa6a8989cd6a96831dff7aa078d4cc2cc7935e7f0526acd45889e009fc5bb`.

## Technical Qualification

The final local qualification result is `pass`, raw SHA-256
`5b8a85672126cbf9fcf4d7978772eb9c8a44415fce9b67b6311eb71559477fc7`.
The exact `riotbox.product_stem_handoff.v2` proof has SHA-256
`9e393c0577557176dfa5efe39e5eaaa868e696813a9735c9c96ddeb906e4b1a1`.
It binds the registered source, 44.1 kHz stereo PCM16 output grid, four bars,
and the unchanged `pcm_sum_v1` rule. Maximum reconstruction error is
`0.000030577183` against `0.000091552734`; RMS reconstruction error is
`0.0000106856205` against `0.000045776367`.

The Session ingress completed `export.stem_package` action `13`, committed
receipt `export-receipt-a-0013`, and wrote the observer's completed lifecycle.
The receipt is correctly `reproducible`, has no unsupported scopes, and passes
all five gates: artifact set, per-stem hash stability, non-silence, lineage, and
fallback comparison. Drums, music, and bass package bytes are hash-identical to
their V2 inputs. The package manifest and proof SHA-256 values are
`ab477d52452bcc486154a6812f102f5d052175ad14491b69c254a3de69c5a37a`
and `250d665506aace770d72bc6d2cf668b4d0e3d16636c0b6354036df69aae2da3b`.

Before playback, the exact source, full mix, drums, music, and bass files were
bound by path and SHA-256, decoded, checked for non-silence, assigned to their
declared roles, and confirmed to end at their announced boundaries. The source
is 3.692 seconds; every rendered artifact is 7.368 seconds. Playback followed
the frozen source, full mix, drums, music, bass order with one-second pauses and
ended in verified silence.

## Human Review

The structured review JSON has SHA-256
`03caee3d07dd0532d9e0b8ed8355edf2abcb7f207c373c3269a0356e813afdda`
and records `human_verdict: inconclusive`.

The listener found the exact artifacts acceptable and not obviously broken,
but would not use the presentation directly as a finished loop and could not
judge stem reuse without a concrete multitrack workflow. This is neither a
rejection of the files nor a musician-usability pass. The review therefore
does not remove the structured-listening blocker and cannot authorize the
predeclared tonal or sparse transfer sources.

## Validation

- Core product-stem handoff tests: 5/5 passed
- App source-matched ingress tests: 10/10 passed
- Python V2 contract mutation fixtures: 13/13 passed
- Tracked JSON validation and Decision Log fixture checks: passed
- Full source-free `just ci`: passed after the review fix

## Consequence

RIOTBOX-1480 proves that one real registered Development source can traverse
the corrected V2 producer-to-Session stem path with reproducible, reconstructing,
non-silent, source-bound artifacts. It also proves that isolated-stem playback
alone is not an adequate musician-facing usability test for this listener.

The next bounded product step is a concrete multitrack placement and mix
context using these unchanged exact stems: demonstrate what muting, balancing,
or arranging the three roles enables, then repeat a structured review. Do not
open another source, tune the renderer, or build broader TUI/Ghost surface work
until that context establishes whether the handoff is genuinely useful.

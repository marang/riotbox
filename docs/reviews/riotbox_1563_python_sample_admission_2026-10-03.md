# Python decoded-sample admission — RIOTBOX-1563

Date: 2026-10-03. Comparison: `abdb8492d3a5282313d28f8de6ba4fe4e0b1212f`
(RIOTBOX-1562, PR #1665) to this branch. P017 maintenance/regression protecting
the accepted P023 source-ingest path, not a new audible mechanism.

## Diagnosis and contract

Rust V1 already admits at most 67108864 interleaved input samples, but App runs
that gate after Python's provider response. A valid mono PCM16 input containing
67108865 samples occupies 134217774 encoded bytes including the ordinary header:
below the existing 268435456-byte encoded cap, yet above the sample cap. Python
previously reached `readframes`, conversion and features before any such check.

The default-limit witness was a metadata-only seam, not a materialized large WAV
or measured out-of-memory event. A second RED used a real tiny stereo WAV and
an injected four-sample test budget: six interleaved samples reached the
`readframes` assertion instead of returning a resource error. These deterministic
probes locate the missing admission boundary; no history bisection, large-memory
experiment or real source inspection was necessary.

RBX-433 / Source Graph §6 freeze the same existing count before implementation.
After complete container/format/whole-frame admission, `source_wave` now requires
`frame_count * channel_count <= 67108864` before materializing PCM or converting
to mono. Counting mono frames instead would undercount multichannel input.

A small pure `source_limits` leaf owns both frozen constants, typed resource
units and their common error. File admission and pure decoding remain separate
consumers; the decoder does not import filesystem machinery. The existing
request-bound, non-retryable `source_resource_limit` envelope handles both
policies. Encoded error wording, supported PCM conversion/averaging, empty and
exact-limit admission, feature algorithms, byte identity and Rust decoding remain
unchanged. Test injection is not a product option or environment override.

## Verification

- RED: the tiny stereo case reaches `PCM payload read before admission` on the
  prior decoder. The early count check makes that exact case green.
- All 23 Python tests pass. New tests cover widths 1/2/3/4, mono/stereo,
  empty/below/exact/over limits, typed units/counts, default numeric limits,
  malformed geometry precedence, and no frame/conversion/feature work on reject.
- The default-limit test mocks only metadata/layout and asserts no `readframes`
  call. Tiny real WAVs test complete layout and the actual parser. Neither is
  represented as a default-sized end-to-end allocation test.
- A real Python process with a tiny injected budget answers rejection, ping and
  exact-limit valid analysis in one synchronized stream.
- Full Rust Sidecar suite passes: 32 unit and four integration tests. The new
  public-client test independently exercises the same error → ping → exact-limit
  analysis sequence, request correlation, units/counts and non-retryable status.
- Five App source-ingest identity tests pass. The new tiny-budget test proves
  rejection before overwriting or creating Session/Graph destinations in both
  embedded/external modes: old bytes remain identical, new directories remain
  absent, and old Session/Graph/cache restore unchanged. This validates resource
  error propagation and publication ordering, not large-file memory behavior.
- `cargo fmt --all` and `git diff --check` pass.
- Full source-free local `just ci` passes: workspace tests, Python/contracts,
  generated-audio smokes and strict Clippy. Native exact-head CI remains the
  merge gate.

Logs: `/tmp/riotbox-1563-red.log`, `/tmp/riotbox-1563-green.log`,
`/tmp/riotbox-1563-python.log`, `/tmp/riotbox-1563-sidecar.log`,
`/tmp/riotbox-1563-app.log`, `/tmp/riotbox-1563-ci.log`.

## Review and ownership

Independent Adversarial Implementation Reviewer retained no P0–P3 findings.
Executed all 23 Python tests, 64 tiny boundary controls, three malformed-input
precedence controls and diff check; inspected the Rust/App deltas and the
coordinator's passing logs, without independently executing those Rust tests.
Verified interleaved units, error precedence, early admission, no public override
and the pure shared-policy dependency direction.

Independent Spec and Evidence Auditor retained no P0–P3 findings in a separate
branch pass. Reran all 23 Python tests and diff check; inspected current Rust/App
tests and coordinator logs, RBX-433/spec agreement, encoded-wording compatibility
and both reports' attribution/limits. The current-state Core/Audio/App auditor
also checked the combined checkpoint record and confirmed its static-only
finding, provenance and scope were represented accurately.

Coordinator self-review covers intent, explicit resource/error ownership,
same-buffer hash/decode, tests, unchanged encoded semantics and scope of claims.
No new ActionCommand, Core/Session/replay schema, persistence model, dependency,
runtime or DSP owner is introduced. Rust changes are only regressions and a small
generated WAV fixture generalization; no mechanical module sharding is needed.

The separate fifth-slice current-state checkpoint is
[recorded here](riotbox_1563_current_state_cadence_2026-10-03.md). Its pre-existing
local-writer receipt mismatch is tracked separately as RIOTBOX-1564, not hidden
inside this source-admission patch or presented as a new regression.

## Limits

Encoded reading/hashing already occur before this gate. The policy bounds input
sample count, not Python object overhead, temporary arrays, aggregate cache/heap,
process RSS, analysis time or scheduling. Admitted large sources can still be
expensive; no universal memory/deadline guarantee is claimed.

No real Development/Holdout/commercial/capture audio, source-directory discovery,
device/DAW execution or human playback. Generated tests are not musical,
source-general, hardness, endurance, host or release qualification. TUI and
Windows expansion remain out of scope; existing CI jobs are unchanged.

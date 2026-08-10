# RIOTBOX-1430 Stage-A-v2 Acquisition Batch-v2 Pre-GET Gate — 2026-08-10

## Verdict

`pass_for_one_exact_registered_development_acquisition_after_pre_GET_commit`

This is a source-blind `contract_enabler` review, not source qualification or
musical evidence. It directly enables `RIOTBOX-1428`: one fresh
development-only `StageAQualificationSession` v2 and, only after admission,
the frozen three-family by four-source by two-event matrix.

At this checkpoint no Batch-v2 attachment GET, Batch-v2 source-directory
discovery, Batch-v2 audio decode, Batch-v2 PCM iteration, Batch-v2 source/event
computation, candidate render, playback, holdout access, or
commercial-reference access has occurred.

## Frozen Inputs

- Protocol P2 raw / semantic SHA-256:
  `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` /
  `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`
- Acquisition Batch v2 raw / semantic SHA-256:
  `d9b92635734e65d0154a7c17143c8759cc758d9b9cf756cda740b08623c53067` /
  `af1605b781004aab984ff75845962130ca22fe936bf9f59a89de7e7ab8942dfb`
- Predecessor Registry R2 raw / semantic SHA-256:
  `af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812` /
  `6cfe11cd10a5947427a09335fbd4795706c71530b6f6a7e5b9883259bcca8ce1`
- Rejected Batch v1 raw / semantic SHA-256:
  `ada49dc778bebe201c399413122765fce08d4476c445af30c2a1982bd524e6c9` /
  `7c103a12b743e8c9406d66008527c0036dac472c78bee5512ee46beb7492b362`
- Batch-v1 rejection report raw SHA-256:
  `2ab9d34888d5ba0a442a408f2e10e9f201fdbfa6291ffdd09d003908c93da619`
- Batch-v1 access-log raw SHA-256:
  `703806c0f6548f1af2e2f51408553e51f060ea51f4f47202079d63145540c174`
- Batch-v2 executable snapshot aggregate SHA-256:
  `690318c53fc5bb43254ce8cfdc99fec00d85a3613317d242f49b1cb6806992f1`

The rejected v1 attempt remains immutable: attempt
`1e6a1070-5df4-4813-8e07-dc30fae7c70a`, request counts `2/1`, rejection stage
`request_2_header`, and observed payload hashes
`00d1ec0b442db60ade056fe24a72c18cc0f8deed23301f5ec961029f3eb810f9`
and `2212c182906ae1b7449e26c31b4c96f132c348a33fdd82c0b00f785f7a677e5f`.
Both payload identities are forbidden in Batch v2; no v1 survivor, retry,
redirect, substitution, or fallback is authorized.

## Complete Registered Batch v2

1. `oga_farfadet46_loopable_beat_ludumdare` — provisional `dense_break` —
   `test.wav` — 1,695,934 bytes — OpenGameArt attachment `45609`
2. `oga_celestialghost8_cc0_scraps_slowdrum` — provisional `sparse_drums` —
   `slowdrum - Track 02 (New song).wav` — 580,400 bytes — attachment `93984`
3. `oga_cosmac_8_bit_disco_loop` — provisional `electronic_drums` —
   display `title.wav`, exact URL basename `title_1.wav` — 676,908 bytes —
   attachment `239763`

All three named pages declare CC0 and did not disclose a third-party source or
sample pack. “Not disclosed” is not proof of absence. Every family assignment
is a metadata hypothesis only; all entries remain unheard, uncomputed, and
unqualified. The sparse hypothesis is based only on its attachment filename;
the page does not claim a loop, at least three onsets, or drums-only content.

## Fail-Closed Gate

Before creating an access log or touching the network, the one-shot runner
requires the exact registered Batch-v2 JSON, executable files, fixtures,
Justfile, predecessor rejection report, and this review to exist as
byte-identical blobs in the current Git HEAD. The large Decision Log is not
read by this gate; targeted RBX-255 authorization is reflected in the frozen
Batch-v2 contract and was reviewed separately. Unrelated user files outside
the closed path set may remain dirty. Every request checkpoint revalidates the
frozen batch, implementation aggregate, and unchanged repository HEAD.

The runner permits exactly three ordered GET requests and no HEAD/probe,
redirect, automatic retry, proxy, authentication, content transform,
directory discovery, survivor reuse, substitution, or fallback. It validates
only bounded response identity and strict RIFF/WAVE PCM headers, stores the
attempt through fsynced atomic transitions, seals the complete batch in
quarantine, and publishes only through one same-filesystem no-replace atomic
rename. Any failure consumes and rejects the whole Batch-v2 attempt. Only an
already durable publication intent may use the no-network reconciler.

## Source-Blind Proof

The final compact Batch-v2 suite passes with:

- 86 fail-closed Batch-v2 contract mutations;
- 87 fail-closed access-log / sealed-manifest and cross-version fixtures;
- 50 one-shot runner no-network fixtures, including exact closed-set Gate-path
  assertions, a missing HEAD runner blob, Worktree/HEAD runner mismatch before
  log or DNS, and in-flight implementation / HEAD drift.

The historical regression suite also passes with 51 Protocol-v2 mutations, 45
Batch-v1 mutations, 20 RIFF mutations, 2 byte-stream mutations, 15
network-metadata mutations, 1 atomic-publication fixture, 73 Batch-v1 artifact
fixtures, and 43 Batch-v1 no-network runner fixtures. P2, R2, Batch v1, its
rejection report, and its ignored durable access log retain their exact pins.

`cargo fmt --check` and `cargo test` completed successfully before the broad CI
entered audio QA; a separate final `cargo clippy --all-targets --all-features
-- -D warnings` also passes. The broad `just ci` run was deliberately stopped
with exit 130 after process inspection showed an existing demo-bank recipe
running against one exact pre-existing repository test-source WAV. That path
was neither Batch v2, holdout, nor a commercial reference, and no result from
it changed any frozen choice or pin. The broad interrupted run is not claimed
as Gate evidence; stopping it prevented further source-backed QA from crossing
this source-blind boundary.

An independent source-blind re-review reports no remaining finding and no GET
blocker. Exact Batch-v2 access-log, `.next`, quarantine, and final paths were
absent before review and must be checked again after the immutable pre-GET
commit and immediately before execution.

## Claims And Next Boundary

This gate makes no family-fitness, event-admission, hardness, force, musical,
or human-listening claim. P2 equations, detector catalog, anatomy catalog,
source-contrast catalog, rhythmic proxy, event ordinals, and numeric
thresholds remain source-blind and frozen. Any required change after source
evidence needs a new version and targeted Decision-Log decision; RBX-255 does
not authorize Batch v3.

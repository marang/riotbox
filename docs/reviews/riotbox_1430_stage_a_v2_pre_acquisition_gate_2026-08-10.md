# RIOTBOX-1430 Stage-A-v2 Pre-Acquisition Gate — 2026-08-10

## Verdict

`pass_for_one_exact_registered_development_acquisition`

This is a source-blind contract-enabler review, not source qualification or
musical evidence. It directly enables exactly one fresh development-only
`RIOTBOX-1428` `StageAQualificationSession` after Registry-v3 and Matrix-v3
are frozen. At this checkpoint no attachment GET, source-directory discovery,
audio decode, PCM iteration, source/event computation, candidate render,
playback, holdout access, or commercial-reference access has occurred.

## Frozen Inputs

- Protocol P2 raw SHA-256:
  `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878`
- Protocol P2 semantic SHA-256:
  `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`
- Acquisition batch v1 raw SHA-256:
  `ada49dc778bebe201c399413122765fce08d4476c445af30c2a1982bd524e6c9`
- Acquisition batch v1 semantic SHA-256:
  `7c103a12b743e8c9406d66008527c0036dac472c78bee5512ee46beb7492b362`
- Predecessor Registry R2 raw SHA-256:
  `af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812`
- Predecessor Registry R2 semantic SHA-256:
  `6cfe11cd10a5947427a09335fbd4795706c71530b6f6a7e5b9883259bcca8ce1`
- Source-blind executable snapshot aggregate SHA-256:
  `4a5f210f0c8ef1d5692f25b7ddf9371656313eb8e933264bf959e1dc18c26df2`

The access log captures the then-current Git HEAD only as provenance and binds
all nine executable files by raw hash. The aggregate excludes HEAD itself, so
this pre-acquisition review document and the pre-GET commit do not alter the
executable snapshot.

The registered non-sequential batch is exactly:

1. `oga_eldritch_grim_pirates_incoming` — provisional `dense_break`
2. `oga_turnovus_simple_drumbeat_start` — provisional `sparse_drums`
3. `oga_fupi_tense_bass_boost_drums` — provisional `electronic_drums`

All family assignments remain metadata hypotheses. All three entries remain
unheard, uncomputed, and unqualified. No substitution, retry, sequential
survivor selection, or fallback is authorized.

## Fail-Closed Gate

The reviewed one-shot runner:

- persists request reservation before DNS/network and permits one exact GET per
  entry in frozen ordinal order;
- resolves only the absolute root-dot provider DNS name, accepts only
  global-unicast answers and one selected endpoint, one TLS connection, exact HTTP
  200/content length, allowed WAV MIME, no redirect/retry/proxy/auth/content
  transform, and exactly bounded response bytes;
- validates only strict RIFF/WAVE PCM identity and header structure before R3;
- rejects payload identity collisions against R2 metadata hashes and within the
  new batch;
- uses exclusive exact paths, no listing/glob/walk, a same-filesystem
  `renameat2(RENAME_NOREPLACE)` capability probe, and a pre-GET free-space gate;
- stores every log transition through a fully fsynced atomic `.next` successor;
- reconciles a durable `publication_pending` state without DNS or network;
- binds manifest and payload device/inode/link-count/byte-count plus exact raw
  hashes and headers;
- seals the batch directory, revalidates manifest and all three payloads
  immediately before rename, immediately after rename, and again after the
  terminal log commit;
- revalidates directory chains plus exact log/final name-to-held-inode identity
  and runs the repository terminal validator before returning success.

Failures after a consumed request stop all later requests. Failures after the
directory rename retain a fail-closed publication state and authorize only the
no-network reconciler. A torn/stale `.next`, changed executable snapshot,
namespace swap, missing/tampered/replaced artifact, or terminal mismatch cannot
produce a successful return.

## Synthetic Proof

`just percussive-force-stage-a-v2-contract-fixtures` passes with:

- 51 fail-closed P2 mutations;
- 45 fail-closed acquisition-batch mutations;
- 20 RIFF, 2 raw-stream, 15 network-metadata, and 1 atomic-publication
  primitive fixtures;
- 73 fail-closed access-log/manifest fixtures;
- 43 no-network runner fixtures.

The runner fixtures include write/truncate/fsync/replace/parent-fsync failures,
torn atomic successors, both quarantine- and already-published reconciliation,
low-space rejection before GET, payload and manifest mutation between sealing
and publication, implementation drift, directory identity replacement, and
in-flight access-log/final-directory namespace swaps in both the main and
reconcile paths, last-read name replacement, and same-inode in-place mutation
of the terminal access log, manifest, and payload.

P2 additionally freezes whole-source DC reduction as exact signed-PCM integer
accumulation followed by one exact-rational-to-binary64 roundTiesToEven step,
including canonical positive zero and a cancellation-heavy PCM24 stereo golden.

Two independent source-blind pre-GET reviews reproduced earlier atomic-log,
publication-TOCTOU, and terminal-namespace weaknesses. The current fixtures
reproduce those attacks as fail-closed, and the final review reports no
remaining GET blocker.

## Trust Boundary And Claims

The repository and executing user session are trusted against arbitrary
same-UID code injection. The gate still detects in-scope accidental or
adversarial namespace, inode, byte, hash, header, and implementation drift at
its declared checkpoints. It does not claim to defend against an attacker who
can continuously replace the running program or mutate state after the final
instruction.

This gate makes no family-fitness, event-admission, hardness, force, musical,
or human-listening claim. P2 equations, detector catalog, anatomy catalog,
source-contrast catalog, rhythmic proxy, event ordinals, and numeric thresholds
remain source-blind and frozen. Any required change after source evidence needs
a new version and a targeted Decision-Log decision before another request.

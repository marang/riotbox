# RIOTBOX-1430 Stage-A-v2 Acquisition Batch-v2 Rejection — 2026-08-10

## Verdict

`rejected_fail_closed_no_publication`

The single authorized acquisition attempt for
`riotbox.percussive_force_stage_a_v2_acquisition_batch.v2` is consumed and
must never be retried. The runner completed all three exact registered GETs
and stopped during the third entry's header-only format gate because its PCM
sample rate was outside the frozen inclusive range.

This is an acquisition/format rejection only. No audio decode, PCM sample
iteration, source feature or event computation, candidate render, source
playback, holdout access, or commercial-reference access occurred.

## Frozen Attempt Evidence

- Attempt ID: `58913eec-8ded-4187-b1e8-e6de07d04f6f`
- Access-log raw SHA-256:
  `ac4faa5cdb51f03d6e41ec5e41caa57643b67370d94177b0ef4a1e84e55fe83c`
- Repository HEAD:
  `5134399e6dbe310d32d976c780dd03d8f0b30cf8`
- Implementation aggregate SHA-256:
  `690318c53fc5bb43254ce8cfdc99fec00d85a3613317d242f49b1cb6806992f1`
- Protocol P2 raw / semantic SHA-256:
  `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` /
  `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`
- Batch-v2 raw / semantic SHA-256:
  `d9b92635734e65d0154a7c17143c8759cc758d9b9cf756cda740b08623c53067` /
  `af1605b781004aab984ff75845962130ca22fe936bf9f59a89de7e7ab8942dfb`

The rejected access log was reopened by exact path, validated against the
frozen Batch-v2 contract and current implementation snapshot, and reproduced
byte-for-byte through the deterministic renderer.

Durable state:

- `attempt_status = rejected`
- `request_count = 3`
- `successful_request_count = 2`
- entry states: `header_verified`, `header_verified`, `body_verified`
- entry request counts: `1`, `1`, `1`
- rejection stage: `request_3_header`
- `further_requests_performed = false`
- `publication_authorized = false`
- `new_versioned_metadata_decision_required = true`
- quarantine cleanup: `removed_exact_known_names`
- publication state: `not_started`
- atomic `.next`, quarantine, final batch, and both publication-probe paths:
  absent

## Identity And Header Evidence

The first registered body was exact-length and header-valid:

- case: `oga_farfadet46_loopable_beat_ludumdare`
- raw payload SHA-256:
  `fd5c5277301069fb9d9fe53706e8eed3deb3ce90cd554c405b26c3e223adc2da`
- bytes: `1695934`
- RIFF/WAVE PCM16, stereo, 44.1 kHz
- frames: `423936`
- duration: `423936 / 44100` seconds, below 16 seconds
- header scope: container headers only, no sample-payload reads

The second registered body was exact-length and header-valid:

- case: `oga_celestialghost8_cc0_scraps_slowdrum`
- raw payload SHA-256:
  `a507d3d8c55e2cf8bb93d9adcc49caa3e34c122f9484e636cd60271f9d1f7e6c`
- bytes: `580400`
- RIFF/WAVE PCM16, mono, 44.1 kHz
- frames: `290178`
- duration: `290178 / 44100` seconds, below 16 seconds
- header scope: container headers only, no sample-payload reads

The third registered body was exact-length before its header gate rejected:

- case: `oga_cosmac_8_bit_disco_loop`
- raw payload SHA-256:
  `494877eb8e6677acc7c9bf7e2dc0c8e857e5a60291f17d9f6ca972bb8bb823d3`
- bytes: `676908`
- rejection: PCM sample rate outside the frozen inclusive range
- no authoritative header record was emitted

All three identities and observed payload hashes are rejection history only.
The first two header-valid entries are not survivors and convey no source,
family, event, hardness, force, or musical fitness.

## Change Control

No Batch-v2 entry may survive into another acquisition based on this result.
Keeping either header-valid item would be forbidden sequential survivor
selection. Another acquisition would require all of the following before any
request:

1. a complete new metadata-only three-family batch with three newly selected
   entries and no Batch-v1 or Batch-v2 survivor;
2. a version bump to acquisition Batch v3 with fresh raw/semantic pins and
   disjoint exact paths;
3. a new targeted Decision-Log entry recording both consumed attempts, all
   forbidden identities and payload hashes, the full replacement batch, and
   unchanged P2 algorithms/numerics;
4. a fresh parallel validator/artifact/runner stack, no-network fixtures,
   absence checks, independent review, and immutable pre-GET commit.

RBX-255 did not authorize Batch v3. Registry v3, Matrix v3, source
qualification, rendering, and listening remain blocked. P2 algorithms,
detector/anatomy/source-contrast catalogs, event ordinals, thresholds, and
numeric passports are unchanged by this rejection.

# RIOTBOX-1430 Stage-A-v2 Acquisition Batch-v1 Rejection — 2026-08-10

## Verdict

`rejected_fail_closed_no_publication`

The single authorized acquisition attempt for
`riotbox.percussive_force_stage_a_v2_acquisition_batch.v1` is consumed and
must never be retried. The runner stopped during the second registered entry's
header-only format gate because its decoded RIFF/WAVE duration exceeded the
frozen 16-second maximum.

This is an acquisition/format rejection only. No audio decode, PCM sample
iteration, source feature or event computation, candidate render, source
playback, holdout access, or commercial-reference access occurred.

## Frozen Attempt Evidence

- Attempt ID: `1e6a1070-5df4-4813-8e07-dc30fae7c70a`
- Access-log raw SHA-256:
  `703806c0f6548f1af2e2f51408553e51f060ea51f4f47202079d63145540c174`
- Repository HEAD:
  `a937e5e8a163ab40ad6dc82ecb1377cd9f077dc4`
- Implementation aggregate SHA-256:
  `4a5f210f0c8ef1d5692f25b7ddf9371656313eb8e933264bf959e1dc18c26df2`
- Protocol P2 raw / semantic SHA-256:
  `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878` /
  `6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb`
- Batch-v1 raw / semantic SHA-256:
  `ada49dc778bebe201c399413122765fce08d4476c445af30c2a1982bd524e6c9` /
  `7c103a12b743e8c9406d66008527c0036dac472c78bee5512ee46beb7492b362`

Durable state:

- `attempt_status = rejected`
- `request_count = 2`
- `successful_request_count = 1`
- entry states: `header_verified`, `body_verified`, `not_requested`
- entry request counts: `1`, `1`, `0`
- rejection stage: `request_2_header`
- `further_requests_performed = false`
- `publication_authorized = false`
- `new_versioned_metadata_decision_required = true`
- quarantine cleanup: `removed_exact_known_names`
- publication state: `not_started`
- atomic `.next`, quarantine, and final batch paths: absent

## Identity And Header Evidence

The first registered body was exact-length and header-valid:

- case: `oga_eldritch_grim_pirates_incoming`
- raw payload SHA-256:
  `00d1ec0b442db60ade056fe24a72c18cc0f8deed23301f5ec961029f3eb810f9`
- bytes: `2048078`
- RIFF/WAVE PCM16, stereo, 44.1 kHz
- frames: `512000`
- duration: `512000 / 44100` seconds, below 16 seconds
- header scope: container headers only, no sample-payload reads

The second registered body was exact-length before its header gate rejected:

- case: `oga_turnovus_simple_drumbeat_start`
- raw payload SHA-256:
  `2212c182906ae1b7449e26c31b4c96f132c348a33fdd82c0b00f785f7a677e5f`
- bytes: `3629150`
- rejection: RIFF/WAVE duration exceeds the frozen 16-second maximum
- no authoritative header record was emitted

The third registered URL was never requested.

## Change Control

No Batch-v1 entry may survive into a retry based on this result. Keeping the
first header-valid item would be forbidden sequential survivor selection.
Another acquisition requires all of the following before any request:

1. a complete new metadata-only three-family batch with three newly selected
   entries and no Batch-v1 survivor;
2. a version bump to acquisition Batch v2 with fresh raw/semantic pins;
3. a targeted Decision-Log entry recording the consumed v1 attempt, the full
   replacement batch, and unchanged P2 algorithms/numerics;
4. fresh fail-closed validators, fixtures, absence checks, review, and a new
   immutable pre-GET commit.

Registry v3, Matrix v3, source qualification, rendering, and listening remain
blocked. P2 algorithms, detector/anatomy/source-contrast catalogs, event
ordinals, thresholds, and numeric passports are unchanged by this rejection.
